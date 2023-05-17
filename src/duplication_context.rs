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
      DXGI_MAP_READ, DXGI_OUTDUPL_FRAME_INFO, DXGI_OUTDUPL_POINTER_SHAPE_INFO, DXGI_OUTPUT_DESC,
      DXGI_RESOURCE_PRIORITY_MAXIMUM,
    },
  },
};

use crate::model::Result;
use crate::utils::OutputDescExt;

/// Stateless.
pub struct DuplicationContext {
  device: ID3D11Device,
  device_context: ID3D11DeviceContext,
  timeout_ms: u32,
  output: IDXGIOutput1,
  output_duplication: IDXGIOutputDuplication,
}

impl DuplicationContext {
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
    unsafe { self.output.GetDesc(&mut desc) }.map_err(|e| format!("GetDesc failed: {:?}", e))?;
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
    .map_err(|e| format!("CreateTexture2D failed: {:?}", e))?;
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
    .map_err(|e| format!("AcquireNextFrame failed: {:?}", e))?;
    let texture: ID3D11Texture2D = resource.unwrap().cast().unwrap();

    // copy GPU texture to readable texture
    unsafe { self.device_context.CopyResource(readable_texture, &texture) };

    // release GPU texture
    unsafe { self.output_duplication.ReleaseFrame() }
      .map_err(|e| format!("ReleaseFrame failed: {:?}", e))?;

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
        .map_err(|e| format!("Map failed: {:?}", e))?;
      ptr::copy_nonoverlapping(mapped_surface.pBits, dest, len);
      frame
        .Unmap()
        .map_err(|e| format!("Unmap failed: {:?}", e))?;
    }

    Ok(info)
  }

  pub fn get_pointer(
    &self,
    info: &DXGI_OUTDUPL_FRAME_INFO,
  ) -> Result<(Vec<u8>, u32, DXGI_OUTDUPL_POINTER_SHAPE_INFO)> {
    let mut buffer = vec![0u8; info.PointerShapeBufferSize as usize];
    let mut size: u32 = 0;
    let mut shape_info = DXGI_OUTDUPL_POINTER_SHAPE_INFO::default();
    unsafe {
      self
        .output_duplication
        .GetFramePointerShape(
          info.PointerShapeBufferSize,
          buffer.as_mut_ptr() as *mut _,
          &mut size,
          &mut shape_info,
        )
        .map_err(|e| format!("GetFramePointerShape failed: {:?}", e))?;
    }
    return Ok((buffer, size, shape_info));
  }
}

#[cfg(test)]
mod tests {
  use std::{thread, time::Duration};

  use crate::{
    manager::Manager,
    utils::{FrameInfoExt, OutputDescExt},
  };

  #[test]
  fn duplication_context() {
    let manager = Manager::default().unwrap();
    assert_ne!(manager.contexts.len(), 0);

    let (texture, desc) = manager.contexts[0].create_readable_texture().unwrap();
    let mut buffer = vec![0u8; desc.calc_buffer_size()];

    // sleep for a while before capture to wait system to update the screen
    thread::sleep(Duration::from_millis(100));

    let info = manager.contexts[0]
      .capture_frame(buffer.as_mut_ptr(), buffer.len(), &texture)
      .unwrap();
    assert!(info.desktop_updated());

    // ensure buffer not all zero
    let mut all_zero = true;
    for i in 0..buffer.len() {
      if buffer[i] != 0 {
        all_zero = false;
        break;
      }
    }
    assert!(!all_zero);
  }
}
