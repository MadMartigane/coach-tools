# Deployment

The deploy scripts run on the production machine only. The machine has no Rust, Trunk or `just` before the first-time setup. Everything the tools need is installed inside the repository under `.tools/`, so no system-wide install and no `sudo` are needed for the toolchain.

Targets:

| Target    | Recipe            | Branch    | URL                                  | Web root                                              |
|-----------|-------------------|-----------|--------------------------------------|-------------------------------------------------------|
| Production | `just prod`      | `main`    | `https://marius.click/coach-tools/`  | `/var/www/marius.click/html/coach-tools`              |
| Pre-production | `just pre-prod` | `develop` | `https://marius.click/coach-tools-preprod/` | `/var/www/marius.click/html/coach-tools-preprod` |

## First-time setup on the production machine

Work through this checklist in order. The scripts do everything not listed here.

1. Install the system packages that the Rust toolchain needs to compile Trunk and `just`, plus `git`, `curl` and `rsync`. Use the command for your distribution:
   - Debian or Ubuntu: `build-essential pkg-config libssl-dev git curl rsync`
   - RHEL or Fedora: `gcc gcc-c++ make pkg-config openssl-devel git curl rsync`
2. Clone the repository into the directory you want to keep it in, for example `~/coach-tools`:
   ```sh
   git clone <repository-url> ~/coach-tools
   ```
3. Choose the branch for this machine. Production uses `main`, pre-production uses `develop`. A machine can have both targets, but each deploy checks out the branch it needs before you run it:
   ```sh
   cd ~/coach-tools
   git checkout main      # or: git checkout develop
   ```
4. Install the toolchain into the repository:
   ```sh
   scripts/bootstrap.sh
   ```
   It installs rustup, the Rust version pinned in `rust-toolchain.toml` with its components and the `wasm32-unknown-unknown` target, Trunk 0.22.1 and just 1.59.0, all under `.tools/`. It is safe to run again.
5. Add this line to your shell startup file (for example `~/.bashrc`), replacing `<repo>` with the absolute path of the clone, and open a new shell:
   ```sh
   source <repo>/scripts/env.sh
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

1. stops unless the current branch is the required one (`main` or `develop`) and the working tree is clean;
2. builds the app with `trunk build --release --config app --public-url <base>`;
3. checks that `app/dist/index.html` references the base path;
4. sets a new service worker cache name in `app/dist/sw.js`;
5. syncs `app/dist/` into the web root with `rsync --delete`, or stages a copy inside the web root and swaps the files in place before pruning old assets when `rsync` is missing.

Before a deploy, switch to the branch you want to publish, for example `git checkout develop && git pull` for pre-production. Then run the recipe.

To check what a deploy would do without building or touching the web root:

```sh
scripts/deploy.sh pre-prod --dry-run
```

The dry run still checks the branch, the clean working tree and the presence of Trunk. It then prints the build, cache rename and sync commands and exits 0.

To deploy into a scratch directory instead of the web root, set `COACH_TOOLS_DEST`:

```sh
COACH_TOOLS_DEST=/tmp/coach-tools-deploy-test scripts/deploy.sh pre-prod
```

### Why `--public-url`

The app is served from a sub-path (`/coach-tools/` or `/coach-tools-preprod/`), not from the domain root. Trunk writes asset URLs such as `/app-….js` and `/app-…_bg.wasm` into `index.html` by default, which point to the domain root and fail on a sub-path. `--public-url` makes Trunk prefix every asset URL with the target's base path. The script refuses to deploy when `index.html` does not contain that base path.

### Why the service worker cache name changes

The service worker in `app/public/sw.js` serves cached files first. Without a change, a browser that has the app installed keeps serving the old bundle after a deploy. Each deploy therefore replaces the cache name `coach-tools-tabata-v1` with `coach-tools-tabata-<target>-<short commit sha>`, where the target is `prod` or `preprod`. The new worker installs a new cache, and the old cache is removed when the new worker activates.
