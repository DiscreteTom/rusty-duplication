use crate::error::Error;
use crate::{model::Result, utils::FrameInfoExt};
use std::ptr;
use windows::Win32::Graphics::Dxgi::DXGI_OUTDUPL_DESC;
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MONITORINFO};
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

  pub fn monitor_info(&self) -> Result<MONITORINFO> {
    let h_monitor = self.dxgi_output_desc()?.Monitor;
    let mut info = MONITORINFO::default();
    info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
    if unsafe { GetMonitorInfoW(h_monitor, &mut info).as_bool() } {
      Ok(info)
    } else {
      Err(Error::new("GetMonitorInfoW"))
    }
  }

  /// This is usually used to get the screen's position and size.
  pub fn dxgi_output_desc(&self) -> Result<DXGI_OUTPUT_DESC> {
    let mut desc = DXGI_OUTPUT_DESC::default();
    unsafe { self.output.GetDesc(&mut desc) }
      .map_err(|e| Error::windows("DXGI_OUTPUT_DESC.GetDesc", e))?;
    Ok(desc)
  }

  /// This is usually used to get the screen's pixel width/height and buffer size.
  pub fn dxgi_outdupl_desc(&self) -> DXGI_OUTDUPL_DESC {
    let mut desc = DXGI_OUTDUPL_DESC::default();
    unsafe { self.output_duplication.GetDesc(&mut desc) };
    desc
  }

  pub fn create_readable_texture(
    &self,
  ) -> Result<(ID3D11Texture2D, DXGI_OUTDUPL_DESC, D3D11_TEXTURE2D_DESC)> {
    let dupl_desc = self.dxgi_outdupl_desc();
    let output_desc = self.dxgi_output_desc()?;

    // create a readable texture description
    let texture_desc = D3D11_TEXTURE2D_DESC {
      BindFlags: D3D11_BIND_FLAG::default(),
      CPUAccessFlags: D3D11_CPU_ACCESS_READ,
      MiscFlags: D3D11_RESOURCE_MISC_FLAG::default(),
      Usage: D3D11_USAGE_STAGING, // A resource that supports data transfer (copy) from the GPU to the CPU.
      Width: if output_desc.Rotation.0 == 2 || output_desc.Rotation.0 == 4 {
        dupl_desc.ModeDesc.Height
      } else {
        dupl_desc.ModeDesc.Width
      },
      Height: if output_desc.Rotation.0 == 2 || output_desc.Rotation.0 == 4 {
        dupl_desc.ModeDesc.Width
      } else {
        dupl_desc.ModeDesc.Height
      },
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
    .map_err(|e| Error::windows("CreateTexture2D", e))?;
    let readable_texture = readable_texture.unwrap();
    // Lower priorities causes stuff to be needlessly copied from gpu to ram,
    // causing huge ram usage on some systems.
    // https://github.com/bryal/dxgcap-rs/blob/208d93368bc64aed783791242410459c878a10fb/src/lib.rs#L225
    unsafe { readable_texture.SetEvictionPriority(DXGI_RESOURCE_PRIORITY_MAXIMUM.0) };

    Ok((readable_texture, dupl_desc, texture_desc))
  }

  fn acquire_next_frame(
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
    .map_err(|e| Error::windows("AcquireNextFrame", e))?;
    let texture: ID3D11Texture2D = resource.unwrap().cast().unwrap();

    // copy GPU texture to readable texture
    unsafe { self.device_context.CopyResource(readable_texture, &texture) };

    Ok((readable_texture.cast().unwrap(), frame_info))
  }

  fn release_frame(&self) -> Result<()> {
    unsafe { self.output_duplication.ReleaseFrame() }.map_err(|e| Error::windows("ReleaseFrame", e))
  }

  pub fn next_frame(
    &self,
    readable_texture: &ID3D11Texture2D,
  ) -> Result<(IDXGISurface1, DXGI_OUTDUPL_FRAME_INFO)> {
    let (surface, frame_info) = self.acquire_next_frame(readable_texture)?;
    self.release_frame()?;
    Ok((surface, frame_info))
  }

  /// If mouse is updated, the `Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>` is `Some`.
  /// and this will resize `pointer_shape_buffer` if needed and update it.
  pub fn next_frame_with_pointer_shape(
    &self,
    readable_texture: &ID3D11Texture2D,
    pointer_shape_buffer: &mut Vec<u8>,
  ) -> Result<(
    IDXGISurface1,
    DXGI_OUTDUPL_FRAME_INFO,
    Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>,
  )> {
    let (surface, frame_info) = self.acquire_next_frame(readable_texture)?;

    if !frame_info.mouse_updated().shape_updated {
      self.release_frame()?;
      return Ok((surface, frame_info, None));
    }

    // resize buffer if needed
    let pointer_shape_buffer_size = frame_info.PointerShapeBufferSize as usize;
    if pointer_shape_buffer.len() < pointer_shape_buffer_size {
      pointer_shape_buffer.resize(pointer_shape_buffer_size, 0);
    }

    // get pointer shape
    let mut size: u32 = 0;
    let mut pointer_shape_info = DXGI_OUTDUPL_POINTER_SHAPE_INFO::default();
    match unsafe {
      self
        .output_duplication
        .GetFramePointerShape(
          pointer_shape_buffer.len() as u32,
          pointer_shape_buffer.as_mut_ptr() as *mut _,
          &mut size,
          &mut pointer_shape_info,
        )
        .map_err(|e| Error::windows("GetFramePointerShape", e))
    } {
      Ok(_) => {
        self.release_frame()?;
        Ok((surface, frame_info, Some(pointer_shape_info)))
      }
      Err(e) => {
        self.release_frame()?;
        return Err(e);
      }
    }
  }

  pub fn capture(
    &self,
    dest: *mut u8,
    len: usize,
    readable_texture: &ID3D11Texture2D,
    texture_desc: &D3D11_TEXTURE2D_DESC,
  ) -> Result<DXGI_OUTDUPL_FRAME_INFO> {
    let (frame, frame_info) = self.next_frame(readable_texture)?;
    let mut mapped_surface = DXGI_MAPPED_RECT::default();
    let line_bytes = texture_desc.Width as usize * 4;

    unsafe {
      frame
        .Map(&mut mapped_surface, DXGI_MAP_READ)
        .map_err(|e| Error::windows("Map", e))?;
      if mapped_surface.Pitch as usize == line_bytes {
        ptr::copy_nonoverlapping(mapped_surface.pBits, dest, len);
      } else {
        // https://github.com/DiscreteTom/rusty-duplication/issues/7
        for i in 0..texture_desc.Height {
          let src = mapped_surface
            .pBits
            .offset((i * mapped_surface.Pitch as u32) as isize);
          let dest = dest.offset((i * line_bytes as u32) as isize);
          ptr::copy_nonoverlapping(src, dest, mapped_surface.Pitch as usize);
        }
      }
      frame.Unmap().map_err(|e| Error::windows("Unmap", e))?;
    }

    Ok(frame_info)
  }

  /// If mouse is updated, the `Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>` is `Some`.
  /// and this will resize `pointer_shape_buffer` if needed and update it.
  pub fn capture_with_pointer_shape(
    &self,
    dest: *mut u8,
    len: usize,
    readable_texture: &ID3D11Texture2D,
    texture_desc: &D3D11_TEXTURE2D_DESC,
    pointer_shape_buffer: &mut Vec<u8>,
  ) -> Result<(
    DXGI_OUTDUPL_FRAME_INFO,
    Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>,
  )> {
    let (frame, frame_info, pointer_shape_info) =
      self.next_frame_with_pointer_shape(readable_texture, pointer_shape_buffer)?;
    let mut mapped_surface = DXGI_MAPPED_RECT::default();
    let line_bytes = texture_desc.Width as usize * 4;

    unsafe {
      frame
        .Map(&mut mapped_surface, DXGI_MAP_READ)
        .map_err(|e| Error::windows("Map", e))?;
      if mapped_surface.Pitch as usize == line_bytes {
        ptr::copy_nonoverlapping(mapped_surface.pBits, dest, len);
      } else {
        // https://github.com/DiscreteTom/rusty-duplication/issues/7
        for i in 0..texture_desc.Height {
          let src = mapped_surface
            .pBits
            .offset((i * mapped_surface.Pitch as u32) as isize);
          let dest = dest.offset((i * line_bytes as u32) as isize);
          ptr::copy_nonoverlapping(src, dest, mapped_surface.Pitch as usize);
        }
      }
      frame.Unmap().map_err(|e| Error::windows("Unmap", e))?;
    }

    Ok((frame_info, pointer_shape_info))
  }
}

#[cfg(test)]
mod tests {
  use std::{thread, time::Duration};

  use crate::{
    manager::Manager,
    utils::{FrameInfoExt, MonitorInfoExt, OutDuplDescExt},
  };

  #[test]
  fn duplication_context() {
    let manager = Manager::default().unwrap();
    assert_ne!(manager.contexts.len(), 0);

    // make sure only one primary monitor
    let mut primary_monitor_count = 0;
    for c in &manager.contexts {
      if c.monitor_info().unwrap().is_primary() {
        primary_monitor_count += 1;
      }
    }
    assert_eq!(primary_monitor_count, 1);

    let (texture, desc, texture_desc) = manager.contexts[0].create_readable_texture().unwrap();
    let mut buffer = vec![0u8; desc.calc_buffer_size()];

    // sleep for a while before capture to wait system to update the screen
    thread::sleep(Duration::from_millis(100));

    let info = manager.contexts[0]
      .capture(buffer.as_mut_ptr(), buffer.len(), &texture, &texture_desc)
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

    // sleep for a while before capture to wait system to update the mouse
    thread::sleep(Duration::from_millis(1000));

    // check pointer
    let mut pointer_shape_buffer = vec![0u8; info.PointerShapeBufferSize as usize];
    let (frame_info, pointer_shape_info) = manager.contexts[0]
      .capture_with_pointer_shape(
        buffer.as_mut_ptr(),
        buffer.len(),
        &texture,
        &texture_desc,
        &mut pointer_shape_buffer,
      )
      .unwrap();
    assert!(frame_info.mouse_updated().position_updated);
    assert!(pointer_shape_info.is_some());

    // ensure pointer_shape_buffer not all zero
    let mut all_zero = true;
    for i in 0..pointer_shape_buffer.len() {
      if pointer_shape_buffer[i] != 0 {
        all_zero = false;
        break;
      }
    }
    assert!(!all_zero);
  }
}
