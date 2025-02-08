mod shared;
mod vec;

use crate::{Error, Monitor, OutDuplDescExt, Result};
use std::ptr;
use windows::{
  core::Interface,
  Win32::Graphics::{
    Direct3D11::{ID3D11Texture2D, D3D11_TEXTURE2D_DESC},
    Dxgi::{
      IDXGISurface1, DXGI_MAPPED_RECT, DXGI_MAP_READ, DXGI_OUTDUPL_FRAME_INFO,
      DXGI_OUTDUPL_POINTER_SHAPE_INFO,
    },
  },
};

pub use shared::*;
pub use vec::*;

/// Capturer is stateful, it holds a buffer of the last captured frame.
pub trait Capturer {
  fn monitor(&self) -> &Monitor;

  /// Get the buffer of the last captured frame.
  /// The buffer is in BGRA32 format.
  fn buffer(&self) -> &[u8];

  /// Get the buffer of the last captured frame.
  /// The buffer is in BGRA32 format.
  fn buffer_mut(&mut self) -> &mut [u8];

  /// Check buffer size.
  fn check_buffer(&self) -> Result<()> {
    if self.buffer().len() < self.monitor().dxgi_outdupl_desc().calc_buffer_size() {
      Err(Error::InvalidBufferLength)
    } else {
      Ok(())
    }
  }
  /// Get the buffer of the captured pointer shape.
  fn pointer_shape_buffer(&self) -> &[u8];

  /// Get the buffer of the captured pointer shape.
  fn pointer_shape_buffer_mut(&mut self) -> &mut [u8];

  /// Capture the screen and return the frame info.
  /// The pixel data is stored in the [`Capturer::buffer`].
  /// # Safety
  /// You have to ensure [`Capturer::buffer`] is large enough to hold the frame.
  /// You can use [`Capturer::check_buffer`] to check the buffer size.
  unsafe fn capture_unchecked(&mut self, timeout_ms: u32) -> Result<DXGI_OUTDUPL_FRAME_INFO>;

  /// Capture the screen and return the frame info.
  /// The pixel data is stored in the [`Capturer::buffer`].
  ///
  /// This will call [`Capturer::check_buffer`] to check the buffer size.
  fn capture(&mut self, timeout_ms: u32) -> Result<DXGI_OUTDUPL_FRAME_INFO> {
    self.check_buffer()?;
    unsafe { self.capture_unchecked(timeout_ms) }
  }
  /// Capture the screen and return the frame info.
  /// The pixel data is stored in the [`Capturer::buffer`].
  ///
  /// If mouse is updated, the `Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>` is Some.
  /// The pointer shape is stored in the [`Capturer::pointer_shape_buffer`].
  /// # Safety
  /// You have to ensure [`Capturer::buffer`] is large enough to hold the frame.
  /// You can use [`Capturer::check_buffer`] to check the buffer size.
  unsafe fn capture_with_pointer_shape_unchecked(
    &mut self,
    timeout_ms: u32,
  ) -> Result<(
    DXGI_OUTDUPL_FRAME_INFO,
    Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>,
  )>;

  /// Check buffer size before capture.
  /// The pixel data is stored in the [`Capturer::buffer`].
  ///
  /// If mouse is updated, the `Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>` is Some.
  /// The pointer shape is stored in the [`Capturer::pointer_shape_buffer`].
  ///
  /// This will call [`Capturer::check_buffer`] to check the buffer size.
  fn capture_with_pointer_shape(
    &mut self,
    timeout_ms: u32,
  ) -> Result<(
    DXGI_OUTDUPL_FRAME_INFO,
    Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>,
  )> {
    self.check_buffer()?;
    unsafe { self.capture_with_pointer_shape_unchecked(timeout_ms) }
  }
}

/// Capture the next frame to the provided buffer.
/// # Safety
/// This function will dereference the provided pointer.
/// The caller must ensure that the buffer is large enough to hold the frame.
unsafe fn capture(
  frame: &ID3D11Texture2D,
  dest: *mut u8,
  len: usize,
  texture_desc: &D3D11_TEXTURE2D_DESC,
) -> Result<()> {
  let frame: IDXGISurface1 = frame.cast().unwrap();
  let mut mapped_surface = DXGI_MAPPED_RECT::default();
  let bytes_per_line = texture_desc.Width as usize * 4; // 4 for BGRA32

  unsafe {
    frame
      .Map(&mut mapped_surface, DXGI_MAP_READ)
      .map_err(Error::from_win_err(stringify!(IDXGISurface1.Map)))?;
    if mapped_surface.Pitch as usize == bytes_per_line {
      ptr::copy_nonoverlapping(mapped_surface.pBits, dest, len);
    } else {
      // https://github.com/DiscreteTom/rusty-duplication/issues/7
      // TODO: add a debug info here
      let mut src_offset = 0;
      let mut dest_offset = 0;
      for _ in 0..texture_desc.Height {
        let src = mapped_surface.pBits.offset(src_offset);
        let dest = dest.offset(dest_offset);
        ptr::copy_nonoverlapping(src, dest, mapped_surface.Pitch as usize);

        src_offset += mapped_surface.Pitch as isize;
        dest_offset += bytes_per_line as isize;
      }
    }
    frame
      .Unmap()
      .map_err(Error::from_win_err(stringify!(IDXGISurface1.Unmap)))
  }
}
