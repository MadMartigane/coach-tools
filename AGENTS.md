# AGENTS.md

Instructions for AI coding agents working on this repository.

## Project

`coach-tools` is a set of small offline web tools for sports coaching. The first tool is TABATA TIMER, an interval timer for phones in portrait mode. The project also explores WebAssembly with Rust. The UI is Leptos 0.8 in client-side rendering, built with Trunk.

## Language

- Code, comments, documentation, commit messages: English.
- Talk with the user in the user's language.

## Branches

- `main`: stable, production-ready. Never commit features directly.
- `develop`: integration. Small changes are committed here directly.
- `feat/<name>`: created from `develop` for important features. Merged back into `develop`.

## Commits

Use Conventional Commits: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `perf`, `build`, `ci`. Keep one logical change per commit.

## Project memory

The shared memory connector is `.memory/config.yml` (key `memory_path`). It points outside the repository. Do not commit memory content to the repository.

## Platform constraints for TABATA TIMER

- iOS cannot lock the screen orientation. Do not rely on `screen.orientation.lock()` or the manifest `orientation` field for iPhone.
- The Screen Wake Lock is released when the page becomes hidden. Re-request it on `visibilitychange`.
- Web Audio needs a user gesture. Create or resume the `AudioContext` inside a tap handler.
- Hidden pages throttle timers. Compute the remaining time from timestamps, never by counting timer ticks.

## Verification

Before a change is finished, run the test or the scenario that covers it. Timer logic must have Rust unit tests with a simulated clock.
Run `just check` before every commit. It runs formatting, both lints and the tests.

## UI conventions

Every UI change follows `docs/design-system.md`. Colours, sizes, radii and durations come from its tokens only. Run its pre-merge checklist before merging any UI change, and update the document when a new token is needed.
