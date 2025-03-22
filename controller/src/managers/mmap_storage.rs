use std::path::PathBuf;

pub struct MmapHandler {
    name: PathBuf,
    ptr: core::ptr::NonNull<core::ffi::c_void>,
    len: core::num::NonZeroUsize,
}
