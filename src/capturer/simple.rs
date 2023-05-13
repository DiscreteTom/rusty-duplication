use windows::Win32::Graphics::{
  Direct3D11::ID3D11Texture2D,
  Dxgi::{DXGI_OUTDUPL_FRAME_INFO, DXGI_OUTPUT_DESC},
};

use crate::{duplicate_context::DuplicateContext, utils::Dimension};

use super::model::Capturer;

/// Capture screen to a `Vec<u8>`.
pub struct SimpleCapturer<'a> {
  desc: DXGI_OUTPUT_DESC, // TODO: not needed?
  buffer: Vec<u8>,
  ctx: &'a DuplicateContext,
  texture: ID3D11Texture2D,
}

impl<'a> SimpleCapturer<'a> {
  pub fn new(ctx: &'a DuplicateContext) -> Self {
    let desc = ctx.get_desc();
    let buffer = vec![0u8; (desc.width() * desc.height() * 4) as usize];
    Self {
      desc,
      buffer,
      ctx,
      texture: ctx.create_readable_texture(),
    }
  }
}

impl Capturer for SimpleCapturer<'_> {
  fn get_buffer(&self) -> &[u8] {
    &self.buffer
  }

  fn get_desc(&self) -> DXGI_OUTPUT_DESC {
    self.desc // TODO: refresh?
  }

  fn capture(&mut self) -> DXGI_OUTDUPL_FRAME_INFO {
    self
      .ctx
      .capture_frame(self.buffer.as_mut_ptr(), self.buffer.len(), &self.texture)
  }
}

impl DuplicateContext {
  pub fn simple_capturer(&self) -> SimpleCapturer {
    SimpleCapturer::new(self)
  }
}
