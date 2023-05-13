use std::result;
use windows::Win32::Graphics::Dxgi::{DXGI_OUTDUPL_FRAME_INFO, DXGI_OUTPUT_DESC};

pub type Result<T> = result::Result<T, &'static str>;

pub trait OutputDescExt {
  fn width(&self) -> u32;
  fn height(&self) -> u32;
  fn calc_buffer_size(&self) -> usize;
}

impl OutputDescExt for DXGI_OUTPUT_DESC {
  fn width(&self) -> u32 {
    (self.DesktopCoordinates.right - self.DesktopCoordinates.left) as u32
  }
  fn height(&self) -> u32 {
    (self.DesktopCoordinates.bottom - self.DesktopCoordinates.top) as u32
  }

  /// Return needed buffer size, in bytes.
  fn calc_buffer_size(&self) -> usize {
    (self.width() * self.height() * 4) as usize // 4 for BGRA32
  }
}

pub trait FrameInfoExt {
  fn is_new_frame(&self) -> bool;
}

impl FrameInfoExt for DXGI_OUTDUPL_FRAME_INFO {
  fn is_new_frame(&self) -> bool {
    self.LastPresentTime > 0
  }
}
