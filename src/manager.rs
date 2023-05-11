use crate::duplicate_context::DuplicateContext;
use windows::core::ComInterface;
use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE_UNKNOWN, D3D_FEATURE_LEVEL_9_1};
use windows::Win32::Graphics::Direct3D11::{
  D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, D3D11_CREATE_DEVICE_FLAG, D3D11_SDK_VERSION,
};
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1, IDXGIOutput1};

pub struct Manager {
  pub dup_ctxs: Vec<DuplicateContext>,
  timeout_ms: u32,
}

impl Manager {
  pub fn default() -> Result<Manager, &'static str> {
    Manager::new(300)
  }

  pub fn new(timeout_ms: u32) -> Result<Manager, &'static str> {
    let mut manager = Manager {
      dup_ctxs: Vec::new(),
      timeout_ms,
    };
    match manager.refresh() {
      Ok(_) => Ok(manager),
      Err(_) => Err("Failed to acquire output duplication"),
    }
  }

  pub fn refresh(&mut self) -> Result<(), ()> {
    self.dup_ctxs.clear();

    unsafe {
      let factory = CreateDXGIFactory1::<IDXGIFactory1>().unwrap();
      let mut adapter_outputs = Vec::new();

      // collect adapters and outputs
      for adapter_index in 0.. {
        let adapter = match factory.EnumAdapters1(adapter_index) {
          Ok(adapter) => adapter,
          Err(_) => break,
        };
        let mut outputs = Vec::new();
        for output_index in 0.. {
          match adapter.EnumOutputs(output_index) {
            Err(_) => break,
            Ok(output) => outputs.push(output),
          }
        }
        if outputs.len() > 0 {
          adapter_outputs.push((adapter, outputs))
        }
      }
      if adapter_outputs.len() == 0 {
        panic!();
      }

      // prepare device and output
      for (adapter, outputs) in adapter_outputs {
        let mut device: Option<ID3D11Device> = None.clone();
        let mut device_context: Option<ID3D11DeviceContext> = None.clone();
        let mut feature_level = D3D_FEATURE_LEVEL_9_1;

        // create device for each adapter
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
        .unwrap();
        let device = device.unwrap();
        let device_context = device_context.unwrap();

        // create duplication output for each output
        for output in outputs {
          let output = output.cast::<IDXGIOutput1>().unwrap();
          let output_duplication = output.DuplicateOutput(&device).unwrap();
          self.dup_ctxs.push(DuplicateContext::new(
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
