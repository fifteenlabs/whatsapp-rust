//! The `<blocklist>` child of an `account_sync` notification.
//!
//! The server tells every device of an account when its blocklist changes,
//! as a child of `<notification type="account_sync">` rather than a
//! notification type of its own. whatsmeow: `handleAccountSyncNotification`
//! → `handleBlocklist`.
//!
//! ```xml
//! <notification type="account_sync" from="user@s.whatsapp.net">
//!   <blocklist dhash="…" prev_dhash="…">
//!     <item jid="…@lid" action="block"/>
//!     <item jid="…@lid" action="unblock"/>
//!   </blocklist>
//! </notification>
//! ```
//!
//! With `action="modify"` the stanza names no entries at all and the list has
//! to be fetched again (`GetBlocklistSpec`).

use crate::WireEnum;
pub use crate::iq::blocklist::BlocklistAction;
use anyhow::{Result, anyhow};
use log::warn;
use serde::Serialize;
use wacore_binary::{Jid, NodeRef};

/// What the stanza says about the list as a whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq, WireEnum)]
pub enum BlocklistListAction {
    /// The list changed in a way the stanza does not spell out; re-fetch it.
    #[wire = "modify"]
    Modify,
}

/// One contact entering or leaving the blocklist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BlocklistChange {
    /// The blocked or unblocked contact, keyed the way the server keys the
    /// list (a LID on current accounts).
    pub jid: Jid,
    pub action: BlocklistAction,
}

/// Parsed `<blocklist>` child of an `account_sync` notification.
#[derive(Debug, Clone, Serialize)]
pub struct BlocklistNotification {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<BlocklistListAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dhash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_dhash: Option<String>,
    pub changes: Vec<BlocklistChange>,
}

impl BlocklistNotification {
    /// Parse the `<blocklist>` node. An `<item>` missing its jid or carrying
    /// an action this client does not know is logged and skipped, as whatsmeow
    /// does, so one odd entry does not hide the rest of the delta.
    pub fn try_parse(node: &NodeRef<'_>) -> Result<Self> {
        if node.tag != "blocklist" {
            return Err(anyhow!("expected <blocklist>, got <{}>", node.tag));
        }
        let mut attrs = node.attrs();
        let action = match attrs.optional_string("action") {
            Some(raw) => Some(
                BlocklistListAction::try_from(raw.as_ref())
                    .map_err(|_| anyhow!("unknown blocklist action '{raw}'"))?,
            ),
            None => None,
        };
        let dhash = attrs.optional_string("dhash").map(|s| s.into_owned());
        let prev_dhash = attrs.optional_string("prev_dhash").map(|s| s.into_owned());

        let mut changes = Vec::new();
        for item in node.children().unwrap_or_default() {
            if item.tag != "item" {
                continue;
            }
            let mut attrs = item.attrs();
            let Some(jid) = attrs.optional_jid("jid") else {
                warn!(target: "Blocking", "blocklist <item> without a jid; skipped");
                continue;
            };
            let Some(action) = attrs
                .optional_string("action")
                .and_then(|raw| BlocklistAction::try_from(raw.as_ref()).ok())
            else {
                warn!(target: "Blocking", "blocklist <item> without a known action; skipped");
                continue;
            };
            changes.push(BlocklistChange { jid, action });
        }

        Ok(Self {
            action,
            dhash,
            prev_dhash,
            changes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wacore_binary::builder::NodeBuilder;

    fn item(jid: &str, action: &str) -> wacore_binary::Node {
        NodeBuilder::new("item")
            .attr("jid", jid)
            .attr("action", action)
            .build()
    }

    #[test]
    fn a_delta_lists_each_change_in_order() {
        let node = NodeBuilder::new("blocklist")
            .attr("dhash", "2:abc")
            .attr("prev_dhash", "2:xyz")
            .children([
                item("123456789012345@lid", "block"),
                item("543210987654321@lid", "unblock"),
            ])
            .build();

        let parsed = BlocklistNotification::try_parse(&node.as_node_ref()).unwrap();

        assert_eq!(parsed.action, None);
        assert_eq!(parsed.dhash.as_deref(), Some("2:abc"));
        assert_eq!(parsed.prev_dhash.as_deref(), Some("2:xyz"));
        assert_eq!(
            parsed.changes,
            vec![
                BlocklistChange {
                    jid: "123456789012345@lid".parse().unwrap(),
                    action: BlocklistAction::Block,
                },
                BlocklistChange {
                    jid: "543210987654321@lid".parse().unwrap(),
                    action: BlocklistAction::Unblock,
                },
            ]
        );
    }

    #[test]
    fn a_modify_names_no_entries() {
        let node = NodeBuilder::new("blocklist")
            .attr("action", "modify")
            .attr("dhash", "2:abc")
            .build();

        let parsed = BlocklistNotification::try_parse(&node.as_node_ref()).unwrap();

        assert_eq!(parsed.action, Some(BlocklistListAction::Modify));
        assert!(parsed.changes.is_empty());
    }

    #[test]
    fn a_malformed_item_is_skipped_not_fatal() {
        let node = NodeBuilder::new("blocklist")
            .children([
                NodeBuilder::new("item").attr("action", "block").build(),
                item("123456789012345@lid", "mute"),
                item("543210987654321@lid", "block"),
            ])
            .build();

        let parsed = BlocklistNotification::try_parse(&node.as_node_ref()).unwrap();

        assert_eq!(parsed.changes.len(), 1);
        assert_eq!(parsed.changes[0].action, BlocklistAction::Block);
    }

    #[test]
    fn the_wrong_tag_is_an_error() {
        let node = NodeBuilder::new("devices").build();
        assert!(BlocklistNotification::try_parse(&node.as_node_ref()).is_err());
    }
}
