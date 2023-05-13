use crate::utils::Result;
use windows::Win32::Graphics::Dxgi::{DXGI_OUTDUPL_FRAME_INFO, DXGI_OUTPUT_DESC};

/// Capturer is stateful, it holds a buffer of the last captured frame.
pub trait Capturer {
  fn get_desc(&self) -> Result<DXGI_OUTPUT_DESC>;

  /// Get the buffer of the last captured frame.
  /// The buffer is in BGRA32 format.
  fn get_buffer(&self) -> &[u8];

  /// Check buffer size.
  fn check_buffer(&self) -> Result<()>;

  /// Capture the screen and return the frame info.
  /// The pixel data is stored in the buffer.
  fn capture(&mut self) -> Result<DXGI_OUTDUPL_FRAME_INFO>;

  /// Check buffer size before capture.
  fn safe_capture(&mut self) -> Result<DXGI_OUTDUPL_FRAME_INFO>;
}
