#!/usr/bin/env bash
#
# Type-checks the game for the iPhone on the Windows PC it is written on,
# which cannot build it. Run it from Git Bash before pushing an `ios-*` tag,
# and after writing anything behind `target_os = "ios"`:
#
#     bash mobile/ios/check_on_windows.sh
#
# It is `cargo check` for `aarch64-apple-ios`, of the same binary and the
# same profile `build_rust.sh` builds on the Mac. Every line the iPhone
# compiles is type- and borrow-checked; nothing is linked, signed, bundled
# or run, so a pass here says the Rust compiles and no more than that.
#
# What stood in the way was three crates that compile a little C for the
# iPhone as they are built (blake3, ring, tracing-oslog), with Apple's
# compiler and SDK. `cargo check` never links what they compile, so a
# stand-in that makes empty objects is enough (`stand_in_cc.rs`). If a crate
# is ever added that reads Apple's headers as it is built (bindgen), this
# stops working, and says so in that crate's build script.
#
# A minute and a half the first time and seconds after; some 700 MB, in the
# temporary folder, which may be deleted whenever. Extra arguments go to
# cargo.

set -euo pipefail

REPO="$(cd "$(dirname "$0")/../.." && pwd)"
WORK="${RT_IOS_CHECK_DIR:-${TEMP:-/tmp}/roundtown-ios-check}"
mkdir -p "$WORK/bin" "$WORK/sdk"

# Cargo and what it starts are Windows programs: hand them Windows paths.
windows() { cygpath -m "$1"; }

# The stand-in has to be called clang for the `cc` crate to take it for one;
# the archiver is the same program under another name.
if [ ! -x "$WORK/bin/clang.exe" ] || [ "$REPO/mobile/ios/stand_in_cc.rs" -nt "$WORK/bin/clang.exe" ]; then
    rustc -O --edition 2021 -o "$(windows "$WORK/bin/clang.exe")" "$(windows "$REPO/mobile/ios/stand_in_cc.rs")"
    cp "$WORK/bin/clang.exe" "$WORK/bin/ar.exe"
fi

if ! rustup target list --installed | grep -qx aarch64-apple-ios; then
    rustup target add aarch64-apple-ios
fi

cd "$REPO"
# SDKROOT has to be a folder that is there, or the `cc` crate asks `xcrun`
# for one; the deployment target is `project.yml`'s. Never `--workspace`.
CARGO_TARGET_DIR="$(windows "$WORK/target")" \
CC_aarch64_apple_ios="$(windows "$WORK/bin/clang.exe")" \
AR_aarch64_apple_ios="$(windows "$WORK/bin/ar.exe")" \
SDKROOT="$(windows "$WORK/sdk")" \
IPHONEOS_DEPLOYMENT_TARGET=16.0 \
    cargo check --locked --release --target aarch64-apple-ios --bin donggeurami_town_pc "$@"
