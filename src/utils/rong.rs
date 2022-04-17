use crate::data::{ChannelPersona, DatabasePool, RongPilot};
use crate::error::RongError;

use std::collections::HashMap;

use serenity::{
    client::Context,
    model::{channel::Message, id::RoleId},
};

pub async fn get_user_id(
    ctx: &Context,
    msg: &Message,
    platform_id: &String,
) -> Result<i32, RongError> {
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();
    match sqlx::query!(
        "SELECT id FROM rong_user WHERE platform_id = $1;",
        platform_id
    )
    .fetch_one(&pool)
    .await
    {
        Ok(row) => Ok(row.id),
        Err(_) => Err(RongError::Custom(format!(
            "Who are you? I've never seen {} before...",
            msg.author_nick(&ctx.http)
                .await
                .unwrap_or_else(|| String::from(&*msg.author.name))
        ))),
    }
}
