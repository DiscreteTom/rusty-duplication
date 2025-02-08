use windows::Win32::Graphics::{
  Dxgi::{DXGI_OUTDUPL_DESC, DXGI_OUTDUPL_FRAME_INFO, DXGI_OUTPUT_DESC},
  Gdi::MONITORINFO,
};

pub trait OutputDescExt {
  fn width(&self) -> u32;
  fn height(&self) -> u32;
}

impl OutputDescExt for DXGI_OUTPUT_DESC {
  #[inline]
  fn width(&self) -> u32 {
    (self.DesktopCoordinates.right - self.DesktopCoordinates.left) as u32
  }
  #[inline]
  fn height(&self) -> u32 {
    (self.DesktopCoordinates.bottom - self.DesktopCoordinates.top) as u32
  }
}

pub trait OutDuplDescExt {
  fn calc_buffer_size(&self) -> usize;
}

impl OutDuplDescExt for DXGI_OUTDUPL_DESC {
  /// Return needed buffer size, in bytes.
  #[inline]
  fn calc_buffer_size(&self) -> usize {
    (self.ModeDesc.Width * self.ModeDesc.Height * 4) as usize // 4 for BGRA32
  }
}

pub trait FrameInfoExt {
  fn desktop_updated(&self) -> bool;
  /// Return `true` if the position or shape of the mouse was updated.
  fn mouse_updated(&self) -> bool;
  fn pointer_shape_updated(&self) -> bool;
}

impl FrameInfoExt for DXGI_OUTDUPL_FRAME_INFO {
  #[inline]
  fn desktop_updated(&self) -> bool {
    self.LastPresentTime != 0
  }

  #[inline]
  fn mouse_updated(&self) -> bool {
    self.LastMouseUpdateTime != 0
  }

  #[inline]
  fn pointer_shape_updated(&self) -> bool {
    self.PointerShapeBufferSize > 0
  }
}

pub trait MonitorInfoExt {
  fn is_primary(&self) -> bool;
}

impl MonitorInfoExt for MONITORINFO {
  #[inline]
  fn is_primary(&self) -> bool {
    self.dwFlags == 0x01 // MONITORINFOF_PRIMARY
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use windows::Win32::Graphics::{
    Dxgi::{DXGI_OUTDUPL_DESC, DXGI_OUTDUPL_FRAME_INFO, DXGI_OUTPUT_DESC},
    Gdi::MONITORINFO,
  };

  #[test]
  fn output_desc_ext() {
    let mut desc = DXGI_OUTPUT_DESC::default();
    desc.DesktopCoordinates.left = 0;
    desc.DesktopCoordinates.top = 0;
    desc.DesktopCoordinates.right = 1920;
    desc.DesktopCoordinates.bottom = 1080;
    assert_eq!(desc.width(), 1920);
    assert_eq!(desc.height(), 1080);
  }

  #[test]
  fn out_dupl_desc_ext() {
    let mut desc = DXGI_OUTDUPL_DESC::default();
    desc.ModeDesc.Width = 1920;
    desc.ModeDesc.Height = 1080;
    assert_eq!(desc.calc_buffer_size(), 1920 * 1080 * 4);
  }

  #[test]
  fn frame_info_ext() {
    let mut desc = DXGI_OUTDUPL_FRAME_INFO::default();
    assert!(!desc.desktop_updated());
    desc.LastPresentTime = 1;
    assert!(desc.desktop_updated());
    assert!(!desc.mouse_updated());
    desc.LastMouseUpdateTime = 1;
    assert!(desc.mouse_updated());
    assert!(!desc.pointer_shape_updated());
    desc.PointerShapeBufferSize = 1;
    assert!(desc.pointer_shape_updated());
  }

  #[test]
  fn monitor_info_ext() {
    let mut info = MONITORINFO::default();
    assert!(!info.is_primary());
    info.dwFlags = 0x01;
    assert!(info.is_primary());
  }
}
