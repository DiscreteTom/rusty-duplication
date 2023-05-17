use std::result;
use windows::Win32::Graphics::Dxgi::DXGI_OUTDUPL_POINTER_SHAPE_INFO;

pub type Result<T> = result::Result<T, String>;

pub struct PointerShape {
  pub info: DXGI_OUTDUPL_POINTER_SHAPE_INFO,
  pub data: Vec<u8>,
}