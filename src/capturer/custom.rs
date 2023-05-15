use windows::Win32::Graphics::{
  Direct3D11::ID3D11Texture2D,
  Dxgi::{DXGI_OUTDUPL_FRAME_INFO, DXGI_OUTPUT_DESC},
};

use crate::utils::Result;
use crate::{duplicate_context::DuplicateContext, utils::OutputDescExt};

use super::model::Capturer;

/// Capture screen to a chunk of memory.
pub struct CustomCapturer<'ctx, 'buffer: 'ctx> {
  buffer: &'buffer mut [u8],
  ctx: &'ctx DuplicateContext,
  texture: ID3D11Texture2D,
}

impl<'ctx, 'buffer> CustomCapturer<'ctx, 'buffer> {
  pub fn with_texture(
    ctx: &'ctx DuplicateContext,
    buffer: &'buffer mut [u8],
    texture: ID3D11Texture2D,
  ) -> Self {
    Self {
      buffer,
      ctx,
      texture,
    }
  }

  pub fn new(ctx: &'ctx DuplicateContext, buffer: &'buffer mut [u8]) -> Result<Self> {
    Ok(Self::with_texture(
      ctx,
      buffer,
      ctx.create_readable_texture()?.0,
    ))
  }
}

impl Capturer for CustomCapturer<'_, '_> {
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
  pub fn custom_capturer<'ctx, 'buffer: 'ctx>(
    &'ctx self,
    buffer: &'buffer mut [u8],
  ) -> Result<CustomCapturer> {
    CustomCapturer::<'ctx, 'buffer>::new(self, buffer)
  }
}
