//! The editor for a new or saved timer: seven programmable parameters, a
//! live total and a two-tap delete.

use leptos::prelude::*;
use timer_core::{InvalidTimer, Timer, format_duration};

use crate::state::AppContext;
use crate::ui::arm_confirmation;
use crate::ui::icons::{IconCheck, IconChevronLeft, IconMinus, IconPlay, IconPlus, IconTrash};

/// Smallest value of a duration parameter, in seconds.
const MIN_DURATION_SECONDS: u32 = 0;

/// Smallest value of a count parameter (cycles or sets).
const MIN_COUNT: u32 = 1;

/// Edits one timer. `index` is the saved timer being edited, `None` for a new one;
/// `initial` is the program shown when the editor opens.
#[component]
pub fn EditorScreen(index: Option<usize>, initial: Timer) -> impl IntoView {
    let ctx = expect_context::<AppContext>();

    let name = RwSignal::new(initial.name);
    let preparation = RwSignal::new(initial.preparation);
    let work = RwSignal::new(initial.work);
    let rest = RwSignal::new(initial.rest);
    let cycles_per_set = RwSignal::new(initial.cycles_per_set);
    let sets = RwSignal::new(initial.sets);
    let rest_between_sets = RwSignal::new(initial.rest_between_sets);
    let cool_down = RwSignal::new(initial.cool_down);

    let draft = Memo::new(move |_| Timer {
        name: name.get().trim().to_string(),
        preparation: preparation.get(),
        work: work.get(),
        rest: rest.get(),
        cycles_per_set: cycles_per_set.get(),
        sets: sets.get(),
        rest_between_sets: rest_between_sets.get(),
        cool_down: cool_down.get(),
    });
    let validation = Memo::new(move |_| draft.get().validate());
    let is_invalid = move || validation.get().is_err();
    let can_start =
        move || draft.with(|timer| timer.validate().is_ok() && timer.total_duration() > 0);
    let total_label = move || format_duration(draft.get().total_duration());

    let confirm_delete = RwSignal::new(false);
    let pending_reset = StoredValue::new(None::<TimeoutHandle>);
    on_cleanup(move || {
        pending_reset.update_value(|pending| {
            if let Some(handle) = pending.take() {
                handle.clear();
            }
        });
    });

    let back_ctx = ctx.clone();
    let save_ctx = ctx.clone();
    let start_ctx = ctx.clone();
    let delete_ctx = ctx.clone();

    let save = move |_| {
        let _ = save_ctx.save_editor(index, draft.get_untracked());
    };
    let start = move |_| {
        let timer = draft.get_untracked();
        if timer.total_duration() > 0 && start_ctx.save_editor(index, timer.clone()).is_ok() {
            start_ctx.start_session(timer);
        }
    };
    let delete = move |_| {
        if let Some(position) = index {
            if confirm_delete.get_untracked() {
                delete_ctx.delete_timer(position);
            } else {
                arm_confirmation(confirm_delete, pending_reset);
            }
        }
    };

    view! {
        <main class="screen-neutral">
            <header class="screen-header">
                <button
                    type="button"
                    class="btn-icon"
                    aria-label="Retour"
                    on:click=move |_| back_ctx.exit_to_list()
                >
                    <IconChevronLeft />
                </button>
                <h1 class="brand-title">
                    {move || if index.is_none() { "Nouveau timer" } else { "Modifier le timer" }}
                </h1>
                <span class="session-chip">{total_label}</span>
            </header>
            <div class="editor-form">
                <div class="field-card">
                    <input
                        class="text-input"
                        type="text"
                        placeholder="Nom du timer"
                        prop:value=move || name.get()
                        on:input=move |event| name.set(event_target_value(&event))
                    />
                    <Show when=move || validation.get() == Err(InvalidTimer::EmptyName)>
                        <p class="validation-msg">"Le nom est requis"</p>
                    </Show>
                </div>
                <div class="field-card">
                    <DurationStepper label="Préparation (s)" value=preparation min=MIN_DURATION_SECONDS />
                    <DurationStepper label="Travail (s)" value=work min=MIN_DURATION_SECONDS />
                    <DurationStepper label="Repos (s)" value=rest min=MIN_DURATION_SECONDS />
                    <DurationStepper label="Cycles par set" value=cycles_per_set min=MIN_COUNT />
                    <DurationStepper label="Sets" value=sets min=MIN_COUNT />
                    <DurationStepper label="Repos entre sets (s)" value=rest_between_sets min=MIN_DURATION_SECONDS />
                    <DurationStepper label="Retour au calme (s)" value=cool_down min=MIN_DURATION_SECONDS />
                </div>
                <div class="editor-actions">
                    <button
                        type="button"
                        class="btn btn-standard"
                        disabled=is_invalid
                        aria-disabled=move || is_invalid().to_string()
                        on:click=save
                    >
                        <IconCheck />
                        <span>"Enregistrer"</span>
                    </button>
                    <button
                        type="button"
                        class="btn btn-hero"
                        disabled=move || !can_start()
                        aria-disabled=move || (!can_start()).to_string()
                        on:click=start
                    >
                        <IconPlay />
                        <span>"Démarrer"</span>
                    </button>
                    {index.map(|_| {
                        view! {
                            <button
                                type="button"
                                class=move || if confirm_delete.get() { "btn btn-danger" } else { "btn btn-ghost" }
                                on:click=delete
                            >
                                <IconTrash />
                                <span>
                                    {move || if confirm_delete.get() { "Confirmer la suppression" } else { "Supprimer" }}
                                </span>
                            </button>
                        }
                    })}
                </div>
            </div>
        </main>
    }
}

/// One labelled duration or count, edited with a minus button, a typed value
/// and a plus button. The value never drops below `min`.
#[component]
fn DurationStepper(label: &'static str, value: RwSignal<u32>, min: u32) -> impl IntoView {
    view! {
        <div class="field-row">
            <span class="field-label">{label}</span>
            <div class="stepper">
                <button
                    type="button"
                    class="stepper-btn"
                    aria-label="Diminuer"
                    on:click=move |_| value.update(|current| *current = current.saturating_sub(1).max(min))
                >
                    <IconMinus />
                </button>
                <input
                    type="number"
                    inputmode="numeric"
                    class="stepper-value text-input"
                    aria-label=label
                    min=min
                    prop:value=move || value.get().to_string()
                    on:input=move |event| {
                        if let Ok(parsed) = event_target_value(&event).parse::<u32>() {
                            value.set(parsed);
                        }
                    }
                />
                <button
                    type="button"
                    class="stepper-btn"
                    aria-label="Augmenter"
                    on:click=move |_| value.update(|current| *current = current.saturating_add(1))
                >
                    <IconPlus />
                </button>
            </div>
        </div>
    }
}
