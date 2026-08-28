import ctypes
from ctypes import c_char_p, c_size_t, c_void_p, CDLL, POINTER, c_int, c_float, Structure
from pathlib import Path
from typing import Optional, TYPE_CHECKING


class SparseBatchStreamProvider(Structure):
    """Empty structure for SparseBatchStreamProvider (opaque pointer)."""
    pass


class SparseBatchStream(Structure):
    """Empty structure for SparseBatchStream (opaque pointer)."""
    pass


class SparseBatch(Structure):
    """Structure representing a sparse batch from the Rust DLL."""
    _fields_ = [
        ('size', c_int),
        ('num_active_our_features', c_int),
        ('num_active_their_features', c_int),
        ('final_score', POINTER(c_float)),
        ('mom_value', POINTER(c_int)),
        ('eval', POINTER(c_int)),
        ('our_feature_indices', POINTER(c_int)),
        ('their_feature_indices', POINTER(c_int)),
    ]


# Type aliases for pointer types
if TYPE_CHECKING:
    SparseBatchStreamProviderPtr = ctypes._Pointer[SparseBatchStreamProvider]
    SparseBatchStreamPtr = ctypes._Pointer[SparseBatchStream]
    SparseBatchPtr = ctypes._Pointer[SparseBatch]
else:
    SparseBatchStreamProviderPtr = ctypes.POINTER(SparseBatchStreamProvider)
    SparseBatchStreamPtr = ctypes.POINTER(SparseBatchStream)
    SparseBatchPtr = ctypes.POINTER(SparseBatch)


class DLLLoader:
    """Loader for the Rust DLL providing sparse batch stream functionality."""
    
    def __init__(self, dll_path: str | Path) -> None:
        """Initialize the DLL loader.
        
        Args:
            dll_path: Path to the DLL file.
        """
        self.dll = CDLL(str(dll_path))
        self._setup_functions()
    
    def _setup_functions(self) -> None:
        """Setup function signatures with proper type hints."""
        # create_sparse_batch_stream_provider
        self.dll.create_sparse_batch_stream_provider.argtypes = [
            c_char_p, c_size_t, c_size_t, c_size_t
        ]
        self.dll.create_sparse_batch_stream_provider.restype = SparseBatchStreamProviderPtr
        
        # get_sparse_batch_stream
        self.dll.get_sparse_batch_stream.argtypes = [SparseBatchStreamProviderPtr, c_size_t, c_size_t]
        self.dll.get_sparse_batch_stream.restype = SparseBatchStreamPtr
        
        # fetch_next_batch
        self.dll.fetch_next_batch.argtypes = [SparseBatchStreamPtr]
        self.dll.fetch_next_batch.restype = SparseBatchPtr
        
        # drop_sparse_batch_stream_provider
        self.dll.drop_sparse_batch_stream_provider.argtypes = [SparseBatchStreamProviderPtr]
        self.dll.drop_sparse_batch_stream_provider.restype = None
        
        # drop_sparse_batch_stream
        self.dll.drop_sparse_batch_stream.argtypes = [SparseBatchStreamPtr]
        self.dll.drop_sparse_batch_stream.restype = None

        # drop_sparse_batch
        self.dll.drop_sparse_batch.argtypes = [SparseBatchPtr]
        self.dll.drop_sparse_batch.restype = None
    
    def create_sparse_batch_stream_provider(
        self,
        file: str,
        batch_size: int,
        num_workers: int,
        step_size: int
    ) -> Optional[SparseBatchStreamProviderPtr]:
        """Create a sparse batch stream provider.
        
        Args:
            file: Path to the data file.
            batch_size: Size of each batch.
            num_workers: Number of worker threads.
            step_size: Step size for streaming.
        
        Returns:
            Pointer to SparseBatchStreamProvider or None if creation failed.
        """
        ptr = self.dll.create_sparse_batch_stream_provider(
            file.encode('utf-8'),
            c_size_t(batch_size),
            c_size_t(num_workers),
            c_size_t(step_size)
        )
        return ptr if ptr else None
    
    def get_sparse_batch_stream(
        self,
        provider: SparseBatchStreamProviderPtr,
        worker_id: int,
        current_epoch: int
    ) -> Optional[SparseBatchStreamPtr]:
        """Get a sparse batch stream from a provider.
        
        Args:
            provider: Pointer to SparseBatchStreamProvider.
            worker_id: ID of the worker.
            current_epoch: Current epoch number.
        
        Returns:
            Pointer to SparseBatchStream or None if retrieval failed.
        """
        ptr = self.dll.get_sparse_batch_stream(
            provider,
            c_size_t(worker_id),
            c_size_t(current_epoch)
        )
        return ptr if ptr else None
    
    def fetch_next_batch(self, stream: SparseBatchStreamPtr) -> Optional[SparseBatchPtr]:
        """Fetch the next batch from a stream.
        
        Args:
            stream: Pointer to SparseBatchStream.
        
        Returns:
            Pointer to SparseBatch or None if no more batches.
        """
        ptr = self.dll.fetch_next_batch(stream)
        return ptr if ptr else None
    
    def drop_sparse_batch_stream_provider(self, provider: SparseBatchStreamProviderPtr) -> None:
        """Free a sparse batch stream provider.
        
        Args:
            provider: Pointer to SparseBatchStreamProvider to free.
        """
        self.dll.drop_sparse_batch_stream_provider(provider)
    
    def drop_sparse_batch_stream(self, stream: SparseBatchStreamPtr) -> None:
        """Free a sparse batch stream.
        
        Args:
            stream: Pointer to SparseBatchStream to free.
        """
        self.dll.drop_sparse_batch_stream(stream)
    
    def drop_sparse_batch(self, batch: SparseBatchPtr) -> None:
        """Free a sparse batch.
        
        Args:
            batch: Pointer to SparseBatch instance to free.
        """
        self.dll.drop_sparse_batch(batch)
