use image::{ImageBuffer, RgbaImage};
use rusty_duplication::{Scanner, VecCapturer};
use std::{thread, time::Duration};

fn main() {
  let monitor = Scanner::new().unwrap().next().unwrap();
  let dupl_desc = monitor.dxgi_outdupl_desc();
  let width = dupl_desc.ModeDesc.Width;
  let height = dupl_desc.ModeDesc.Height;

  let mut capturer: VecCapturer = monitor.try_into().unwrap();

  // sleep for a while before capture to wait system to update the screen
  thread::sleep(Duration::from_millis(100));
  capturer.capture().unwrap();

  // convert BGRA32 to RGBA32
  let mut buffer = Vec::with_capacity(capturer.buffer.len());
  for i in (0..capturer.buffer.len()).step_by(4) {
    buffer.push(capturer.buffer[i + 2]);
    buffer.push(capturer.buffer[i + 1]);
    buffer.push(capturer.buffer[i]);
    buffer.push(capturer.buffer[i + 3]);
  }

  let img: RgbaImage = ImageBuffer::from_raw(width, height, buffer).unwrap();

  img.save("desktop.png").unwrap();
}
