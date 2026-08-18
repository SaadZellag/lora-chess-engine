mod batch_provider;
mod utils;

use std::{ffi::CStr, ptr};

use libc::{c_char, size_t};

use crate::{batch_provider::SparseBatchStream, utils::SparseBatch};

#[unsafe(no_mangle)]
pub extern "C" fn create_sparse_batch_stream(
    file: *const c_char,
    batch_size: size_t,
) -> *const SparseBatchStream {
    fn create_stream(file: *const c_char, batch_size: size_t) -> Option<SparseBatchStream> {
        let file = unsafe { CStr::from_ptr(file).to_str().ok()? };
        SparseBatchStream::new(file, batch_size)
    }

    if let Some(stream) = create_stream(file, batch_size) {
        let stream = Box::new(stream);
        return Box::leak(stream) as *const SparseBatchStream;
    }
    ptr::null()
}

#[unsafe(no_mangle)]
pub extern "C" fn fetch_next_batch(stream: *mut SparseBatchStream) -> *const SparseBatch {
    if stream.is_null() {
        return ptr::null();
    }

    let stream = unsafe { stream.as_mut_unchecked() };

    if let Some(batch) = stream.next() {
        let batch = Box::new(batch);
        return Box::leak(batch) as *const SparseBatch;
    }
    ptr::null()
}

#[unsafe(no_mangle)]
pub extern "C" fn drop_sparse_batch_stream(stream: *mut SparseBatchStream) {
    if stream.is_null() {
        return;
    }
    let _ = unsafe { Box::from_raw(stream) };
}

#[unsafe(no_mangle)]
pub extern "C" fn drop_sparse_batch(batch: *mut SparseBatch) {
    if batch.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(batch);
    }
}
