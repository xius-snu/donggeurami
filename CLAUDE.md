# 동그라미타운 (RoundTown)

A third-person 3D world in Bevy 0.19. One codebase, three targets: Windows
(`src/main.rs`), Android (`mobile/android`, built as a cdylib via cargo-ndk),
iOS (`mobile/ios`, built as a plain Rust binary that Xcode bundles).

## Before touching rendering, read `RENDERING.md`

It documents why the APK used to look different from the Windows build, what was
tested on the actual phone, and what not to try again. The short version:

- **Windows and Android run the same rendering code.** `src/day_night.rs` has no
  `#[cfg(target_os = ...)]` in it. Keep it that way — materials, lights and
  geometry are shared, with real `AlphaMode::Blend` and `AlphaMode::Add` on both.
- **`NoIndirectDrawing` on the camera (`src/lib.rs:373`) is load-bearing.**
  Without it the Adreno 840 silently drops Bevy's sorted transparent phase and
  the water, clouds, sun/moon glow and stars all disappear — with no error in
  logcat. Do not remove it as a cleanup.
- **Don't fake transparency with opaque geometry.** An earlier attempt added an
  opaque haze dome and giant "sun wash" spheres; that is what made the sky look
  grey and washed out. `RENDERING.md` has the details and the anti-patterns.
- **Test on the device, don't infer from the engine source.** The adb
  build-install-screenshot loop is in `RENDERING.md` and takes about ten seconds
  per cycle.

## Building

Desktop: `cargo run`.

Android needs the `release-android` profile, a manual strip step, and Gradle —
see the "Build gotchas" section of `RENDERING.md`. The command in
`mobile/android/app/build.gradle`'s comment is out of date.

iOS cannot be built on Windows at all — it needs macOS. `codemagic.yaml` rents
one per build and ships the result to TestFlight. **Read `IOS.md` before
touching anything under `mobile/ios`**; it covers the Apple-side setup, why the
`.xcodeproj` is generated rather than committed, and what breaks if the shell
scripts are checked out with CRLF line endings.

## Models

Authored in Blender and exported by `blender/export_glb.py` into `assets/`. A
metre in Blender is a metre in the world: anything the wrong size gets resized in
Blender and re-exported rather than scaled at spawn time (`src/lib.rs:381`).
