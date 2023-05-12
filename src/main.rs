mod capturer;
mod duplicate_context;
mod manager;
mod utils;

use std::{fs::File, io::Write, thread, time::Duration};

use manager::Manager;

use crate::utils::Dimension;

fn main() -> std::io::Result<()> {
  let manager = Manager::default().unwrap();
  let mut capturer = manager.dup_ctxs[0].simple_capturer();
  println!("size: {}x{}", capturer.desc.width(), capturer.desc.height());

  thread::sleep(Duration::from_millis(100));

  capturer.capture();

  let mut file = File::create("capture.bin")?;
  file.write_all(&capturer.buffer)?;
  Ok(())
}
