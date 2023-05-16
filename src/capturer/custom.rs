use windows::Win32::Graphics::{
  Direct3D11::ID3D11Texture2D,
  Dxgi::{DXGI_OUTDUPL_FRAME_INFO, DXGI_OUTDUPL_POINTER_SHAPE_INFO, DXGI_OUTPUT_DESC},
};

use crate::utils::Result;
use crate::{duplication_context::DuplicationContext, utils::OutputDescExt};

use super::model::Capturer;

/// Capture screen to a chunk of memory.
pub struct CustomCapturer<'a> {
  buffer: &'a mut [u8],
  ctx: &'a DuplicationContext,
  texture: ID3D11Texture2D,
}

impl<'a> CustomCapturer<'a> {
  pub fn with_texture(
    ctx: &'a DuplicationContext,
    buffer: &'a mut [u8],
    texture: ID3D11Texture2D,
  ) -> Self {
    Self {
      buffer,
      ctx,
      texture,
    }
  }

  pub fn new(ctx: &'a DuplicationContext, buffer: &'a mut [u8]) -> Result<Self> {
    Ok(Self::with_texture(
      ctx,
      buffer,
      ctx.create_readable_texture()?.0,
    ))
  }
}

impl Capturer for CustomCapturer<'_> {
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
      Err("Invalid buffer length".into())
    } else {
      Ok(())
    }
  }

  fn capture(&mut self) -> Result<DXGI_OUTDUPL_FRAME_INFO> {
    self
      .ctx
      .capture_frame(self.buffer.as_mut_ptr(), self.buffer.len(), &self.texture)
  }

  fn get_pointer(
    &self,
    frame_info: &DXGI_OUTDUPL_FRAME_INFO,
  ) -> Result<(Vec<u8>, u32, DXGI_OUTDUPL_POINTER_SHAPE_INFO)> {
    self.ctx.get_pointer(&frame_info)
  }

  fn safe_capture(&mut self) -> Result<DXGI_OUTDUPL_FRAME_INFO> {
    self.check_buffer()?;
    self.capture()
  }
}

impl DuplicationContext {
  pub fn custom_capturer<'a>(&'a self, buffer: &'a mut [u8]) -> Result<CustomCapturer> {
    CustomCapturer::<'a>::new(self, buffer)
  }
}

#[cfg(test)]
mod tests {
  use std::{thread, time::Duration};

  use crate::{
    capturer::model::Capturer,
    manager::Manager,
    utils::{FrameInfoExt, OutputDescExt},
  };

  #[test]
  fn custom_capturer() {
    let manager = Manager::default().unwrap();
    assert_ne!(manager.contexts.len(), 0);

    let ctx = &manager.contexts[0];
    let desc = ctx.desc().unwrap();
    let mut buffer = vec![0u8; desc.calc_buffer_size()];
    let mut capturer = ctx.custom_capturer(&mut buffer).unwrap();

    // sleep for a while before capture to wait system to update the screen
    thread::sleep(Duration::from_millis(100));

    let info = capturer.safe_capture().unwrap();
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
  }
}
