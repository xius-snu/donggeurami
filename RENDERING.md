# Rendering notes — matching the APK to the Windows build

Written 2026-09-21 after three wrong fixes and one right one. Read this before
changing anything about materials, transparency, colour or the camera's render
settings. It will save you a day.

Engine: Bevy 0.19.1. Device the Android findings come from: **Galaxy S26 Ultra
(SM-S948N), Adreno 840 / SM8850, Vulkan, Android 16.**

---

## The one rule

**Windows and Android run the same rendering code.** `src/day_night.rs` has zero
`#[cfg(target_os = ...)]` in it and should stay that way. The sea, sky, clouds,
sun, moon and stars are identical code on both platforms, using real
`AlphaMode::Blend` and `AlphaMode::Add`.

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
performance" because it gives up GPU culling. At this scene's size — ~160 stars,
~144 cloud puffs, a handful of meshes, three directional lights — that is not
measurable. If the world grows a lot, revisit whether it can be scoped more
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
| GPU clustering | Bevy disables it on all Android and falls back to CPU clustering. Expected, logged, and irrelevant here — it only affects point and spot lights, and this scene has none. |

---

## Bevy 0.19 facts worth knowing

Things that are easy to get wrong and hard to notice:

**`unlit = true` ignores `emissive` completely.** In `bevy_pbr/src/render/pbr.wgsl`
the unlit branch is `out.color = pbr_input.material.base_color;` — emissive is
only applied inside `apply_pbr_lighting`, which unlit skips. Several materials in
`day_night.rs` still set `emissive` on unlit materials; those assignments do
nothing. Harmless, but don't tune them expecting a result. For an unlit material,
**`base_color` is the whole story.**

**`AlphaMode::Add` is premultiplied blending, not a separate blend state.** It
compiles to `BlendState::PREMULTIPLIED_ALPHA_BLENDING` and the shader writes
alpha 0 (`pbr_functions.wgsl::premultiply_alpha`), which turns
`src + (1-0)*dst` into plain addition. No extension, no downlevel flag, one
colour attachment, so no `INDEPENDENT_BLEND` requirement either.

**The GPU blends in linear space.** If you ever composite a translucent colour by
hand, convert to linear first, mix, and emit `Color::linear_rgb`. Mixing the sRGB
component values directly lands on a visibly different colour. Note that the sky
gradient constants in `day_night.rs` *are* interpolated in sRGB space — that is
fine, they are authored that way, but it is not the same operation as a blend.

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
- **Wait for night to check stars.** `TIME_SCALE = 20` makes a full cycle 60 s:
  ~30 s day, ~4.5 s sunset, ~21 s night, ~4.5 s sunrise. Stars are hidden unless
  `stars > 0.02`, so a daytime screenshot proves nothing about them.
- **Read Bevy's own log lines** with
  `adb logcat -d | grep RustStdoutStderr`. `AdapterInfo`, the clustering line and
  the GPU preprocessing line all print at INFO and tell you which paths are
  active.

---

## Build gotchas

- Use the **`release-android`** profile, not `release`. NDK + thin LTO is slow
  and can fail the linker; `release-android` sets `lto = false`,
  `codegen-units = 8`. The comment in `mobile/android/app/build.gradle` still
  says `build --release` and is wrong.
- **`cargo ndk` does not strip for a custom profile.** The library comes out at
  162 MB instead of 85 MB, nearly all `.symtab` and `.strtab`. Strip it with the
  NDK's `llvm-strip`, then confirm `GameActivity_onCreate` survives in `.dynsym`
  — that is the entry point `MainActivity` loads.
- The target-dir and jniLibs copies of the `.so` are **hardlinked**, so strip
  with `-o` to a temp file and move it over rather than editing in place.
- Only `gradlew.bat` exists. From git bash, run it through `cmd //c`.

---

## The process lesson

Two sessions shipped broken builds by reasoning from the Bevy and wgpu source
about what the driver "must" support, and by treating a previous developer's
on-device comment as a myth because the source disagreed with it. The source
tells you what the engine *asks for*. It cannot tell you what a specific driver
*does*. When an on-device report conflicts with source reading, the device wins —
plug the phone in and take a screenshot.
