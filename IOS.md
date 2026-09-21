# Shipping 동그라미타운 to the App Store

Written 2026-09-21, when the Windows and Android builds were already working
and iOS was an empty directory. Read this before changing anything under
`mobile/ios` or in `codemagic.yaml`.

**The one constraint everything follows from: an iOS binary can only be
produced on macOS.** There is no cross-compiler and no Windows path. Apple's
linker, `actool`, `codesign` and `xcodebuild` are all macOS-only, and the App
Store will not accept a bundle assembled any other way.

So the question is never "how do I build iOS on Windows", it is "whose Mac".
The answer here is Codemagic's, for the length of each build.

---

## The route

```
  Windows (you)        GitHub          Codemagic (macOS)         Apple
  -------------        ------          -----------------         -----
  edit Rust       -->  git push  -->   rustup + cargo
  edit project.yml                     xcodegen generate
                                       xcodebuild archive
                                       codesign      <---- certificate +
                                       build .ipa           profile, minted
                                             |              via the API key
                                             |                     |
                                        TestFlight  <--------------+
                                             |
                                        your iPhone
```

**You never need a Mac, not even once.** The part people assume requires one —
creating a signing certificate, which normally means generating a certificate
signing request in Keychain Access — is done for you: an App Store Connect API
key lets Codemagic create both the distribution certificate and the
provisioning profile on Apple's servers.

GitHub is in the picture only because Codemagic builds from a git remote. It is
not doing any building. GitHub Actions could host the Mac instead, but its
macOS runners bill at 10x the Linux rate on private repos and you would have to
assemble the signing dance (fastlane match, or manual keychain work) by hand.
Codemagic does that part as a build setting. You already have an account there;
use it.

---

## Part 1 — Apple, in a browser (~10 minutes, once)

### 1.1 Register the App ID

developer.apple.com/account -> **Certificates, Identifiers & Profiles** ->
**Identifiers** -> **+** -> *App IDs* -> *App*.

- Description: `Donggeurami Town`
- Bundle ID: **Explicit**, `town.donggeurami.app`
- Capabilities: none. The game has no push, no iCloud, no sign-in.

`town.donggeurami.app` matches the Android `applicationId` exactly. It is baked
into `mobile/ios/project.yml` and `codemagic.yaml`, two lines each; change it
in both or not at all. `mobile/ios/Info.plist` reads
`$(PRODUCT_BUNDLE_IDENTIFIER)` and never carries the literal.
**It cannot be changed after 1.3.**

The plain `town.donggeurami` was the first choice and is now permanently
unusable. It was attached to an app record that was later deleted, and Apple
retires a bundle ID the moment a record claims it — deleting the app does not
give it back, and it will never reappear in the **New App** dropdown no matter
how long you wait or whether the identifier still exists under **Identifiers**.
There is no appeal and no workaround; a new identifier is the only route, which
is what the `.app` suffix is. Do not delete an app record you might want the ID
for again.

`bundleIdPrefix` in `project.yml` deliberately stays `town.donggeurami`:
XcodeGen only applies it to targets that have no explicit
`PRODUCT_BUNDLE_IDENTIFIER`, and it is still a true prefix of the new ID.

### 1.2 The App Store Connect API key — already done

This account already ships another app through Codemagic, and the Developer
Portal integration is **team-level**: the key added for that app authorises
this one too. Codemagic also holds the private key for the distribution
certificate from that app, and a distribution certificate is shared across all
apps on the account, so it is reused rather than minting a second one against
Apple's per-account limit.

**But a working key and certificate are not enough, and this is the trap.** The
`ios_signing` block in `codemagic.yaml` does not ask Apple for anything. It is
a lookup over the signing files already stored in **Team settings** -> **Code
signing identities**, and a *provisioning profile is per bundle ID* even though
the certificate is not. The other app having shipped successfully proves the
key, the role and the certificate all work, and proves nothing whatsoever about
this app.

So before the first TestFlight build, generate the profile once: Codemagic ->
**Team settings** -> **Code signing identities** -> **iOS provisioning
profiles**, using the API key, bundle ID `town.donggeurami.app`, type **App
Store**. Skipping this fails the build inside Codemagic's own signing step,
before any script runs, with a single line and no other log output:

```
No matching profiles found for bundle identifier "town.donggeurami.app"
and distribution type "app_store"
```

That message names profiles, so it reads like an Apple permissions problem. It
is not. Do not go audit the key role, the identifier or the certificate — they
are almost certainly fine. The alternative, if you would rather the build be
self-sufficient, is the CLI path in the note at the end of this section.

One thing worth checking rather than assuming, at appstoreconnect.apple.com ->
**Users and Access** -> **Integrations**: the key needs the **App Manager**
role. *Developer* cannot create provisioning profiles, and the build dies at
the signing step with an error that never mentions the role.

(If the key ever has to be replaced: create it under **Team Keys** -> **+**,
download the `.p8` immediately — Apple shows it exactly once — and keep the
Issuer ID from the top of the page and the Key ID from the row with it.)

### 1.3 Create the app record

appstoreconnect.apple.com -> **Apps** -> **+** -> **New App**

- Platform: iOS
- Name: `Donggeurami Town`. The store name must be unique across the entire
  App Store. `CFBundleDisplayName` in `mobile/ios/Info.plist` was set to match
  it, so the home screen label and the store listing agree. They are allowed to
  differ — Apple does not object — but if you ever want them to, change the
  plist rather than the record: renaming the record is the harder side.

  Android still labels the icon `동그라미타운`
  (`mobile/android/app/src/main/AndroidManifest.xml`). The two platforms
  deliberately disagree; there is no Play listing yet to match.
- Primary language: English (U.S.). This matches the `en` that
  `$(DEVELOPMENT_LANGUAGE)` resolves to for `CFBundleDevelopmentRegion`.
- Bundle ID: `town.donggeurami.app`, the identifier from 1.1
- SKU: `donggeurami-town-001`

Registered 2026-09-21. None of this is visible from the repo, which is why it
is written down: the record is the thing `submit_to_testflight` uploads into.

Without this record the build still compiles and signs, but the TestFlight
upload at the end fails — there is nowhere to put it.

---

## Part 2 — Codemagic, in a browser (~2 minutes, once)

1. **Connect the repository** and add it as a new app. Codemagic finds
   `codemagic.yaml` at the repo root by itself; there is nothing to configure
   in the UI. The team's existing Developer Portal integration applies
   automatically — apps do not each get their own.
2. **Match the key name.** `integrations.app_store_connect` in
   `codemagic.yaml` is a lookup by *name* into **Team settings** ->
   **Integrations** -> **Developer Portal** -> **Manage keys**. It is the name,
   not the Key ID. Getting this wrong fails the build immediately, before any
   compiling, which at least makes it cheap to discover.

   Here the key is named **`codemagicflutter`** — inherited from the Flutter
   app this account already ships, since the integration is team-level. The
   name has nothing to do with this project and is not a mistake to correct;
   renaming the key in Codemagic would break the other app.

---

## Part 3 — GitHub (once)

The repo has no commits yet. From the project root:

```bash
git add -A
git commit -m "Windows, Android and iOS builds"
git branch -M main
git remote add origin https://github.com/<your-account>/RoundTown.git
git push -u origin main
```

Private is fine — Codemagic builds private repos on the free tier.

`.gitignore` already keeps `target/`, the `.exe` and the `.apk` out. Run
`git status` before the first push anyway; a 100 MB APK in git history is not
worth the cleanup.

---

## Part 4 — The first build

In Codemagic, pick the **iOS · TestFlight** workflow and press *Start new
build*. Expect **15–25 minutes** the first time — it is compiling Bevy from
scratch. Later builds reuse the cached `target/` and take a few minutes.

When it finishes, the build appears in App Store Connect -> your app ->
**TestFlight**, in *Processing* for another 5–30 minutes. Add yourself under
**Internal Testing** and it shows up in the TestFlight app on your phone.

After that, releases are tag-driven:

```bash
git tag ios-1.0.1 && git push origin ios-1.0.1
```

---

## How the build actually works

Worth understanding, because it is not the arrangement most iOS tutorials
describe.

**The app target compiles nothing.** Bevy 0.19 builds iOS apps as a plain Rust
*binary*, not as a static library that Xcode links — see
`examples/mobile/build_rust_deps.sh` upstream, which `mobile/ios/build_rust.sh`
follows. `cargo build --target aarch64-apple-ios --bin donggeurami_town_pc`
produces a finished Mach-O executable, `lipo` copies it to
`$TARGET_BUILD_DIR/$EXECUTABLE_PATH`, and Xcode's only jobs are wrapping it in
a bundle, copying the assets, compiling the icon, signing and packaging.

Three consequences that look like bugs but are not:

- **There is no dSYM.** Xcode never ran a link step, so `dsymutil` has nothing
  to read. TestFlight crash reports will not be symbolicated. Apple treats a
  missing dSYM as a warning, not a rejection.
- **`#[bevy_main]` does nothing on iOS.** In Bevy 0.19.1 that macro only emits
  the Android `android_main` entry point. Older Bevy versions also emitted a
  `main_rs` symbol for an Objective-C shim to call; that is gone, and it is not
  needed, because a Rust `bin` already has a real C `main`. UIKit gets started
  from inside `EventLoop::run`, by winit.
- **`assets/` is a folder reference, not a group.** The tree is copied verbatim
  to `RoundTown.app/assets/`, which is exactly where Bevy looks: `bevy_asset`'s
  `get_base_path()` returns `current_exe()`'s parent, and on iOS that is the
  bundle root. This also ships the `.blend` sources, about 700 KB, the same way
  the APK does. Filter them in `project.yml` if that ever matters.

**The `.xcodeproj` is generated, not committed.** `mobile/ios/project.yml` is
89 readable lines; the `project.pbxproj` it expands to is 300 lines of plist
that cannot be meaningfully reviewed from Windows. `xcodegen generate` runs on
the build machine. The only thing a committed project would add is signing
configuration, which Codemagic rewrites at build time anyway.

**The SDK requirement is satisfied for free.** Since 2026-04-28 Apple rejects
uploads not built with the iOS 26 SDK. `rustc` takes the SDK from `$SDKROOT`
and the minimum OS from `$IPHONEOS_DEPLOYMENT_TARGET`, both exported by Xcode,
and records them in the binary's `LC_BUILD_VERSION`. `environment.xcode` is set
to `latest` rather than pinned for exactly this reason — the requirement moves
every year.

---

## Screenshots and the rest of the listing

Run the **iOS · Simulator smoke test + store screenshots** workflow. It boots a
6.9" iPhone and a 13" iPad simulator, installs the app, lets it render, and
returns PNGs at the two sizes App Store Connect still demands — **2868×1320**
and **2752×2064** in landscape, from which Apple scales every smaller device.
The rotation direction is a guess; `mobile/ios/simulator_screenshots.sh` says
how to flip it if the world comes out upside down.

It also captures the app's stdout to `screenshots/*.log`. That is the iOS
equivalent of the `adb logcat -d | grep RustStdoutStderr` loop in
`RENDERING.md`, and it is how you learn that the game panicked on launch
instead of guessing from a black screenshot.

**Treat a clean simulator run as "it starts", not "it renders correctly."** The
simulator's Metal is not the phone's Metal. `RENDERING.md` is a long argument
for trusting hardware over inference, and it applies here too.

Still to fill in by hand, in App Store Connect:

| Item | Notes |
|---|---|
| Privacy policy URL | **Required for every app.** A GitHub Pages page saying the game collects no data is enough. This is the item that most often blocks a first submission. |
| App Privacy | Answer *Data Not Collected*. It is true — the game has no networking. |
| Age rating | The questionnaire. All "None" for this game. |
| Category | Games, then likely Adventure or Casual |
| Description, keywords, support URL | Free text |
| Export compliance | Already answered by `ITSAppUsesNonExemptEncryption` in `Info.plist`, so it will not ask on every upload. |

When TestFlight looks right on the phone, flip `submit_to_app_store` to `true`
in `codemagic.yaml`, or press *Submit for Review* in App Store Connect. A first
review is typically 24–48 hours.

---

## When it breaks

| Symptom | Cause |
|---|---|
| `env: bash\r: No such file or directory` | A script was checked out with CRLF. `.gitattributes` pins `*.sh` to LF; confirm it was committed. |
| `ld: library 'System' not found` | Xcode's toolchain shadowed the system `cc`. `build_rust.sh` resets `PATH` for exactly this (rust-lang/rust#80817) — check that reset survived an edit. |
| `No matching profiles found for bundle identifier` | The App ID from 1.1 was never registered, or the API key has *Developer* rather than *App Manager*. |
| `does not contain a scheme named "RoundTown"` | `xcodegen generate` did not run, or the `schemes:` block was deleted. Schemes Xcode auto-creates are per-user and invisible to `xcodebuild`. |
| `Invalid Bundle. ... does not contain a bundle executable` | `lipo` wrote somewhere other than the archive. `$TARGET_BUILD_DIR` is the correct variable for both normal and archive builds, but upstream Bevy only ever exercises the normal one — this is the least-proven line in the pipeline. |
| `ITMS-91053: Missing API declaration` | A required-reason API Apple found in the binary is missing from `PrivacyInfo.xcprivacy`. The mail names the category; add it with the matching reason code. `NSPrivacyAccessedAPICategoryUserDefaults` / `CA92.1` is the likely next one. |
| Build stuck in *Processing* | Normal. 5–30 minutes. |
| Runs in the simulator, dies on the phone | The hard case, because reading device logs normally needs a Mac. Without one: App Store Connect -> TestFlight -> **Crashes**. |

---

## Do not

- **Do not remove `NoIndirectDrawing`** from the camera spawn. `RENDERING.md`
  records what it cost to find out why it is there. On Metal it is probably
  unnecessary, and it does give up GPU culling — but "probably" is the word
  that wasted two days on Android. If you want it gone on iOS, prove it on a
  device first.
- **Do not add `#[cfg(target_os = "ios")]` to `src/day_night.rs`.** One shared
  rendering path across all three platforms is the rule that made the APK
  correct. iOS does not get an exception.
- **Do not commit the generated `.xcodeproj`.** It will drift from
  `project.yml`, and then nobody can tell which one is authoritative.
- **Do not edit `CFBundleVersion` by hand.** CI overwrites it with the
  Codemagic build number, the only value guaranteed to keep increasing.
  `CFBundleShortVersionString` is the one you bump for a release.
- **Do not upload a portrait screenshot.** App Store Connect rejects dimensions
  that are not on its list, and the game is landscape-only.

---

## Cost

The free Codemagic tier is **500 macOS minutes a month**, roughly 20 cold
builds and many more warm ones — `$HOME/.cargo`, `$HOME/.rustup` and `target/`
are all cached between builds. That is why the TestFlight workflow is
tag-triggered rather than push-triggered: a cold build on every commit would
burn the month in a week. Apple's side is the $99/year developer program you
already have.
