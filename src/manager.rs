use crate::duplication_context::DuplicationContext;
use crate::Error;
use crate::Result;
use windows::core::Interface;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE_UNKNOWN, D3D_FEATURE_LEVEL_9_1};
use windows::Win32::Graphics::Direct3D11::{
  D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, D3D11_CREATE_DEVICE_FLAG, D3D11_SDK_VERSION,
};
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1, IDXGIOutput1};

pub struct Manager {
  pub contexts: Vec<DuplicationContext>,
  timeout_ms: u32,
}

impl Default for Manager {
  #[inline]
  fn default() -> Self {
    Manager::new(300)
  }
}

impl Manager {
  /// Create a new manager with the provided timeout.
  #[inline]
  pub const fn new(timeout_ms: u32) -> Manager {
    Manager {
      contexts: Vec::new(),
      timeout_ms,
    }
  }

  /// Refresh monitors info.
  pub fn refresh(&mut self) -> Result<()> {
    self.contexts.clear();

    let factory = unsafe { CreateDXGIFactory1::<IDXGIFactory1>() }
      .map_err(Error::from_win_err(stringify!(CreateDXGIFactory1)))?;
    let mut adapter_outputs = Vec::new();

    // collect adapters and outputs
    for adapter_index in 0.. {
      let adapter = match unsafe { factory.EnumAdapters1(adapter_index) } {
        Ok(adapter) => adapter,
        Err(_) => break,
      };
      let mut outputs = Vec::new();
      for output_index in 0.. {
        match unsafe { adapter.EnumOutputs(output_index) } {
          Err(_) => break,
          Ok(output) => outputs.push(output),
        }
      }
      if !outputs.is_empty() {
        adapter_outputs.push((adapter, outputs))
      }
    }
    if adapter_outputs.is_empty() {
      return Err(Error::NoOutput);
    }

    // prepare device and output
    for (adapter, outputs) in adapter_outputs {
      let mut device: Option<ID3D11Device> = None;
      let mut device_context: Option<ID3D11DeviceContext> = None;
      let mut feature_level = D3D_FEATURE_LEVEL_9_1;

      // create device for each adapter
      unsafe {
        D3D11CreateDevice(
          &adapter,
          D3D_DRIVER_TYPE_UNKNOWN,
          HMODULE(std::ptr::null_mut()),
          D3D11_CREATE_DEVICE_FLAG(0),
          None,
          D3D11_SDK_VERSION,
          Some(&mut device),
          Some(&mut feature_level),
          Some(&mut device_context),
        )
      }
      .map_err(Error::from_win_err(stringify!(D3D11CreateDevice)))?;
      let Some(device) = device else {
        continue;
      };
      let device_context = device_context.unwrap();

      // create duplication output for each output
      for output in outputs {
        let output = output.cast::<IDXGIOutput1>().unwrap();
        let output_duplication = unsafe { output.DuplicateOutput(&device) }
          .map_err(Error::from_win_err(stringify!(DuplicateOutput)))?;
        self.contexts.push(DuplicationContext::new(
          device.clone(),
          device_context.clone(),
          output,
          output_duplication,
          self.timeout_ms,
        ))
      }
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::Manager;
  use serial_test::serial;

  #[test]
  #[serial]
  fn manager() {
    let mut manager = Manager::default();
    manager.refresh().unwrap();
    assert_ne!(manager.contexts.len(), 0);
    manager.refresh().unwrap();
    assert_ne!(manager.contexts.len(), 0);
  }
}
