use std::ptr;

use windows::{
  core::ComInterface,
  Win32::Graphics::{
    Direct3D11::{
      ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D, D3D11_BIND_FLAG, D3D11_CPU_ACCESS_READ,
      D3D11_RESOURCE_MISC_FLAG, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
    },
    Dxgi::{
      Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
      IDXGIOutput1, IDXGIOutputDuplication, IDXGIResource, IDXGISurface1, DXGI_MAPPED_RECT,
      DXGI_MAP_READ, DXGI_OUTDUPL_FRAME_INFO, DXGI_OUTPUT_DESC, DXGI_RESOURCE_PRIORITY_MAXIMUM,
    },
  },
};

use crate::utils::OutputDescExt;
use crate::utils::Result;

/// Stateless.
pub struct DuplicateContext {
  device: ID3D11Device,
  device_context: ID3D11DeviceContext,
  timeout_ms: u32,
  output: IDXGIOutput1,
  output_duplication: IDXGIOutputDuplication,
}

impl DuplicateContext {
  pub fn new(
    device: ID3D11Device,
    device_context: ID3D11DeviceContext,
    output: IDXGIOutput1,
    output_duplication: IDXGIOutputDuplication,
    timeout_ms: u32,
  ) -> Self {
    Self {
      device,
      device_context,
      timeout_ms,
      output,
      output_duplication,
    }
  }

  pub fn desc(&self) -> Result<DXGI_OUTPUT_DESC> {
    let mut desc = DXGI_OUTPUT_DESC::default();
    unsafe { self.output.GetDesc(&mut desc) }.map_err(|_| "GetDesc failed")?;
    Ok(desc)
  }

  pub fn create_readable_texture(&self) -> Result<(ID3D11Texture2D, DXGI_OUTPUT_DESC)> {
    let desc = self.desc()?;

    // create a readable texture description
    let texture_desc = D3D11_TEXTURE2D_DESC {
      BindFlags: D3D11_BIND_FLAG::default(),
      CPUAccessFlags: D3D11_CPU_ACCESS_READ,
      MiscFlags: D3D11_RESOURCE_MISC_FLAG::default(),
      Usage: D3D11_USAGE_STAGING, // A resource that supports data transfer (copy) from the GPU to the CPU.
      Width: desc.width(),
      Height: desc.height(),
      MipLevels: 1,
      ArraySize: 1,
      Format: DXGI_FORMAT_B8G8R8A8_UNORM,
      SampleDesc: DXGI_SAMPLE_DESC {
        Count: 1,
        Quality: 0,
      },
    };

    // create a readable texture in GPU memory
    let mut readable_texture: Option<ID3D11Texture2D> = None.clone();
    unsafe {
      self
        .device
        .CreateTexture2D(&texture_desc, None, Some(&mut readable_texture))
    }
    .map_err(|_| "CreateTexture2D failed")?;
    let readable_texture = readable_texture.unwrap();
    // Lower priorities causes stuff to be needlessly copied from gpu to ram,
    // causing huge ram usage on some systems.
    // https://github.com/bryal/dxgcap-rs/blob/208d93368bc64aed783791242410459c878a10fb/src/lib.rs#L225
    unsafe { readable_texture.SetEvictionPriority(DXGI_RESOURCE_PRIORITY_MAXIMUM.0) };

    Ok((readable_texture, desc))
  }

  pub fn acquire_next_frame(
    &self,
    readable_texture: &ID3D11Texture2D,
  ) -> Result<(IDXGISurface1, DXGI_OUTDUPL_FRAME_INFO)> {
    // acquire GPU texture
    let mut frame_info = DXGI_OUTDUPL_FRAME_INFO::default();
    let mut resource: Option<IDXGIResource> = None.clone();
    unsafe {
      self
        .output_duplication
        .AcquireNextFrame(self.timeout_ms, &mut frame_info, &mut resource)
    }
    .map_err(|_| "AcquireNextFrame failed")?;
    let texture: ID3D11Texture2D = resource.unwrap().cast().unwrap();

    // copy GPU texture to readable texture
    unsafe { self.device_context.CopyResource(readable_texture, &texture) };

    // release GPU texture
    unsafe { self.output_duplication.ReleaseFrame() }.map_err(|_| "ReleaseFrame failed")?;

    Ok((readable_texture.cast().unwrap(), frame_info))
  }

  pub fn capture_frame(
    &self,
    dest: *mut u8,
    len: usize,
    readable_texture: &ID3D11Texture2D,
  ) -> Result<DXGI_OUTDUPL_FRAME_INFO> {
    let (frame, info) = self.acquire_next_frame(readable_texture)?;
    let mut mapped_surface = DXGI_MAPPED_RECT::default();

    unsafe {
      frame
        .Map(&mut mapped_surface, DXGI_MAP_READ)
        .map_err(|_| "Map failed")?;
      ptr::copy_nonoverlapping(mapped_surface.pBits, dest, len);
      frame.Unmap().map_err(|_| "Unmap failed")?;
    }

    Ok(info)
  }
}
