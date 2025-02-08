use image::{ImageBuffer, RgbaImage};
use rusty_duplication::{capturer::model::Capturer, Scanner};
use std::{thread, time::Duration};

fn main() {
  let monitor = Scanner::new().unwrap().next().unwrap();
  let mut capturer = monitor.simple_capturer().unwrap();

  // sleep for a while before capture to wait system to update the screen
  thread::sleep(Duration::from_millis(100));
  capturer.safe_capture(300).unwrap();

  // convert BGRA32 to RGBA32
  let mut buffer = Vec::with_capacity(capturer.buffer().len());
  for i in (0..capturer.buffer().len()).step_by(4) {
    buffer.push(capturer.buffer()[i + 2]);
    buffer.push(capturer.buffer()[i + 1]);
    buffer.push(capturer.buffer()[i]);
    buffer.push(capturer.buffer()[i + 3]);
  }

  let dimension = monitor.dxgi_outdupl_desc();
  let width = dimension.ModeDesc.Width;
  let height = dimension.ModeDesc.Height;

  let img: RgbaImage = ImageBuffer::from_raw(width, height, buffer).unwrap();

  img.save("desktop.png").unwrap();
}
