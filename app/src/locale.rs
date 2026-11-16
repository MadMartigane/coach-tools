//! The interface language: supported locales, the first choice, its storage, and the copy that needs logic (phase labels, timer summary).

use leptos::prelude::{document, window};
use timer_core::{PhaseKind, Timer, format_duration};

/// The languages the interface is written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    Fr,
    En,
}

/// Every translated string of the interface, one field per catalogue key.
/// `FRENCH` and `ENGLISH` hold the values; both must define every field.
#[cfg_attr(test, derive(serde::Serialize))]
pub struct Catalog {
    pub back_to_home: &'static str,
    pub back: &'static str,
    pub language: &'static str,
    pub no_saved_timers: &'static str,
    pub new_timer: &'static str,
    pub edit: &'static str,
    pub edit_timer: &'static str,
    pub timer_name: &'static str,
    pub name_required: &'static str,
    pub preparation_seconds: &'static str,
    pub work_seconds: &'static str,
    pub rest_seconds: &'static str,
    pub cycles_per_set: &'static str,
    pub sets: &'static str,
    pub rest_between_sets_seconds: &'static str,
    pub cool_down_seconds: &'static str,
    pub decrease: &'static str,
    pub increase: &'static str,
    pub save: &'static str,
    pub start: &'static str,
    pub delete: &'static str,
    pub confirm_delete: &'static str,
    pub ready_title: &'static str,
    pub paused_title: &'static str,
    pub pause: &'static str,
    pub resume: &'static str,
    pub stop: &'static str,
    pub confirm_stop: &'static str,
    pub elapsed: &'static str,
    pub total: &'static str,
    pub remaining: &'static str,
    pub done_title: &'static str,
    pub replay: &'static str,
    pub my_timers: &'static str,
    pub phase_preparation: &'static str,
    pub phase_work: &'static str,
    pub phase_rest: &'static str,
    pub phase_rest_between_sets: &'static str,
    pub phase_cool_down: &'static str,
    pub summary_preparation: &'static str,
    pub set_singular: &'static str,
    pub set_plural: &'static str,
}

static FRENCH: Catalog = Catalog {
    back_to_home: "Retour à l'accueil",
    back: "Retour",
    language: "Langue",
    no_saved_timers: "Aucun timer enregistré.",
    new_timer: "Nouveau timer",
    edit: "Modifier",
    edit_timer: "Modifier le timer",
    timer_name: "Nom du timer",
    name_required: "Le nom est requis",
    preparation_seconds: "Préparation (s)",
    work_seconds: "Travail (s)",
    rest_seconds: "Repos (s)",
    cycles_per_set: "Cycles par set",
    sets: "Sets",
    rest_between_sets_seconds: "Repos entre sets (s)",
    cool_down_seconds: "Retour au calme (s)",
    decrease: "Diminuer",
    increase: "Augmenter",
    save: "Enregistrer",
    start: "Démarrer",
    delete: "Supprimer",
    confirm_delete: "Confirmer la suppression",
    ready_title: "PRÊT",
    paused_title: "PAUSE",
    pause: "Pause",
    resume: "Reprendre",
    stop: "Arrêter",
    confirm_stop: "Confirmer l'arrêt",
    elapsed: "Écoulé",
    total: "Total",
    remaining: "Restant",
    done_title: "Terminé",
    replay: "Rejouer",
    my_timers: "Mes timers",
    phase_preparation: "Préparation",
    phase_work: "Travail",
    phase_rest: "Repos",
    phase_rest_between_sets: "Repos entre sets",
    phase_cool_down: "Retour au calme",
    summary_preparation: "Prép",
    set_singular: "set",
    set_plural: "sets",
};

static ENGLISH: Catalog = Catalog {
    back_to_home: "Back to home",
    back: "Back",
    language: "Language",
    no_saved_timers: "No saved timers.",
    new_timer: "New timer",
    edit: "Edit",
    edit_timer: "Edit timer",
    timer_name: "Timer name",
    name_required: "Name is required",
    preparation_seconds: "Preparation (s)",
    work_seconds: "Work (s)",
    rest_seconds: "Rest (s)",
    cycles_per_set: "Cycles per set",
    sets: "Sets",
    rest_between_sets_seconds: "Rest between sets (s)",
    cool_down_seconds: "Cool-down (s)",
    decrease: "Decrease",
    increase: "Increase",
    save: "Save",
    start: "Start",
    delete: "Delete",
    confirm_delete: "Confirm delete",
    ready_title: "READY",
    paused_title: "PAUSED",
    pause: "Pause",
    resume: "Resume",
    stop: "Stop",
    confirm_stop: "Confirm stop",
    elapsed: "Elapsed",
    total: "Total",
    remaining: "Remaining",
    done_title: "Done",
    replay: "Replay",
    my_timers: "My timers",
    phase_preparation: "Preparation",
    phase_work: "Work",
    phase_rest: "Rest",
    phase_rest_between_sets: "Rest between sets",
    phase_cool_down: "Cool-down",
    summary_preparation: "Prep",
    set_singular: "set",
    set_plural: "sets",
};

/// The language used when neither the stored choice nor the browser language
/// selects one.
pub const DEFAULT_LOCALE: Locale = Locale::Fr;

/// The `localStorage` key holding the chosen locale code.
pub const LOCALE_STORAGE_KEY: &str = "coach-tools.locale";

/// The stored and `lang` code of the French locale.
pub const FRENCH_CODE: &str = "fr";
/// The stored and `lang` code of the English locale.
pub const ENGLISH_CODE: &str = "en";

/// The `<html>` attribute carrying the interface language.
const LANG_ATTRIBUTE: &str = "lang";

/// The separator between the primary and the region part of a language tag.
const LANGUAGE_TAG_SEPARATOR: char = '-';

impl Locale {
    /// The translated strings of this locale.
    pub fn catalog(self) -> &'static Catalog {
        match self {
            Locale::Fr => &FRENCH,
            Locale::En => &ENGLISH,
        }
    }
}

/// The short code of `locale`, as stored and written to `<html lang>`.
pub fn locale_code(locale: Locale) -> &'static str {
    match locale {
        Locale::Fr => FRENCH_CODE,
        Locale::En => ENGLISH_CODE,
    }
}

/// The locale named by `code`, matched exactly; `None` for any other value.
pub fn locale_from_code(code: &str) -> Option<Locale> {
    match code {
        FRENCH_CODE => Some(Locale::Fr),
        ENGLISH_CODE => Some(Locale::En),
        _ => None,
    }
}

/// English for a tag whose primary subtag is `en` (any case), French otherwise.
pub fn locale_from_language_tag(tag: &str) -> Locale {
    let primary_subtag = tag
        .split_once(LANGUAGE_TAG_SEPARATOR)
        .map_or(tag, |(primary, _region)| primary);
    if primary_subtag.eq_ignore_ascii_case(ENGLISH_CODE) {
        Locale::En
    } else {
        Locale::Fr
    }
}

/// The stored choice when it is a known code, else the browser language, else
/// [`DEFAULT_LOCALE`].
pub fn resolve_locale(stored: Option<&str>, language_tag: Option<&str>) -> Locale {
    stored
        .and_then(locale_from_code)
        .or_else(|| language_tag.map(locale_from_language_tag))
        .unwrap_or(DEFAULT_LOCALE)
}

/// The locale to start with on this device.
pub fn initial_locale() -> Locale {
    let window = window();
    let stored = window
        .local_storage()
        .ok()
        .flatten()
        .and_then(|storage| storage.get_item(LOCALE_STORAGE_KEY).ok().flatten());
    let browser_language = window.navigator().language();
    resolve_locale(stored.as_deref(), browser_language.as_deref())
}

/// Saves `locale` to `localStorage`, silently skipping the write when storage
/// is unavailable or full.
pub fn save_locale(locale: Locale) {
    if let Some(storage) = window().local_storage().ok().flatten() {
        let _ = storage.set_item(LOCALE_STORAGE_KEY, locale_code(locale));
    }
}

/// Sets `<html lang>` to `locale`, silently skipping when there is no root
/// element.
pub fn set_document_lang(locale: Locale) {
    if let Some(root) = document().document_element() {
        let _ = root.set_attribute(LANG_ATTRIBUTE, locale_code(locale));
    }
}

/// Whether `count` takes the singular form of a noun in `locale`.
pub fn is_singular(locale: Locale, count: u32) -> bool {
    match locale {
        Locale::Fr => count <= 1,
        Locale::En => count == 1,
    }
}

/// The name of a phase kind in `locale`.
pub fn phase_label(locale: Locale, kind: PhaseKind) -> &'static str {
    let catalog = locale.catalog();
    match kind {
        PhaseKind::Preparation => catalog.phase_preparation,
        PhaseKind::Work => catalog.phase_work,
        PhaseKind::Rest => catalog.phase_rest,
        PhaseKind::RestBetweenSets => catalog.phase_rest_between_sets,
        PhaseKind::CoolDown => catalog.phase_cool_down,
    }
}

/// Summarises a program in `locale`, e.g. `Prép 10 · 20/10 ×8 · 4 sets · 31:00`.
pub fn timer_summary(locale: Locale, timer: &Timer) -> String {
    let catalog = locale.catalog();
    let set_word = if is_singular(locale, timer.sets) {
        catalog.set_singular
    } else {
        catalog.set_plural
    };
    format!(
        "{} {} · {}/{} ×{} · {} {} · {}",
        catalog.summary_preparation,
        timer.preparation,
        timer.work,
        timer.rest,
        timer.cycles_per_set,
        timer.sets,
        set_word,
        format_duration(timer.total_duration()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const CATALOG_KEY_COUNT: usize = 42;

    fn locale(code: &str) -> Locale {
        locale_from_code(code).expect("supported locale code")
    }

    fn tabata() -> Timer {
        Timer {
            name: String::from("Tabata"),
            preparation: 10,
            work: 20,
            rest: 10,
            cycles_per_set: 8,
            sets: 1,
            rest_between_sets: 0,
            cool_down: 0,
        }
    }

    fn catalog_entries(catalog: &Catalog) -> serde_json::Map<String, serde_json::Value> {
        match serde_json::to_value(catalog).expect("serialize catalog") {
            serde_json::Value::Object(entries) => entries,
            other => panic!("catalog serializes to {other}, expected an object"),
        }
    }

    #[test]
    fn stored_choice_wins_over_browser_language() {
        assert_eq!(locale_code(resolve_locale(Some("en"), Some("fr-FR"))), "en");
        assert_eq!(locale_code(resolve_locale(Some("fr"), Some("en-US"))), "fr");
    }

    #[test]
    fn unknown_stored_value_is_ignored() {
        assert_eq!(locale_code(resolve_locale(Some("de"), Some("en-GB"))), "en");
        assert_eq!(locale_code(resolve_locale(Some("EN"), None)), "fr");
    }

    #[test]
    fn browser_language_picks_english_or_french() {
        for tag in ["en", "en-US", "EN-gb"] {
            assert_eq!(locale_code(locale_from_language_tag(tag)), "en", "{tag}");
        }
        for tag in ["fr-CA", "de-DE", ""] {
            assert_eq!(locale_code(locale_from_language_tag(tag)), "fr", "{tag}");
        }
    }

    #[test]
    fn default_is_french() {
        assert_eq!(locale_code(resolve_locale(None, None)), "fr");
    }

    #[test]
    fn locale_codes_round_trip() {
        assert_eq!(locale_code(locale("fr")), "fr");
        assert_eq!(locale_code(locale("en")), "en");
        assert!(locale_from_code("FR").is_none());
    }

    #[test]
    fn french_treats_zero_and_one_as_singular() {
        assert!(is_singular(locale("fr"), 0));
        assert!(is_singular(locale("fr"), 1));
        assert!(!is_singular(locale("fr"), 2));
    }

    #[test]
    fn english_treats_only_one_as_singular() {
        assert!(!is_singular(locale("en"), 0));
        assert!(is_singular(locale("en"), 1));
        assert!(!is_singular(locale("en"), 2));
    }

    #[test]
    fn summary_in_french() {
        assert_eq!(
            timer_summary(locale("fr"), &tabata()),
            "Prép 10 · 20/10 ×8 · 1 set · 04:00"
        );
        let four_sets = Timer {
            sets: 4,
            rest_between_sets: 60,
            ..tabata()
        };
        assert_eq!(
            timer_summary(locale("fr"), &four_sets),
            "Prép 10 · 20/10 ×8 · 4 sets · 18:30"
        );
    }

    #[test]
    fn summary_in_english() {
        assert_eq!(
            timer_summary(locale("en"), &tabata()),
            "Prep 10 · 20/10 ×8 · 1 set · 04:00"
        );
        let four_sets = Timer {
            sets: 4,
            rest_between_sets: 60,
            ..tabata()
        };
        assert_eq!(
            timer_summary(locale("en"), &four_sets),
            "Prep 10 · 20/10 ×8 · 4 sets · 18:30"
        );
    }

    #[test]
    fn phase_labels_in_both_languages() {
        let french = [
            (PhaseKind::Preparation, "Préparation"),
            (PhaseKind::Work, "Travail"),
            (PhaseKind::Rest, "Repos"),
            (PhaseKind::RestBetweenSets, "Repos entre sets"),
            (PhaseKind::CoolDown, "Retour au calme"),
        ];
        let english = [
            (PhaseKind::Preparation, "Preparation"),
            (PhaseKind::Work, "Work"),
            (PhaseKind::Rest, "Rest"),
            (PhaseKind::RestBetweenSets, "Rest between sets"),
            (PhaseKind::CoolDown, "Cool-down"),
        ];
        for (kind, label) in french {
            assert_eq!(phase_label(locale("fr"), kind), label);
        }
        for (kind, label) in english {
            assert_eq!(phase_label(locale("en"), kind), label);
        }
    }

    #[test]
    fn every_locale_defines_every_key() {
        let french = catalog_entries(&FRENCH);
        let english = catalog_entries(&ENGLISH);

        assert_eq!(french.len(), CATALOG_KEY_COUNT);
        assert_eq!(english.len(), CATALOG_KEY_COUNT);
        assert!(french.keys().eq(english.keys()));
        for value in french.values().chain(english.values()) {
            assert!(
                value.as_str().is_some_and(|text| !text.is_empty()),
                "every catalogue value is a non-empty string, got {value}"
            );
        }
    }
}
