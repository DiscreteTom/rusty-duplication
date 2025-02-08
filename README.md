# rusty-duplication

[![license](https://img.shields.io/crates/l/rusty-duplication?style=flat-square)](https://crates.io/crates/rusty-duplication)
[![version](https://img.shields.io/crates/v/rusty-duplication?style=flat-square)](https://crates.io/crates/rusty-duplication)
[![docs.rs](https://img.shields.io/docsrs/rusty-duplication?style=flat-square)](https://docs.rs/rusty-duplication/latest)

Capture the screen on Windows using the Desktop Duplication API in Rust, with shared memory support.

## Installation

```sh
cargo add rusty-duplication
```

## Usage

### Basic Usage

```rust
use rusty_duplication::{FrameInfoExt, Scanner, VecCapturer};
use std::{fs::File, io::Write, thread, time::Duration};

// create a scanner to scan for monitors
let mut scanner = Scanner::new().unwrap();

// scanner implements Iterator, you can use it to iterate through monitors
let monitor = scanner.next().unwrap();

// get monitor info
monitor.dxgi_output_desc().unwrap();
monitor.dxgi_outdupl_desc();

// create a vec capturer for a monitor
// this will allocate memory buffer to store pixel data
let mut capturer: VecCapturer = monitor.try_into().unwrap();

// you can also get monitor info from a capturer
let dxgi_outdupl_desc = capturer.monitor().dxgi_outdupl_desc();
let dxgi_output_desc = capturer.monitor().dxgi_output_desc().unwrap();
// get resolution width/height
println!(
  "size: {}x{}",
  dxgi_outdupl_desc.ModeDesc.Width, dxgi_outdupl_desc.ModeDesc.Height
);
// get position
println!(
  "left: {}, top: {}, right: {}, bottom: {}",
  dxgi_output_desc.DesktopCoordinates.left,
  dxgi_output_desc.DesktopCoordinates.top,
  dxgi_output_desc.DesktopCoordinates.right,
  dxgi_output_desc.DesktopCoordinates.bottom
);

// sleep for a while before capture to wait system to update the screen
thread::sleep(Duration::from_millis(100));

// capture desktop image and get the frame info
let info = capturer.capture().unwrap();

// we have some extension methods for the frame info
if info.desktop_updated() {
  println!("captured!");
}
if info.mouse_updated() {
  println!("mouse updated!");
}
if info.pointer_shape_updated() {
  println!("pointer shape updated!");
}

// write to a file
let mut file = File::create("capture.bin").unwrap();
// the buffer is in BGRA32 format
file.write_all(&capturer.buffer).unwrap();
```

### Shared Memory

You can use shared memory to share the frame buffer between processes.

```rust
use rusty_duplication::{CapturerBuffer, FrameInfoExt, Scanner, SharedMemoryCapturer};
use std::{fs::File, io::Write, thread, time::Duration};

let monitor = Scanner::new().unwrap().next().unwrap();

// create a shared memory capturer by creating a shared memory with the provided name
let mut capturer = SharedMemoryCapturer::create(monitor, "SharedMemoryName").unwrap();
// you can also use `SharedMemoryCapturer::open` to open an existing shared memory

// sleep for a while before capture to wait system to update the screen
thread::sleep(Duration::from_millis(50));

let info = capturer.capture().unwrap();
assert!(info.desktop_updated());

// write to a file
let mut file = File::create("capture.bin").unwrap();
// the buffer is in BGRA32 format
file.write_all(capturer.buffer.as_bytes()).unwrap();
```

> [!NOTE]
> If your shared memory name starts with `Global\`, you may need to run your app in administrator mode. See https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createfilemappinga.

### Customized Capturer

You can implement `CapturerBuffer` for your own type to create a customized capturer. You can refer to [`VecCapturer`](./src/capturer/vec.rs)'s implementation.

## [Examples](./examples/)

## [Documentation](https://docs.rs/rusty-duplication/)

## Credit

This project is based on the following projects:

- https://github.com/bryal/dxgcap-rs
- https://github.com/microsoft/windows-rs
- https://github.com/hecomi/uDesktopDuplication

## [CHANGELOG](./CHANGELOG.md)
