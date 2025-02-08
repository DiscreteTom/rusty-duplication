use crate::{Error, FrameInfoExt, Result};
use windows::{
  core::Interface,
  Win32::Graphics::{
    Direct3D11::{
      ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D, D3D11_BIND_FLAG, D3D11_CPU_ACCESS_READ,
      D3D11_RESOURCE_MISC_FLAG, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
    },
    Dxgi::{
      Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
      IDXGIOutput1, IDXGIOutputDuplication, IDXGIResource, IDXGISurface1, DXGI_OUTDUPL_DESC,
      DXGI_OUTDUPL_FRAME_INFO, DXGI_OUTDUPL_POINTER_SHAPE_INFO, DXGI_OUTPUT_DESC,
      DXGI_RESOURCE_PRIORITY_MAXIMUM,
    },
    Gdi::{GetMonitorInfoW, MONITORINFO},
  },
};

/// Monitor context for screen duplication.
/// This is stateless and immutable.
///
/// To create a new instance, use [`Scanner`](crate::Scanner).
#[derive(Debug, Clone)]
pub struct Monitor {
  device: ID3D11Device,
  device_context: ID3D11DeviceContext,
  output: IDXGIOutput1,
  output_duplication: IDXGIOutputDuplication,
}

impl Monitor {
  #[inline]
  pub(crate) fn new(
    device: ID3D11Device,
    device_context: ID3D11DeviceContext,
    output: IDXGIOutput1,
    output_duplication: IDXGIOutputDuplication,
  ) -> Self {
    Self {
      device,
      device_context,
      output,
      output_duplication,
    }
  }

  /// This is usually used to check if the monitor is primary.
  /// # Examples
  /// ```
  /// use rusty_duplication::{Scanner, MonitorInfoExt};
  ///
  /// let monitor = Scanner::new().unwrap().next().unwrap();
  /// monitor.monitor_info().unwrap().is_primary();
  /// ```
  pub fn monitor_info(&self) -> Result<MONITORINFO> {
    let h_monitor = self.dxgi_output_desc()?.Monitor;
    let mut info = MONITORINFO {
      cbSize: size_of::<MONITORINFO>() as u32,
      ..Default::default()
    };
    if unsafe { GetMonitorInfoW(h_monitor, &mut info).as_bool() } {
      Ok(info)
    } else {
      Err(Error::last_win_err(stringify!(GetMonitorInfoW)))
    }
  }

  /// This is usually used to get the screen's position and size.
  /// # Examples
  /// ```
  /// use rusty_duplication::{Scanner, OutputDescExt};
  ///
  /// let monitor = Scanner::new().unwrap().next().unwrap();
  /// let desc = monitor.dxgi_output_desc().unwrap();
  /// println!("{}x{}", desc.width(), desc.height());
  /// ```
  #[inline]
  pub fn dxgi_output_desc(&self) -> Result<DXGI_OUTPUT_DESC> {
    unsafe { self.output.GetDesc() }
      .map_err(Error::from_win_err(stringify!(DXGI_OUTPUT_DESC.GetDesc)))
  }

  /// This is usually used to get the screen's pixel width/height and buffer size.
  #[inline]
  pub fn dxgi_outdupl_desc(&self) -> DXGI_OUTDUPL_DESC {
    unsafe { self.output_duplication.GetDesc() }
  }

  pub(crate) fn create_readable_texture(
    &self,
  ) -> Result<(ID3D11Texture2D, DXGI_OUTDUPL_DESC, D3D11_TEXTURE2D_DESC)> {
    let dupl_desc = self.dxgi_outdupl_desc();
    let output_desc = self.dxgi_output_desc()?;

    // create a readable texture description
    let texture_desc = D3D11_TEXTURE2D_DESC {
      BindFlags: D3D11_BIND_FLAG::default().0 as u32,
      CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
      MiscFlags: D3D11_RESOURCE_MISC_FLAG::default().0 as u32,
      Usage: D3D11_USAGE_STAGING, // A resource that supports data transfer (copy) from the GPU to the CPU.
      Width: match output_desc.Rotation.0 {
        2 | 4 => dupl_desc.ModeDesc.Height,
        _ => dupl_desc.ModeDesc.Width,
      },
      Height: match output_desc.Rotation.0 {
        2 | 4 => dupl_desc.ModeDesc.Width,
        _ => dupl_desc.ModeDesc.Height,
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
    let mut readable_texture: Option<ID3D11Texture2D> = None;
    unsafe {
      self
        .device
        .CreateTexture2D(&texture_desc, None, Some(&mut readable_texture))
    }
    .map_err(Error::from_win_err(stringify!(
      ID3D11Device.CreateTexture2D
    )))?;
    let readable_texture = readable_texture.unwrap();
    // Lower priorities causes stuff to be needlessly copied from gpu to ram,
    // causing huge ram usage on some systems.
    // https://github.com/bryal/dxgcap-rs/blob/208d93368bc64aed783791242410459c878a10fb/src/lib.rs#L225
    unsafe { readable_texture.SetEvictionPriority(DXGI_RESOURCE_PRIORITY_MAXIMUM.0) };

    Ok((readable_texture, dupl_desc, texture_desc))
  }

  /// Try to process the next frame with the provided `cb`.
  fn process_next_frame<R>(
    &self,
    timeout_ms: u32,
    readable_texture: &ID3D11Texture2D,
    cb: impl FnOnce((IDXGISurface1, DXGI_OUTDUPL_FRAME_INFO)) -> R,
  ) -> Result<R> {
    // acquire GPU texture
    let mut frame_info = DXGI_OUTDUPL_FRAME_INFO::default();
    let mut resource: Option<IDXGIResource> = None;
    unsafe {
      self
        .output_duplication
        .AcquireNextFrame(timeout_ms, &mut frame_info, &mut resource)
    }
    .map_err(Error::from_win_err(stringify!(
      IDXGIOutputDuplication.AcquireNextFrame
    )))?;
    let texture: ID3D11Texture2D = resource.unwrap().cast().unwrap();

    // copy GPU texture to readable texture
    // TODO: is this needed?
    unsafe { self.device_context.CopyResource(readable_texture, &texture) };

    let r = cb((readable_texture.cast().unwrap(), frame_info));

    unsafe { self.output_duplication.ReleaseFrame() }.map_err(Error::from_win_err(stringify!(
      IDXGIOutputDuplication.ReleaseFrame
    )))?;

    Ok(r)
  }

  /// Get the next frame without pointer shape.
  ///
  /// To get the pointer shape, use [`Self::next_frame_with_pointer_shape`].
  pub fn next_frame(
    &self,
    timeout_ms: u32,
    readable_texture: &ID3D11Texture2D,
  ) -> Result<(IDXGISurface1, DXGI_OUTDUPL_FRAME_INFO)> {
    self.process_next_frame(timeout_ms, readable_texture, |r| r)
  }

  /// If mouse is updated, the `Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>` will be [`Some`].
  /// This will resize `pointer_shape_buffer` if needed and update it.
  pub fn next_frame_with_pointer_shape(
    &self,
    timeout_ms: u32,
    readable_texture: &ID3D11Texture2D,
    pointer_shape_buffer: &mut Vec<u8>,
  ) -> Result<(
    IDXGISurface1,
    DXGI_OUTDUPL_FRAME_INFO,
    Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>,
  )> {
    self
      .process_next_frame(timeout_ms, readable_texture, |(surface, frame_info)| {
        if !frame_info.pointer_shape_updated() {
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
        unsafe {
          self.output_duplication.GetFramePointerShape(
            pointer_shape_buffer.len() as u32,
            pointer_shape_buffer.as_mut_ptr() as *mut _,
            &mut size,
            &mut pointer_shape_info,
          )
        }
        .map_err(Error::from_win_err(stringify!(
          IDXGIOutputDuplication.GetFramePointerShape
        )))?;
        // fix buffer size
        pointer_shape_buffer.truncate(size as usize);

        Ok((surface, frame_info, Some(pointer_shape_info)))
      })
      .and_then(|r| r)
  }
}

#[cfg(test)]
mod tests {
  use crate::{MonitorInfoExt, Scanner};
  use serial_test::serial;

  #[test]
  #[serial]
  fn monitor() {
    let contexts = Scanner::new().unwrap().collect::<Vec<_>>();

    // make sure only one primary monitor
    let mut primary_monitor_count = 0;
    for c in &contexts {
      if c.monitor_info().unwrap().is_primary() {
        primary_monitor_count += 1;
      }
    }
    assert_eq!(primary_monitor_count, 1);
  }
}
