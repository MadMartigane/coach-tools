#!/usr/bin/env bash
# Prepares the repository-local toolchain under .tools/: rustup with the
# toolchain pinned in rust-toolchain.toml, plus Trunk and just.
# Idempotent and needs no sudo. It downloads from the network.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/env.sh"
cd "$ROOT"

readonly TRUNK_VERSION="0.22.1"
readonly JUST_VERSION="1.59.0"
readonly RUSTUP_INIT_URL="https://sh.rustup.rs"
readonly RUSTUP_INIT_SCRIPT="$ROOT/.tools/rustup-init.sh"

die() {
  printf 'error: %s\n' "$1" >&2
  exit 1
}

# Downloads the rustup installer and runs it without installing a toolchain.
# The toolchain itself is installed later by `rustup show`, from rust-toolchain.toml.
install_rustup() {
  mkdir -p "$ROOT/.tools"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL -o "$RUSTUP_INIT_SCRIPT" "$RUSTUP_INIT_URL"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$RUSTUP_INIT_SCRIPT" "$RUSTUP_INIT_URL"
  else
    die "neither curl nor wget is installed"
  fi
  sh "$RUSTUP_INIT_SCRIPT" -y --no-modify-path --profile minimal --default-toolchain none
}

# Succeeds when $CARGO_HOME/bin/<tool> exists, is executable, and reports exactly "<tool> <version>".
# Matching binaries elsewhere on PATH are deliberately ignored.
has_tool_version() {
  local tool="$1" expected_version="$2"
  local bin="$CARGO_HOME/bin/$tool"
  [[ -x "$bin" ]] || return 1
  [[ "$("$bin" --version)" == "$tool $expected_version" ]]
}

# Installs a crates.io tool at a pinned version into $CARGO_HOME, unless already present.
install_cargo_tool() {
  local tool="$1" version="$2"
  if has_tool_version "$tool" "$version"; then
    echo "$tool $version is already installed"
    return
  fi
  echo "Installing $tool $version"
  if ! "$CARGO_HOME/bin/cargo" install --locked --version "$version" "$tool" --root "$CARGO_HOME"; then
    cat >&2 <<EOF
error: cargo install $tool $version failed.
It compiles native code, so the system build packages are needed first.
  Debian: build-essential pkg-config libssl-dev
  RHEL:   gcc gcc-c++ make pkg-config openssl-devel
Install them with your system package manager, then run this script again.
EOF
    exit 1
  fi
}

if [[ ! -x "$CARGO_HOME/bin/rustup" ]]; then
  install_rustup
fi

"$CARGO_HOME/bin/rustup" show
install_cargo_tool trunk "$TRUNK_VERSION"
install_cargo_tool just "$JUST_VERSION"

echo
echo "Resolved versions:"
"$CARGO_HOME/bin/rustc" --version
"$CARGO_HOME/bin/trunk" --version
"$CARGO_HOME/bin/just" --version

echo
echo "Add this line to your shell startup file (for example ~/.bashrc):"
echo "source $ROOT/scripts/env.sh"
