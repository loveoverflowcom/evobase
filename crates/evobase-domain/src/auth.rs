use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{OrderRecord, UserId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    PayOrder,
    ShipOrder,
    CancelOrder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    pub user_id: UserId,
    pub verified: bool,
    pub capabilities: BTreeSet<Capability>,
}

impl Actor {
    pub fn has(&self, capability: &Capability) -> bool {
        self.capabilities.contains(capability)
    }

    pub fn is_buyer_of(&self, order: &OrderRecord) -> bool {
        self.user_id == order.buyer_id
    }
}
