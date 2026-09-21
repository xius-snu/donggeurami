#!/usr/bin/env bash
#
# Xcode "Run Script" build phase: compiles the Rust binary for whichever iOS
# architecture Xcode asked for, and drops it in as the app's executable.
#
# Bevy 0.19 builds iOS apps as a plain Rust *binary*, not as a static library
# that Xcode links — see `examples/mobile/build_rust_deps.sh` upstream, which
# this follows. So the Xcode target has no compile sources at all: this script
# produces `RoundTown.app/RoundTown` on its own, and Xcode only wraps it in a
# bundle, copies the assets in, signs it and packages the .ipa.
#
# That is also how the App Store's "must be built with the iOS 26 SDK" rule is
# satisfied without doing anything special: rustc takes the SDK from $SDKROOT
# and the minimum OS from $IPHONEOS_DEPLOYMENT_TARGET, both of which Xcode
# exports, and records them in the binary's LC_BUILD_VERSION.
#
# Xcode gives us: ARCHS, CONFIGURATION, LLVM_TARGET_TRIPLE_SUFFIX, SRCROOT,
# TARGET_BUILD_DIR, EXECUTABLE_PATH.

set -euo pipefail

: "${SRCROOT:?this script only runs as an Xcode build phase}"
: "${ARCHS:?}"
: "${TARGET_BUILD_DIR:?}"
: "${EXECUTABLE_PATH:?}"

# The [[bin]] from Cargo.toml. The `_pc` suffix is historical — it keeps the
# MSVC .pdb name distinct from the lib's — but it is the same `fn main` on
# every platform, so iOS builds it too rather than adding a second bin target
# that would double desktop link times.
BIN=donggeurami_town_pc

REPO="$(cd "$SRCROOT/../.." && pwd)"

# rustc shells out to `cc` to link. Xcode puts its own toolchain at the front
# of PATH and the `cc` found there cannot resolve libSystem:
#     ld: library 'System' not found
# (rust-lang/rust#80817). Reset PATH to the system tools, then put cargo and
# Homebrew back on the end.
export PATH="/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin"

# Keep cargo's output in the repo's own target/ rather than Xcode's
# DerivedData, so CI can cache it as one directory.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$REPO/target}"

if [ "${CONFIGURATION:-Release}" = "Debug" ]; then
    CARGO_PROFILE=dev
    PROFILE_DIR=debug
else
    # The plain `release` profile, not a stripped-down one. RENDERING.md's
    # reason for `release-android` was the NDK linker choking on thin LTO;
    # Apple's ld64 has no such problem. Named explicitly rather than passed
    # as --release so the argument list is never empty: macOS ships bash 3.2,
    # where expanding an empty array under `set -u` is a fatal error.
    CARGO_PROFILE=release
    PROFILE_DIR=release
fi

IS_SIMULATOR=0
if [ "${LLVM_TARGET_TRIPLE_SUFFIX-}" = "-simulator" ]; then
    IS_SIMULATOR=1
fi

EXECUTABLES=()
for arch in $ARCHS; do
    case "$arch" in
        arm64)
            if [ "$IS_SIMULATOR" -eq 1 ]; then
                TRIPLE=aarch64-apple-ios-sim
            else
                TRIPLE=aarch64-apple-ios
            fi
            ;;
        x86_64)
            if [ "$IS_SIMULATOR" -eq 0 ]; then
                echo "error: x86_64 asked for on a device build" >&2
                exit 2
            fi
            TRIPLE=x86_64-apple-ios
            ;;
        *)
            echo "error: unsupported arch '$arch'" >&2
            exit 2
            ;;
    esac

    if ! rustup target list --installed | grep -qx "$TRIPLE"; then
        rustup target add "$TRIPLE"
    fi

    cargo build --profile "$CARGO_PROFILE" \
        --manifest-path "$REPO/Cargo.toml" \
        --target "$TRIPLE" \
        --bin "$BIN"

    EXECUTABLES+=("$CARGO_TARGET_DIR/$TRIPLE/$PROFILE_DIR/$BIN")
done

mkdir -p "$(dirname "$TARGET_BUILD_DIR/$EXECUTABLE_PATH")"
lipo -create -output "$TARGET_BUILD_DIR/$EXECUTABLE_PATH" "${EXECUTABLES[@]}"
