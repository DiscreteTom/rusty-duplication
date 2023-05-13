mod capturer;
mod duplicate_context;
mod manager;
mod utils;

use capturer::model::Capturer;
use manager::Manager;
use std::{fs::File, io::Write, thread, time::Duration};
use utils::OutputDescExt;

use crate::utils::FrameInfoExt;

fn main() {
  // manager will collect monitor info when created
  let manager = Manager::default().unwrap();
  // you can also refresh monitor info manually
  // manager.refresh();

  // get monitor info before capturing start
  for ctx in &manager.contexts {
    ctx.get_desc().unwrap();
  }

  // create capturer for a display
  // this will allocate memory buffer to store pixel data
  let mut capturer = manager.contexts[0].simple_capturer().unwrap();

  // you can also get monitor info from a capturer
  let desc = capturer.get_desc().unwrap();
  // we have some extension methods for you such as `width/height`
  println!("size: {}x{}", desc.width(), desc.height());

  // sleep for a while before capture to wait system update
  thread::sleep(Duration::from_millis(100));

  // capture desktop image and get the frame info
  // `safe_capture` will check if the buffer's size is enough
  let info = capturer.safe_capture().unwrap();

  // check if this is a new frame using the extension method `is_new_frame`
  if info.is_new_frame() {
    println!("captured!");
  }

  // write to a file
  let mut file = File::create("capture.bin").unwrap();
  // `get_buffer` will return `&[u8]` in BGRA32 format
  file.write_all(capturer.get_buffer()).unwrap();
}
