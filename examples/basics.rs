use rusty_duplication::{FrameInfoExt, Scanner, VecCapturer};
use std::{fs::File, io::Write, thread, time::Duration};

fn main() {
  // create a scanner to scan for monitors
  let mut scanner = Scanner::new().unwrap();

  // scanner implements Iterator, you can use it to iterate through monitors
  let monitor = scanner.next().unwrap();

  // get monitor info
  monitor.dxgi_output_desc().unwrap();
  monitor.dxgi_outdupl_desc();

  // create a vec capturer for a monitor
  // this will allocate memory buffer to store pixel data
  let mut capturer: VecCapturer = monitor.try_into().unwrap();

  // you can also get monitor info from a capturer
  let dxgi_outdupl_desc = capturer.monitor().dxgi_outdupl_desc();
  let dxgi_output_desc = capturer.monitor().dxgi_output_desc().unwrap();
  // get resolution width/height
  println!(
    "size: {}x{}",
    dxgi_outdupl_desc.ModeDesc.Width, dxgi_outdupl_desc.ModeDesc.Height
  );
  // get position
  println!(
    "left: {}, top: {}, right: {}, bottom: {}",
    dxgi_output_desc.DesktopCoordinates.left,
    dxgi_output_desc.DesktopCoordinates.top,
    dxgi_output_desc.DesktopCoordinates.right,
    dxgi_output_desc.DesktopCoordinates.bottom
  );

  // sleep for a while before capture to wait system to update the screen
  thread::sleep(Duration::from_millis(100));

  // capture desktop image and get the frame info
  let info = capturer.capture().unwrap();

  // we have some extension methods for the frame info
  if info.desktop_updated() {
    println!("captured!");
  }
  if info.mouse_updated() {
    println!("mouse updated!");
  }
  if info.pointer_shape_updated() {
    println!("pointer shape updated!");
  }

  // write to a file
  let mut file = File::create("capture.bin").unwrap();
  // the buffer is in BGRA32 format
  file.write_all(&capturer.buffer).unwrap();
}
