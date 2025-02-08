use super::model::Capturer;
use crate::duplication_context::DuplicationContext;
use crate::utils::OutDuplDescExt;
use crate::Error;
use crate::Result;
use windows::Win32::Graphics::Direct3D11::D3D11_TEXTURE2D_DESC;
use windows::Win32::Graphics::Dxgi::{
  DXGI_OUTDUPL_DESC, DXGI_OUTDUPL_FRAME_INFO, DXGI_OUTDUPL_POINTER_SHAPE_INFO,
};
use windows::Win32::Graphics::{Direct3D11::ID3D11Texture2D, Dxgi::DXGI_OUTPUT_DESC};

/// Capture screen to a `Vec<u8>`.
pub struct SimpleCapturer<'a> {
  buffer: Vec<u8>,
  ctx: &'a DuplicationContext,
  texture: ID3D11Texture2D,
  texture_desc: D3D11_TEXTURE2D_DESC,
  pointer_shape_buffer: Vec<u8>,
  pointer_shape_buffer_size: usize,
}

impl<'a> SimpleCapturer<'a> {
  pub fn new(ctx: &'a DuplicationContext) -> Result<Self> {
    let (buffer, texture, texture_desc) = Self::allocate(ctx)?;
    Ok(Self {
      buffer,
      ctx,
      texture,
      texture_desc,
      pointer_shape_buffer: Vec::new(),
      pointer_shape_buffer_size: 0,
    })
  }

  fn allocate(
    ctx: &'a DuplicationContext,
  ) -> Result<(Vec<u8>, ID3D11Texture2D, D3D11_TEXTURE2D_DESC)> {
    let (texture, desc, texture_desc) = ctx.create_readable_texture()?;
    let buffer = vec![0u8; desc.calc_buffer_size()];
    Ok((buffer, texture, texture_desc))
  }
}

impl Capturer for SimpleCapturer<'_> {
  fn dxgi_output_desc(&self) -> Result<DXGI_OUTPUT_DESC> {
    self.ctx.dxgi_output_desc()
  }

  fn dxgi_outdupl_desc(&self) -> DXGI_OUTDUPL_DESC {
    self.ctx.dxgi_outdupl_desc()
  }

  fn buffer(&self) -> &[u8] {
    &self.buffer
  }

  fn buffer_mut(&mut self) -> &mut [u8] {
    &mut self.buffer
  }

  fn check_buffer(&self) -> Result<()> {
    if self.buffer.len() < self.dxgi_outdupl_desc().calc_buffer_size() {
      Err(Error::InvalidBufferLength)
    } else {
      Ok(())
    }
  }

  fn pointer_shape_buffer(&self) -> &[u8] {
    &self.pointer_shape_buffer[..self.pointer_shape_buffer_size]
  }

  fn capture(&mut self, timeout_ms: u32) -> Result<DXGI_OUTDUPL_FRAME_INFO> {
    self.ctx.capture(
      self.buffer.as_mut_ptr(),
      self.buffer.len(),
      timeout_ms,
      &self.texture,
      &self.texture_desc,
    )
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
    let (frame_info, pointer_shape_info) = self.ctx.capture_with_pointer_shape(
      self.buffer.as_mut_ptr(),
      self.buffer.len(),
      timeout_ms,
      &self.texture,
      &self.texture_desc,
      &mut self.pointer_shape_buffer,
    )?;

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

impl DuplicationContext {
  pub fn simple_capturer(&self) -> Result<SimpleCapturer> {
    SimpleCapturer::new(self)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    capturer::model::Capturer, duplication_context::DuplicationContext, utils::FrameInfoExt,
  };
  use serial_test::serial;
  use std::{thread, time::Duration};

  #[test]
  #[serial]
  fn simple_capturer() {
    let ctx = DuplicationContext::factory().unwrap().next().unwrap();
    let mut capturer = ctx.simple_capturer().unwrap();

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
    assert!(frame_info.mouse_updated().position_updated);
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
