#![cfg(target_vendor = "apple")]

use aster_fs::AsyncFileExt;
use std::io::{Seek, SeekFrom, Write};

const SPARSE_FILE_BYTES: u64 = 128 * 1024 * 1024;

#[tokio::test(flavor = "current_thread")]
async fn sparse_async_allocation_materializes_holes() {
  let directory = tempfile::TempDir::with_prefix("aster-fs-apple-async").unwrap();
  let path = directory.path().join("reservation.bin");
  let mut source = std::fs::OpenOptions::new()
    .read(true)
    .write(true)
    .create_new(true)
    .open(&path)
    .unwrap();
  source.set_len(SPARSE_FILE_BYTES).unwrap();
  source
    .seek(SeekFrom::Start(SPARSE_FILE_BYTES - 4096))
    .unwrap();
  source.write_all(&[0x5a; 4096]).unwrap();
  source.sync_all().unwrap();
  drop(source);

  let file = tokio::fs::OpenOptions::new()
    .read(true)
    .write(true)
    .open(path)
    .await
    .unwrap();
  file.allocate(SPARSE_FILE_BYTES).await.unwrap();

  assert!(
    file.allocated_size().await.unwrap() >= SPARSE_FILE_BYTES,
    "async allocation must materialize every sparse hole",
  );
}
