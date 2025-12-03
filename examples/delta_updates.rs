use rusty_duplication::{FrameInfoExt, Scanner, VecCapturer};
use std::{thread, time::Duration};

fn main() {
  // create a scanner to scan for monitors
  let mut scanner = Scanner::new().unwrap();

  // get the first monitor
  let monitor = scanner.next().unwrap();

  // create a vec capturer for a monitor
  let mut capturer: VecCapturer = monitor.try_into().unwrap();

  println!("Delta updates enabled by default: {}", capturer.use_delta_updates);

  // First capture - always does a full copy
  println!("Capturing first frame (full copy)...");
  thread::sleep(Duration::from_millis(100));
  let info = capturer.capture().unwrap();
  println!("First frame captured, desktop updated: {}", info.desktop_updated());

  // Subsequent captures - uses delta updates (only dirty/moved regions)
  for i in 1..=5 {
    thread::sleep(Duration::from_millis(100));
    println!("\nCapturing frame {} (delta updates)...", i + 1);
    let info = capturer.capture().unwrap();
    println!("Frame {} captured, desktop updated: {}", i + 1, info.desktop_updated());
    if info.mouse_updated() {
      println!("Mouse position updated");
    }
  }

  // You can disable delta updates to always do a full copy
  println!("\n\nDisabling delta updates...");
  capturer.use_delta_updates = false;
  
  for i in 1..=3 {
    thread::sleep(Duration::from_millis(100));
    println!("\nCapturing frame (full copy)...");
    let info = capturer.capture().unwrap();
    println!("Frame captured, desktop updated: {}", info.desktop_updated());
  }

  println!("\n\nDelta updates demonstration complete!");
  println!("With delta updates, only changed regions are copied from GPU to CPU memory,");
  println!("significantly improving performance for applications that capture at high frame rates.");
}
