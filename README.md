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

```rs
use rusty_duplication::{
  capturer::model::Capturer,
  manager::Manager,
  utils::{FrameInfoExt, OutputDescExt},
};
use std::{fs::File, io::Write, thread, time::Duration};

fn main() {
  // manager will collect monitor info when created
  let manager = Manager::default().unwrap();
  // you can also refresh monitor info manually
  // manager.refresh();

  // you can get monitor info before capturing start
  // for ctx in &manager.contexts {
  //   ctx.desc().unwrap();
  // }

  // create capturer for a display
  // this will allocate memory buffer to store pixel data
  let mut capturer = manager.contexts[0].simple_capturer().unwrap();

  // you can also get monitor info from a capturer
  let desc = capturer.desc().unwrap();
  // we have some extension methods for you such as `width/height`
  println!("size: {}x{}", desc.width(), desc.height());

  // sleep for a while before capture to wait system to update the screen
  thread::sleep(Duration::from_millis(100));

  // capture desktop image and get the frame info
  // `safe_capture` will check if the buffer's size is enough
  let info = capturer.safe_capture().unwrap();

  // check if this is a new frame using the extension method `desktop_updated`
  if info.desktop_updated() {
    println!("captured!");
  }

  // write to a file
  // `buffer()` will return `&[u8]` in BGRA32 format
  let mut file = File::create("capture.bin").unwrap();
  file.write_all(capturer.buffer()).unwrap();
}
```

## Advanced Usage

### Shared Memory

You can use shared memory to share the buffer between processes.

This lib provides a `SharedCapturer` which will use Windows shared memory to store the buffer. Just call `DuplicateContext.shared_capturer` with a name.

```rs
manager.contexts[0].shared_capturer("Global\\MyFileMappingObject").unwrap();
```

> **Note**: if your memory name starts with `Global\\`, you may need to run this in administrator mode. See the [doc](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createfilemappinga).

### Customized Capturer

This lib provides low-level APIs like [`DuplicateContext`](https://github.com/DiscreteTom/rusty-duplication/blob/main/src/duplicate_context.rs), so you can write your own capturer. You can refer to [`SimpleCapturer`](https://github.com/DiscreteTom/rusty-duplication/blob/main/src/capturer/simple.rs)'s implementation.

### Test

Run the tests using `cargo test -- --test-threads=1` to prevent the tests from running in parallel.

> **Note**: move your mouse during the test to make sure the mouse pointer is captured, also make sure your mouse is in the primary monitor.

## Credit

This project is based on the following projects:

- https://github.com/bryal/dxgcap-rs
- https://github.com/microsoft/windows-rs
- https://github.com/hecomi/uDesktopDuplication

## [CHANGELOG](https://github.com/DiscreteTom/rusty-duplication/blob/main/CHANGELOG.md)
