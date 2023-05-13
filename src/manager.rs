use crate::duplicate_context::DuplicateContext;
use windows::core::ComInterface;
use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE_UNKNOWN, D3D_FEATURE_LEVEL_9_1};
use windows::Win32::Graphics::Direct3D11::{
  D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, D3D11_CREATE_DEVICE_FLAG, D3D11_SDK_VERSION,
};
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1, IDXGIOutput1};

pub struct Manager {
  pub contexts: Vec<DuplicateContext>,
  timeout_ms: u32,
}

impl Manager {
  /// Create a new manager and refresh monitor info.
  pub fn default() -> Result<Manager, &'static str> {
    Manager::new(300)
  }

  /// Create a new manager and refresh monitor info.
  pub fn new(timeout_ms: u32) -> Result<Manager, &'static str> {
    let mut manager = Manager {
      contexts: Vec::new(),
      timeout_ms,
    };
    match manager.refresh() {
      Ok(_) => Ok(manager),
      Err(e) => Err(e),
    }
  }

  /// Refresh monitor info.
  pub fn refresh(&mut self) -> Result<(), &'static str> {
    self.contexts.clear();

    let factory =
      unsafe { CreateDXGIFactory1::<IDXGIFactory1>() }.map_err(|_| "CreateDXGIFactory1 failed")?;
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
      if outputs.len() > 0 {
        adapter_outputs.push((adapter, outputs))
      }
    }
    if adapter_outputs.len() == 0 {
      return Err("No output");
    }

    // prepare device and output
    for (adapter, outputs) in adapter_outputs {
      let mut device: Option<ID3D11Device> = None.clone();
      let mut device_context: Option<ID3D11DeviceContext> = None.clone();
      let mut feature_level = D3D_FEATURE_LEVEL_9_1;

      // create device for each adapter
      unsafe {
        D3D11CreateDevice(
          &adapter,
          D3D_DRIVER_TYPE_UNKNOWN,
          None,
          D3D11_CREATE_DEVICE_FLAG(0),
          None,
          D3D11_SDK_VERSION,
          Some(&mut device),
          Some(&mut feature_level),
          Some(&mut device_context),
        )
      }
      .map_err(|_| "D3D11CreateDevice failed")?;
      let device = device.unwrap();
      let device_context = device_context.unwrap();

      // create duplication output for each output
      for output in outputs {
        let output = output.cast::<IDXGIOutput1>().unwrap();
        let output_duplication =
          unsafe { output.DuplicateOutput(&device) }.map_err(|_| "DuplicateOutput failed")?;
        self.contexts.push(DuplicateContext::new(
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

  }
}
