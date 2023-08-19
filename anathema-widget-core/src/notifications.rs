use std::sync::OnceLock;

use anathema_generator::NodeId;
use anathema_values::{StoreRef, Listen, Listeners, ValueRef, Container as AVValue} ;
use parking_lot::Mutex;

use crate::Value;

static LISTENERS: OnceLock<Mutex<Listeners<NodeId, Value>>> = OnceLock::new();

fn sub_to_value(node_id: NodeId, val: ValueRef<AVValue<Value>>) {
    let listeners = LISTENERS.get_or_init(|| Mutex::new(Listeners::empty()));
    listeners.lock().subscribe_to_value(node_id, val);
}

// TODO: rename this because X is a bad name
pub struct Listener;

impl Listen for Listener {
    type Key = NodeId;
    type Value = Value;

    fn subscribe(value: ValueRef<AVValue<Self::Value>>, key: Self::Key) {
        sub_to_value(key, value);
    }
}
