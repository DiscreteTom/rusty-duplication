# CHANGELOG

## v0.4.0

- **_Breaking Change_**: move `Result` to `model` module.

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
