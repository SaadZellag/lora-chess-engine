mod stream_provider;
mod batch;
mod stream;

use std::{ffi::CStr, ptr};

use libc::{c_char, size_t};

use crate::{batch::SparseBatch, stream::SparseBatchStream, stream_provider::SparseBatchStreamProvider};

#[unsafe(no_mangle)]
pub extern "C" fn create_sparse_batch_stream_provider(
    file: *const c_char,
    batch_size: size_t, 
    num_workers: size_t, 
    step_size: size_t
) -> *const SparseBatchStreamProvider {

    unsafe { CStr::from_ptr(file) }.to_str().ok()
        .map(|file| SparseBatchStreamProvider::new(file, batch_size, num_workers, step_size))
        .flatten()
        .map(to_const_ptr)
        .unwrap_or(ptr::null())

}

#[unsafe(no_mangle)]

pub extern "C" fn get_sparse_batch_stream(
    provider: *const SparseBatchStreamProvider,
    worker_id: size_t,
    current_epoch: size_t
) -> *const SparseBatchStream {

    unsafe { provider.as_ref() }
        .map(|provider| provider.get_stream(worker_id, current_epoch))
        .map(to_const_ptr)
        .unwrap_or(ptr::null())
}

#[unsafe(no_mangle)]
pub extern "C" fn fetch_next_batch(stream: *const SparseBatchStream) -> *const SparseBatch {

    unsafe { (stream as *mut SparseBatchStream).as_mut() }
        .map(|stream| stream.next())
        .flatten()
        .map(to_const_ptr)
        .unwrap_or(ptr::null())

}

#[unsafe(no_mangle)]
pub extern "C" fn drop_sparse_batch_stream_provider(stream: *mut SparseBatchStreamProvider) {
    if stream.is_null() {
        return;
    }
    let _ = unsafe { Box::from_raw(stream) };
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


fn to_const_ptr<T>(t: T) -> *const T {
    Box::into_raw(Box::new(t)) as *const T
}