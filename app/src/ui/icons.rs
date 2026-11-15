//! Inline SVG icons: the coach-tools app mark, the Tabata logo mark and the Lucide icon set (ISC, see `THIRD_PARTY_LICENSES.md`).
//!
//! Colors and sizes come from CSS classes and `currentColor`, never inline
//! dimensions or color codes.

use leptos::prelude::*;

/// Shared Lucide wrapper: 24x24 viewBox, stroke-based, sized and colored via CSS.
#[component]
fn LucideIcon(children: Children) -> impl IntoView {
    view! {
        <svg
            class="icon"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            {children()}
        </svg>
    }
}

/// Three vertical rounded bars: tall outer bars around a short middle bar.
#[component]
pub fn LogoMark() -> impl IntoView {
    view! {
        <svg class="logo-mark" viewBox="0 0 24 24" aria-hidden="true">
            <rect x="3" y="3" width="4" height="18" rx="2" class="logo-bar-outer" />
            <rect x="10" y="7" width="4" height="10" rx="2" class="logo-bar-mid" />
            <rect x="17" y="3" width="4" height="18" rx="2" class="logo-bar-outer" />
        </svg>
    }
}

/// The coach-tools mark, a whistle seen from the side. Its hole takes the page
/// background colour, so it is shown on `--color-bg` only (the home header).
#[component]
pub fn AppMark() -> impl IntoView {
    view! {
        <svg class="logo-mark" viewBox="10 12 160 160" aria-hidden="true">
            <circle cx="72" cy="92" r="50" class="app-mark-figure" />
            <path d="M72 42H148A10 10 0 0 1 158 52V72A10 10 0 0 1 148 82H72Z" class="app-mark-figure" />
            <circle cx="72" cy="92" r="16" class="app-mark-ground" />
        </svg>
    }
}

/// Lucide `plus`.
#[component]
pub fn IconPlus() -> impl IntoView {
    view! {
        <LucideIcon>
            <path d="M5 12h14" />
            <path d="M12 5v14" />
        </LucideIcon>
    }
}

/// Lucide `minus`.
#[component]
pub fn IconMinus() -> impl IntoView {
    view! {
        <LucideIcon>
            <path d="M5 12h14" />
        </LucideIcon>
    }
}

/// Lucide `play`.
#[component]
pub fn IconPlay() -> impl IntoView {
    view! {
        <LucideIcon>
            <polygon points="6 3 20 12 6 21 6 3" />
        </LucideIcon>
    }
}

/// Lucide `pause`.
#[component]
pub fn IconPause() -> impl IntoView {
    view! {
        <LucideIcon>
            <rect x="14" y="4" width="4" height="16" rx="1" />
            <rect x="6" y="4" width="4" height="16" rx="1" />
        </LucideIcon>
    }
}

/// Lucide `square`.
#[component]
pub fn IconSquare() -> impl IntoView {
    view! {
        <LucideIcon>
            <rect width="18" height="18" x="3" y="3" rx="2" />
        </LucideIcon>
    }
}

/// Lucide `rotate-ccw`.
#[component]
pub fn IconRotateCcw() -> impl IntoView {
    view! {
        <LucideIcon>
            <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
            <path d="M3 3v5h5" />
        </LucideIcon>
    }
}

/// Lucide `list`.
#[component]
pub fn IconList() -> impl IntoView {
    view! {
        <LucideIcon>
            <path d="M3 5h.01" />
            <path d="M3 12h.01" />
            <path d="M3 19h.01" />
            <path d="M8 5h13" />
            <path d="M8 12h13" />
            <path d="M8 19h13" />
        </LucideIcon>
    }
}

/// Lucide `pencil`.
#[component]
pub fn IconPencil() -> impl IntoView {
    view! {
        <LucideIcon>
            <path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z" />
            <path d="m15 5 4 4" />
        </LucideIcon>
    }
}

/// Lucide `trash-2`.
#[component]
pub fn IconTrash() -> impl IntoView {
    view! {
        <LucideIcon>
            <path d="M3 6h18" />
            <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
            <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
            <line x1="10" x2="10" y1="11" y2="17" />
            <line x1="14" x2="14" y1="11" y2="17" />
        </LucideIcon>
    }
}

/// Lucide `chevron-left`.
#[component]
pub fn IconChevronLeft() -> impl IntoView {
    view! {
        <LucideIcon>
            <path d="m15 18-6-6 6-6" />
        </LucideIcon>
    }
}

/// Lucide `check`.
#[component]
pub fn IconCheck() -> impl IntoView {
    view! {
        <LucideIcon>
            <path d="M20 6 9 17l-5-5" />
        </LucideIcon>
    }
}
