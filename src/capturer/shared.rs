use super::CapturerBuffer;
use crate::{Capturer, Error, Monitor, Result};
use std::ffi::CString;
use std::slice;
use windows::{
  core::PCSTR,
  Win32::{
    Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE},
    System::Memory::{
      CreateFileMappingA, MapViewOfFile, OpenFileMappingA, UnmapViewOfFile, FILE_MAP_ALL_ACCESS,
      MEMORY_MAPPED_VIEW_ADDRESS, PAGE_READWRITE,
    },
  },
};

/// This is not clone-able.
#[derive(Debug)]
pub struct SharedMemory {
  ptr: *mut u8,
  len: usize,
  file: HANDLE,
}

impl CapturerBuffer for SharedMemory {
  #[inline]
  fn as_bytes(&self) -> &[u8] {
    unsafe { slice::from_raw_parts(self.ptr, self.len) }
  }

  #[inline]
  fn as_bytes_mut(&mut self) -> &mut [u8] {
    unsafe { slice::from_raw_parts_mut(self.ptr, self.len) }
  }
}

impl Drop for SharedMemory {
  fn drop(&mut self) {
    // TODO: add debug log
    unsafe {
      UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
        Value: self.ptr as _,
      })
      .ok();
      CloseHandle(self.file).ok();
    }
  }
}

/// Capture screen to a chunk of shared memory.
///
/// To create an instance, see [`SharedMemoryCapturer::create`] and [`SharedMemoryCapturer::open`].
pub type SharedMemoryCapturer = Capturer<SharedMemory>;

impl SharedMemoryCapturer {
  /// Create an instance by creating a new shared memory with the provided name.
  pub fn create(monitor: Monitor, name: &str) -> Result<Self> {
    Self::new(monitor, move |len| unsafe {
      let file = CreateFileMappingA(
        INVALID_HANDLE_VALUE,
        None,
        PAGE_READWRITE,
        0,
        len as u32,
        str_to_pc_str(name),
      )
      .map_err(Error::from_win_err(stringify!(CreateFileMappingA)))?;

      Ok(SharedMemory {
        ptr: map_view_of_file(file, len)?,
        len,
        file,
      })
    })
  }

  /// Create an instance by opening an existing shared memory with the provided name.
  pub fn open(monitor: Monitor, name: &str) -> Result<Self> {
    Self::new(monitor, move |len| unsafe {
      let file = OpenFileMappingA(FILE_MAP_ALL_ACCESS.0, false, str_to_pc_str(name))
        .map_err(Error::from_win_err(stringify!(OpenFileMappingA)))?;

      Ok(SharedMemory {
        ptr: map_view_of_file(file, len)?,
        len,
        file,
      })
    })
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

#[inline]
fn str_to_pc_str(s: &str) -> PCSTR {
  let c_str = CString::new(s).unwrap(); // make the name null terminated
  PCSTR(c_str.as_ptr() as _)
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
    let mut capturer = SharedMemoryCapturer::create(monitor, "RustyDuplicationTest").unwrap();

    // sleep for a while before capture to wait system to update the screen
    thread::sleep(Duration::from_millis(50));

    let info = capturer.capture().unwrap();
    assert!(info.desktop_updated());

    // ensure buffer not all zero
    assert!(!capturer.buffer.as_bytes().iter().all(|&n| n == 0));

    thread::sleep(Duration::from_millis(50));

    // check mouse
    let (frame_info, pointer_shape_info) = capturer.capture_with_pointer_shape().unwrap();
    if frame_info.mouse_updated() {
      assert!(pointer_shape_info.is_some());
      // make sure pointer shape buffer is not all zero
      assert!(!capturer.pointer_shape_buffer.iter().all(|&n| n == 0));
    } else {
      panic!("Move your mouse during the test to check mouse capture");
    }
  }
}
