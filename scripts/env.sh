# Sourced, not executed: `source scripts/env.sh`.
# Points rustup and cargo at the repository's .tools/ directory, so the
# toolchain never touches a system-wide installation.

# Repository root: the parent of scripts/, whatever the caller's working directory.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

export RUSTUP_HOME="$ROOT/.tools/rustup"
export CARGO_HOME="$ROOT/.tools/cargo"

case ":$PATH:" in
  *":$CARGO_HOME/bin:"*) ;;
  *) export PATH="$CARGO_HOME/bin:$PATH" ;;
esac
