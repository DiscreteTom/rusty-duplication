use super::model::Capturer;
use crate::duplication_context::DuplicationContext;
use crate::model::Result;
use crate::utils::{FrameInfoExt, OutputDescExt};
use std::slice;
use windows::core::PCSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Graphics::Dxgi::DXGI_OUTDUPL_POINTER_SHAPE_INFO;
use windows::Win32::System::Memory::{
  CreateFileMappingA, MapViewOfFile, OpenFileMappingA, UnmapViewOfFile, FILE_MAP_ALL_ACCESS,
  MEMORYMAPPEDVIEW_HANDLE,
};
use windows::Win32::{
  Foundation::INVALID_HANDLE_VALUE,
  Graphics::{
    Direct3D11::ID3D11Texture2D,
    Dxgi::{DXGI_OUTDUPL_FRAME_INFO, DXGI_OUTPUT_DESC},
  },
  System::Memory::PAGE_READWRITE,
};

/// Capture screen to a chunk of shared memory.
pub struct SharedCapturer<'a> {
  buffer: *mut u8,
  buffer_size: usize,
  file: HANDLE,
  ctx: &'a DuplicationContext,
  texture: ID3D11Texture2D,
  last_pointer_shape_buffer: Vec<u8>,
  last_pointer_shape_buffer_size: usize,
  pointer_shape_buffer: Vec<u8>,
  pointer_shape_buffer_size: usize,
}

impl<'a> SharedCapturer<'a> {
  pub fn new(ctx: &'a DuplicationContext, name: &str) -> Result<Self> {
    let (buffer, buffer_size, file, texture) = Self::allocate(ctx, name)?;
    Ok(Self {
      buffer,
      buffer_size,
      file,
      texture,
      ctx,
      last_pointer_shape_buffer: Vec::new(),
      last_pointer_shape_buffer_size: 0,
      pointer_shape_buffer: Vec::new(),
      pointer_shape_buffer_size: 0,
    })
  }

  pub fn open(ctx: &'a DuplicationContext, name: &str) -> Result<Self> {
    let (buffer, buffer_size, file, texture) = Self::open_file(ctx, name)?;
    Ok(Self {
      buffer,
      buffer_size,
      file,
      texture,
      ctx,
      last_pointer_shape_buffer: Vec::new(),
      last_pointer_shape_buffer_size: 0,
      pointer_shape_buffer: Vec::new(),
      pointer_shape_buffer_size: 0,
    })
  }

  fn allocate(
    ctx: &'a DuplicationContext,
    name: &str,
  ) -> Result<(*mut u8, usize, HANDLE, ID3D11Texture2D)> {
    let (texture, desc) = ctx.create_readable_texture()?;
    let buffer_size = desc.calc_buffer_size();

    unsafe {
      let file = CreateFileMappingA(
        INVALID_HANDLE_VALUE,
        None,
        PAGE_READWRITE,
        0,
        buffer_size as u32,
        PCSTR(name.as_ptr()),
      )
      .map_err(|e| format!("CreateFileMappingA failed: {:?}", e))?;

      let buffer = MapViewOfFile(
        file,                // handle to map object
        FILE_MAP_ALL_ACCESS, // read/write permission
        0,
        0,
        buffer_size,
      )
      .map_err(|e| format!("MapViewOfFile failed: {:?}", e))?
      .0 as *mut u8;
      Ok((buffer, buffer_size, file, texture))
    }
  }

  fn open_file(
    ctx: &'a DuplicationContext,
    name: &str,
  ) -> Result<(*mut u8, usize, HANDLE, ID3D11Texture2D)> {
    let (texture, desc) = ctx.create_readable_texture()?;
    let buffer_size = desc.calc_buffer_size();

    unsafe {
      let file = OpenFileMappingA(FILE_MAP_ALL_ACCESS.0, false, PCSTR(name.as_ptr()))
        .map_err(|e| format!("CreateFileMappingA failed: {:?}", e))?;

      let buffer = MapViewOfFile(
        file,                // handle to map object
        FILE_MAP_ALL_ACCESS, // read/write permission
        0,
        0,
        buffer_size,
      )
      .map_err(|e| format!("MapViewOfFile failed: {:?}", e))?
      .0 as *mut u8;
      Ok((buffer, buffer_size, file, texture))
    }
  }

  fn free(&self) {
    unsafe {
      UnmapViewOfFile(MEMORYMAPPEDVIEW_HANDLE(self.buffer as isize));
      CloseHandle(self.file);
    }
  }
}

impl<'a> Capturer for SharedCapturer<'a> {
  fn buffer(&self) -> &[u8] {
    unsafe { slice::from_raw_parts(self.buffer, self.buffer_size) }
  }

  fn buffer_mut(&mut self) -> &mut [u8] {
    unsafe { slice::from_raw_parts_mut(self.buffer, self.buffer_size) }
  }

  fn pointer_shape_buffer(&self) -> &[u8] {
    &self.pointer_shape_buffer[..self.pointer_shape_buffer_size]
  }

  fn desc(&self) -> Result<DXGI_OUTPUT_DESC> {
    self.ctx.desc()
  }

  fn capture(&mut self) -> Result<DXGI_OUTDUPL_FRAME_INFO> {
    self
      .ctx
      .capture_frame(self.buffer, self.buffer_size, &self.texture)
  }

  fn capture_with_pointer_shape(
    &mut self,
  ) -> Result<(
    DXGI_OUTDUPL_FRAME_INFO,
    Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>,
  )> {
    let (frame_info, pointer_shape_info) = self.ctx.capture_frame_with_pointer_shape(
      self.buffer,
      self.buffer_size,
      &self.texture,
      &mut self.pointer_shape_buffer,
    )?;

    // record the pointer shape buffer size
    if frame_info.mouse_updated() {
      self.pointer_shape_buffer_size = frame_info.PointerShapeBufferSize as usize;

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
}

impl DuplicationContext {
  pub fn shared_capturer(&self, name: &str) -> Result<SharedCapturer> {
    SharedCapturer::new(self, name)
  }

  pub fn shared_capturer_open(&self, name: &str) -> Result<SharedCapturer> {
    SharedCapturer::open(self, name)
  }
}

impl<'a> Drop for SharedCapturer<'a> {
  fn drop(&mut self) {
    self.free()
  }
}

// #[cfg(test)]
// mod tests {
//   use std::{thread, time::Duration};

//   use crate::{capturer::model::Capturer, manager::Manager, utils::FrameInfoExt};

//   #[test]
//   fn shared_capturer() {
//     let manager = Manager::default().unwrap();
//     assert_ne!(manager.contexts.len(), 0);

//     let mut capturer = manager.contexts[0]
//       .shared_capturer("RustyDuplicationTest")
//       .unwrap();

//     // sleep for a while before capture to wait system to update the screen
//     thread::sleep(Duration::from_millis(100));

//     let info = capturer.safe_capture().unwrap();
//     assert!(info.desktop_updated());

//     let buffer = capturer.buffer();
//     // ensure buffer not all zero
//     let mut all_zero = true;
//     for i in 0..buffer.len() {
//       if buffer[i] != 0 {
//         all_zero = false;
//         break;
//       }
//     }
//     assert!(!all_zero);

//     // check pointer shape
//     let pointer_shape = capturer.get_pointer_shape(&info).unwrap();
//     // make sure pointer shape buffer is not all zero
//     let mut all_zero = true;
//     for i in 0..pointer_shape.data.len() {
//       if pointer_shape.data[i] != 0 {
//         all_zero = false;
//         break;
//       }
//     }
//     assert!(!all_zero);
//   }
// }
