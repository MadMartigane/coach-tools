#!/usr/bin/env bash
# Builds the coach-tools app for one target and publishes it to that target's web root.
# Runs on the production machine only.
#
# Usage: scripts/deploy.sh prod|pre-prod [--dry-run] [--bump patch|minor|major | --no-bump]
# --bump and --no-bump apply to prod only. prod bumps the patch version by default, commits, tags and pushes it before building.
# Environment: COACH_TOOLS_DEST overrides the destination directory.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/env.sh"
cd "$ROOT"

readonly USAGE="usage: scripts/deploy.sh prod|pre-prod [--dry-run] [--bump patch|minor|major | --no-bump]"
readonly PUBLIC_HOST="https://marius.click"
readonly BUILD_DIR="app/dist"
readonly INDEX_FILE="$BUILD_DIR/index.html"
readonly SERVICE_WORKER_FILE="$BUILD_DIR/sw.js"
# Cache name as committed in app/public/sw.js; rewritten on every deploy.
readonly PLACEHOLDER_CACHE_NAME="coach-tools-v1"
readonly WORKSPACE_MANIFEST="Cargo.toml"
readonly LOCK_FILE="Cargo.lock"
readonly REMOTE="origin"
readonly PULL_REQUEST_URL="https://github.com/MadMartigane/coach-tools/compare/develop...main"
readonly BUILD_SHA_VARIABLE="COACH_TOOLS_BUILD_SHA"

dry_run=0
TARGET=""
BASE_PATH=""
DEST=""
REQUIRED_BRANCH=""
CACHE_TARGET=""
COMMIT_SHA=""
bump_level="patch"
bump_given=0
no_bump=0
VERSION=""

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
      # Pre-production accepts any branch (feature branches included); production stays on main.
      REQUIRED_BRANCH=""
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
  # An empty REQUIRED_BRANCH means the target accepts any branch.
  if [[ -z "$REQUIRED_BRANCH" ]]; then
    return
  fi
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

# Runs the repository gate (formatting, host and browser lints, tests) so a commit that fails them is never published.
run_checks() {
  if ! run just check; then
    die "just check failed: fix the formatting, lint or test errors above before deploying"
  fi
}

# Production builds unset the build SHA so Home shows only the version; pre-production embeds the commit.
build_app() {
  local -a build_env
  if [[ "$TARGET" == prod ]]; then
    build_env=(-u "$BUILD_SHA_VARIABLE")
  else
    build_env=("$BUILD_SHA_VARIABLE=$COMMIT_SHA")
  fi
  run env "${build_env[@]}" trunk build --release --config app --public-url "$BASE_PATH"
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
  local cache_name="coach-tools-$CACHE_TARGET-$COMMIT_SHA"
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
    echo "Dry run complete: would deploy v$VERSION (commit $COMMIT_SHA) to $PUBLIC_HOST$BASE_PATH"
    return
  fi
  echo "Deployed v$VERSION (commit $COMMIT_SHA) to $PUBLIC_HOST$BASE_PATH"
  if [[ "$TARGET" == prod ]]; then
    echo "Open the pull request main -> develop: $PULL_REQUEST_URL"
  fi
}

# Parses the options that follow the target, in any order.
parse_options() {
  while (( $# > 0 )); do
    case "$1" in
      --dry-run)
        dry_run=1
        ;;
      --bump)
        if (( $# < 2 )) || [[ "$2" != patch && "$2" != minor && "$2" != major ]]; then
          die "--bump expects patch, minor or major. $USAGE"
        fi
        bump_level="$2"
        bump_given=1
        shift
        ;;
      --no-bump)
        no_bump=1
        ;;
      *)
        die "$USAGE"
        ;;
    esac
    shift
  done

  if (( bump_given && no_bump )); then
    die "--bump and --no-bump cannot be combined"
  fi
  if [[ "$TARGET" != prod ]] && (( bump_given || no_bump )); then
    die "--bump and --no-bump apply to prod only"
  fi
}

# Prints the version from the single 'version = "X.Y.Z"' line of WORKSPACE_MANIFEST.
read_workspace_version() {
  local -a versions
  mapfile -t versions < <(sed -n 's/^version = "\([^"]*\)"$/\1/p' "$WORKSPACE_MANIFEST")
  if (( ${#versions[@]} != 1 )) || [[ -z "${versions[0]}" ]]; then
    die "$WORKSPACE_MANIFEST must hold exactly one 'version = \"X.Y.Z\"' line, in [workspace.package]"
  fi
  printf '%s\n' "${versions[0]}"
}

# Fetches the remote and refuses to release from a local branch that lacks remote commits.
check_origin() {
  if ! git fetch --tags "$REMOTE"; then
    die "git fetch $REMOTE failed; a production deploy needs network access to GitHub"
  fi
  local remote_commits_missing
  remote_commits_missing="$(git rev-list --count "HEAD..$REMOTE/$REQUIRED_BRANCH")"
  if (( remote_commits_missing > 0 )); then
    die "local $REQUIRED_BRANCH is behind $REMOTE/$REQUIRED_BRANCH or has diverged; run 'git pull --ff-only' and deploy again"
  fi
}

# Bumps the workspace version, then commits, tags and pushes the release before anything is built.
release() {
  local current next tag escaped_current
  current="$(read_workspace_version)"
  next="$(cargo run --quiet --locked --package bump-version -- "$current" "$bump_level")" \
    || die "cannot compute the next version from '$current'"
  tag="v$next"
  if git rev-parse --quiet --verify "refs/tags/$tag" >/dev/null; then
    die "tag $tag already exists locally or on $REMOTE; the version in $WORKSPACE_MANIFEST is behind the latest release"
  fi

  escaped_current="${current//./\\.}"
  # Until the release commit exists, any exit restores the manifest and lock file, so a rerun starts from a clean tree.
  trap 'git checkout HEAD -- "$WORKSPACE_MANIFEST" "$LOCK_FILE"' EXIT
  run sed -i "s/^version = \"$escaped_current\"\$/version = \"$next\"/" "$WORKSPACE_MANIFEST"
  run cargo update --workspace

  if (( dry_run == 0 )); then
    local bumped_version changed_files expected_changes
    # git status --porcelain sorts paths, so the lock file comes first.
    expected_changes=" M $LOCK_FILE"$'\n'" M $WORKSPACE_MANIFEST"
    bumped_version="$(read_workspace_version)"
    changed_files="$(git status --porcelain)"
    if [[ "$bumped_version" != "$next" || "$changed_files" != "$expected_changes" ]]; then
      die "the version bump must change only $WORKSPACE_MANIFEST and $LOCK_FILE; git status shows: $changed_files"
    fi
  fi

  run git add -- "$WORKSPACE_MANIFEST" "$LOCK_FILE"
  if ! run git commit -m "chore(release): $tag"; then
    die "the release commit failed (the pre-commit hook runs just audit, which needs network access); the version bump was reverted"
  fi
  trap - EXIT
  run git tag --annotate "$tag" --message "Release $tag"
  if ! run git push --atomic "$REMOTE" "$REQUIRED_BRANCH" "refs/tags/$tag"; then
    die "push to $REMOTE failed; the release commit and tag $tag exist locally only. Fix the access to $REMOTE, then run scripts/deploy.sh prod --no-bump"
  fi
  VERSION="$next"
}

# Pushes an existing release (the branch and its local tag) without bumping, committing or tagging.
push_pending() {
  VERSION="$(read_workspace_version)"
  local tag="v$VERSION"
  local -a push_refs=("$REQUIRED_BRANCH")
  if git rev-parse --quiet --verify "refs/tags/$tag" >/dev/null; then
    push_refs+=("refs/tags/$tag")
  fi
  if ! run git push --atomic "$REMOTE" "${push_refs[@]}"; then
    die "push to $REMOTE failed; nothing was deployed"
  fi
}

main() {
  (( $# >= 1 )) || die "$USAGE"
  resolve_target "$1"
  shift
  parse_options "$@"

  check_branch
  check_clean_tree
  check_trunk
  if [[ "$TARGET" == prod ]]; then
    check_origin
  fi
  run_checks

  if [[ "$TARGET" != prod ]]; then
    VERSION="$(read_workspace_version)"
  elif (( no_bump )); then
    push_pending
  else
    release
  fi
  COMMIT_SHA="$(git rev-parse --short HEAD)"

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
