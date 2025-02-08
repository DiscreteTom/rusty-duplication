use super::CapturerBuffer;
use crate::{Capturer, Monitor, OutDuplDescExt, Result};
use windows::Win32::Graphics::Direct3D11::{ID3D11Texture2D, D3D11_TEXTURE2D_DESC};

impl CapturerBuffer for Vec<u8> {
  fn as_bytes(&self) -> &[u8] {
    self
  }

  fn as_bytes_mut(&mut self) -> &mut [u8] {
    self
  }
}

/// Capture screen to a `Vec<u8>`.
pub type VecCapturer = Capturer<Vec<u8>>;

impl VecCapturer {
  pub fn new(monitor: Monitor) -> Result<Self> {
    let (buffer, texture, texture_desc) = Self::allocate(&monitor)?;
    Ok(Self {
      buffer,
      monitor,
      texture,
      texture_desc,
      pointer_shape_buffer: Vec::new(),
    })
  }

  fn allocate(monitor: &Monitor) -> Result<(Vec<u8>, ID3D11Texture2D, D3D11_TEXTURE2D_DESC)> {
    let dupl_desc = monitor.dxgi_outdupl_desc();
    let (texture, texture_desc) =
      monitor.create_texture(&dupl_desc, &monitor.dxgi_output_desc()?)?;
    let buffer = vec![0u8; dupl_desc.calc_buffer_size()];
    Ok((buffer, texture, texture_desc))
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
  fn simple_capturer() {
    let monitor = Scanner::new().unwrap().next().unwrap();
    let mut capturer = VecCapturer::new(monitor).unwrap();

    // sleep for a while before capture to wait system to update the screen
    thread::sleep(Duration::from_millis(100));

    let info = capturer.capture(300).unwrap();
    assert!(info.desktop_updated());

    let buffer = capturer.buffer.as_bytes();
    // ensure buffer not all zero
    let mut all_zero = true;
    for i in 0..buffer.len() {
      if buffer[i] != 0 {
        all_zero = false;
        break;
      }
    }
    assert!(!all_zero);

    // sleep for a while before capture to wait system to update the mouse
    thread::sleep(Duration::from_millis(1000));

    // check pointer shape
    let (frame_info, pointer_shape_info) = capturer.capture_with_pointer_shape(300).unwrap();
    assert!(frame_info.mouse_updated());
    assert!(pointer_shape_info.is_some());
    let pointer_shape_data = capturer.pointer_shape_buffer;
    // make sure pointer shape buffer is not all zero
    let mut all_zero = true;
    for i in 0..pointer_shape_data.len() {
      if pointer_shape_data[i] != 0 {
        all_zero = false;
        break;
      }
    }
    assert!(!all_zero);
  }
}
