//! Notices: their keys and lists, argument names, the two answer keys, the events, and what each one carries.

use super::*;
use keys::ALL_KEYS;
use std::collections::HashSet;

/// A notice that asks nothing.
fn plain() -> Notice {
    Notice::plain(keys::KEYS_NOT_SENT)
}

#[test]
fn a_plain_notice_names_nothing_and_one_about_a_module_names_it() {
    let plain = plain();
    assert_eq!((plain.module, plain.args.len(), plain.ask), (None, 0, None));
    let id = ModuleId::new("mouse");
    let about = Notice::about(keys::MOUSE_STOPPED, id);
    assert_eq!(
        (about.key, about.module, about.ask),
        (keys::MOUSE_STOPPED, Some(id), None)
    );
    assert_eq!(
        about.args,
        [(args::MODULE, NoticeArg::Text("mouse".into()))]
    );
}

#[test]
fn keys_are_unique_notice_keys() {
    let unique: HashSet<_> = ALL_KEYS.iter().collect();
    assert_eq!(unique.len(), ALL_KEYS.len());
    assert!(ALL_KEYS.iter().all(|k| k.starts_with("notice.")));
}

#[test]
fn the_power_notices_are_listed() {
    assert_eq!(keys::POWER_FAILED, "notice.power-failed");
    assert_eq!(keys::POWER_STOPPED, "notice.power-stopped");
    assert!(ALL_KEYS.contains(&keys::POWER_FAILED));
    assert!(ALL_KEYS.contains(&keys::POWER_STOPPED));
}

#[test]
fn arguments_keep_the_names_translations_use() {
    assert_eq!(
        (args::MODULE, args::KEYS, args::MORE, args::LANG, args::APP),
        ("module", "keys", "more", "lang", "app")
    );
    assert_eq!(args::SITE, "site");
}

#[test]
fn the_translate_notices_keep_their_keys_and_are_listed() {
    assert_eq!(keys::TRANSLATE_ASK, "notice.translate-ask");
    assert_eq!(keys::TRANSLATE_FAILED, "notice.translate-failed");
    let translate = [keys::TRANSLATE_ASK, keys::TRANSLATE_FAILED];
    assert!(translate.iter().all(|k| ALL_KEYS.contains(k)));
}

#[test]
fn the_words_notices_keep_their_keys_and_are_listed() {
    assert_eq!(keys::WORDS_DAMAGED, "notice.words-damaged");
    assert_eq!(keys::WORDS_MISSING, "notice.words-missing");
    assert_eq!(keys::WORDS_LOCKED, "notice.words-locked");
    let words = [keys::WORDS_DAMAGED, keys::WORDS_MISSING, keys::WORDS_LOCKED];
    assert!(words.iter().all(|k| ALL_KEYS.contains(k)));
}

#[test]
fn the_clipboard_notices_keep_their_keys_and_are_listed() {
    assert_eq!(keys::CLIP_DAMAGED, "notice.clip-damaged");
    assert_eq!(keys::CLIP_LOCKED, "notice.clip-locked");
    let clip = [keys::CLIP_DAMAGED, keys::CLIP_LOCKED];
    assert!(clip.iter().all(|k| ALL_KEYS.contains(k)));
}

#[test]
fn an_asking_notice_keeps_its_two_answer_keys() {
    let ask = &NoticeAsk::YES_NO;
    assert_eq!((ask.yes, ask.no), ("button.yes", "button.no"));
    let asking = Notice {
        ask: Some(ask),
        ..plain()
    };
    assert_eq!(asking.ask, Some(ask));
    assert_ne!(asking, plain(), "the ask tells two notices apart");
}

#[test]
fn the_vault_notices_keep_their_keys_and_are_listed() {
    assert_eq!(keys::VAULT_DAMAGED, "notice.vault-damaged");
    assert_eq!(keys::VAULT_CHECK, "notice.vault-check");
    assert_eq!(keys::VAULT_OFFER, "notice.vault-offer");
    assert_eq!(keys::VAULT_BACKUP_DAMAGED, "notice.vault-backup-damaged");
    let vault = [
        keys::VAULT_DAMAGED,
        keys::VAULT_CHECK,
        keys::VAULT_OFFER,
        keys::VAULT_BACKUP_DAMAGED,
    ];
    assert!(vault.iter().all(|k| ALL_KEYS.contains(k)));
}

#[test]
fn a_memory_check_asks_check_or_later_and_the_offer_yes_or_later() {
    let check = NoticeAsk::CHECK_LATER;
    assert_eq!((check.yes, check.no), ("button.check", "button.later"));
    let offer = NoticeAsk::YES_LATER;
    assert_eq!((offer.yes, offer.no), ("button.yes", "button.later"));
}

#[test]
fn an_answer_needs_no_capability() {
    assert_eq!(NoticeAnswered::CAPABILITY, None);
    assert_eq!(NoticeAnswered::NAME, "notice-answered");
}

#[test]
fn event_names_are_distinct() {
    let names = [
        Notice::NAME,
        NoticeAnswered::NAME,
        ModuleStateChanged::NAME,
        SettingsChanged::NAME,
    ];
    let unique: HashSet<_> = names.iter().collect();
    assert_eq!(unique.len(), names.len());
}

#[test]
fn a_state_change_carries_module_and_state() {
    let module = ModuleId::new("keyboard");
    let change = ModuleStateChanged {
        module,
        state: ModuleState::Failed,
    };
    assert_eq!((change.module, change.state), (module, ModuleState::Failed));
}

#[test]
fn the_hidden_run_notices_keep_their_keys_and_are_listed() {
    assert_eq!(keys::HIDDEN_RUN, "notice.hidden-run");
    assert_eq!(keys::HIDDEN_COPY, "notice.hidden-copy");
    assert_eq!(keys::HIDDEN_COPY_FAILED, "notice.hidden-copy-failed");
    let hidden = [
        keys::HIDDEN_RUN,
        keys::HIDDEN_COPY,
        keys::HIDDEN_COPY_FAILED,
    ];
    assert!(hidden.iter().all(|k| ALL_KEYS.contains(k)));
}
