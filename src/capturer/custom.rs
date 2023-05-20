use super::model::Capturer;
use crate::model::Result;
use crate::utils::FrameInfoExt;
use crate::{duplication_context::DuplicationContext, utils::OutputDescExt};
use windows::Win32::Graphics::Dxgi::DXGI_OUTDUPL_POINTER_SHAPE_INFO;
use windows::Win32::Graphics::{
  Direct3D11::ID3D11Texture2D,
  Dxgi::{DXGI_OUTDUPL_FRAME_INFO, DXGI_OUTPUT_DESC},
};

/// Capture screen to a chunk of memory.
pub struct CustomCapturer<'a> {
  buffer: &'a mut [u8],
  ctx: &'a DuplicationContext,
  texture: ID3D11Texture2D,
  last_pointer_shape_buffer: Vec<u8>,
  last_pointer_shape_buffer_size: usize,
  pointer_shape_buffer: Vec<u8>,
  pointer_shape_buffer_size: usize,
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
      last_pointer_shape_buffer: Vec::new(),
      last_pointer_shape_buffer_size: 0,
      pointer_shape_buffer: Vec::new(),
      pointer_shape_buffer_size: 0,
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

  fn check_buffer(&self) -> Result<()> {
    let desc = self.dxgi_output_desc()?;
    let dpi = self.ctx.effective_dpi(&desc)?;
    if self.buffer.len() < desc.calc_buffer_size(dpi) {
      Err("Invalid buffer length".into())
    } else {
      Ok(())
    }
  }

  fn pointer_shape_buffer(&self) -> &[u8] {
    &self.pointer_shape_buffer[..self.pointer_shape_buffer_size]
  }

  fn pointer_shape_updated(&self) -> bool {
    self.pointer_shape_buffer_size != self.last_pointer_shape_buffer_size || {
      let len = self.pointer_shape_buffer_size;
      self.pointer_shape_buffer[..len] != self.last_pointer_shape_buffer[..len]
    }
  }

  fn dxgi_output_desc(&self) -> Result<DXGI_OUTPUT_DESC> {
    self.ctx.dxgi_output_desc()
  }

  fn capture(&mut self) -> Result<DXGI_OUTDUPL_FRAME_INFO> {
    self
      .ctx
      .capture(self.buffer.as_mut_ptr(), self.buffer.len(), &self.texture)
  }

  fn safe_capture(&mut self) -> Result<DXGI_OUTDUPL_FRAME_INFO> {
    self.check_buffer()?;
    self.capture()
  }

  fn capture_with_pointer_shape(
    &mut self,
  ) -> Result<(
    DXGI_OUTDUPL_FRAME_INFO,
    Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>,
  )> {
    let (frame_info, pointer_shape_info) = self.ctx.capture_with_pointer_shape(
      self.buffer.as_mut_ptr(),
      self.buffer.len(),
      &self.texture,
      &mut self.last_pointer_shape_buffer, // IMPORTANT: write to last pointer shape buffer
    )?;

    if frame_info.mouse_updated() {
      // record the pointer shape buffer size
      // IMPORTANT: change last pointer shape buffer size
      self.last_pointer_shape_buffer_size = frame_info.PointerShapeBufferSize as usize;

      // swap the pointer shape buffer
      std::mem::swap(
        &mut self.pointer_shape_buffer,
        &mut self.last_pointer_shape_buffer,
      );
      std::mem::swap(
        &mut self.pointer_shape_buffer_size,
        &mut self.last_pointer_shape_buffer_size,
      );
    }

    Ok((frame_info, pointer_shape_info))
  }

  fn safe_capture_with_pointer_shape(
    &mut self,
  ) -> Result<(
    DXGI_OUTDUPL_FRAME_INFO,
    Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>,
  )> {
    self.check_buffer()?;
    self.capture_with_pointer_shape()
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
    let desc = ctx.dxgi_output_desc().unwrap();
    let dpi = ctx.effective_dpi(&desc).unwrap();
    let mut buffer = vec![0u8; desc.calc_buffer_size(dpi)];
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

    // sleep for a while before capture to wait system to update the mouse
    thread::sleep(Duration::from_millis(1000));

    // check pointer shape
    let (frame_info, pointer_shape_info) = capturer.safe_capture_with_pointer_shape().unwrap();
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
