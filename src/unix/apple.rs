use rustix::fd::{AsRawFd, BorrowedFd};
use std::io;

static ZERO_BLOCK: [u8; 64 * 1024] = [0; 64 * 1024];

/// Reserves the requested logical range while preserving existing data and cursor state.
pub(super) fn allocate(fd: BorrowedFd<'_>, len: u64, logical_size: u64) -> io::Result<()> {
  let target_len =
    libc::off_t::try_from(len).map_err(|_| io::Error::from_raw_os_error(libc::EFBIG))?;
  let existing_range_end = logical_size.min(len);

  if existing_range_end > 0 {
    fill_sparse_holes(fd, existing_range_end)?;
  }

  if logical_size < len {
    reserve_from_physical_end(fd, len - logical_size)?;
    let result = unsafe { libc::ftruncate(fd.as_raw_fd(), target_len) };
    if result == -1 {
      return Err(io::Error::last_os_error());
    }
  }

  Ok(())
}

/// Reserves new extents atomically beyond the file's current physical end.
fn reserve_from_physical_end(fd: BorrowedFd<'_>, bytes: u64) -> io::Result<()> {
  if bytes == 0 {
    return Ok(());
  }
  let bytes =
    libc::off_t::try_from(bytes).map_err(|_| io::Error::from_raw_os_error(libc::EFBIG))?;
  let mut store = libc::fstore_t {
    fst_flags: libc::F_ALLOCATEALL,
    fst_posmode: libc::F_PEOFPOSMODE,
    fst_offset: 0,
    fst_length: bytes,
    fst_bytesalloc: 0,
  };
  let result = unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_PREALLOCATE, &mut store) };
  if result == -1 {
    return Err(io::Error::last_os_error());
  }
  if store.fst_bytesalloc < bytes {
    return Err(io::Error::other(format!(
      "F_ALLOCATEALL reserved only {} of {bytes} requested bytes",
      store.fst_bytesalloc
    )));
  }
  Ok(())
}

/// Materializes sparse holes and restores the caller's original file cursor.
fn fill_sparse_holes(fd: BorrowedFd<'_>, end: u64) -> io::Result<()> {
  let original_offset = current_offset(fd)?;
  let fill_result = fill_sparse_holes_inner(fd, end);
  let restore_result = set_offset(fd, original_offset);
  fill_result.and(restore_result)
}

/// Locates and writes every sparse hole in the target logical range.
fn fill_sparse_holes_inner(fd: BorrowedFd<'_>, end: u64) -> io::Result<()> {
  let mut cursor = 0_u64;
  while cursor < end {
    let Some(hole_start) = seek_region(fd, cursor, libc::SEEK_HOLE)? else {
      break;
    };
    if hole_start >= end {
      break;
    }
    let data_start = seek_region(fd, hole_start, libc::SEEK_DATA)?
      .unwrap_or(end)
      .min(end);
    write_zeros(fd, hole_start, data_start)?;
    cursor = data_start;
  }

  if seek_region(fd, 0, libc::SEEK_HOLE)?.is_some_and(|offset| offset < end) {
    return Err(io::Error::other(
      "Apple filesystem kept a sparse hole after zero-fill allocation",
    ));
  }
  Ok(())
}

/// Reads the shared file description's current cursor.
fn current_offset(fd: BorrowedFd<'_>) -> io::Result<u64> {
  let result = unsafe { libc::lseek(fd.as_raw_fd(), 0, libc::SEEK_CUR) };
  if result == -1 {
    Err(io::Error::last_os_error())
  } else {
    Ok(result as u64)
  }
}

/// Restores the shared file description's cursor.
fn set_offset(fd: BorrowedFd<'_>, offset: u64) -> io::Result<()> {
  let offset =
    libc::off_t::try_from(offset).map_err(|_| io::Error::from_raw_os_error(libc::EFBIG))?;
  let result = unsafe { libc::lseek(fd.as_raw_fd(), offset, libc::SEEK_SET) };
  if result == -1 {
    Err(io::Error::last_os_error())
  } else {
    Ok(())
  }
}

/// Finds the next data or hole boundary, treating `ENXIO` as end-of-range.
fn seek_region(fd: BorrowedFd<'_>, offset: u64, whence: libc::c_int) -> io::Result<Option<u64>> {
  let offset =
    libc::off_t::try_from(offset).map_err(|_| io::Error::from_raw_os_error(libc::EFBIG))?;
  let result = unsafe { libc::lseek(fd.as_raw_fd(), offset, whence) };
  if result >= 0 {
    return Ok(Some(result as u64));
  }
  let error = io::Error::last_os_error();
  if error.raw_os_error() == Some(libc::ENXIO) {
    Ok(None)
  } else {
    Err(error)
  }
}

/// Writes zeros into a logical hole without changing the shared file cursor.
fn write_zeros(fd: BorrowedFd<'_>, start: u64, end: u64) -> io::Result<()> {
  let mut offset = start;
  while offset < end {
    let remaining = end - offset;
    let chunk_len = usize::try_from(remaining.min(ZERO_BLOCK.len() as u64))
      .map_err(|_| io::Error::from_raw_os_error(libc::EFBIG))?;
    let file_offset =
      libc::off_t::try_from(offset).map_err(|_| io::Error::from_raw_os_error(libc::EFBIG))?;
    let written = unsafe {
      libc::pwrite(
        fd.as_raw_fd(),
        ZERO_BLOCK.as_ptr().cast(),
        chunk_len,
        file_offset,
      )
    };
    if written == -1 {
      let error = io::Error::last_os_error();
      if error.kind() == io::ErrorKind::Interrupted {
        continue;
      }
      return Err(error);
    }
    if written == 0 {
      return Err(io::Error::from(io::ErrorKind::WriteZero));
    }
    offset = offset
      .checked_add(written as u64)
      .ok_or_else(|| io::Error::from_raw_os_error(libc::EFBIG))?;
  }
  Ok(())
}
