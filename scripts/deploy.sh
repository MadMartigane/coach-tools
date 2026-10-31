#!/usr/bin/env bash
# Builds the TABATA TIMER app for one target and publishes it to that target's web root.
# Runs on the production machine only.
#
# Usage: scripts/deploy.sh prod|pre-prod [--dry-run]
# Environment: COACH_TOOLS_DEST overrides the destination directory.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/env.sh"
cd "$ROOT"

readonly USAGE="usage: scripts/deploy.sh prod|pre-prod [--dry-run]"
readonly PUBLIC_HOST="https://marius.click"
readonly BUILD_DIR="app/dist"
readonly INDEX_FILE="$BUILD_DIR/index.html"
readonly SERVICE_WORKER_FILE="$BUILD_DIR/sw.js"
# Cache name as committed in app/public/sw.js; rewritten on every deploy.
readonly PLACEHOLDER_CACHE_NAME="coach-tools-tabata-v1"

dry_run=0
TARGET=""
BASE_PATH=""
DEST=""
REQUIRED_BRANCH=""
CACHE_TARGET=""
COMMIT_SHA=""

die() {
  printf 'error: %s\n' "$1" >&2
  exit 1
}

# Prints the command in dry-run mode; otherwise runs it.
run() {
  if (( dry_run )); then
    printf '[dry-run] %s\n' "$(printf '%q ' "$@")"
  else
    "$@"
  fi
}

# Sets the per-target settings. The destination can be overridden for testing.
resolve_target() {
  TARGET="$1"
  case "$TARGET" in
    prod)
      BASE_PATH="/coach-tools/"
      REQUIRED_BRANCH="main"
      CACHE_TARGET="prod"
      DEST="/var/www/marius.click/html/coach-tools"
      ;;
    pre-prod)
      BASE_PATH="/coach-tools-preprod/"
      REQUIRED_BRANCH="develop"
      CACHE_TARGET="preprod"
      DEST="/var/www/marius.click/html/coach-tools-preprod"
      ;;
    *)
      die "unknown target '$TARGET'. $USAGE"
      ;;
  esac
  DEST="${COACH_TOOLS_DEST:-$DEST}"
}

check_branch() {
  local current_branch
  current_branch="$(git rev-parse --abbrev-ref HEAD)"
  if [[ "$current_branch" != "$REQUIRED_BRANCH" ]]; then
    die "$TARGET must be deployed from branch '$REQUIRED_BRANCH', but the current branch is '$current_branch'"
  fi
}

check_clean_tree() {
  if [[ -n "$(git status --porcelain)" ]]; then
    die "working tree is dirty; commit or stash your changes before deploying"
  fi
}

check_trunk() {
  if ! command -v trunk >/dev/null 2>&1; then
    die "trunk not found; run scripts/bootstrap.sh first"
  fi
}

build_app() {
  run trunk build --release --config app --public-url "$BASE_PATH"
}

verify_build() {
  if [[ ! -f "$INDEX_FILE" ]]; then
    die "build did not produce $INDEX_FILE"
  fi
  if ! grep -qF "$BASE_PATH" "$INDEX_FILE"; then
    die "$INDEX_FILE does not reference $BASE_PATH; the build did not use the expected public URL"
  fi
}

# Gives each deploy its own service worker cache, so browsers drop the previous app.
rewrite_cache_name() {
  local cache_name="coach-tools-tabata-$CACHE_TARGET-$COMMIT_SHA"
  run sed -i "s/\"$PLACEHOLDER_CACHE_NAME\"/\"$cache_name\"/" "$SERVICE_WORKER_FILE"
  if (( dry_run == 0 )) && ! grep -qF "\"$cache_name\"" "$SERVICE_WORKER_FILE"; then
    die "$SERVICE_WORKER_FILE does not contain \"$PLACEHOLDER_CACHE_NAME\"; cannot set the cache name"
  fi
}

prepare_destination() {
  if (( dry_run )); then
    run mkdir -p "$DEST"
    return
  fi
  if ! mkdir -p "$DEST" || [[ ! -w "$DEST" ]]; then
    die "cannot write to $DEST. Run once: sudo mkdir -p $DEST && sudo chown \"\$USER\": $DEST"
  fi
}

# Copies the build without rsync. The copy is staged inside $DEST, so a failed copy
# leaves the live files untouched; staged files then replace the live ones in place.
sync_destination() {
  if command -v rsync >/dev/null 2>&1; then
    run rsync -a --delete "$BUILD_DIR/" "$DEST/"
  else
    sync_by_copy
  fi
}

sync_by_copy() {
  local stage_dir="$DEST/.deploy-stage-$$"
  local stage_name="${stage_dir##*/}"
  local enumerate_dir destination_path staged_path entry name

  run rm -rf "$stage_dir"
  run mkdir -p "$stage_dir"
  if ! run cp -a "$BUILD_DIR/." "$stage_dir/"; then
    run rm -rf "$stage_dir"
    die "failed to stage build into $stage_dir"
  fi

  # Moves entries one by one. mv cannot replace a non-empty directory with a directory,
  # and a directory cannot be moved over an existing entry, so such destinations are
  # removed first. A dry run stages nothing, so it enumerates the build output instead.
  enumerate_dir=""
  if [[ -d "$stage_dir" ]]; then
    enumerate_dir="$stage_dir"
  elif (( dry_run )) && [[ -d "$BUILD_DIR" ]]; then
    enumerate_dir="$BUILD_DIR"
  fi

  if [[ -n "$enumerate_dir" ]]; then
    while IFS= read -r -d '' entry; do
      name="${entry##*/}"
      destination_path="$DEST/$name"
      staged_path="$stage_dir/$name"
      if [[ -d "$destination_path" && ! -L "$destination_path" ]]; then
        run rm -rf -- "$destination_path"
      elif [[ -d "$entry" && ! -L "$entry" && ( -e "$destination_path" || -L "$destination_path" ) ]]; then
        run rm -rf -- "$destination_path"
      fi
      run mv -f -T -- "$staged_path" "$destination_path"
    done < <(find "$enumerate_dir" -mindepth 1 -maxdepth 1 -print0)
  fi

  # Lists the destination only to find stale entries; nothing is removed here.
  # In dry-run either directory may be missing, so there is nothing to prune yet.
  if [[ -d "$DEST" && -d "$BUILD_DIR" ]]; then
    while IFS= read -r -d '' entry; do
      name="${entry##*/}"
      if [[ ! -e "$BUILD_DIR/$name" && ! -L "$BUILD_DIR/$name" ]]; then
        run rm -rf -- "$entry"
      fi
    done < <(find "$DEST" -mindepth 1 -maxdepth 1 -not -name "$stage_name" -print0)
  fi

  run rmdir "$stage_dir"
}

report() {
  if (( dry_run )); then
    echo "Dry run complete: would deploy commit $COMMIT_SHA to $PUBLIC_HOST$BASE_PATH"
  else
    echo "Deployed commit $COMMIT_SHA to $PUBLIC_HOST$BASE_PATH"
  fi
}

main() {
  if (( $# < 1 || $# > 2 )); then
    die "$USAGE"
  fi
  if (( $# == 2 )); then
    if [[ "$2" != "--dry-run" ]]; then
      die "$USAGE"
    fi
    dry_run=1
  fi

  resolve_target "$1"
  COMMIT_SHA="$(git rev-parse --short HEAD)"

  check_branch
  check_clean_tree
  check_trunk

  build_app
  if (( dry_run == 0 )); then
    verify_build
  fi
  rewrite_cache_name
  prepare_destination
  sync_destination
  report
}

main "$@"
