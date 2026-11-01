# Sourced, not executed: `source scripts/env.fish`.
# Fish counterpart of env.sh: points rustup and cargo at the repository's
# .tools/ directory, so the toolchain never touches a system-wide installation.

# Repository root: the parent of scripts/, whatever the caller's working directory.
set -l root (cd (dirname (status dirname)); and pwd)

set -gx RUSTUP_HOME $root/.tools/rustup
set -gx CARGO_HOME $root/.tools/cargo

if not contains -- $CARGO_HOME/bin $PATH
    set -gx PATH $CARGO_HOME/bin $PATH
end
