use crate::model::Result;
use windows::Win32::Graphics::Dxgi::{
  DXGI_OUTDUPL_FRAME_INFO, DXGI_OUTDUPL_POINTER_SHAPE_INFO, DXGI_OUTPUT_DESC,
};

/// Capturer is stateful, it holds a buffer of the last captured frame.
pub trait Capturer {
  fn desc(&self) -> Result<DXGI_OUTPUT_DESC>;

  /// Get the buffer of the last captured frame.
  /// The buffer is in BGRA32 format.
  fn buffer(&self) -> &[u8];

  /// Get the buffer of the last captured frame.
  /// The buffer is in BGRA32 format.
  fn buffer_mut(&mut self) -> &mut [u8];

  fn pointer_shape_buffer(&self) -> &[u8];

  /// Capture the screen and return the frame info.
  /// The pixel data is stored in the buffer.
  fn capture(&mut self) -> Result<DXGI_OUTDUPL_FRAME_INFO>;

  fn capture_with_pointer_shape(
    &mut self,
  ) -> Result<(
    DXGI_OUTDUPL_FRAME_INFO,
    Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>,
  )>;
}
