use windows::Win32::Graphics::{
  Direct3D11::ID3D11Texture2D,
  Dxgi::{DXGI_OUTDUPL_FRAME_INFO, DXGI_OUTPUT_DESC},
};

use crate::utils::Result;
use crate::{duplicate_context::DuplicateContext, utils::Dimension};

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
    let buffer = vec![0u8; (desc.width() * desc.height() * 4) as usize];
    Ok((buffer, texture))
  }
}

impl Capturer for SimpleCapturer<'_> {
  fn get_buffer(&self) -> &[u8] {
    &self.buffer
  }

  fn get_desc(&self) -> Result<DXGI_OUTPUT_DESC> {
    self.ctx.get_desc()
  }

  fn capture(&mut self) -> Result<DXGI_OUTDUPL_FRAME_INFO> {
    self
      .ctx
      .capture_frame(self.buffer.as_mut_ptr(), self.buffer.len(), &self.texture)
  }
}

impl DuplicateContext {
  pub fn simple_capturer(&self) -> Result<SimpleCapturer> {
    SimpleCapturer::new(self)
  }
}
