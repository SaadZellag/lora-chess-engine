mod batch_provider;
mod utils;

use std::{ffi::CStr, ptr};

use libc::{c_char, size_t};

use crate::{batch_provider::SparseBatchStream, utils::SparseBatch};

#[no_mangle]
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

#[no_mangle]
pub extern "C" fn fetch_next_batch(stream: *mut SparseBatchStream) -> *const SparseBatch {
    if stream.is_null() {
        return ptr::null();
    }

    let mut stream = unsafe { Box::from_raw(stream) };

    if let Some(batch) = stream.next() {
        Box::leak(stream); // Don't let the borrow checker drop it
        let batch = Box::new(batch);
        return Box::leak(batch) as *const SparseBatch;
    }
    Box::leak(stream); // Don't let the borrow checker drop it
    ptr::null()
}

#[no_mangle]
pub extern "C" fn stream_len(stream: *mut SparseBatchStream) -> size_t {
    if stream.is_null() {
        return 0;
    }

    let stream = unsafe { Box::from_raw(stream) };
    let len = stream.len();
    Box::leak(stream); // Don't let the borrow checker drop it
    len
}

#[no_mangle]
pub extern "C" fn drop_sparse_batch_stream(stream: *mut SparseBatchStream) {
    if stream.is_null() {
        return;
    }
    let _ = unsafe { Box::from_raw(stream) };
}

#[no_mangle]
pub extern "C" fn drop_sparse_batch(batch: *mut SparseBatch) {
    if batch.is_null() {
        return;
    }
    unsafe {
        SparseBatch::drop_batch(batch);
    }
}
