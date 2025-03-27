use nix::errno::Errno;
use nix::fcntl::OFlag;
use nix::sys::mman::{mmap, shm_open, MapFlags, ProtFlags};
use nix::sys::stat::Mode;
use nix::unistd::ftruncate;
use std::path::{Path, PathBuf};

use super::runtime::RuntimeError;

pub struct MmapHandler {
    name: PathBuf,                              // shared memory name
    ptr: core::ptr::NonNull<core::ffi::c_void>, // pointer to shared memory
    len: core::num::NonZeroUsize,               // 0 is not allowed.
}

// The developer has determined that this struct is safe to use across multiple threads
unsafe impl Send for MmapHandler {}
unsafe impl Sync for MmapHandler {}

// Convert nix::errno::Errno to std::io::Error
#[inline]
fn _e2e(e: Errno) -> std::io::Error {
    std::io::Error::from_raw_os_error(e as core::ffi::c_int)
}

impl MmapHandler {
    // ref: https://stackoverflow.com/questions/62320764/how-to-create-shared-memory-after-fork
    pub fn new(name: impl AsRef<Path>) -> Result<Self, RuntimeError> {
        // We assume that data is sufficiently small.
        let length = core::num::NonZero::new(10000).unwrap();
        // Open without creation
        let fd = shm_open(name.as_ref(), OFlag::O_RDWR, Mode::S_IRUSR | Mode::S_IWUSR);
        let fd = match fd {
            Ok(fd) => fd,
            Err(Errno::ENOENT) => {
                let fd = shm_open(
                    name.as_ref(),
                    OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_RDWR,
                    Mode::S_IRUSR | Mode::S_IWUSR,
                )
                .map_err(_e2e)
                .map_err(RuntimeError::FileOpen)?;
                // We must truncate size of shared memory at the time of initial creation.
                ftruncate(&fd, Into::<usize>::into(length) as i64)
                    .map_err(_e2e)
                    .map_err(RuntimeError::FileOpen)?;
                fd
            }
            Err(err) => return Err(RuntimeError::FileOpen(_e2e(err))),
        };
        let ptr = unsafe {
            mmap(
                None,
                length,
                ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                MapFlags::MAP_NORESERVE | MapFlags::MAP_SHARED,
                fd,
                0,
            )
            .map_err(_e2e)
            .map_err(RuntimeError::FileOpen)?
        };
        Ok(MmapHandler {
            ptr,
            len: length,
            name: name.as_ref().to_path_buf(),
        })
    }
}
