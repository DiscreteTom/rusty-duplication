use windows::Win32::Graphics::{
  Direct3D11::ID3D11Texture2D,
  Dxgi::{DXGI_OUTDUPL_FRAME_INFO, DXGI_OUTPUT_DESC},
};

use crate::utils::Result;
use crate::{duplicate_context::DuplicateContext, utils::OutputDescExt};

use super::model::Capturer;

/// Capture screen to a `Vec<u8>`.
pub struct SimpleCapturer<'a> {
  buffer: Vec<u8>,
  ctx: &'a DuplicateContext,
  texture: ID3D11Texture2D,
}

impl<'a> SimpleCapturer<'a> {
  pub fn new(ctx: &'a DuplicateContext) -> Result<Self> {
    let (buffer, texture) = Self::allocate(ctx)?;
    Ok(Self {
      buffer,
      ctx,
      texture,
    })
  }

  fn allocate(ctx: &'a DuplicateContext) -> Result<(Vec<u8>, ID3D11Texture2D)> {
    let (texture, desc) = ctx.create_readable_texture()?;
    let buffer = vec![0u8; desc.calc_buffer_size()];
    Ok((buffer, texture))
  }
}

impl Capturer for SimpleCapturer<'_> {
  fn buffer(&self) -> &[u8] {
    &self.buffer
  }

  fn buffer_mut(&mut self) -> &mut [u8] {
    &mut self.buffer
  }

  fn desc(&self) -> Result<DXGI_OUTPUT_DESC> {
    self.ctx.desc()
  }

  fn check_buffer(&self) -> Result<()> {
    if self.buffer.len() < self.desc()?.calc_buffer_size() {
      Err("Invalid buffer length")
    } else {
      Ok(())
    }
  }

  fn capture(&mut self) -> Result<DXGI_OUTDUPL_FRAME_INFO> {
    self
      .ctx
      .capture_frame(self.buffer.as_mut_ptr(), self.buffer.len(), &self.texture)
  }

  fn safe_capture(&mut self) -> Result<DXGI_OUTDUPL_FRAME_INFO> {
    self.check_buffer()?;
    self.capture()
  }
}

impl DuplicateContext {
  pub fn simple_capturer(&self) -> Result<SimpleCapturer> {
    SimpleCapturer::new(self)
  }
}

#[cfg(test)]
mod tests {
  use std::{thread, time::Duration};

  use crate::{capturer::model::Capturer, manager::Manager, utils::FrameInfoExt};

  #[test]
  fn simple_capturer() {
    let manager = Manager::default().unwrap();
    assert_ne!(manager.contexts.len(), 0);

    let mut capturer = manager.contexts[0].simple_capturer().unwrap();

    // sleep for a while before capture to wait system to update the screen
    thread::sleep(Duration::from_millis(100));

    let info = capturer.safe_capture().unwrap();
    assert!(info.is_new_frame());

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
  }
}
