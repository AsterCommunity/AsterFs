#![cfg(target_vendor = "apple")]

use aster_fs::{available_space, FileExt};
use std::fs::OpenOptions;

const LARGE_RESERVATION_BYTES: u64 = 128 * 1024 * 1024;

#[test]
fn large_reservation_is_fully_allocated() {
  let directory = tempfile::TempDir::with_prefix("aster-fs-apple-allocation").unwrap();
  let path = directory.path().join("reservation.bin");
  let file = OpenOptions::new()
    .read(true)
    .write(true)
    .create_new(true)
    .open(path)
    .unwrap();

  FileExt::allocate(&file, LARGE_RESERVATION_BYTES).unwrap();

  assert_eq!(file.metadata().unwrap().len(), LARGE_RESERVATION_BYTES);
  assert!(
    FileExt::allocated_size(&file).unwrap() >= LARGE_RESERVATION_BYTES,
    "Apple allocation must reserve every requested byte",
  );
}

#[test]
fn sparse_existing_file_recovers_missing_physical_allocation() {
  let directory = tempfile::TempDir::with_prefix("aster-fs-apple-recovery").unwrap();
  let path = directory.path().join("reservation.bin");
  let mut file = OpenOptions::new()
    .read(true)
    .write(true)
    .create_new(true)
    .open(path)
    .unwrap();
  file.set_len(LARGE_RESERVATION_BYTES).unwrap();
  std::io::Write::write_all(&mut file, &[0_u8; 4096]).unwrap();
  file.sync_all().unwrap();
  let partial_allocation = FileExt::allocated_size(&file).unwrap();
  assert!(partial_allocation < LARGE_RESERVATION_BYTES);

  FileExt::allocate(&file, LARGE_RESERVATION_BYTES).unwrap();

  assert_eq!(file.metadata().unwrap().len(), LARGE_RESERVATION_BYTES);
  assert!(
    FileExt::allocated_size(&file).unwrap() >= LARGE_RESERVATION_BYTES,
    "Apple allocation must fill the missing physical reservation",
  );
}

#[test]
fn insufficient_space_is_all_or_nothing_on_limited_volume() {
  let Some(root) = std::env::var_os("ASTER_FS_LIMITED_TEST_ROOT") else {
    eprintln!("ASTER_FS_LIMITED_TEST_ROOT is unset; limited-volume assertion skipped");
    return;
  };
  let directory = tempfile::TempDir::with_prefix_in("aster-fs-apple-enospc", &root).unwrap();
  let path = directory.path().join("reservation.bin");
  let file = OpenOptions::new()
    .read(true)
    .write(true)
    .create_new(true)
    .open(path)
    .unwrap();
  let available = available_space(&root).unwrap();
  let requested = available.checked_add(64 * 1024 * 1024).unwrap();

  let error = FileExt::allocate(&file, requested).unwrap_err();

  assert_eq!(error.raw_os_error(), Some(libc::ENOSPC));
  assert_eq!(file.metadata().unwrap().len(), 0);
  assert_eq!(FileExt::allocated_size(&file).unwrap(), 0);
}
