import time
import numpy as np
import ctypes
import torch
import os
import sys
import glob
import threading
import queue
from torch.utils.data import Dataset
from consts import *
from pathlib import Path

import torch

torch.sparse.check_sparse_tensor_invariants.disable()

def validate_sparse_coo_inputs(
    indices: torch.Tensor,
    values: torch.Tensor,
    size: tuple[int, ...],
    is_coalesced: bool = False
) -> None:
    """
    Validates inputs before calling torch.sparse_coo_tensor.
    Raises ValueError or TypeError if constraints are violated.
    """
    # 1. Type Checks
    if not isinstance(indices, torch.Tensor):
        raise TypeError(f"`indices` must be a torch.Tensor, got {type(indices).__name__}")
    if not isinstance(values, torch.Tensor):
        raise TypeError(f"`values` must be a torch.Tensor, got {type(values).__name__}")
    if not isinstance(size, (tuple, list)):
        raise TypeError(f"`size` must be a tuple or list of ints, got {type(size).__name__}")

    # 2. Indices Tensor Checks
    if indices.ndim != 2:
        raise ValueError(f"`indices` must be 2D of shape (sparse_dims, nnz), got shape {tuple(indices.shape)}")
    if indices.dtype not in (torch.int64, torch.int32):
        raise ValueError(f"`indices` dtype must be int64 or int32, got {indices.dtype}")

    sparse_dims, nnz = indices.shape

    # 3. Size Checks
    if len(size) < sparse_dims:
        raise ValueError(f"Size tuple length ({len(size)}) cannot be smaller than sparse dimensions ({sparse_dims})")
    if any(s < 0 for s in size):
        raise ValueError(f"All dimensions in `size` must be non-negative, got {size}")

    # 4. Values Tensor Alignment
    # values shape must start with nnz, followed by dense dimensions if hybrid sparse tensor
    expected_values_shape = (nnz,) + tuple(size[sparse_dims:])
    if values.shape != expected_values_shape:
        raise ValueError(f"`values` shape mismatch. Expected {expected_values_shape}, got {tuple(values.shape)}")

    # 5. Out-of-Bounds Index Checks
    if nnz > 0:
        if torch.any(indices < 0):
            raise ValueError("Found negative values in `indices`")
        
        # Check boundary per sparse dimension
        size_tensor = torch.tensor(size[:sparse_dims], dtype=indices.dtype, device=indices.device).unsqueeze(1)
        oob_mask = indices >= size_tensor
        if torch.any(oob_mask):
            bad_dim, bad_idx = torch.where(oob_mask)
            dim_val = bad_dim[0].item()
            invalid_val = indices[bad_dim[0], bad_idx[0]].item()
            raise ValueError(
                f"Index out of bounds in dimension {dim_val}: "
                f"value {invalid_val} >= dim max size {size[dim_val]}"
            )

    # 6. Optional Coalescence Check (Duplicate Indices)
    if is_coalesced and nnz > 1:
        # Transpose to shape (nnz, sparse_dims) for row comparison
        idx_t = indices.T
        sorted_idx, _ = torch.sort(idx_t, dim=0)
        duplicates = (idx_t[1:] == idx_t[:-1]).all(dim=1)
        if torch.any(duplicates):
            raise ValueError("`is_coalesced=True` was passed, but duplicate indices were detected.")


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


        # The values are all ones, so we can create these tensors in place easly.
        # No need to go through a copy.
        our_features_values_t = torch.ones(self.num_active_our_features)
        their_features_values_t = torch.ones(self.num_active_their_features)

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
            our_features_indices_t, our_features_values_t, (self.size, NUM_FEATURES))
        their_features_t = torch.sparse_coo_tensor(
            their_features_indices_t, their_features_values_t, (self.size, NUM_FEATURES))

        # What is coalescing?! It makes sure the indices are unique and ordered.
        # We **know** the indices come out of the DLL already sorted/unique per
        # row, so we can tell pytorch it doesn't need to redo that work (an
        # O(n log n) operation) on every forward pass.
        our_features_t._coalesced_(True)
        their_features_t._coalesced_(True)

        return our_features_t, their_features_t, final_eval_t


SparseBatchPtr = ctypes.POINTER(SparseBatch)


class _StopSentinel:
    """Marker put on the prefetch queue to signal the stream is exhausted."""
    pass


class SparseBatchDataset(torch.utils.data.IterableDataset):
    """
    Iterable dataset backed by a single DLL-driven data stream.

    Fetching happens on a background *thread* (not a separate process/worker).
    This is intentional: `fetch_next_batch` is a ctypes call into a C shared
    library, and ctypes releases the GIL for the duration of foreign calls.
    That means the background thread's DLL fetch can genuinely run in
    parallel with the main thread doing GPU work, without any of the
    downsides of multiprocessing DataLoader workers:
      - no duplicated passes over the data (each `num_workers > 1` worker
        would otherwise open its own independent stream over the *entire*
        file, since there's no sharding logic for this custom stream)
      - no cross-process pickling / shared-memory IPC overhead for the
        sparse tensors
      - exactly one stream, one source of truth, correct epoch semantics

    Use this with `DataLoader(dataset, batch_size=None, num_workers=0)` —
    batching is already done inside the DLL, and num_workers must stay 0
    since we're doing our own threaded prefetching instead.
    """

    def __init__(self, dll_filename: str, filename: str, batch_size: int, prefetch: int = 2):
        self.load_dll(dll_filename)
        self.compute_approximate_size(filename, batch_size)

        self.filename = filename.encode('utf-8')
        self.batch_size = batch_size
        self.prefetch = prefetch

        self.stream = None
        self._thread = None
        self._queue = None
        self._stop_event = None

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
        # Tear down any previous stream/thread (e.g. from the last epoch)
        # before starting a fresh pass over the data.
        self._stop_prefetch_thread()
        self.delete_stream()

        self.stream = self.create_sparse_batch_stream(self.filename, self.batch_size)

        self._queue = queue.Queue(maxsize=self.prefetch)
        self._stop_event = threading.Event()
        self._thread = threading.Thread(
            target=self._prefetch_loop,
            daemon=True,
        )
        self._thread.start()

        return self

    def _prefetch_loop(self):
        """
        Runs on a background thread. Repeatedly fetches batches from the DLL
        stream and pushes ready-to-use tensors onto the queue. The
        `fetch_next_batch` call releases the GIL while it runs, so this
        overlaps with whatever the main thread is doing (e.g. a forward/
        backward pass on the previous batch).
        """
        try:
            while not self._stop_event.is_set():
                batch = self.fetch_next_batch(self.stream)

                if not batch:
                    self._queue.put(_StopSentinel())
                    return

                tensors = batch.contents.get_tensors()
                self.drop_sparse_batch(batch)

                # Blocks if the queue is full, i.e. we're already prefetch
                # batches ahead of the main thread — that's the desired
                # backpressure so we don't run unboundedly far ahead.
                self._queue.put(tensors)
        except Exception as e:
            # Surface the error on the main thread instead of dying silently
            # in the background.
            self._queue.put(e)

    def __next__(self):
        item = self._queue.get()

        if isinstance(item, _StopSentinel):
            raise StopIteration
        if isinstance(item, Exception):
            raise item

        return item

    def __del__(self):
        self._stop_prefetch_thread()
        self.delete_stream()

    def _stop_prefetch_thread(self):
        if self._stop_event is not None:
            self._stop_event.set()
        if self._thread is not None and self._thread.is_alive():
            # Drain one item in case the thread is blocked on a full queue
            # put(), so it can see the stop event and exit.
            try:
                self._queue.get_nowait()
            except (queue.Empty, AttributeError):
                pass
            self._thread.join(timeout=5)
        self._thread = None
        self._queue = None
        self._stop_event = None

    def delete_stream(self):
        if hasattr(self, 'stream') and self.stream:
            self.drop_sparse_batch_stream(self.stream)
            self.stream = None


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
