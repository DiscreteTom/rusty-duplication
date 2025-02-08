use super::CapturerBuffer;
use crate::{Capturer, Error, Monitor, OutDuplDescExt, Result};
use std::ffi::CString;
use std::slice;
use windows::{
  core::PCSTR,
  Win32::{
    Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE},
    Graphics::Direct3D11::{ID3D11Texture2D, D3D11_TEXTURE2D_DESC},
    System::Memory::{
      CreateFileMappingA, MapViewOfFile, OpenFileMappingA, UnmapViewOfFile, FILE_MAP_ALL_ACCESS,
      MEMORY_MAPPED_VIEW_ADDRESS, PAGE_READWRITE,
    },
  },
};

#[derive(Debug)]
pub struct SharedMemory {
  buffer: *mut u8,
  buffer_size: usize,
  file: HANDLE,
}

impl CapturerBuffer for SharedMemory {
  fn as_bytes(&self) -> &[u8] {
    unsafe { slice::from_raw_parts(self.buffer, self.buffer_size) }
  }

  fn as_bytes_mut(&mut self) -> &mut [u8] {
    unsafe { slice::from_raw_parts_mut(self.buffer, self.buffer_size) }
  }
}

impl Drop for SharedMemory {
  fn drop(&mut self) {
    // TODO: add debug log
    unsafe {
      UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
        Value: self.buffer as _,
      })
      .ok();
      CloseHandle(self.file).ok();
    }
  }
}

/// Capture screen to a chunk of shared memory.
pub type SharedCapturer = Capturer<SharedMemory>;

impl SharedCapturer {
  pub fn create(monitor: Monitor, name: &str) -> Result<Self> {
    let (buffer, buffer_size, file, texture, texture_desc) = Self::allocate(&monitor, name)?;
    Ok(Self {
      buffer: SharedMemory {
        buffer,
        buffer_size,
        file,
      },
      texture,
      texture_desc,
      monitor,
      pointer_shape_buffer: Vec::new(),
    })
  }

  pub fn open(monitor: Monitor, name: &str) -> Result<Self> {
    let (buffer, buffer_size, file, texture, texture_desc) = Self::open_file(&monitor, name)?;
    Ok(Self {
      buffer: SharedMemory {
        buffer,
        buffer_size,
        file,
      },
      texture,
      texture_desc,
      monitor,
      pointer_shape_buffer: Vec::new(),
    })
  }

  fn allocate(
    monitor: &Monitor,
    name: &str,
  ) -> Result<(
    *mut u8,
    usize,
    HANDLE,
    ID3D11Texture2D,
    D3D11_TEXTURE2D_DESC,
  )> {
    let dupl_desc = monitor.dxgi_outdupl_desc();
    let (texture, texture_desc) =
      monitor.create_texture(&dupl_desc, &monitor.dxgi_output_desc()?)?;
    let buffer_size = dupl_desc.calc_buffer_size();
    let name = CString::new(name).unwrap(); // make the name null terminated

    unsafe {
      let file = CreateFileMappingA(
        INVALID_HANDLE_VALUE,
        None,
        PAGE_READWRITE,
        0,
        buffer_size as u32,
        PCSTR(name.as_ptr() as *const _),
      )
      .map_err(Error::from_win_err(stringify!(CreateFileMappingA)))?;

      let buffer = Self::map_view_of_file(file, buffer_size)?;
      Ok((buffer, buffer_size, file, texture, texture_desc))
    }
  }

  fn open_file(
    monitor: &Monitor,
    name: &str,
  ) -> Result<(
    *mut u8,
    usize,
    HANDLE,
    ID3D11Texture2D,
    D3D11_TEXTURE2D_DESC,
  )> {
    let dupl_desc = monitor.dxgi_outdupl_desc();
    let (texture, texture_desc) =
      monitor.create_texture(&dupl_desc, &monitor.dxgi_output_desc()?)?;
    let buffer_size = dupl_desc.calc_buffer_size();
    let name = CString::new(name).unwrap(); // make the name null terminated

    unsafe {
      let file = OpenFileMappingA(
        FILE_MAP_ALL_ACCESS.0,
        false,
        PCSTR(name.as_ptr() as *const _),
      )
      .map_err(Error::from_win_err(stringify!(OpenFileMappingA)))?;

      let buffer = Self::map_view_of_file(file, buffer_size)?;
      Ok((buffer, buffer_size, file, texture, texture_desc))
    }
  }

  unsafe fn map_view_of_file(file: HANDLE, buffer_size: usize) -> Result<*mut u8> {
    let buffer_ptr = MapViewOfFile(
      file,                // handle to map object
      FILE_MAP_ALL_ACCESS, // read/write permission
      0,
      0,
      buffer_size,
    );

    if buffer_ptr.Value.is_null() {
      CloseHandle(file).ok();
      return Err(Error::last_win_err(stringify!(MapViewOfFile)));
    }

    Ok(buffer_ptr.Value as *mut u8)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{FrameInfoExt, Scanner};
  use serial_test::serial;
  use std::{thread, time::Duration};

  #[test]
  #[serial]
  fn shared_capturer() {
    let monitor = Scanner::new().unwrap().next().unwrap();
    let mut capturer = SharedCapturer::create(monitor, "RustyDuplicationTest").unwrap();

    // sleep for a while before capture to wait system to update the screen
    thread::sleep(Duration::from_millis(100));

    let info = capturer.capture(300).unwrap();
    assert!(info.desktop_updated());

    let buffer = capturer.buffer.as_bytes();
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
    let (frame_info, pointer_shape_info) = capturer.capture_with_pointer_shape(300).unwrap();
    assert!(frame_info.mouse_updated());
    assert!(pointer_shape_info.is_some());
    let pointer_shape_data = capturer.pointer_shape_buffer;
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
