use std::result;

use windows::Win32::Graphics::Dxgi::DXGI_OUTPUT_DESC;

pub trait Dimension {
  fn width(&self) -> u32;
  fn height(&self) -> u32;
}

impl Dimension for DXGI_OUTPUT_DESC {
  fn width(&self) -> u32 {
    (self.DesktopCoordinates.right - self.DesktopCoordinates.left) as u32
  }
  fn height(&self) -> u32 {
    (self.DesktopCoordinates.bottom - self.DesktopCoordinates.top) as u32
  }
}

pub type Result<T> = result::Result<T, &'static str>;

/// Return needed buffer size, in bytes.
pub fn calc_buffer_size(desc: DXGI_OUTPUT_DESC) -> usize {
  (desc.width() * desc.height() * 4) as usize // 4 for BGRA32
}
