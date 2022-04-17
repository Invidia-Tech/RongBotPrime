use chrono::{DateTime, Utc};
use serenity::client::bridge::gateway::ShardManager;
use serenity::prelude::TypeMapKey;
use sqlx::PgPool;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

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

#[derive(Debug)]
pub enum CbStatus {
    Future,
    Active,
    Past,
}

#[derive(Debug, sqlx::Type, strum_macros::EnumString)]
#[sqlx(type_name = "channel_persona", rename_all = "lowercase")]
pub enum ChannelPersona {
    #[strum(ascii_case_insensitive)]
    Cb,
    #[strum(ascii_case_insensitive)]
    Pvp,
    #[strum(ascii_case_insensitive)]
    Clan,
    #[strum(ascii_case_insensitive)]
    Public,
}

#[derive(Debug, sqlx::Type, strum_macros::EnumString)]
#[sqlx(type_name = "flight_status", rename_all = "lowercase")]
pub enum FlightStatus {
    #[strum(ascii_case_insensitive)]
    Canceled,
    #[strum(ascii_case_insensitive, serialize = "in flight")]
    #[sqlx(rename = "in flight")]
    InFlight,
    #[strum(ascii_case_insensitive)]
    Landed,
    #[strum(ascii_case_insensitive)]
    Crashed,
    #[strum(ascii_case_insensitive)]
    Amb,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ChannelType {
    pub channel_id: String,
    pub clan_id: i32,
    pub persona: ChannelPersona,
}

#[derive(Debug, sqlx::FromRow)]
pub struct RongPilot {
    pub id: i32,
    pub nickname: Option<String>,
    pub motto: Option<String>,
    pub code: Option<String>,
    pub clan_id: i32,
    pub user_id: i32,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CbInfo {
    pub id: i32,
    pub name: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub clan_id: i32,
    pub current_boss: Option<i32>,
    pub current_hp: Option<i32>,
    pub current_lap: Option<i32>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Flight {
    pub id: i32,
    pub call_sign: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub clan_id: i32,
    pub cb_id: i32,
    pub pilot_id: i32,
    pub passenger_id: Option<i32>,
    pub status: FlightStatus,
    pub team_id: Option<i32>,
}
