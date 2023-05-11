mod capturer;
mod duplicate_context;
mod manager;
mod utils;

use manager::Manager;

fn main() {
  let manager = Manager::default().unwrap();
  let ctx = &manager.dup_ctxs[0];
  let desc = ctx.get_desc();
  let width = desc.DesktopCoordinates.right - desc.DesktopCoordinates.left;
  let height = desc.DesktopCoordinates.bottom - desc.DesktopCoordinates.top;
  let mut buffer = vec![0u8; (width * height * 4) as usize];
  ctx.capture_frame(buffer.as_mut_ptr(), width as u32, height as u32);
}
