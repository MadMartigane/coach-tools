# coach-tools

A small web application that groups the tools and utilities a sports coach uses.

## Tools

### TABATA TIMER (in design)

An interval timer that runs fully offline in a phone browser.

- Save as many named timers as needed.
- Each timer defines, in order: preparation, work, rest, cycles per set, sets, rest between sets, cool-down.
- Portrait layout, large countdown, phase title and background colour per phase.
- Bottom progress bar: current phase, total cycles, elapsed time, remaining time, total duration.
- Sound cues for each phase change.
- The screen stays on during the whole session, pauses included, while the browser allows it.

## Technology

Rust compiled to WebAssembly, served as a static Progressive Web App. The UI framework is not chosen yet.

## Status

Early stage. The repository holds the project foundations only.

## Branches

- `main`: stable branch.
- `develop`: integration branch. Small changes are committed here directly.
- `feat/<name>`: branch for an important feature, merged into `develop`.

Commits follow [Conventional Commits](https://www.conventionalcommits.org/).

## Limits by platform

- iOS does not lock the screen orientation. The app works in both orientations on iPhone.
- The browser releases the screen wake lock when the page is hidden or the phone locks.
- Sounds need one tap on Start before they can play.

## License

MIT. See [LICENSE](LICENSE).
