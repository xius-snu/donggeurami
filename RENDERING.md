# Rendering notes — matching the APK to the Windows build

Written 2026-09-21 after three wrong fixes and one right one. Read this before
changing anything about materials, transparency, colour or the camera's render
settings. It will save you a day.

Engine: Bevy 0.19.1. Device the Android findings come from: **Galaxy S26 Ultra
(SM-S948N), Adreno 840 / SM8850, Vulkan, Android 16.**

---

## The one rule

**Windows and Android run the same rendering code.** `src/sky.rs` has zero
`#[cfg(target_os = ...)]` in it and should stay that way. The sea, sky, clouds
and sun are identical code on both platforms, using real `AlphaMode::Blend` and
`AlphaMode::Add`.

(When this was written there was also a day–night cycle, with a moon and stars.
It has since been removed — the sun is fixed at mid-morning — so where the
history below mentions the moon, the stars or night, that is what it refers to.)

There is exactly **one** Android-specific rendering line in the whole project:

```rust
// src/lib.rs, in the camera spawn
#[cfg(any(target_os = "android", target_os = "ios"))]
NoIndirectDrawing,
```

Do not remove it. Everything else about mobile rendering follows from that.

---

## Why `NoIndirectDrawing` is load-bearing

Bevy logs this on the Adreno 840 at startup:

```
INFO bevy_render::batching::gpu_preprocessing: GPU preprocessing is fully supported on this device.
```

So Bevy drives its render phases with indirect draws and GPU culling. The driver
accepts all of it and then **silently drops everything queued into the sorted
`Transparent3d` phase.** The opaque phase draws perfectly. The result is a scene
with no water, no clouds, no sun/moon glow and no stars, and a sky that is just
`ClearColor`.

The thing that makes this so expensive to diagnose: **logcat is completely
clean.** No validation error, no wgpu error, no naga warning, no pipeline
creation failure. The pipelines compile, the render pass is recorded, the draws
produce nothing. You cannot find this by reading the engine source, because from
the engine's side everything is correct.

Bevy already carries workarounds for exactly this class of bug — see
`is_non_supported_android_device` in
`bevy_render-0.19.1/src/batching/gpu_preprocessing.rs`, which disables GPU
preprocessing for Adreno ≤730 and Mali drivers <48. The Adreno 840 is not on that
list but has the same problem. If a newer chip shows the same symptom, this is
the first thing to try.

`NoIndirectDrawing` (`bevy::render::view::NoIndirectDrawing`) drops the camera
back to direct draws. Bevy's docs note it "generally reduces rendering
performance" because it gives up GPU culling. At this scene's size — ~144 cloud
puffs, a handful of meshes, two directional lights — that is not measurable. If the world grows a lot, revisit whether it can be scoped more
narrowly. It must be added **when the camera is first spawned**; adding or
removing it later is documented as unspecified behaviour.

---

## Ruled out on device — do not re-test these

Each of these was tested on the actual phone by installing a build and taking a
screenshot. They are **not** the problem, and two of them cost a wasted build
each:

| Suspect | Verdict |
|---|---|
| MSAA | 4× MSAA works fine once indirect drawing is off. Android uses the same `Msaa::Sample4` default as Windows. No `Msaa::Off`, no FXAA needed. |
| `AlphaMode::Add` | Works. It is ordinary premultiplied blending, not a special GPU feature. |
| `cull_mode: None` / `double_sided: true` | Fine. The original build drew opaque water with both set. |
| Tonemapping | LUTs are `include_bytes!`-embedded at compile time, so there is no Android asset-loading difference. |
| sRGB surface format | Bevy explicitly prefers `Rgba8UnormSrgb`/`Bgra8UnormSrgb` and adds an sRGB view format otherwise. Gamma matches. |
| Directional light limits | `MAX_DIRECTIONAL_LIGHTS` (10) and `MAX_CASCADES_PER_LIGHT` (4) are only reduced under WebGL, never Android. |
| GPU clustering | Bevy disables it on all Android and falls back to CPU clustering. Expected and logged. It only affects point and spot lights, and the scene has none. (For a while each tree carried a `PointLight`; that was removed on 2026-09-26. If point lights come back, the CPU path on the phone is untested — that is where to look if their lighting looks wrong on device.) |

---

## Bevy 0.19 facts worth knowing

Things that are easy to get wrong and hard to notice:

**`unlit = true` ignores `emissive` completely.** In `bevy_pbr/src/render/pbr.wgsl`
the unlit branch is `out.color = pbr_input.material.base_color;` — emissive is
only applied inside `apply_pbr_lighting`, which unlit skips. Several materials in
`sky.rs` still set `emissive` on unlit materials; those assignments do
nothing. Harmless, but don't tune them expecting a result. For an unlit material,
**`base_color` is the whole story.**

**`AlphaMode::Add` is premultiplied blending, not a separate blend state.** It
compiles to `BlendState::PREMULTIPLIED_ALPHA_BLENDING` and the shader writes
alpha 0 (`pbr_functions.wgsl::premultiply_alpha`), which turns
`src + (1-0)*dst` into plain addition. No extension, no downlevel flag, one
colour attachment, so no `INDEPENDENT_BLEND` requirement either.

**The GPU blends in linear space.** If you ever composite a translucent colour by
hand, convert to linear first, mix, and emit `Color::linear_rgb`. Mixing the sRGB
component values directly lands on a visibly different colour.

**`ClearColor` is not tonemapped, but fragments are.** So a translucent surface
blended over the sky is a tonemapped colour over a non-tonemapped one. Exact
colorimetric matching of a hand-composited approximation is not achievable;
approximate and move on.

---

## Anti-patterns — what an earlier attempt did, and why it looked wrong

If you find yourself reaching for any of these, stop. They were all in the
codebase at one point and they were the cause of the original "the APK looks
wrong" report:

- **An opaque haze dome around the camera.** A `uv_ball(170)` squashed to
  `y * 0.34`, unlit and opaque, spawned at the origin. The camera sits inside it,
  so it replaces the entire sky. At noon its colour computes to roughly
  `srgb(0.76, 0.86, 0.79)` — pale grey-green. **This is what "the sky is greyish"
  meant.** It also occludes the distant ocean and z-fights the sun disc, which
  both sit at the same 170-unit radius.
- **Giant opaque "sun wash" spheres** of radius 46 and 105 parented to the sun,
  to fake a glow. They flatten the whole area around the sun into a disc of
  washed-out colour.
- **Opaque water with a pre-baked colour.** Correct as a last resort, wrong as a
  default — see below.
- **Cranking `base_color` 3–6× brighter in linear space on mobile**, and making
  stars 6× larger and opaque. These were compensating for the missing transparent
  pass, not for any real brightness difference.

All of these were written to work around the transparent pass being dropped. The
actual fix for that is one line; none of this is needed.

---

## If you ever genuinely need an opaque fallback

On a device where `NoIndirectDrawing` does not help and the transparent pass is
truly unusable, the working approach is to composite against the sky up front and
draw in the opaque phase. This shipped once and worked. Keep these details:

- Composite in **linear** space (see above).
- For an additive shell, a desktop sphere with `cull_mode: None` shows **both**
  faces and each one adds, so the baked colour must add **twice** to match.
- **Nested glow shells need front-face culling.** An opaque shell wrapped around
  the sun sits in front of it and hides it. Set
  `cull_mode: Some(Face::Front)` so only the far side of the sphere draws — at
  181 and 187 units versus the sun's 162 — and the sun disc then covers the
  middle of it, leaving a ring. That is how you rebuild a layered glow out of
  opaque geometry.
- Accept the limits honestly: water will not show the island's submerged wall,
  and clouds will occlude the sun and stars instead of tinting them.

This code is deleted from the repo. It is in git history and in this note; do not
resurrect it unless a device actually needs it.

---

## Verifying on device — do this instead of guessing

This loop is what found the bug in three cycles after two days of wrong
inference. It is fast: about 6 s for cargo, 2 s for gradle.

```bash
# build, strip, package
cargo ndk -t arm64-v8a -P 26 -o mobile/android/app/src/main/jniLibs \
    build --profile release-android --lib
llvm-strip -o tmp.so mobile/android/app/src/main/jniLibs/arm64-v8a/libdonggeurami_town.so
# replace the .so with tmp.so, then:
cmd //c "mobile\android\gradlew.bat -p mobile\android assembleRelease"
cp mobile/android/app/build/outputs/apk/release/app-release.apk donggeurami_town-release.apk

# install, launch, look
adb install -r donggeurami_town-release.apk
adb logcat -c
adb shell am start -n town.donggeurami.app/town.donggeurami.MainActivity
adb exec-out screencap -p > shot.png     # then read the PNG
```

- **The launch component is not `<package>/.MainActivity`.** `applicationId` is
  `town.donggeurami.app` but the Java package is still `town.donggeurami`, so
  `am start` needs both halves spelled out, as above. A relative `.MainActivity`
  resolves against the application ID and fails with
  `Error type 3 ... does not exist`. `adb uninstall` and `adb shell pm clear`
  take the application ID alone.
- **Drive the camera with `adb shell input swipe`.** A swipe outside the stick
  and jump circles is the look control. Swiping **up** tilts the view up toward
  the sky; swiping horizontally orbits. Screen is 3120×1440 in landscape.
- **Read Bevy's own log lines** with
  `adb logcat -d | grep RustStdoutStderr`. `AdapterInfo`, the clustering line and
  the GPU preprocessing line all print at INFO and tell you which paths are
  active.

---

## Measuring performance on the phone

Measured 2026-09-27 on the same SM-S948N, at 1440×3120 with 4× MSAA. The
things that looked like the obvious tools did not work:

- **Bevy's per-pass GPU timers (`RenderDiagnosticsPlugin`) are meaningless on
  the Adreno.** Every render pass reads about 0.0005 ms: a tiling GPU does the
  real work after the timestamps are written. Only compute passes and the
  final upscaling blit (0.33 ms) read true. Measure a feature by switching it
  off and comparing, not by its timer.
- **`dumpsys SurfaceFlinger --latency` returns no frames on Android 16.** Use
  timestats instead: `--timestats -clear -enable`, play for a while, then
  `--timestats -dump` and read the block for
  `SurfaceView[town.donggeurami.app/…]`. Its `present2present` histogram is
  the frame pacing (at 120 fps, `8ms` is on time and `12ms` a late frame).
- **GPU load and clock:** `/sys/class/kgsl/kgsl-3d0/gpu_busy_percentage` and
  `clock_mhz` (the maximum is 1300). **CPU per thread:**
  `/proc/<pid>/task/*/stat`, fields 14–15, at 100 ticks a second.
- **Temperatures:** `dumpsys thermalservice`, the *Current temperatures from
  HAL* block. The block printed before it is a stale cache.
- **The phone has to be unlocked with the game in front.** A locked phone
  draws nothing, and every number reads idle. `settings put global
  stay_on_while_plugged_in 7` keeps it awake for a session; put it back to 0.
- **Charging heats it.** Plugged in and after a few minutes at 120 fps, Android
  caps the GPU at about 750 MHz even while it is 98% busy. Numbers from a cool
  phone and a hot one are not comparable, so note the temperature with each.

What it found, at 120 fps: the GPU is the limit. Its work per frame, roughly
(busy × clock ÷ frame rate, from runs at different temperatures):

| Setting | GPU work per frame |
|---|---|
| Four shadow maps (Bevy's default) | ~7.4 M cycles |
| **Two shadow maps (what `sky.rs` uses)** | **~4.4 M** |
| 2× MSAA instead of 4× | ~6.1 M |
| 1024² shadow maps instead of 2048² | ~6.7 M |

Also found: asking Android for the 1080×2340 display mode through
`preferredDisplayModeId` is ignored (the game stays at 1440×3120; since
2026-10-03 `MainActivity` shrinks the game's own surface with
`setFixedSize` instead, on trial and not yet measured), and
`ClusterConfig::None` on the camera crashes Bevy 0.19 on its first frame
("clustering dummy texture"), where `ClusterConfig::Single` works.

---

## The iPhone's black screen: the render world on one thread (2026-10-08)

iOS 1.0.1 to 1.0.3 opened on a black screen and stayed there. The phone's own
`log.txt` (Files app, `src/log_file.rs`) had it in one line:

```
PANIC on the unnamed thread: ... raw-window-metal ...: can only access UIView on the main thread
  ... wgpu_hal::metal ... create_surface
  ... bevy_render::view::window::create_surfaces
  ... SingleThreadedExecutor ... PipelinedRenderingPlugin (the render thread)
```

`create_surfaces`, which makes the window's surface, has to run on the main
thread on iOS and macOS (it takes a `NonSendMarker` there and nowhere else),
as it touches the window's UIView. With pipelined rendering the render world
runs on a thread of its own, and Bevy's multi-threaded executor hands such a
system back to the main thread. `one_thread_per_world` in `src/lib.rs` had put
every schedule of the render world on the single-threaded executor, which
runs everything on the thread it is on: the render thread. It panicked on the
first frame; `renderer_extract` found it gone and asked the app to quit;
winit ignores that on iOS ("`ControlFlow::Exit` ignored on iOS"); and the app
stopped, still open, having drawn nothing. Android and Windows have no such
system, which is why they ran. The render world now keeps Bevy's executor on
Apple platforms.

Before the log there were two guesses. The server's log showed the iPhone
making its account at the login, a thread of its own started on the first
frame, and never joining the game, which the main loop does a moment later:
a render thread dying on its first frame, rightly read. Its cause was guessed
to be GPU light clustering, which Bevy 0.19 runs on an iPhone but neither on
Android nor in the iOS simulator (`make_global_cluster_settings` in
`bevy_pbr/src/cluster/mod.rs`), and 1.0.2 turned it off, to no effect. It
stays off, on every platform (`cluster_lights_on_the_cpu`): the world has no
light to cluster, only the sun and the fill, which are directional, so on the
CPU it is nothing at all, where the GPU still ran compute and raster passes
for it every frame.

**A device-only failure: get the device's log before guessing.** It took one
build to add the log and one look at it, after two builds of inference.

---

## Mirrors: tried and taken out (2026-10-02)

A reflecting mirror was built and worked on desktop: a second camera behind
the glass, opposite your eye, with an off-axis projection cut to the glass
and the glass as its near plane, drawing into a small HDR texture laid on the
glass, and switched on only while the glass was in sight. Hajun had it taken
out as too costly for the phone, before it was measured there.

The cost that decided it: **Bevy 0.19 gives every active camera its own sun
shadow maps** (`Cascades` in `bevy_light/src/cascade.rs` is per view), and
there is no setting to turn them off for one camera. So any second camera
that draws the world — a mirror, a portal, a picture-in-picture — adds the
two shadow maps again, roughly the 3 M cycles a frame that going from four
maps to two saved, on top of drawing the scene again. Weigh that before
adding one. The code is not in git history (it was never committed).

---

## Build gotchas

- Use the **`release-android`** profile, not `release`. NDK + thin LTO is slow
  and can fail the linker; `release-android` sets `lto = false`,
  `codegen-units = 8`. The comment at the end of
  `mobile/android/app/build.gradle` has the right commands (it once said
  `build --release`).
- **`cargo ndk` does not strip for a custom profile.** The library comes out at
  162 MB instead of 85 MB, nearly all `.symtab` and `.strtab`. Strip it with the
  NDK's `llvm-strip`, then confirm `GameActivity_onCreate` survives in `.dynsym`
  — that is the entry point `MainActivity` loads.
- The target-dir and jniLibs copies of the `.so` are **hardlinked**, so strip
  with `-o` to a temp file and move it over rather than editing in place.
- Only `gradlew.bat` exists. From git bash, run it through `cmd //c`.
- **The APK leaves out what the game never loads**: every name in
  `mobile/android/app/assets-left-out.txt`, handed to aapt as
  `ignoreAssetsPattern` in `build.gradle`. That is Blender's `.blend` and
  `.blend1` files, a reference photo and the models no longer used; 12.6 MB
  of a 111 MB APK on 2026-10-03, before Hajun deleted most of the unused
  models outright. The patterns match names, not paths: a name on the list
  leaves out every file called that, in any folder. A reference picture or a
  retired model added later goes on the list by hand. `cargo test` fails if a piece the
  shop sells would be left out (`build::tests::no_piece_is_left_out_of_the_apk`).
  The iOS app still bundles the whole of `assets/` (`mobile/ios/project.yml`).

---

## The process lesson

Two sessions shipped broken builds by reasoning from the Bevy and wgpu source
about what the driver "must" support, and by treating a previous developer's
on-device comment as a myth because the source disagreed with it. The source
tells you what the engine *asks for*. It cannot tell you what a specific driver
*does*. When an on-device report conflicts with source reading, the device wins —
plug the phone in and take a screenshot.
