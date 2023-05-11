use windows::Win32::Graphics::Dxgi::DXGI_OUTPUT_DESC;

use crate::{duplicate_context::DuplicateContext, utils::Dimension};

pub struct SimpleCapturer<'a> {
  ctx: &'a DuplicateContext,
  pub desc: DXGI_OUTPUT_DESC,
  pub buffer: Vec<u8>,
}

impl<'a> SimpleCapturer<'a> {
  pub fn new(ctx: &'a DuplicateContext) -> Self {
    let desc = ctx.get_desc();
    let buffer = vec![0u8; (desc.width() * desc.height() * 4) as usize];
    Self { ctx, buffer, desc }
  }

  pub fn capture(&mut self) {
    self.ctx.capture_frame(
      self.buffer.as_mut_ptr(),
      self.desc.width(),
      self.desc.height(),
    );
  }
}
