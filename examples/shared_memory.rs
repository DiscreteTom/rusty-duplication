use rusty_duplication::{CapturerBuffer, FrameInfoExt, Scanner, SharedMemoryCapturer};
use std::{fs::File, io::Write, thread, time::Duration};

fn main() {
  let monitor = Scanner::new().unwrap().next().unwrap();

  // create a shared memory capturer by creating a shared memory with the provided name
  let mut capturer = SharedMemoryCapturer::create(monitor, "SharedMemoryName").unwrap();
  // you can also use `SharedMemoryCapturer::open` to open an existing shared memory

  // sleep for a while before capture to wait system to update the screen
  thread::sleep(Duration::from_millis(50));

  let info = capturer.capture().unwrap();
  assert!(info.desktop_updated());

  // write to a file
  let mut file = File::create("capture.bin").unwrap();
  // the buffer is in BGRA32 format
  file.write_all(capturer.buffer.as_bytes()).unwrap();
}
