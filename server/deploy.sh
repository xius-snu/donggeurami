#!/usr/bin/env bash
# Builds the game server for Linux on this PC and puts it on the Vultr VM.
#
#   server/deploy.sh            build, upload, restart
#   server/deploy.sh --logs     the server's log, following it
#
# Run from Git Bash on Windows, in the repo. It cross-compiles: the server is
# pure Rust, so the musl target and rust-lld (both from rustup:
# `rustup target add x86_64-unknown-linux-musl`) make a static Linux binary
# with no C toolchain and no Docker. The VM needs nothing installed for it.
#
# The key is the one made for the VM on 2026-10-08, ~/.ssh/roundtown_vultr;
# RT_HOST and RT_KEY say otherwise.

set -euo pipefail

HOST="${RT_HOST:-root@141.164.62.193}"
KEY="${RT_KEY:-$HOME/.ssh/roundtown_vultr}"
SSH=(ssh -i "$KEY" -o BatchMode=yes "$HOST")
TARGET=x86_64-unknown-linux-musl

cd "$(dirname "$0")/.."

if [[ "${1:-}" == "--logs" ]]; then
    exec "${SSH[@]}" journalctl -u roundtown-server -n 100 -f
fi

export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=rust-lld
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-C linker-flavor=ld.lld -C target-feature=+crt-static"
cargo build -p roundtown_server --release --target "$TARGET"

scp -i "$KEY" -o BatchMode=yes \
    "target/$TARGET/release/roundtown-server" \
    server/roundtown-server.service \
    "$HOST:/tmp/"

"${SSH[@]}" 'bash -s' <<'EOF'
set -e
install -m 755 /tmp/roundtown-server /opt/roundtown/roundtown-server.new
mv /opt/roundtown/roundtown-server.new /opt/roundtown/roundtown-server
install -m 644 /tmp/roundtown-server.service /etc/systemd/system/roundtown-server.service
rm -f /tmp/roundtown-server /tmp/roundtown-server.service
[ -f /etc/roundtown/env ] || install -m 640 -g roundtown /dev/null /etc/roundtown/env
systemctl daemon-reload
systemctl enable --quiet roundtown-server
systemctl restart roundtown-server
sleep 1
systemctl --no-pager --lines=0 status roundtown-server | head -5
journalctl -u roundtown-server -n 5 --no-pager -o cat
EOF
