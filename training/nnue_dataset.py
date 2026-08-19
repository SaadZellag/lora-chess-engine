import time
import numpy as np
import ctypes
import torch
import os
import sys
import glob
from torch.utils.data import Dataset
from consts import *

local_dllpath = [n for n in glob.glob(
    '../target/release/*libdataloader.*') if n.endswith('.so') or n.endswith('.dll') or n.endswith('.dylib')]
if not local_dllpath:
    print('Cannot find data_loader shared library.')
    sys.exit(1)
dllpath = os.path.abspath(local_dllpath[0])
dll = ctypes.cdll.LoadLibrary(dllpath)


class SparseBatch(ctypes.Structure):
    _fields_ = [
        ('size', ctypes.c_int),
        ('num_active_our_features', ctypes.c_int),
        ('num_active_their_features', ctypes.c_int),

        ('final_score', ctypes.POINTER(ctypes.c_float)),
        ('ply', ctypes.POINTER(ctypes.c_int)),
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

        final_eval_t = torch.sigmoid(
            eval_t / WEIGHT_SCALE / ACTIVATION_RANGE * 8) * LAMBDA + score_t * (1 - LAMBDA)

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

create_sparse_batch_stream = dll.create_sparse_batch_stream
create_sparse_batch_stream.argtypes = [ctypes.c_char_p, ctypes.c_int]
create_sparse_batch_stream.restype = ctypes.c_void_p

fetch_next_batch = dll.fetch_next_batch
fetch_next_batch.argtypes = [ctypes.c_void_p]
fetch_next_batch.restype = SparseBatchPtr

drop_sparse_batch_stream = dll.drop_sparse_batch_stream
drop_sparse_batch_stream.argtypes = [ctypes.c_void_p]

drop_sparse_batch = dll.drop_sparse_batch
drop_sparse_batch.argtypes = [ctypes.c_void_p]


class SparseBatchDataset(torch.utils.data.IterableDataset):
    def __init__(self, filename, batch_size):
        self.stream = create_sparse_batch_stream(filename, batch_size)

    def __iter__(self):
        return self

    def __next__(self):
        batch = fetch_next_batch(self.stream)

        if batch:
            tensors = batch.contents.get_tensors()
            drop_sparse_batch(batch)
            return tensors
        else:
            raise StopIteration

    def __del__(self):
        drop_sparse_batch_stream(self.stream)


if __name__ == '__main__':
    provider = SparseBatchDataset(b"../games/training_data.bin", 8192)

    start = time.time()
    total = 0
    for batch in provider:
        total += 1
        if total == 2500:
            break

    end = time.time()

    print('total', total)
    print('time', end-start)

    # import torch

    # # LITTERALLY  COORDINATES
    # features = [[0, 1], [0, 2], [0, 5], [1, 2]]

    # i = torch.transpose(torch.tensor(features), 0, 1)
    # v = torch.ones(len(features), dtype=torch.int64)
    # k = torch.sparse_coo_tensor(i, v)

    # print(k)
    # print(k.to_dense())
