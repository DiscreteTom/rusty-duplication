mod capturer;
mod duplicate_context;
mod manager;
mod utils;

use std::{fs::File, io::Write, thread, time::Duration};

use manager::Manager;

use crate::{capturer::model::Capturer, utils::Dimension};

fn main() -> std::io::Result<()> {
  let manager = Manager::default().unwrap();
  let mut capturer = manager.contexts[0]
    .shared_capturer("test".to_string())
    .unwrap();
  println!(
    "size: {}x{}",
    capturer.get_desc().width(),
    capturer.get_desc().height()
  );

  thread::sleep(Duration::from_millis(100));

  capturer.capture().unwrap();

  let mut file = File::create("capture.bin")?;
  file.write_all(capturer.get_buffer())?;
  Ok(())
}
