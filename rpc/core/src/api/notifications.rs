use crate::model::message::*;
use async_channel::{Receiver, Sender};
use borsh::{BorshDeserialize, BorshSerialize};
use kaspa_notify::{
    events::EventType,
    notification::{full_featured, Notification as NotificationTrait},
    scope::{Scope, UtxosChangedScope, VirtualSelectedParentChainChangedScope},
    subscription::{
        single::{OverallSubscription, UtxosChangedSubscription, VirtualSelectedParentChainChangedSubscription},
        Single,
    },
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::sync::Arc;

full_featured! {
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum Notification {
    BlockAdded(BlockAddedNotification),
    VirtualSelectedParentChainChanged(VirtualSelectedParentChainChangedNotification),
    FinalityConflict(FinalityConflictNotification),
    FinalityConflictResolved(FinalityConflictResolvedNotification),
    UtxosChanged(UtxosChangedNotification),
    VirtualSelectedParentBlueScoreChanged(VirtualSelectedParentBlueScoreChangedNotification),
    VirtualDaaScoreChanged(VirtualDaaScoreChangedNotification),
    PruningPointUtxoSetOverride(PruningPointUtxoSetOverrideNotification),
    NewBlockTemplate(NewBlockTemplateNotification),
}
}

impl NotificationTrait for Notification {
    fn apply_overall_subscription(&self, subscription: &OverallSubscription) -> Option<Self> {
        match subscription.active() {
            true => Some(self.clone()),
            false => None,
        }
    }

    fn apply_virtual_selected_parent_chain_changed_subscription(
        &self,
        subscription: &VirtualSelectedParentChainChangedSubscription,
    ) -> Option<Self> {
        match subscription.active() {
            true => {
                if let Notification::VirtualSelectedParentChainChanged(ref payload) = self {
                    if !subscription.include_accepted_transaction_ids() && !payload.accepted_transaction_ids.is_empty() {
                        return Some(Notification::VirtualSelectedParentChainChanged(VirtualSelectedParentChainChangedNotification {
                            removed_chain_block_hashes: payload.removed_chain_block_hashes.clone(),
                            added_chain_block_hashes: payload.added_chain_block_hashes.clone(),
                            accepted_transaction_ids: Arc::new(vec![]),
                        }));
                    }
                }
                Some(self.clone())
            }
            false => None,
        }
    }

    fn apply_utxos_changed_subscription(&self, _subscription: &UtxosChangedSubscription) -> Option<Self> {
        todo!()
    }

    fn event_type(&self) -> EventType {
        self.into()
    }
}

pub type NotificationSender = Sender<Notification>;
pub type NotificationReceiver = Receiver<Notification>;

pub enum NotificationHandle {
    Existing(u64),
    New(NotificationSender),
}
