# TABATA TIMER Design System

Version 1. Single source of truth for every visual decision in this app.
Any colour, size, radius, duration or font used by the UI MUST come from the
tokens below. If a needed value does not exist, add the token here first,
then use it. Tokens are defined once, in `:root`, in one tokens stylesheet.

Every rule below is checkable by a human in under one minute, with browser
DevTools and the WebAIM contrast checker (https://webaim.org/resources/contrastchecker/).

## 1. Colour tokens

### Neutrals (dark theme — the only theme)

| Token                  | Value     | Use                                          |
|------------------------|-----------|----------------------------------------------|
| `--color-bg`           | `#0E1116` | Page background, ready screen                |
| `--color-surface`      | `#171C23` | Cards, progress strip                        |
| `--color-surface-raised` | `#222933` | Inputs, pressed surfaces                   |
| `--color-border`       | `#323B47` | Decorative separators ONLY (never an affordance) |
| `--color-text`         | `#F2F5F8` | Primary text (17.3:1 on `--color-bg`)        |
| `--color-text-muted`   | `#A3AEBB` | Secondary text (8.4:1 on bg, 7.6:1 on surface) |

### Actions

| Token             | Value     | Use                                                |
|-------------------|-----------|----------------------------------------------------|
| `--color-accent`  | `#B44A00` | Hero buttons on neutral screens; label `#FFFFFF` (5.4:1; 3.5:1 vs bg) |
| `--color-danger`  | `#D5393E` | Destructive buttons; label `#FFFFFF` (4.7:1)       |
| `--color-focus`   | `#6FB3FF` | Focus ring on neutral surfaces (8.6:1 vs bg)       |
| `--color-success` | `#2BD58C` | Terminé check mark, icons only (6.4:1 on done bg)  |

### Contrast law

- Text: >= 4.5:1 against its actual background. No exception, never rely on
  the WCAG large-text 3:1 allowance.
- UI boundaries (button edge, progress track, icons): >= 3:1 against adjacent
  colours (WCAG 1.4.11).
- Never lower text contrast with `opacity`; use a muted token or nothing.
- `--color-border` carries no meaning. An interactive element must remain
  identifiable without it.

## 2. Phase colour mapping

Base palette: Okabe & Ito "Color Universal Design" (colourblind-safe),
darkened where needed so labels reach 4.5:1. The phase name is ALWAYS written
as text on the screen; colour is never the only carrier of meaning.

| Phase             | Background token (value)          | Text token (value)        | Polarity | Ratio |
|-------------------|-----------------------------------|---------------------------|----------|-------|
| Préparation       | `--phase-prep-bg` `#E69F00`       | `--phase-prep-fg` `#10151B` | light  | 8.1:1 |
| Travail           | `--phase-work-bg` `#C24E00`       | `--phase-work-fg` `#FFFFFF` | dark   | 4.8:1 |
| Repos             | `--phase-rest-bg` `#0072B2`       | `--phase-rest-fg` `#FFFFFF` | dark   | 5.2:1 |
| Repos entre sets  | `--phase-rest-sets-bg` `#008063`  | `--phase-rest-sets-fg` `#FFFFFF` | dark | 4.9:1 |
| Retour au calme   | `--phase-cooldown-bg` `#56B4E9`   | `--phase-cooldown-fg` `#10151B` | light | 8.0:1 |
| Pause             | `--phase-paused-bg` `#262B33`     | `--phase-paused-fg` `#FFFFFF` | dark | 14.2:1 |
| Terminé           | `--phase-done-bg` `#0F3D2E`       | `--phase-done-fg` `#FFFFFF` | dark   | 12.2:1 |

Why it is safe: work (vermillion) vs rest (blue) differ on the blue-yellow
axis, which survives deuteranopia and protanopia (~8% of men). The two light
phases (prep, cool-down) use dark text and the dark phases white text, so
light vs dark phases stay distinct even in greyscale.

## 3. Typography

| Token             | Value                                                        | Use                       |
|-------------------|--------------------------------------------------------------|---------------------------|
| `--font-sans`     | `system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif` | Everything |
| `--text-xs`       | `13px`                                                       | Progress strip labels ONLY |
| `--text-sm`       | `16px`                                                       | Body text, summaries      |
| `--text-md`       | `20px`                                                       | Button labels, row titles |
| `--text-lg`       | `24px`                                                       | Screen titles, stepper values |
| `--text-phase`    | `clamp(28px, 8vw, 48px)`                                     | Phase title               |
| `--text-countdown` | `clamp(72px, min(30vw, 32dvh), 256px)`                      | Countdown digits          |

Rules:

- No web fonts: the app is offline and system numerals are excellent.
- Every numeric display uses `font-variant-numeric: tabular-nums`.
- Countdown is zero-padded `mm:ss` (`08:07`) so its width never changes.
- Countdown digits never animate: no flip, no pulse, no scale, ever.
- Phase titles: uppercase, weight 700, letter-spacing `0.04em`.

## 4. Spacing, radius, sizes, motion

Spacing: `--space-1` `4px`, `--space-2` `8px`, `--space-3` `12px`, `--space-4`
`16px`, `--space-5` `24px`, `--space-6` `32px`, `--space-7` `48px`, `--space-8` `64px`.

Radius: `--radius-sm` `8px` (chips), `--radius-md` `14px` (buttons, rows),
`--radius-lg` `24px` (hero), `--radius-full` `999px` (pills).

Control heights (minimums, width follows content unless stated):

| Token                  | Value | Applies to                                            |
|------------------------|-------|-------------------------------------------------------|
| `--size-hero`          | `96px` | Start / Pause / Resume, full width minus `--space-4` |
| `--size-hero-landscape`| `72px` | Same, in landscape                                    |
| `--size-btn`           | `64px` | Standard buttons (Save, New timer)                   |
| `--size-ghost`         | `56px` | Secondary (Modify, End session) and steppers         |
| `--size-icon`          | `48px` | Icon buttons (back, delete), square                  |
| `--size-row`           | `80px` | Timer list rows, whole row tappable                  |

Absolute floor for any target: 48x48px (Material asks 48dp, Apple HIG 44pt,
WCAG 2.2 AA 2.5.8 asks 24px — this project exceeds all three).

Motion tokens: `--motion-fast` `150ms` (presses), `--motion-phase` `250ms`
(phase background crossfade), `--ease-standard` `cubic-bezier(0, 0, 0.2, 1)`.

Under `prefers-reduced-motion: reduce`: no transform, scale or parallax
anywhere; only opacity/colour transitions of at most 250ms; the progress fill
steps instantly.

## 5. Component rules

### Controls on a phase screen (Pause / Resume / Rejouer / Accueil)

Fill with the phase's `-fg` token and label with the phase's `-bg` token
(the control inverts the phase colours). Every pair already passes 4.5:1 and
the boundary passes 3:1 by construction. No extra tokens, works on all seven
states, including light phases where a white button would fail.

### Buttons on neutral screens

- Hero: `--color-accent` fill, `#FFFFFF` label, `--size-hero`, `--radius-lg`,
  full width. One hero per screen, maximum.
- Standard: `--color-surface-raised` fill, `--color-text` label, `--size-btn`.
- Ghost: transparent fill, 2px `--color-border`, `--color-text` label,
  `--size-ghost`. `--color-border` is decorative; the label is the affordance.
- Pressed: `filter: brightness(0.85)` on the fill, nothing else. Brightness
  only, no transforms, so the state also works under reduced motion.
- Disabled: `opacity: 0.5` on the whole button plus `aria-disabled="true"`;
  never colour alone.
- Focus (all interactive elements, no exceptions): `outline: 3px solid`
  `--color-focus` on neutral screens, the phase's `-bg` colour on phase
  screens; `outline-offset: 3px`. The ring must be visible against both the
  element and its background.
- Destructive (delete timer, end session): `--color-danger` or ghost, trash
  icon plus text label, then a second tap on a changed label within 3s
  ("Confirmer l'arrêt"). Two taps minimum, always.

### Countdown

- The countdown zone occupies at least one third of the screen height
  (`min-height: 33.4dvh`) and grows (`flex: 1`).
- Digits use `--text-countdown`, phase `-fg` colour, centred, tabular-nums.
- "88:88" must fit with `--space-4` side margins at 360px width.

### Progress strip (session screen, bottom, full width)

- Sits on `--color-surface`, above `env(safe-area-inset-bottom)`, so its text
  contrast is constant regardless of the phase colour.
- Row 1: phase name, `Cycle x/y`, `Set n/m` (`--text-sm`, `--color-text`).
- Track: 8px tall, one segment per phase instance, segment width proportional
  to duration, segment fill = phase `-bg` colour. Future and current segments
  full opacity (>= 3:1 vs surface); past segments at 25% opacity.
- Row 2, three columns: elapsed / total / remaining. Values `--text-sm`
  `--color-text`, labels `--text-xs` `--color-text-muted`. Formats `mm:ss`,
  `h:mm:ss` over one hour.

### Timer list rows and editor

- Row: timer name (`--text-md`, 700) above a summary line (`--text-sm`,
  muted, e.g. `Prép 10 · 20/10 ×8 · 4 sets · 31:00`). Height >= `--size-row`,
  the entire row is one tap target.
- Steppers: value in `--text-lg`, minus/plus buttons at `--size-ghost`.
- List and editor scroll normally; the session screen never scrolls.

### Icons and logo

- Icons: Lucide (ISC licence), inline SVG components in `src/ui/icons.rs`,
  24px grid, `currentColor`. No icon fonts, no CDN, no external assets.
  The ISC notice lives in `THIRD_PARTY_LICENSES.md`.
- Actions show icon plus text label. Icon-only allowed for the back chevron
  only, with `aria-label`.
- Logo: three vertical rounded bars, tall-short-tall (two efforts around a
  rest): outer bars `--color-text`, middle bar `--phase-cooldown-bg`, on
  `--color-bg`. No gradients, no text in the mark.
- Manifest icons: 192px, 512px, maskable 512px (mark inside the central 80%
  circle), apple-touch-icon 180px. `theme_color` `#0E1116`, updated to the
  phase colour during a session if the platform allows it.

## 6. Layout rules

- Portrait session screen is a fixed-height column: header (logo mark small,
  timer name, `Set n/m` chip) / countdown zone / controls / progress strip.
  `height: 100vh; height: 100dvh;` (fallback first), no scrolling, ever.
- `viewport-fit=cover`; top and bottom safe-area insets reserved on the
  session screen so the iPhone home indicator never overlaps the hero button
  or the progress strip.
- Landscape (media query `orientation: landscape` with `max-height: 500px`):
  header on top, countdown zone on the left (~60% width), controls stacked on
  the right, progress strip full width at the bottom. No scroll at 568x320.
- Ready and Terminé screens reuse the session column structure on
  `--color-bg` and `--phase-done-bg`.
- Verification matrix: portrait 360x640, 360x800, 390x844, 430x932;
  landscape 568x320, 844x390, 932x430.

## 7. Do and don't

Do:

- Take every colour, size, radius and duration from a token.
- Write the phase name on every phase screen.
- Invert the phase colours for controls on phase screens.
- Keep the countdown zone at least one third of the screen height.
- One hero per screen; full-width primary actions.
- Two-tap confirm for anything destructive or session-ending.

Don't:

- No raw hex or magic px values outside the tokens stylesheet.
- No text under 13px; 13px only inside the progress strip.
- No colour-only meaning or state; no `opacity` on text.
- No animation of countdown digits; no transform animations at all.
- No second theme, no gradients, no shadows used as affordances.
- No icon-only actions without an accessible label.

## 8. Pre-merge UI checklist (each item under one minute)

1. Grep the diff for `#[0-9a-fA-F]{3,8}` and for `px`: hits allowed only in
   the tokens stylesheet or this document.
2. Every new or changed colour pair passes WebAIM: text >= 4.5:1, boundaries
   >= 3:1.
3. Phase screens show background, written phase name, matching polarity pair.
4. DevTools inspect: hero >= 96px (72px landscape); every target >= 48x48px.
5. Session screens at 360x640 portrait and 932x430 landscape: no scroll, no
   overflow, controls clear of safe areas.
6. Countdown zone >= 1/3 of the height and "88:88" fits with 16px margins.
7. Tab through the screen: focus ring visible on every control, including on
   a phase-coloured background.
8. OS reduced-motion enabled: no transform or parallax remains, transitions
   all <= 250ms.
9. Pressed and disabled states are perceivable without colour vision.
10. Destructive and session-ending flows still require two taps.
11. Numeric displays use tabular-nums: no width jitter from 9 to 10.
12. The change introduces no new screen structure outside section 6.
