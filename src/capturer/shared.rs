use std::slice;

use windows::core::PCSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Memory::{
  CreateFileMappingA, MapViewOfFile, UnmapViewOfFile, FILE_MAP_ALL_ACCESS, MEMORYMAPPEDVIEW_HANDLE,
};
use windows::Win32::{
  Foundation::INVALID_HANDLE_VALUE,
  Graphics::{
    Direct3D11::ID3D11Texture2D,
    Dxgi::{DXGI_OUTDUPL_FRAME_INFO, DXGI_OUTPUT_DESC},
  },
  System::Memory::PAGE_READWRITE,
};

use crate::utils::Result;
use crate::{duplicate_context::DuplicateContext, utils::Dimension};

use super::model::Capturer;

/// Capture screen to a chunk of shared memory.
pub struct SharedCapturer<'a> {
  desc: DXGI_OUTPUT_DESC,
  buffer: *mut u8,
  buffer_size: usize,
  file: HANDLE,
  ctx: &'a DuplicateContext,
  texture: ID3D11Texture2D,
}

impl<'a> SharedCapturer<'a> {
  pub fn new(ctx: &'a DuplicateContext, name: String) -> Result<Self> {
    let (texture, desc) = ctx.create_readable_texture()?;
    let buffer_size = (desc.width() * desc.height() * 4) as usize;

    unsafe {
      let file = CreateFileMappingA(
        INVALID_HANDLE_VALUE,
        None,
        PAGE_READWRITE,
        0,
        buffer_size as u32,
        PCSTR(name.as_ptr()),
      )
      .unwrap();

      let buffer = MapViewOfFile(
        file,                // handle to map object
        FILE_MAP_ALL_ACCESS, // read/write permission
        0,
        0,
        buffer_size,
      )
      .unwrap()
      .0 as *mut u8;

      Ok(Self {
        desc,
        buffer,
        buffer_size,
        file,
        ctx,
        texture,
      })
    }
  }
}

impl<'a> Capturer for SharedCapturer<'a> {
  fn get_buffer(&self) -> &[u8] {
    unsafe { slice::from_raw_parts(self.buffer, self.buffer_size) }
  }

  fn get_desc(&self) -> DXGI_OUTPUT_DESC {
    self.desc
  }

  fn capture(&mut self) -> Result<DXGI_OUTDUPL_FRAME_INFO> {
    self
      .ctx
      .capture_frame(self.buffer, self.buffer_size, &self.texture)
  }
}

impl DuplicateContext {
  pub fn shared_capturer(&self, name: String) -> Result<SharedCapturer> {
    SharedCapturer::new(self, name)
  }
}

impl<'a> Drop for SharedCapturer<'a> {
  fn drop(&mut self) {
    unsafe {
      UnmapViewOfFile(MEMORYMAPPEDVIEW_HANDLE(self.buffer as isize));
      CloseHandle(self.file);
    }
  }
}
