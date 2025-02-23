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

/// Provides a buffer for the capturer to store the captured frame.
pub trait CapturerBuffer {
  fn as_bytes(&self) -> &[u8];
  fn as_bytes_mut(&mut self) -> &mut [u8];
}

/// This is stateful and holds a buffer of the last captured frame.
#[derive(Debug, Clone)]
pub struct Capturer<Buffer> {
  pub pointer_shape_buffer: Vec<u8>,
  /// The buffer to store the captured frame.
  /// The data is stored in BGRA32 format.
  pub buffer: Buffer,
  /// Timeout in milliseconds for the next frame.
  /// By default it is 300ms.
  pub timeout_ms: u32,

  monitor: Monitor,
  texture: ID3D11Texture2D,
  texture_desc: D3D11_TEXTURE2D_DESC,
}

impl<Buffer> Capturer<Buffer> {
  /// Create a new capturer with the provided monitor and buffer factory.
  ///
  /// The parameter of `buffer_factory` is the size of the buffer, in bytes.
  pub fn new(
    monitor: Monitor,
    buffer_factory: impl FnOnce(usize) -> Result<Buffer>,
  ) -> Result<Self> {
    let dupl_desc = monitor.dxgi_outdupl_desc();
    let (texture, texture_desc) =
      monitor.create_texture(&dupl_desc, &monitor.dxgi_output_desc()?)?;

    Ok(Self {
      buffer: buffer_factory(dupl_desc.calc_buffer_size())?,
      monitor,
      texture,
      texture_desc,
      pointer_shape_buffer: Vec::new(),
      timeout_ms: 300,
    })
  }

  #[inline]
  pub const fn monitor(&self) -> &Monitor {
    &self.monitor
  }

  /// Ensure [`Self::buffer`] is large enough to hold the frame.
  pub fn check_buffer(&self) -> Result<()>
  where
    Buffer: CapturerBuffer,
  {
    // TODO: can we cache the result of `self.monitor.dxgi_outdupl_desc().calc_buffer_size()`?
    if self.buffer.as_bytes().len() < self.monitor.dxgi_outdupl_desc().calc_buffer_size() {
      Err(Error::InvalidBufferLength)
    } else {
      Ok(())
    }
  }

  /// Capture the screen and return the frame info.
  /// The pixel data is stored in the [`Self::buffer`].
  /// # Safety
  /// You have to ensure [`Self::buffer`] is large enough to hold the frame.
  /// You can use [`Self::check_buffer`] to check the buffer size.
  pub unsafe fn capture_unchecked(&mut self) -> Result<DXGI_OUTDUPL_FRAME_INFO>
  where
    Buffer: CapturerBuffer,
  {
    let frame_info = self.monitor.next_frame(self.timeout_ms, &self.texture)?;

    capture(
      &self.texture,
      self.buffer.as_bytes_mut(),
      &self.texture_desc,
    )?;

    Ok(frame_info)
  }

  /// Capture the screen and return the frame info.
  /// The pixel data is stored in the [`Self::buffer`].
  ///
  /// This will call [`Self::check_buffer`] to check the buffer size.
  #[inline]
  pub fn capture(&mut self) -> Result<DXGI_OUTDUPL_FRAME_INFO>
  where
    Buffer: CapturerBuffer,
  {
    self.check_buffer()?;
    unsafe { self.capture_unchecked() }
  }

  /// Capture the screen and return the frame info.
  /// The pixel data is stored in the [`Self::buffer`].
  ///
  /// If the pointer shape is updated, the `Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>` will be [`Some`].
  /// The pointer shape is stored in the [`Self::pointer_shape_buffer`].
  /// # Safety
  /// You have to ensure [`Self::buffer`] is large enough to hold the frame.
  /// You can use [`Self::check_buffer`] to check the buffer size.
  pub unsafe fn capture_with_pointer_shape_unchecked(
    &mut self,
  ) -> Result<(
    DXGI_OUTDUPL_FRAME_INFO,
    Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>,
  )>
  where
    Buffer: CapturerBuffer,
  {
    let (frame_info, pointer_shape_info) = self.monitor.next_frame_with_pointer_shape(
      self.timeout_ms,
      &self.texture,
      &mut self.pointer_shape_buffer,
    )?;

    capture(
      &self.texture,
      self.buffer.as_bytes_mut(),
      &self.texture_desc,
    )?;

    Ok((frame_info, pointer_shape_info))
  }

  /// Capture the screen and return the frame info.
  /// The pixel data is stored in the [`Self::buffer`].
  ///
  /// If the pointer shape is updated, the `Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>` will be [`Some`].
  /// The pointer shape is stored in the [`Self::pointer_shape_buffer`].
  ///
  /// This will call [`Self::check_buffer`] to check the buffer size.
  #[inline]
  pub fn capture_with_pointer_shape(
    &mut self,
  ) -> Result<(
    DXGI_OUTDUPL_FRAME_INFO,
    Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>,
  )>
  where
    Buffer: CapturerBuffer,
  {
    self.check_buffer()?;
    unsafe { self.capture_with_pointer_shape_unchecked() }
  }
}

/// Capture the next frame to the provided buffer.
/// # Safety
/// This function will dereference the provided pointer.
/// The caller must ensure that the buffer is large enough to hold the frame.
unsafe fn capture(
  frame: &ID3D11Texture2D,
  dest: &mut [u8],
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
      ptr::copy_nonoverlapping(mapped_surface.pBits, dest.as_mut_ptr(), dest.len());
    } else {
      // https://github.com/DiscreteTom/rusty-duplication/issues/7
      // TODO: add a debug info here
      let mut src_offset = 0;
      let mut dest_offset = 0;
      for _ in 0..texture_desc.Height {
        let src = mapped_surface.pBits.offset(src_offset);
        let dest = dest.as_mut_ptr().offset(dest_offset);
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
