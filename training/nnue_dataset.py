import time
import numpy as np
import ctypes
import torch
import os
import sys
import glob
from torch.utils.data import Dataset
from consts import *
from pathlib import Path



class SparseBatch(ctypes.Structure):
    _fields_ = [
        ('size', ctypes.c_int),
        ('num_active_our_features', ctypes.c_int),
        ('num_active_their_features', ctypes.c_int),

        ('final_score', ctypes.POINTER(ctypes.c_float)),
        ('mom_value', ctypes.POINTER(ctypes.c_int)),
        ('eval', ctypes.POINTER(ctypes.c_int)),
        ('our_feature_indices', ctypes.POINTER(ctypes.c_int)),
        ('their_feature_indices', ctypes.POINTER(ctypes.c_int)),
    ]

    def get_tensors(self):

        # This is illustrative. In reality you might need to transfer these
        # to the GPU. You can also do it asynchronously, but remember to make
        # sure the source lives long enough for the copy to finish.

        score_t = torch.from_numpy(np.ctypeslib.as_array(
            self.final_score, shape=(self.size, 1)))

        eval_t = torch.from_numpy(np.ctypeslib.as_array(
            self.eval, shape=(self.size, 1)))

        mom_t = torch.from_numpy(np.ctypeslib.as_array(
            self.mom_value, shape=(self.size, 1)))
        mom_t = torch.clamp(mom_t / MOM_TARGET, 0, 1)

        eval_t = score(eval_t, mom_t)

        final_eval_t = eval_t * mom_t + (1 - mom_t) * score_t

        # Now we don't have to bother with the sparse pytorch tensors!
        # And no transpositions required too because we have control over the layout!
        our_features_indices_t = torch.transpose(torch.from_numpy(
            np.ctypeslib.as_array(self.our_feature_indices, shape=(self.num_active_our_features, 2))), 0, 1)
        their_features_indices_t = torch.transpose(torch.from_numpy(
            np.ctypeslib.as_array(self.their_feature_indices, shape=(self.num_active_their_features, 2))), 0, 1)

        # print("=" * 50)
        # # print(our_features_indices_t[1])
        # print((our_features_indices_t[1] == 65533).nonzero(as_tuple=True))
        # print(our_features_indices_t[1][1040:1060])
        # print(our_features_indices_t[1][17470:17490])
        # print("=" * 50)

        # The values are all ones, so we can create these tensors in place easly.
        # No need to go through a copy.
        our_features_values_t = torch.ones(self.num_active_our_features)
        their_features_values_t = torch.ones(self.num_active_their_features)

        # Now the magic. We construct a sparse tensor by giving the indices of
        # non-zero values (active feature indices) and the values itself (all ones!).
        # The size of the tensor is batch_size*NUM_FEATURES, which would
        # normally be insanely large, but since the density is ~0.1% it takes
        # very little space and allows for faster forward pass.
        # For maximum performance we do cheat somewhat though. Normally pytorch
        # checks the correctness, which is an expensive O(n) operation.
        # By using _sparse_coo_tensor_unsafe we avoid that.
        our_features_t = torch.sparse_coo_tensor(
            our_features_indices_t, our_features_values_t, (self.size, NUM_FEATURES))
        their_features_t = torch.sparse_coo_tensor(
            their_features_indices_t, their_features_values_t, (self.size, NUM_FEATURES))

        # What is coalescing?! It makes sure the indices are unique and ordered.
        # Now you probably see why we said the inputs must be ordered from the start.
        # This is normally a O(n log n) operation and takes a significant amount of
        # time. But here we **know** that the tensor is already in a coalesced form,
        # therefore we can just tell pytorch that it can use that assumption.
        # our_features_t._coalesced_(True)
        # their_features_t._coalesced_(True)

        return our_features_t, their_features_t, final_eval_t


SparseBatchPtr = ctypes.POINTER(SparseBatch)

class SparseBatchDataset(torch.utils.data.IterableDataset):
    def __init__(self, dll_filename: str, filename:str, batch_size: int):
        self.load_dll(dll_filename)
        self.compute_approximate_size(filename, batch_size)

        self.filename = filename.encode('utf-8')
        self.batch_size = batch_size

    def load_dll(self, dll_filename: str):
        if not os.path.exists(dll_filename):
            raise FileNotFoundError(f"Shared library {dll_filename} not found.")
        dll = ctypes.cdll.LoadLibrary(dll_filename)

        self.create_sparse_batch_stream = dll.create_sparse_batch_stream
        self.create_sparse_batch_stream.argtypes = [ctypes.c_char_p, ctypes.c_int]
        self.create_sparse_batch_stream.restype = ctypes.c_void_p

        self.fetch_next_batch = dll.fetch_next_batch
        self.fetch_next_batch.argtypes = [ctypes.c_void_p]
        self.fetch_next_batch.restype = SparseBatchPtr

        self.drop_sparse_batch_stream = dll.drop_sparse_batch_stream
        self.drop_sparse_batch_stream.argtypes = [ctypes.c_void_p]

        self.drop_sparse_batch = dll.drop_sparse_batch
        self.drop_sparse_batch.argtypes = [ctypes.c_void_p]

    def compute_approximate_size(self, filename: str, batch_size: int):
        BYTES_PER_ENTRY = 2.5
        file_size = Path(filename).stat().st_size
        self.approx_num_batches = int(file_size // (batch_size * BYTES_PER_ENTRY))

    def __len__(self):
        return self.approx_num_batches

    def __iter__(self):
        self.delete_stream()
        self.stream = self.create_sparse_batch_stream(self.filename, self.batch_size)
        return self

    def __next__(self):
        batch = self.fetch_next_batch(self.stream)

        if batch:
            tensors = batch.contents.get_tensors()
            self.drop_sparse_batch(batch)
            return tensors
        else:
            raise StopIteration

    def __del__(self):
        self.delete_stream()

    def delete_stream(self):
        if hasattr(self, 'stream') and self.stream:
            self.drop_sparse_batch_stream(self.stream)


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description='Test SparseBatchDataset')
    parser.add_argument('--dll-path', type=str, required=True,
                        help='Path to the shared library (DLL/SO/DYLIB)')
    parser.add_argument('--data-path', type=str, required=True,
                        help='Path to the data file')
    parser.add_argument('--batch-size', type=int, default=8192,
                        help='Batch size for training (default: 8192)')
    args = parser.parse_args()


    dataset = SparseBatchDataset(args.dll_path, args.data_path, args.batch_size)

    start_time = time.time()
    batches_read = 0

    try:
        for i, data in enumerate(dataset):
            batches_read = i + 1
    except KeyboardInterrupt:
        print("Interrupted by user.")

    end_time = time.time()
    elapsed_time = end_time - start_time
    print(f"Read {batches_read} batches in {elapsed_time:.2f} seconds.")
    print(f"Batches per second: {batches_read / elapsed_time:.2f}")
        