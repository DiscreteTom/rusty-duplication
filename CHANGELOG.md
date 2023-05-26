# CHANGELOG

## v0.4.3

- Fix: wrong c-style file name which cause https://github.com/DiscreteTom/HyperDesktopDuplication/issues/4.
- Fix: unclosed file if `MapViewOfFile` failed.

## v0.4.2

- Fix: copy memory using `mapped_surface.Pitch`. #7

## v0.4.1

- Feat: add `DuplicationContext.monitor_info` and `MONITORINFO.is_primary`.

## v0.4.0

- **_Breaking Change_**: move `Result` to `model` module.
- **_Breaking Change_**: remove `DuplicationContext.acquire_next_frame`, add `DuplicationContext.next_frame/next_frame_with_pointer_shape`.
- **_Breaking Change_**: remove `DuplicationContext.get_pointer`, add `DuplicationContext.capture_frame_with_pointer_shape`.
- **_Breaking Change_**: remove `Capturer.get_pointer`, add `Capturer.capture_with_pointer_shape/pointer_shape_buffer`.
- **_Breaking Change_**: rename `DuplicationContext.capture_frame` to `DuplicationContext.capture`.
- **_Breaking Change_**: rename `desc` to `dxgi_output_desc`.
- **_Breaking Change_**: remove `OutputDescExt.calc_buffer_size`, use `OutDuplDescExt.calc_buffer_size` instead.
- **_Breaking Change_**: apply new `Error` type for better error handling.
- Feat: add `Capturer.pointer_shape_updated`.
- Feat: add `DuplicationContext/Capturer.dxgi_outdupl_desc`.
- Fix: wrong screen size in high dpi. #5

## v0.3.0

- **_Breaking Change_**: error type changed from `&static str` to `String`.
- **_Breaking Change_**: rename `DXGI_OUTDUPL_FRAME_INFO.is_new_frame` to `DXGI_OUTDUPL_FRAME_INFO.desktop_updated`, add `DXGI_OUTDUPL_FRAME_INFO.mouse_updated`.
- **_Breaking Change_**: rename `DuplicateContext` to `DuplicationContext`.
- Feat: add `CustomCapturer`.
- Feat: add `SharedCapturer.open`.
- Feat: add `DuplicationContext/Capturer.get_pointer`.

## v0.2.0

- **_Breaking Change_**: move `calc_buffer_size` into trait `OutputDescExt`.
- **_Breaking Change_**: rename methods `get_xxx` to `xxx`.
- Feat: add `Capturer.buffer_mut`.

## v0.1.0

The initial release.
