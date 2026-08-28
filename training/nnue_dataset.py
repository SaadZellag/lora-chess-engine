import time
import numpy as np
import ctypes
import torch
import os
from torch.utils.data import Dataset
from consts import *
from pathlib import Path

import torch

torch.sparse.check_sparse_tensor_invariants.disable()

from dll_loader import *


def get_tensors(batch: SparseBatch) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor]:

        # This is illustrative. In reality you might need to transfer these
        # to the GPU. You can also do it asynchronously, but remember to make
        # sure the source lives long enough for the copy to finish.

        score_t = torch.from_numpy(np.ctypeslib.as_array(
            batch.final_score, shape=(batch.size, 1)))

        eval_t = torch.from_numpy(np.ctypeslib.as_array(
            batch.eval, shape=(batch.size, 1)))

        mom_t = torch.from_numpy(np.ctypeslib.as_array(
            batch.mom_value, shape=(batch.size, 1)))
        mom_t = torch.clamp(mom_t / MOM_TARGET, 0, 1)

        eval_t = score(eval_t, mom_t)

        final_eval_t = eval_t * mom_t + (1 - mom_t) * score_t

        # Now we don't have to bother with the sparse pytorch tensors!
        # And no transpositions required too because we have control over the layout!
        our_features_indices_t = torch.transpose(torch.from_numpy(
            np.ctypeslib.as_array(batch.our_feature_indices, shape=(batch.num_active_our_features, 2))), 0, 1)
        their_features_indices_t = torch.transpose(torch.from_numpy(
            np.ctypeslib.as_array(batch.their_feature_indices, shape=(batch.num_active_their_features, 2))), 0, 1)


        # The values are all ones, so we can create these tensors in place easly.
        # No need to go through a copy.
        our_features_values_t = torch.ones(batch.num_active_our_features)
        their_features_values_t = torch.ones(batch.num_active_their_features)

        # validate_sparse_coo_inputs(
        #     indices=our_features_indices_t,
        #     values=our_features_values_t,
        #     size=(self.size, NUM_FEATURES),
        #     is_coalesced=True
        # )

        # validate_sparse_coo_inputs(
        #     indices=their_features_indices_t,
        #     values=their_features_values_t,
        #     size=(self.size, NUM_FEATURES),
        #     is_coalesced=True
        # )

        # Now the magic. We construct a sparse tensor by giving the indices of
        # non-zero values (active feature indices) and the values itself (all ones!).
        # The size of the tensor is batch_size*NUM_FEATURES, which would
        # normally be insanely large, but since the density is ~0.1% it takes
        # very little space and allows for faster forward pass.
        our_features_t = torch.sparse_coo_tensor(
            our_features_indices_t, our_features_values_t, (batch.size, NUM_FEATURES))
        their_features_t = torch.sparse_coo_tensor(
            their_features_indices_t, their_features_values_t, (batch.size, NUM_FEATURES))

        # What is coalescing?! It makes sure the indices are unique and ordered.
        # We **know** the indices come out of the DLL already sorted/unique per
        # row, so we can tell pytorch it doesn't need to redo that work (an
        # O(n log n) operation) on every forward pass.
        our_features_t._coalesced_(True)
        their_features_t._coalesced_(True)

        return our_features_t, their_features_t, final_eval_t


class SparseBatchStreamIterator:
    def __init__(self, stream_ptr: SparseBatchStreamPtr, dll_loader: DLLLoader):
        self.stream_ptr = stream_ptr
        self.dll_loader = dll_loader
        self.prev_batch_ptr = None

    def __iter__(self):
        return self

    def __next__(self):
        if self.prev_batch_ptr is not None:
            self.dll_loader.drop_sparse_batch(self.prev_batch_ptr)
            self.prev_batch_ptr = None
        batch_ptr = self.dll_loader.fetch_next_batch(self.stream_ptr)
        if not batch_ptr:
            raise StopIteration

        self.prev_batch_ptr = batch_ptr

        return get_tensors(batch_ptr.contents)
        


class SparseBatchDataset(torch.utils.data.IterableDataset):
    def __init__(self, dll_filename: str, filename: str, batch_size: int, num_workers: int, step_size: int):
        # self.load_dll(dll_filename)
        # self.compute_approximate_size(filename, batch_size)

        self.dll_filename = dll_filename
        self.filename = filename
        self.batch_size = batch_size
        self.num_workers = num_workers
        self.step_size = step_size

        self.dll = None
        self.stream_loader = None
        self.stream = None
        self.current_epoch = 0

    # def compute_approximate_size(self, filename: str, batch_size: int):
    #     BYTES_PER_ENTRY = 2.5
    #     file_size = Path(filename).stat().st_size
    #     self.approx_num_batches = int(file_size // (batch_size * BYTES_PER_ENTRY))

    # def __len__(self):
    #     return self.approx_num_batches

    def __iter__(self):
        if self.dll is None:
            self.dll = DLLLoader(self.dll_filename)

        if self.stream_loader is None:
            self.stream_loader = self.dll.create_sparse_batch_stream_provider(
                self.filename, 
                self.batch_size, 
                self.num_workers, 
                self.step_size
            )

        if self.stream_loader is None:
            raise RuntimeError("Failed to create sparse batch stream provider.")

        if self.stream:
            self.dll.drop_sparse_batch_stream(self.stream)
            self.stream = None

        worker_info = torch.utils.data.get_worker_info()
        if worker_info is None:
            worker_id = 0
        else:
            worker_id = worker_info.id

        self.stream = self.dll.get_sparse_batch_stream(
            self.stream_loader,
            worker_id,
            self.current_epoch
        )

        self.current_epoch += 1
        
        return SparseBatchStreamIterator(self.stream, self.dll)

    def __del__(self):
        if self.dll and self.stream_loader:
            self.dll.drop_sparse_batch_stream_provider(self.stream_loader)


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description='Test SparseBatchDataset')
    parser.add_argument('--dll-path', type=str, required=True,
                        help='Path to the shared library (DLL/SO/DYLIB)')
    parser.add_argument('--data-path', type=str, required=True,
                        help='Path to the data file')
    parser.add_argument('--batch-size', type=int, default=8192,
                        help='Batch size for training (default: 8192)')
    parser.add_argument('--num-workers', type=int, default=4,
                        help='Number of worker threads (default: 4)')
    parser.add_argument('--step-size', type=int, default=1,
                        help='Step size for the dataset (default: 1)')
    args = parser.parse_args()


    dataset = SparseBatchDataset(
        args.dll_path, 
        args.data_path, 
        args.batch_size,
        args.num_workers,
        args.step_size
        )

    start_time = time.time()
    batches_read = 0

    dataloader = torch.utils.data.DataLoader(dataset, batch_size=None, num_workers=args.num_workers, persistent_workers=True)

    try:
        for i, data in enumerate(dataloader):
            batches_read = i + 1
    except KeyboardInterrupt:
        print("Interrupted by user.")

    end_time = time.time()
    elapsed_time = end_time - start_time
    print(f"Read {batches_read} batches in {elapsed_time:.2f} seconds.")
    print(f"Batches per second: {batches_read / elapsed_time:.2f}")
