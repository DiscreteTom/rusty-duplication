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

  /// Check buffer size.
  fn check_buffer(&self) -> Result<()>;

  /// Get the buffer of the captured pointer shape.
  fn pointer_shape_buffer(&self) -> &[u8];

  /// Return true if the pointer shape is updated.
  fn pointer_shape_updated(&self) -> bool;

  /// Capture the screen and return the frame info.
  /// The pixel data is stored in the `buffer`.
  fn capture(&mut self) -> Result<DXGI_OUTDUPL_FRAME_INFO>;

  /// Check buffer size before capture.
  /// The pixel data is stored in the `buffer`.
  fn safe_capture(&mut self) -> Result<DXGI_OUTDUPL_FRAME_INFO>;

  /// Capture the screen and return the frame info.
  /// The pixel data is stored in the `buffer`.
  /// If mouse is updated, the `Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>` is Some.
  /// The pointer shape is stored in the `pointer_shape_buffer`.
  fn capture_with_pointer_shape(
    &mut self,
  ) -> Result<(
    DXGI_OUTDUPL_FRAME_INFO,
    Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>,
  )>;

  /// Check buffer size before capture.
  /// The pixel data is stored in the `buffer`.
  /// If mouse is updated, the `Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>` is Some.
  /// The pointer shape is stored in the `pointer_shape_buffer`.
  fn safe_capture_with_pointer_shape(
    &mut self,
  ) -> Result<(
    DXGI_OUTDUPL_FRAME_INFO,
    Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>,
  )>;
}
