use crate::{Error, FrameInfoExt, Result};
use windows::{
  core::Interface,
  Win32::{
    Foundation::RECT,
    Graphics::{
      Direct3D11::{
        ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D, D3D11_BIND_FLAG, D3D11_BOX,
        D3D11_CPU_ACCESS_READ, D3D11_RESOURCE_MISC_FLAG, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
      },
      Dxgi::{
        Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
        IDXGIOutput1, IDXGIOutputDuplication, IDXGIResource, DXGI_OUTDUPL_DESC,
        DXGI_OUTDUPL_FRAME_INFO, DXGI_OUTDUPL_MOVE_RECT, DXGI_OUTDUPL_POINTER_SHAPE_INFO,
        DXGI_OUTPUT_DESC, DXGI_RESOURCE_PRIORITY_MAXIMUM,
      },
      Gdi::{GetMonitorInfoW, MONITORINFO},
    },
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

  pub(crate) fn create_texture(
    &self,
    dupl_desc: &DXGI_OUTDUPL_DESC,
    output_desc: &DXGI_OUTPUT_DESC,
  ) -> Result<(ID3D11Texture2D, D3D11_TEXTURE2D_DESC)> {
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

    // create a texture in GPU memory
    let mut texture: Option<ID3D11Texture2D> = None;
    unsafe {
      self
        .device
        .CreateTexture2D(&texture_desc, None, Some(&mut texture))
    }
    .map_err(Error::from_win_err(stringify!(
      ID3D11Device.CreateTexture2D
    )))?;
    let texture = texture.unwrap();
    // Lower priorities causes stuff to be needlessly copied from gpu to ram,
    // causing huge ram usage on some systems.
    // https://github.com/bryal/dxgcap-rs/blob/208d93368bc64aed783791242410459c878a10fb/src/lib.rs#L225
    unsafe { texture.SetEvictionPriority(DXGI_RESOURCE_PRIORITY_MAXIMUM.0) };

    Ok((texture, texture_desc))
  }

  /// Try to process the next frame with the provided `cb`.
  fn process_next_frame<R>(
    &self,
    timeout_ms: u32,
    texture: &ID3D11Texture2D,
    cb: impl FnOnce(DXGI_OUTDUPL_FRAME_INFO) -> R,
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
    let new_texture: ID3D11Texture2D = resource.unwrap().cast().unwrap();

    // copy texture using GPU
    unsafe { self.device_context.CopyResource(texture, &new_texture) };

    let r = cb(frame_info);

    unsafe { self.output_duplication.ReleaseFrame() }.map_err(Error::from_win_err(stringify!(
      IDXGIOutputDuplication.ReleaseFrame
    )))?;

    Ok(r)
  }

  /// Try to process the next frame with delta updates (only copying dirty and moved regions).
  /// If `use_delta` is false or this is the first frame, does a full copy.
  fn process_next_frame_delta<R>(
    &self,
    timeout_ms: u32,
    texture: &ID3D11Texture2D,
    use_delta: bool,
    cb: impl FnOnce(DXGI_OUTDUPL_FRAME_INFO) -> R,
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
    let new_texture: ID3D11Texture2D = resource.unwrap().cast().unwrap();

    if use_delta {
      // Get dirty rectangles
      let dirty_rects = self.get_frame_dirty_rects()?;
      
      // Get move rectangles
      let move_rects = self.get_frame_move_rects()?;

      // Copy dirty regions
      for rect in &dirty_rects {
        self.copy_subresource_region(texture, &new_texture, rect);
      }

      // Copy moved regions
      // For move rects, we need to copy from the source position to the destination position
      for move_rect in &move_rects {
        // Copy from source position
        let src_rect = RECT {
          left: move_rect.SourcePoint.x,
          top: move_rect.SourcePoint.y,
          right: move_rect.SourcePoint.x + (move_rect.DestinationRect.right - move_rect.DestinationRect.left),
          bottom: move_rect.SourcePoint.y + (move_rect.DestinationRect.bottom - move_rect.DestinationRect.top),
        };
        
        // Copy to destination position
        self.copy_subresource_region_to_dest(
          texture,
          &new_texture,
          &src_rect,
          move_rect.DestinationRect.left as u32,
          move_rect.DestinationRect.top as u32,
        );
      }
    } else {
      // Do full copy for first frame or when delta is disabled
      unsafe { self.device_context.CopyResource(texture, &new_texture) };
    }

    let r = cb(frame_info);

    unsafe { self.output_duplication.ReleaseFrame() }.map_err(Error::from_win_err(stringify!(
      IDXGIOutputDuplication.ReleaseFrame
    )))?;

    Ok(r)
  }

  /// Get dirty rectangles for the current frame.
  fn get_frame_dirty_rects(&self) -> Result<Vec<RECT>> {
    // First call to get the required buffer size
    let mut required_size: u32 = 0;
    unsafe {
      self.output_duplication.GetFrameDirtyRects(
        0,
        std::ptr::null_mut(),
        &mut required_size,
      )
    }
    .map_err(Error::from_win_err(stringify!(
      IDXGIOutputDuplication.GetFrameDirtyRects
    )))?;

    // If there are no dirty rects, return empty vector
    if required_size == 0 {
      return Ok(Vec::new());
    }

    // Allocate buffer and get dirty rects
    let rect_count = (required_size as usize) / std::mem::size_of::<RECT>();
    let mut dirty_rects = vec![RECT::default(); rect_count];
    
    unsafe {
      self.output_duplication.GetFrameDirtyRects(
        required_size,
        dirty_rects.as_mut_ptr(),
        &mut required_size,
      )
    }
    .map_err(Error::from_win_err(stringify!(
      IDXGIOutputDuplication.GetFrameDirtyRects
    )))?;

    Ok(dirty_rects)
  }

  /// Get move rectangles for the current frame.
  fn get_frame_move_rects(&self) -> Result<Vec<DXGI_OUTDUPL_MOVE_RECT>> {
    // First call to get the required buffer size
    let mut required_size: u32 = 0;
    unsafe {
      self.output_duplication.GetFrameMoveRects(
        0,
        std::ptr::null_mut(),
        &mut required_size,
      )
    }
    .map_err(Error::from_win_err(stringify!(
      IDXGIOutputDuplication.GetFrameMoveRects
    )))?;

    // If there are no move rects, return empty vector
    if required_size == 0 {
      return Ok(Vec::new());
    }

    // Allocate buffer and get move rects
    let rect_count = (required_size as usize) / std::mem::size_of::<DXGI_OUTDUPL_MOVE_RECT>();
    let mut move_rects = vec![DXGI_OUTDUPL_MOVE_RECT::default(); rect_count];
    
    unsafe {
      self.output_duplication.GetFrameMoveRects(
        required_size,
        move_rects.as_mut_ptr(),
        &mut required_size,
      )
    }
    .map_err(Error::from_win_err(stringify!(
      IDXGIOutputDuplication.GetFrameMoveRects
    )))?;

    Ok(move_rects)
  }

  /// Copy a subresource region from source to destination texture.
  fn copy_subresource_region(&self, dst: &ID3D11Texture2D, src: &ID3D11Texture2D, rect: &RECT) {
    let src_box = D3D11_BOX {
      left: rect.left as u32,
      top: rect.top as u32,
      front: 0,
      right: rect.right as u32,
      bottom: rect.bottom as u32,
      back: 1,
    };

    unsafe {
      self.device_context.CopySubresourceRegion(
        dst,
        0, // DstSubresource
        rect.left as u32, // DstX
        rect.top as u32,  // DstY
        0,                // DstZ
        src,
        0,                // SrcSubresource
        Some(&src_box),
      );
    }
  }

  /// Copy a subresource region from source to a specific destination position.
  fn copy_subresource_region_to_dest(
    &self,
    dst: &ID3D11Texture2D,
    src: &ID3D11Texture2D,
    src_rect: &RECT,
    dst_x: u32,
    dst_y: u32,
  ) {
    let src_box = D3D11_BOX {
      left: src_rect.left as u32,
      top: src_rect.top as u32,
      front: 0,
      right: src_rect.right as u32,
      bottom: src_rect.bottom as u32,
      back: 1,
    };

    unsafe {
      self.device_context.CopySubresourceRegion(
        dst,
        0,      // DstSubresource
        dst_x,  // DstX
        dst_y,  // DstY
        0,      // DstZ
        src,
        0,      // SrcSubresource
        Some(&src_box),
      );
    }
  }

  /// Get the next frame without pointer shape.
  ///
  /// To get the pointer shape, use [`Self::next_frame_with_pointer_shape`].
  #[inline]
  pub(crate) fn next_frame(
    &self,
    timeout_ms: u32,
    texture: &ID3D11Texture2D,
  ) -> Result<DXGI_OUTDUPL_FRAME_INFO> {
    self.process_next_frame(timeout_ms, texture, |r| r)
  }

  /// Get the next frame without pointer shape, using delta updates.
  /// If `use_delta` is false, does a full copy.
  #[inline]
  pub(crate) fn next_frame_delta(
    &self,
    timeout_ms: u32,
    texture: &ID3D11Texture2D,
    use_delta: bool,
  ) -> Result<DXGI_OUTDUPL_FRAME_INFO> {
    self.process_next_frame_delta(timeout_ms, texture, use_delta, |r| r)
  }

  /// If the pointer shape is updated, the `Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>` will be [`Some`].
  /// This will resize `pointer_shape_buffer` if needed and update it.
  pub(crate) fn next_frame_with_pointer_shape(
    &self,
    timeout_ms: u32,
    texture: &ID3D11Texture2D,
    pointer_shape_buffer: &mut Vec<u8>,
  ) -> Result<(
    DXGI_OUTDUPL_FRAME_INFO,
    Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>,
  )> {
    self
      .process_next_frame(timeout_ms, texture, |frame_info| {
        if !frame_info.pointer_shape_updated() {
          return Ok((frame_info, None));
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

        Ok((frame_info, Some(pointer_shape_info)))
      })
      .and_then(|r| r)
  }

  /// If the pointer shape is updated, the `Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>` will be [`Some`].
  /// This will resize `pointer_shape_buffer` if needed and update it.
  /// Uses delta updates if `use_delta` is true.
  pub(crate) fn next_frame_with_pointer_shape_delta(
    &self,
    timeout_ms: u32,
    texture: &ID3D11Texture2D,
    pointer_shape_buffer: &mut Vec<u8>,
    use_delta: bool,
  ) -> Result<(
    DXGI_OUTDUPL_FRAME_INFO,
    Option<DXGI_OUTDUPL_POINTER_SHAPE_INFO>,
  )> {
    self
      .process_next_frame_delta(timeout_ms, texture, use_delta, |frame_info| {
        if !frame_info.pointer_shape_updated() {
          return Ok((frame_info, None));
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

        Ok((frame_info, Some(pointer_shape_info)))
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
