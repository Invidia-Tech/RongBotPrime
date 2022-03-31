use std::{
    collections::HashMap,
    sync::Arc,
};
use serenity::client::bridge::gateway::ShardManager;
use tokio::sync::Mutex;
use serenity::prelude::TypeMapKey;
use sqlx::PgPool;


pub struct DatabasePool;

impl TypeMapKey for DatabasePool {
    type Value = PgPool;
}

// This allows data to be shared across the shard, so that all frameworks
// and handlers can see the same data as long as they have a copy of the
// `data` Arc. Arc is an atomic reference counter btw. It's a thread safe
// way to share immutable data.
pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct CommandCounter;

impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
}

pub enum CbStatus {
    FUTURE,
    ACTIVE,
    PAST
}
