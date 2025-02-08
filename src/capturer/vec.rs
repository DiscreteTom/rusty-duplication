use super::CapturerBuffer;
use crate::{Capturer, Error, Monitor, Result};

impl CapturerBuffer for Vec<u8> {
  #[inline]
  fn as_bytes(&self) -> &[u8] {
    self
  }

  #[inline]
  fn as_bytes_mut(&mut self) -> &mut [u8] {
    self
  }
}

/// Capture screen to a `Vec<u8>`.
/// # Examples
/// ```
/// use rusty_duplication::{Scanner, VecCapturer};
///
/// let monitor = Scanner::new().unwrap().next().unwrap();
/// let mut capturer: VecCapturer = monitor.try_into().unwrap();
/// ```
pub type VecCapturer = Capturer<Vec<u8>>;

impl TryFrom<Monitor> for VecCapturer {
  type Error = Error;

  fn try_from(monitor: Monitor) -> Result<Self> {
    Capturer::new(monitor, |size| Ok(vec![0u8; size]))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{FrameInfoExt, Scanner};
  use serial_test::serial;
  use std::{thread, time::Duration};

  #[test]
  #[serial]
  fn vec_capturer() {
    let monitor = Scanner::new().unwrap().next().unwrap();
    let mut capturer: VecCapturer = monitor.try_into().unwrap();

    // sleep for a while before capture to wait system to update the screen
    thread::sleep(Duration::from_millis(50));

    let info = capturer.capture(300).unwrap();
    assert!(info.desktop_updated());

    // ensure buffer not all zero
    assert!(!capturer.buffer.as_bytes().iter().all(|&n| n == 0));

    thread::sleep(Duration::from_millis(50));

    // check mouse
    let (frame_info, pointer_shape_info) = capturer.capture_with_pointer_shape(300).unwrap();
    assert!(frame_info.mouse_updated());
    assert!(pointer_shape_info.is_some());
    // make sure pointer shape buffer is not all zero
    assert!(!capturer.pointer_shape_buffer.iter().all(|&n| n == 0));
  }
}
