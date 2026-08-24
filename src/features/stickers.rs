//! Sticker tray state via app state sync (syncd).
//!
//! Two account-scoped actions in the `regular_low` collection, both keyed by
//! the sticker's `filehash` (the base64 SHA-256 of the webp) at `index[1]`:
//!
//! - `favoriteSticker` (`WAWebStickersFavoriteSyncAction`): carries a
//!   [`wa::sync_action_value::StickerAction`] — the full CDN descriptor plus
//!   `is_favorite`. Unfavoriting is the same `Set` with
//!   `is_favorite == Some(false)`, mirroring quick replies' `deleted` flag.
//! - `removeRecentSticker` (`WAWebStickersRemoveRecentSyncAction`): hides the
//!   sticker from the recents tray for sends up to
//!   `last_sticker_sent_ts`; a later re-send restores it.
//!
//! Collection, action version and index shape come from the generated
//! `schemas::{FAVORITE_STICKER, REMOVE_RECENT_STICKER}` registry entries.

use crate::appstate_sync::Mutation;
use wacore::appstate::schemas;
use wacore::types::events::{Event, FavoriteStickerUpdate, RecentStickerRemoved};
use waproto::whatsapp as wa;

/// Dispatch inbound sticker-tray mutations synced from a linked device.
/// Returns `true` if handled, `false` if the mutation is not sticker-related.
pub(crate) fn dispatch_sticker_mutation(
    event_bus: &wacore::types::events::CoreEventBus,
    m: &Mutation,
    full_sync: bool,
) -> bool {
    if m.operation != wa::syncd_mutation::SyncdOperation::Set {
        return false;
    }

    let name = m.index.first().map(String::as_str);
    let is_favorite = name == Some(schemas::FAVORITE_STICKER.name);
    let is_remove_recent = name == Some(schemas::REMOVE_RECENT_STICKER.name);
    if !is_favorite && !is_remove_recent {
        return false;
    }

    let Some(filehash) = m.index.get(1).cloned() else {
        log::warn!("Skipping sticker mutation: missing filehash in index");
        return true;
    };

    let ts = m
        .action_value
        .as_ref()
        .and_then(|v| v.timestamp)
        .unwrap_or(0);
    let time = wacore::time::from_millis_or_now(ts);

    if is_favorite {
        if let Some(val) = &m.action_value
            && let Some(act) = val.sticker_action.as_option()
        {
            event_bus.dispatch(Event::FavoriteStickerUpdate(
                FavoriteStickerUpdate::builder()
                    .filehash(filehash)
                    .timestamp(time)
                    .action(Box::new(act.clone()))
                    .from_full_sync(full_sync)
                    .build(),
            ));
        } else {
            // Claimed but undeliverable, like quick replies: without the log
            // the mutation would vanish with no signal. The hash is opaque,
            // not user content.
            log::warn!("Skipping favoriteSticker mutation: missing stickerAction value");
        }
    } else if let Some(val) = &m.action_value
        && let Some(act) = val.remove_recent_sticker_action.as_option()
    {
        event_bus.dispatch(Event::RecentStickerRemoved(
            RecentStickerRemoved::builder()
                .filehash(filehash)
                .timestamp(time)
                .action(Box::new(act.clone()))
                .from_full_sync(full_sync)
                .build(),
        ));
    } else {
        log::warn!(
            "Skipping removeRecentSticker mutation: missing removeRecentStickerAction value"
        );
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use wacore::types::events::{CoreEventBus, EventHandler, EventInterest};

    #[derive(Default)]
    struct Recorder {
        events: Mutex<Vec<Arc<Event>>>,
    }
    impl EventHandler for Recorder {
        fn handle_event(&self, event: Arc<Event>) {
            self.events.lock().unwrap().push(event);
        }
        fn interest(&self) -> EventInterest {
            EventInterest::ALL
        }
    }

    fn run(m: &Mutation) -> (bool, Vec<Arc<Event>>) {
        let bus = CoreEventBus::new();
        let rec = Arc::new(Recorder::default());
        bus.subscribe_handler(rec.clone()).detach();
        let handled = dispatch_sticker_mutation(&bus, m, false);
        let events = rec.events.lock().unwrap().clone();
        (handled, events)
    }

    const FILEHASH: &str = "A1B2c3d4e5F6g7H8i9J0k1L2m3N4o5P6q7R8s9T0u1V2w3X4=";

    fn favorite_mutation(action: Option<wa::sync_action_value::StickerAction>) -> Mutation {
        Mutation {
            action_value: action.map(|a| wa::SyncActionValue {
                timestamp: Some(1_700_000_000_000),
                sticker_action: buffa::MessageField::some(a),
                ..Default::default()
            }),
            index: vec![
                schemas::FAVORITE_STICKER.name.to_string(),
                FILEHASH.to_string(),
            ],
            operation: wa::syncd_mutation::SyncdOperation::Set,
        }
    }

    #[test]
    fn sticker_schemas_match_wa_web() {
        assert_eq!(schemas::FAVORITE_STICKER.name, "favoriteSticker");
        assert_eq!(
            schemas::FAVORITE_STICKER.collection,
            schemas::Collection::RegularLow
        );
        assert_eq!(schemas::FAVORITE_STICKER.value_field, Some("stickerAction"));
        assert_eq!(schemas::REMOVE_RECENT_STICKER.name, "removeRecentSticker");
        assert_eq!(
            schemas::REMOVE_RECENT_STICKER.collection,
            schemas::Collection::RegularLow
        );
        assert_eq!(
            schemas::REMOVE_RECENT_STICKER.value_field,
            Some("removeRecentStickerAction")
        );
    }

    #[test]
    fn favorite_set_dispatches_update() {
        let m = favorite_mutation(Some(wa::sync_action_value::StickerAction {
            direct_path: Some("/v/sticker".into()),
            media_key: Some(vec![1u8; 32]),
            file_enc_sha256: Some(vec![2u8; 32]),
            file_length: Some(4096),
            width: Some(512),
            height: Some(512),
            is_favorite: Some(true),
            mimetype: Some("image/webp".into()),
            ..Default::default()
        }));
        let (handled, events) = run(&m);
        assert!(handled);
        assert_eq!(events.len(), 1);
        match &*events[0] {
            Event::FavoriteStickerUpdate(u) => {
                assert_eq!(u.filehash, FILEHASH);
                assert_eq!(u.action.is_favorite, Some(true));
                assert_eq!(u.action.direct_path.as_deref(), Some("/v/sticker"));
                assert!(!u.from_full_sync);
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn unfavorite_is_a_set_with_the_flag_off() {
        let m = favorite_mutation(Some(wa::sync_action_value::StickerAction {
            is_favorite: Some(false),
            ..Default::default()
        }));
        let (handled, events) = run(&m);
        assert!(handled);
        match &*events[0] {
            Event::FavoriteStickerUpdate(u) => assert_eq!(u.action.is_favorite, Some(false)),
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn remove_recent_dispatches_removal() {
        let m = Mutation {
            action_value: Some(wa::SyncActionValue {
                timestamp: Some(1_700_000_000_000),
                remove_recent_sticker_action: buffa::MessageField::some(
                    wa::sync_action_value::RemoveRecentStickerAction {
                        last_sticker_sent_ts: Some(1_699_999_999_000),
                    },
                ),
                ..Default::default()
            }),
            index: vec![
                schemas::REMOVE_RECENT_STICKER.name.to_string(),
                FILEHASH.to_string(),
            ],
            operation: wa::syncd_mutation::SyncdOperation::Set,
        };
        let (handled, events) = run(&m);
        assert!(handled);
        assert_eq!(events.len(), 1);
        match &*events[0] {
            Event::RecentStickerRemoved(u) => {
                assert_eq!(u.filehash, FILEHASH);
                assert_eq!(u.action.last_sticker_sent_ts, Some(1_699_999_999_000));
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn missing_action_value_is_claimed_without_event() {
        let (handled, events) = run(&favorite_mutation(None));
        assert!(handled);
        assert!(events.is_empty());
    }

    #[test]
    fn missing_filehash_is_claimed_without_event() {
        let mut m = favorite_mutation(None);
        m.index.truncate(1);
        let (handled, events) = run(&m);
        assert!(handled);
        assert!(events.is_empty());
    }

    #[test]
    fn foreign_mutations_are_not_claimed() {
        let mut m = favorite_mutation(None);
        m.index[0] = "archive".to_string();
        let (handled, events) = run(&m);
        assert!(!handled);
        assert!(events.is_empty());

        let mut m = favorite_mutation(None);
        m.operation = wa::syncd_mutation::SyncdOperation::Remove;
        let (handled, _) = run(&m);
        assert!(!handled);
    }
}
