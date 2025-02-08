use super::{capture, model::Capturer};
use crate::{Error, Monitor, OutDuplDescExt, Result};
use windows::Win32::Graphics::{
  Direct3D11::{ID3D11Texture2D, D3D11_TEXTURE2D_DESC},
  Dxgi::{DXGI_OUTDUPL_FRAME_INFO, DXGI_OUTDUPL_POINTER_SHAPE_INFO},
};

/// Capture screen to a chunk of memory.
pub struct CustomCapturer<'a> {
  buffer: &'a mut [u8],
  monitor: Monitor,
  texture: ID3D11Texture2D,
  texture_desc: D3D11_TEXTURE2D_DESC,
  pointer_shape_buffer: Vec<u8>,
  pointer_shape_buffer_size: usize,
}

impl<'a> CustomCapturer<'a> {
  pub fn with_texture(
    monitor: Monitor,
    buffer: &'a mut [u8],
    texture: ID3D11Texture2D,
    texture_desc: D3D11_TEXTURE2D_DESC,
  ) -> Self {
    Self {
      buffer,
      monitor,
      texture,
      texture_desc,
      pointer_shape_buffer: Vec::new(),
      pointer_shape_buffer_size: 0,
    }
  }

  pub fn new(ctx: Monitor, buffer: &'a mut [u8]) -> Result<Self> {
    let (texture, texture_desc) =
      ctx.create_texture(&ctx.dxgi_outdupl_desc(), &ctx.dxgi_output_desc()?)?;
    Ok(Self::with_texture(ctx, buffer, texture, texture_desc))
  }
}

impl Capturer for CustomCapturer<'_> {
  fn monitor(&self) -> &Monitor {
    &self.monitor
  }

  fn buffer(&self) -> &[u8] {
    self.buffer
  }

  fn buffer_mut(&mut self) -> &mut [u8] {
    self.buffer
  }

  fn check_buffer(&self) -> Result<()> {
    if self.buffer.len() < self.monitor.dxgi_outdupl_desc().calc_buffer_size() {
      Err(Error::InvalidBufferLength)
    } else {
      Ok(())
    }
  }

  fn pointer_shape_buffer(&self) -> &[u8] {
    &self.pointer_shape_buffer[..self.pointer_shape_buffer_size]
  }

  fn capture(&mut self, timeout_ms: u32) -> Result<DXGI_OUTDUPL_FRAME_INFO> {
    let frame_info = self.monitor.next_frame(timeout_ms, &self.texture)?;

    unsafe {
      capture(
        &self.texture,
        self.buffer.as_mut_ptr(),
        self.buffer.len(),
        &self.texture_desc,
      )?;
    }

    Ok(frame_info)
  }

  fn safe_capture(&mut self, timeout_ms: u32) -> Result<DXGI_OUTDUPL_FRAME_INFO> {
    self.check_buffer()?;
    self.capture(timeout_ms)
  }

  fn capture_with_pointer_shape(
    &mut self,
    timeout_ms: u32,
  ) -> Result<(
    DXGI_OUTDUPL_FRAME_INFO,
    Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>,
  )> {
    let (frame_info, pointer_shape_info) = self.monitor.next_frame_with_pointer_shape(
      timeout_ms,
      &self.texture,
      &mut self.pointer_shape_buffer,
    )?;

    unsafe {
      capture(
        &self.texture,
        self.buffer.as_mut_ptr(),
        self.buffer.len(),
        &self.texture_desc,
      )
    }?;

    if pointer_shape_info.is_some() {
      // record the pointer shape buffer size
      self.pointer_shape_buffer_size = frame_info.PointerShapeBufferSize as usize;
    }

    Ok((frame_info, pointer_shape_info))
  }

  fn safe_capture_with_pointer_shape(
    &mut self,
    timeout_ms: u32,
  ) -> Result<(
    DXGI_OUTDUPL_FRAME_INFO,
    Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>,
  )> {
    self.check_buffer()?;
    self.capture_with_pointer_shape(timeout_ms)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{capturer::model::Capturer, FrameInfoExt, OutDuplDescExt, Scanner};
  use serial_test::serial;
  use std::{thread, time::Duration};

  #[test]
  #[serial]
  fn custom_capturer() {
    let monitor = Scanner::new().unwrap().next().unwrap();
    let desc = monitor.dxgi_outdupl_desc();
    let mut buffer = vec![0u8; desc.calc_buffer_size()];
    let mut capturer = CustomCapturer::new(monitor, &mut buffer).unwrap();

    // sleep for a while before capture to wait system to update the screen
    thread::sleep(Duration::from_millis(100));

    let info = capturer.safe_capture(300).unwrap();
    assert!(info.desktop_updated());

    let buffer = capturer.buffer();
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
    let (frame_info, pointer_shape_info) = capturer.safe_capture_with_pointer_shape(300).unwrap();
    assert!(frame_info.mouse_updated());
    assert!(pointer_shape_info.is_some());
    let pointer_shape_data = capturer.pointer_shape_buffer();
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
