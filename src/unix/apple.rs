use rustix::fd::{AsRawFd, BorrowedFd};
use std::io;

pub(super) fn allocate(fd: BorrowedFd<'_>, len: u64, allocated_size: u64) -> io::Result<()> {
  let target_len =
    libc::off_t::try_from(len).map_err(|_| io::Error::from_raw_os_error(libc::EFBIG))?;
  let missing = len.saturating_sub(allocated_size.min(len));

  if missing > 0 {
    let missing =
      libc::off_t::try_from(missing).map_err(|_| io::Error::from_raw_os_error(libc::EFBIG))?;
    let mut store = libc::fstore_t {
      fst_flags: libc::F_ALLOCATEALL,
      fst_posmode: libc::F_PEOFPOSMODE,
      fst_offset: 0,
      fst_length: missing,
      fst_bytesalloc: 0,
    };
    let result = unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_PREALLOCATE, &mut store) };
    if result == -1 {
      return Err(io::Error::last_os_error());
    }
    if store.fst_bytesalloc < missing {
      return Err(io::Error::other(format!(
        "F_ALLOCATEALL reserved only {} of {missing} requested bytes",
        store.fst_bytesalloc
      )));
    }
  }

  let result = unsafe { libc::ftruncate(fd.as_raw_fd(), target_len) };
  if result == -1 {
    Err(io::Error::last_os_error())
  } else {
    Ok(())
  }
}
