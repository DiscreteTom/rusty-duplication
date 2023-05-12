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

  pub fn get_desc(&self) -> DXGI_OUTPUT_DESC {
    unsafe {
      let mut desc = DXGI_OUTPUT_DESC::default();
      self.output.GetDesc(&mut desc).unwrap();
      desc
    }
  }

  pub fn acquire_next_frame(&self, width: u32, height: u32) -> IDXGISurface1 {
    unsafe {
      // create a readable texture description
      let texture_desc = D3D11_TEXTURE2D_DESC {
        BindFlags: D3D11_BIND_FLAG::default(),
        CPUAccessFlags: D3D11_CPU_ACCESS_READ,
        MiscFlags: D3D11_RESOURCE_MISC_FLAG::default(),
        Usage: D3D11_USAGE_STAGING, // A resource that supports data transfer (copy) from the GPU to the CPU.
        Width: width,
        Height: height,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC {
          Count: 1,
          Quality: 0,
        },
      };

      // copy a readable texture in GPU memory
      let mut readable_texture: Option<ID3D11Texture2D> = None.clone();
      self
        .device
        .CreateTexture2D(&texture_desc, None, Some(&mut readable_texture))
        .unwrap(); // TODO: cache this texture
      let readable_texture = readable_texture.unwrap();
      // Lower priorities causes stuff to be needlessly copied from gpu to ram,
      // causing huge ram usage on some systems.
      // https://github.com/bryal/dxgcap-rs/blob/208d93368bc64aed783791242410459c878a10fb/src/lib.rs#L225
      readable_texture.SetEvictionPriority(DXGI_RESOURCE_PRIORITY_MAXIMUM.0);

      // acquire GPU texture
      let mut frame_info = DXGI_OUTDUPL_FRAME_INFO::default();
      let mut resource: Option<IDXGIResource> = None.clone();
      self
        .output_duplication
        .AcquireNextFrame(self.timeout_ms, &mut frame_info, &mut resource)
        .unwrap();
      let texture: ID3D11Texture2D = resource.unwrap().cast().unwrap();

      // copy GPU texture to readable texture
      self
        .device_context
        .CopyResource(&readable_texture, &texture);

      // release GPU texture
      self.output_duplication.ReleaseFrame().unwrap();

      readable_texture.cast().unwrap()
    }
  }

  pub fn capture_frame(&self, dest: *mut u8, width: u32, height: u32) {
    unsafe {
      let frame = self.acquire_next_frame(width, height);
      let mut mapped_surface = DXGI_MAPPED_RECT::default();
      frame.Map(&mut mapped_surface, DXGI_MAP_READ).unwrap();

      ptr::copy_nonoverlapping(mapped_surface.pBits, dest, (width * height * 4) as usize); // 4 for BGRA

      frame.Unmap().unwrap();
    }
  }
}
