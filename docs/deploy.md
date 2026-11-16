# Deployment

The deploy scripts run on the production machine only. The machine has no Rust, Trunk or `just` before the first-time setup. Everything the tools need is installed inside the repository under `.tools/`, so no system-wide install and no `sudo` are needed for the toolchain.

Targets:

| Target    | Recipe            | Branch    | URL                                  | Web root                                              |
|-----------|-------------------|-----------|--------------------------------------|-------------------------------------------------------|
| Production | `just prod`      | `main`    | `https://marius.click/coach-tools/`  | `/var/www/marius.click/html/coach-tools`              |
| Pre-production | `just pre-prod` | any branch | `https://marius.click/coach-tools-preprod/` | `/var/www/marius.click/html/coach-tools-preprod` |

## First-time setup on the production machine

Work through this checklist in order. The scripts do everything not listed here.

1. Install the system packages that the Rust toolchain needs to compile Trunk, `just` and cargo-deny, plus `git`, `curl` and `rsync`. Use the command for your distribution:
   - Debian or Ubuntu: `build-essential pkg-config libssl-dev git curl rsync`
   - RHEL or Fedora: `gcc gcc-c++ make pkg-config openssl-devel git curl rsync`
2. Clone the repository into the directory you want to keep it in, for example `~/coach-tools`:
   ```sh
   git clone <repository-url> ~/coach-tools
   ```
3. Choose the branch for this machine. Production deploys from `main`; pre-production deploys from any branch, so it can publish a feature branch for review. A machine can have both targets, but a production deploy requires the `main` branch to be checked out before you run it:
   ```sh
   cd ~/coach-tools
   git checkout main      # required only for `just prod`
   ```
4. Install the toolchain into the repository:
   ```sh
   scripts/bootstrap.sh
   ```
   It installs rustup, the Rust version pinned in `rust-toolchain.toml` with its components and the `wasm32-unknown-unknown` target, Trunk 0.22.1, just 1.59.0 and cargo-deny 0.20.2, all under `.tools/`. It also enables the repository Git hooks with `git config core.hooksPath .githooks`. It is safe to run again.

   `just prod` and `just pre-prod` run `just check` before building, so the production machine needs `just`, the `clippy` and `rustfmt` components and the `wasm32-unknown-unknown` target, all installed by `scripts/bootstrap.sh` from `rust-toolchain.toml`; check them with `rustup component list --installed` and `rustup target list --installed`.
5. Add the line for your shell to its startup file, replacing `<repo>` with the absolute path of the clone, and open a new shell:
   - bash, in `~/.bashrc`:
     ```sh
     source <repo>/scripts/env.sh
     ```
   - fish, in `~/.config/fish/config.fish`:
     ```fish
     source <repo>/scripts/env.fish
     ```
6. Create the two web roots and give the deploy user write access to them:
   ```sh
   sudo mkdir -p /var/www/marius.click/html/coach-tools /var/www/marius.click/html/coach-tools-preprod
   sudo chown "$USER": /var/www/marius.click/html/coach-tools /var/www/marius.click/html/coach-tools-preprod
   ```
   Deploys only write into these directories, so no further `sudo` is needed.
7. Declare the WebAssembly MIME type in the web server. Without it, browsers refuse to compile the app. For nginx, add this inside the `http` block or the `server` block that serves `marius.click`:
   ```nginx
   types { application/wasm wasm; }
   ```
   Then test and reload nginx, for example with `sudo nginx -t && sudo systemctl reload nginx`.

## Deploying

Run the recipes from the repository root, on the production machine:

```sh
just prod        # builds the main branch and publishes it to /coach-tools/
just pre-prod    # builds the develop branch and publishes it to /coach-tools-preprod/
```

Each recipe calls `scripts/deploy.sh <target>`, which:

1. stops unless the current branch is the required one (`main` for `prod`; pre-production accepts any branch) and the working tree is clean. For `prod` it then runs `git fetch --tags origin` and stops when the local `main` is behind or has diverged from `origin/main`;
2. runs `just check` (formatting, the host and browser lints, the tests) and stops with `error: just check failed: …` when it fails;
3. for `prod` only, bumps `[workspace.package] version` in `Cargo.toml` (patch by default), refreshes `Cargo.lock` with `cargo update --workspace`, commits both as `chore(release): vX.Y.Z` through the pre-commit hook (it needs network access), creates the annotated tag `vX.Y.Z` and pushes `main` and the tag to `origin`;
4. builds the app with `trunk build --release --config app --public-url <base>`; pre-production also passes `COACH_TOOLS_BUILD_SHA=<short sha>`, so its home screen shows the commit;
5. checks that `app/dist/index.html` references the base path;
6. sets a new service worker cache name in `app/dist/sw.js`;
7. syncs `app/dist/` into the web root with `rsync --delete`, or stages a copy inside the web root and swaps the files in place before pruning old assets when `rsync` is missing;
8. for `prod`, prints `Open the pull request main -> develop: https://github.com/MadMartigane/coach-tools/compare/develop...main`.

Before a deploy, check out the branch you want to publish, for example a feature branch for pre-production, `git checkout main && git pull` for production. Then run the recipe.

After a production deploy, open the pull request `main` -> `develop` that the deploy prints, and merge it with a merge commit, like the existing `develop` -> `main` pull requests, so `develop` receives the release commit.

To check what a deploy would do without building or touching the web root:

```sh
scripts/deploy.sh pre-prod --dry-run
```

The dry run still checks the branch, the clean working tree and the presence of Trunk. A `prod` dry run also fetches `origin`, checks that `main` is not behind, and computes the next version (it builds the small `bump-version` helper). It then prints the `just check`, build, cache rename and sync commands, and for `prod` the bump, commit, tag and push commands, without running them, and exits 0. A `pre-prod` dry run prints no Git command.

To deploy into a scratch directory instead of the web root, set `COACH_TOOLS_DEST`:

```sh
COACH_TOOLS_DEST=/tmp/coach-tools-deploy-test scripts/deploy.sh pre-prod
```

### Production options

`--bump patch|minor|major` (default `patch`) chooses the SemVer part that `prod` increments. `--no-bump` redeploys the current version without a new commit or tag; when `origin` lacks the local `main` or the local tag `vX.Y.Z`, it pushes them first. Both options apply to `prod` only. `just prod` passes no option, so run `scripts/deploy.sh prod --bump minor` directly for a minor release.

### When a production deploy fails

- Commit failure (for example an offline pre-commit hook): the bump is reverted and the working tree is clean. Rerun `just prod` once the hook can run.
- Push failure: the release commit and tag stay local and nothing is built. Fix the access to `origin`, then run `scripts/deploy.sh prod --no-bump`.
- Build or sync failure after the push: the version is already on GitHub. Fix the cause, then run `scripts/deploy.sh prod --no-bump`; do not bump a second time.
- Local `main` behind `origin/main`: the deploy stops before any change. Run `git pull --ff-only`, then rerun the deploy.

### Versioning

The app follows [Semantic Versioning](https://semver.org). The single source of truth is `[workspace.package] version` in the root `Cargo.toml`; every crate inherits it with `version.workspace = true`. Only `scripts/deploy.sh prod` changes it. Never edit it by hand on `develop` or a feature branch: that keeps the pull request `main` -> `develop` free of conflicts on `Cargo.toml` and `Cargo.lock`.

The home screen shows `vX.Y.Z` in production and `vX.Y.Z+<short sha>` in pre-production. `CURRENT_VERSION` in `app/src/platform/storage.rs` is the storage schema version, not the app version, and is unrelated.

### Why `--public-url`

The app is served from a sub-path (`/coach-tools/` or `/coach-tools-preprod/`), not from the domain root. Trunk writes asset URLs such as `/app-….js` and `/app-…_bg.wasm` into `index.html` by default, which point to the domain root and fail on a sub-path. `--public-url` makes Trunk prefix every asset URL with the target's base path. The script refuses to deploy when `index.html` does not contain that base path.

### Why the service worker cache name changes

The service worker in `app/public/sw.js` serves cached files first. Without a change, a browser that has the app installed keeps serving the old bundle after a deploy. Each deploy therefore replaces the cache name `coach-tools-v1` with `coach-tools-<target>-<short commit sha>`, where the target is `prod` or `preprod`. The new worker installs a new cache, and the old cache is removed when the new worker activates. The activating worker deletes every other cache, so the caches named before the app became coach-tools are removed by the first deploy after the rename. When the new worker has installed, the app shows a banner that follows the app language: "Nouvelle version disponible — Recharger" in French, "New version available — Reload" in English; tapping the button reloads the page, so a deploy applies within a single launch of the installed PWA. The cache name keeps the commit SHA rather than the version, because a pre-production deploy or a `--no-bump` redeploy keeps the same version but must still change the bytes of `sw.js`.
