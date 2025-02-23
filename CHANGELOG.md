# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.1] - 2025-02-11

### Fixed

- Fix documentations in docs.rs.

## [0.6.0-fix-doc.rs] - 2025-02-11

### Fixed

- Fix documentations in docs.rs.

## [0.6.0] - 2025-02-09

### Added

- `Scanner` to create `Monitor`.
- `FrameInfoExt::pointer_shape_updated`.
- `Capturer::monitor`.
- `Capturer::capture_unchecked` and `Capturer::capture_with_pointer_shape_unchecked`.
- `Capturer::new`, `Capturer::try_into` for `VecCapturer`, `Capturer::create` and `Capturer::open` for `SharedMemoryCapturer`.
- `Capturer::timeout_ms`.
- `SharedMemory`.
- `CapturerBuffer`.

### Changed

- Rewrite `Error`.
- Re-organize export.
- Rename `DuplicationContext` to `Monitor`.
- `FrameInfoExt::mouse_updated` will return a `bool`.
- Capturers will consume the `Monitor`. You can clone `Monitor` to create multiple capturers.
- `Capturer::capture` and `Capturer::capture_with_pointer_shape` are safe now.
- `Capture` is now a struct instead of a trait.
- Rename `SimpleCapturer` to `VecCapturer`.
- Rename `SharedCapturer` to `SharedMemoryCapturer`.

### Removed

- `Manager`. Use `Scanner` instead.
- `Monitor::new`. Use `Scanner` instead.
- `Monitor::create_readable_texture`, `Monitor::capture`, `Monitor::capture_with_pointer_shape`, `Monitor::next_frame` and `Monitor::next_frame_with_pointer_shape`.
- `MouseUpdateStatus`. Use `FrameInfoExt::mouse_updated` and `FrameInfoExt::pointer_shape_updated` instead.
- `Monitor::simple_capturer`, `Monitor::shared_capturer`, `Monitor::shared_capturer_open`, `Monitor::custom_capturer`. Use capturer's `new` to create capturers.
- `Capturer::dxgi_output_desc` and `Capturer::dxgi_outdupl_desc`. Use `Capturer::monitor` instead.
- `Capturer::safe_capture` and `Capturer::safe_capture_with_pointer_shape`. Use `Capturer::capture` and `Capturer::capture_with_pointer_shape` instead.
- `Capturer::buffer_mut`. Use `Capturer::buffer` instead.
- `CustomCapturer`. Use `Capturer` instead.

## [0.5.0] - 2023-05-08

### Removed

- `Capturer.pointer_shape_updated`.

### Fixed

- Don't retrieve pointer shape when it's not updated.

## [0.4.5] - 2023-05-26

### Fixed

- Texture dimension when rotating screen.
  - https://github.com/DiscreteTom/shremdup/issues/2
  - https://github.com/DiscreteTom/HyperDesktopDuplication/issues/5

## [0.4.4] - 2023-05-26

### Fixed

- Wrong C-style file name when opening shared memory.

## [0.4.3] - 2023-05-26

### Fixed

- Wrong C-style file name causing https://github.com/DiscreteTom/HyperDesktopDuplication/issues/4.
- Unclosed file if `MapViewOfFile` failed.

## [0.4.2] - 2023-05-25

### Fixed

- Copy memory using `mapped_surface.Pitch`. [#7](https://github.com/DiscreteTom/rusty-duplication/issues/7)

## [0.4.1] - 2023-05-21

### Added

- `DuplicationContext.monitor_info` and `MONITORINFO.is_primary`.

## [0.4.0] - 2023-05-21

### Changed

- Move `Result` to `model` module.
- Rename `DuplicationContext.capture_frame` to `DuplicationContext.capture`.
- Rename `desc` to `dxgi_output_desc`.
- Apply new `Error` type for better error handling.

### Added

- `Capturer.pointer_shape_updated`.
- `DuplicationContext/Capturer.dxgi_outdupl_desc`.
- `DuplicationContext.next_frame/next_frame_with_pointer_shape`.
- `DuplicationContext.capture_frame_with_pointer_shape`.
- `Capturer.capture_with_pointer_shape/pointer_shape_buffer`.

### Fixed

- Wrong screen size in high DPI. [#5](https://github.com/DiscreteTom/rusty-duplication/issues/5).

### Removed

- `DuplicationContext.acquire_next_frame`, use `DuplicationContext.next_frame/next_frame_with_pointer_shape` instead.
- `DuplicationContext.get_pointer`, use `DuplicationContext.capture_frame_with_pointer_shape` instead.
- `Capturer.get_pointer`, use `Capturer.capture_with_pointer_shape/pointer_shape_buffer` instead.
- `OutputDescExt.calc_buffer_size`, use `OutDuplDescExt.calc_buffer_size` instead.

## [0.3.0] - 2023-05-16

### Changed

- Error type changed from `&static str` to `String`.
- Rename `DXGI_OUTDUPL_FRAME_INFO.is_new_frame` to `DXGI_OUTDUPL_FRAME_INFO.desktop_updated`, `DXGI_OUTDUPL_FRAME_INFO.mouse_updated`.
- Rename `DuplicateContext` to `DuplicationContext`.

### Added

- `CustomCapturer`.
- `SharedCapturer.open`.
- `DuplicationContext/Capturer.get_pointer`.

## [0.2.0] - 2023-05-13

### Changed

- Move `calc_buffer_size` into trait `OutputDescExt`.
- Rename methods `get_xxx` to `xxx`.

### Added

- `Capturer.buffer_mut`.

## [0.1.0] - 2023-05-13

### Added

- Initial release.

[unreleased]: https://github.com/DiscreteTom/rusty-duplication/compare/v0.6.1...HEAD
[0.6.1]: https://github.com/DiscreteTom/rusty-duplication/releases/tag/v0.6.1
[0.6.0-fix-doc.rs]: https://github.com/DiscreteTom/rusty-duplication/releases/tag/v0.6.0-fix-doc.rs
[0.6.0]: https://github.com/DiscreteTom/rusty-duplication/releases/tag/v0.6.0
[0.5.0]: https://github.com/DiscreteTom/rusty-duplication/releases/tag/v0.5.0
[0.4.5]: https://github.com/DiscreteTom/rusty-duplication/releases/tag/v0.4.5
[0.4.4]: https://github.com/DiscreteTom/rusty-duplication/releases/tag/v0.4.4
[0.4.3]: https://github.com/DiscreteTom/rusty-duplication/releases/tag/v0.4.3
[0.4.2]: https://github.com/DiscreteTom/rusty-duplication/releases/tag/v0.4.2
[0.4.1]: https://github.com/DiscreteTom/rusty-duplication/releases/tag/v0.4.1
[0.4.0]: https://github.com/DiscreteTom/rusty-duplication/releases/tag/v0.4.0
[0.3.0]: https://github.com/DiscreteTom/rusty-duplication/releases/tag/v0.3.0
[0.2.0]: https://github.com/DiscreteTom/rusty-duplication/releases/tag/v0.2.0
[0.1.0]: https://github.com/DiscreteTom/rusty-duplication/releases/tag/v0.1.0
