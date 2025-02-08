use crate::{Error, Result};
use std::ptr;
use windows::Win32::Graphics::{
  Direct3D11::D3D11_TEXTURE2D_DESC,
  Dxgi::{IDXGISurface1, DXGI_MAPPED_RECT, DXGI_MAP_READ},
};

pub mod custom;
pub mod model;
pub mod shared;
pub mod simple;

/// Capture the next frame to the provided buffer.
/// # Safety
/// This function will dereference the provided pointer.
/// The caller must ensure that the buffer is large enough to hold the frame.
unsafe fn capture(
  frame: &IDXGISurface1,
  dest: *mut u8,
  len: usize,
  texture_desc: &D3D11_TEXTURE2D_DESC,
) -> Result<()> {
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
