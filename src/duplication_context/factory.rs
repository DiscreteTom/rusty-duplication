use super::DuplicationContext;
use crate::{Error, Result};
use std::ptr::null_mut;
use windows::core::Interface;
use windows::Win32::{
  Foundation::HMODULE,
  Graphics::{
    Direct3D::D3D_DRIVER_TYPE_UNKNOWN,
    Direct3D11::{
      D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, D3D11_CREATE_DEVICE_FLAG,
      D3D11_SDK_VERSION,
    },
    Dxgi::{CreateDXGIFactory1, IDXGIAdapter1, IDXGIFactory1, IDXGIOutput1},
  },
};

/// Factory of [`DuplicationContext`].
/// # Examples
/// ```no_run
/// use rusty_duplication::duplication_context::Factory;
///
/// // create a new factory
/// let mut factory = Factory::new().unwrap();
/// // get the next available context
/// let ctx = factory.next().unwrap();
/// ```
#[derive(Debug)]
pub struct Factory {
  next_adapter_index: u32,
  next_output_index: u32,
  factory: IDXGIFactory1,
  adapter: IDXGIAdapter1,
  device: ID3D11Device,
  device_context: ID3D11DeviceContext,
}

impl Factory {
  /// Try to create a new factory.
  /// Return [`Err`] if no adapter is found.
  pub fn new() -> Result<Self> {
    let factory = unsafe { CreateDXGIFactory1::<IDXGIFactory1>() }
      .map_err(Error::from_win_err(stringify!(CreateDXGIFactory1)))?;

    let adapter_index = 0;
    let (adapter, device, device_context) = get_adapter(&factory, adapter_index)?;

    Ok(Self {
      next_adapter_index: adapter_index + 1,
      next_output_index: 0,
      factory,
      adapter,
      device,
      device_context,
    })
  }

  fn get_current_ctx(&mut self) -> Option<DuplicationContext> {
    let output_index = self.next_output_index;
    self.next_output_index += 1;

    // TODO: add debug log for Result::ok()
    let output = unsafe { self.adapter.EnumOutputs(output_index) }.ok()?;
    let output = output.cast::<IDXGIOutput1>().unwrap();
    let output_duplication = unsafe { output.DuplicateOutput(&self.device) }.ok()?;
    Some(DuplicationContext::new(
      self.device.clone(),
      self.device_context.clone(),
      output,
      output_duplication,
    ))
  }
}

impl Iterator for Factory {
  type Item = DuplicationContext;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if let Some(ctx) = self.get_current_ctx() {
        return Some(ctx);
      }

      // no more available outputs, try next adapter
      let adapter_index = self.next_adapter_index;
      self.next_adapter_index += 1;
      let Ok((adapter, device, device_context)) = get_adapter(&self.factory, adapter_index) else {
        break;
      };
      self.adapter = adapter;
      self.device = device;
      self.device_context = device_context;
      self.next_output_index = 0;
    }

    None
  }
}

fn get_adapter(
  factory: &IDXGIFactory1,
  adapter_index: u32,
) -> Result<(IDXGIAdapter1, ID3D11Device, ID3D11DeviceContext)> {
  let adapter = unsafe { factory.EnumAdapters1(adapter_index) }
    .map_err(Error::from_win_err(stringify!(IDXGIFactory1.EnumAdapters1)))?;

  let mut device: Option<ID3D11Device> = None;
  let mut device_context: Option<ID3D11DeviceContext> = None;

  unsafe {
    D3D11CreateDevice(
      &adapter,
      D3D_DRIVER_TYPE_UNKNOWN,
      HMODULE(null_mut()),
      D3D11_CREATE_DEVICE_FLAG(0),
      None,
      D3D11_SDK_VERSION,
      Some(&mut device),
      None,
      Some(&mut device_context),
    )
  }
  .map_err(Error::from_win_err(stringify!(D3D11CreateDevice)))?;

  Ok((adapter, device.unwrap(), device_context.unwrap()))
}

#[cfg(test)]
mod tests {
  use super::*;
  use serial_test::serial;

  #[test]
  #[serial]
  fn manager() {
    let mut factory = Factory::new().unwrap();
    assert!(factory.next().is_some());
  }
}
