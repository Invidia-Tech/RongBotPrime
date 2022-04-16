use crate::data::{ChannelPersona, DatabasePool, RongPilot};
use crate::error::RongError;

use std::collections::HashMap;

use serenity::{
    client::Context,
    model::{channel::Message, id::RoleId},
};

pub async fn update_pilot_info(ctx: &Context, pilot_info: &RongPilot) -> Result<(), RongError> {
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();
    match sqlx::query!(
        "UPDATE rongbot.pilot SET (nickname, motto, code) = ($1, $2, $3)
         WHERE pilot_id = $4 RETURNING pilot_id;",
        pilot_info.nickname,
        pilot_info.motto,
        pilot_info.code,
        pilot_info.pilot_id
    )
    .fetch_one(&pool)
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(RongError::Database(e)),
    }
}

pub async fn get_pilot_info_or_create_new(
    ctx: &Context,
    user_id: &i32,
    clan_id: &i32,
) -> Result<RongPilot, RongError> {
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();
    match sqlx::query_as!(
        RongPilot,
        "SELECT * FROM rongbot.pilot WHERE user_id = $1 AND clan_id = $2;",
        user_id,
        clan_id
    )
    .fetch_one(&pool)
    .await
    {
        Ok(row) => Ok(row),
        Err(_) => {
            match sqlx::query_as!(
                RongPilot,
                "INSERT INTO rongbot.pilot (user_id, clan_id)
                 VALUES ($1, $2) RETURNING *",
                user_id,
                clan_id
            )
            .fetch_one(&pool)
            .await
            {
                Ok(row) => Ok(row),
                Err(e) => Err(RongError::Database(e)),
            }
        }
    }
}
