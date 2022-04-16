use crate::data::{ChannelPersona, DatabasePool, RongPilot};
use crate::error::RongError;

use std::collections::HashMap;

use serenity::{
    client::Context,
    model::{channel::Message, id::RoleId},
};

pub async fn get_clan_from_channel_context(
    ctx: &Context,
    msg: &Message,
    persona: ChannelPersona,
) -> Result<(i32, String), RongError> {
    let cache = &ctx.cache;
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();
    let clans_info = match sqlx::query_unchecked!(
        "SELECT clan_id, name AS clan_name, platform_id
             FROM rongbot.channel_type channel
             JOIN public.rong_clan clan
               ON channel.clan_id = clan.id
             WHERE persona = $1
                   AND channel_id = $2
                   AND platform_id != 'unset'",
        persona,
        msg.channel_id.to_string()
    )
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => rows,
        Err(_) => {
            return Err(RongError::Custom(
                "There are no clans within Rong.".to_string(),
            ))
        }
    };

    if clans_info.is_empty() {
        return Err(RongError::Custom(
            "This channel does not allow cb commands.".to_string(),
        ));
    }

    let mut clan_lookup = HashMap::new();
    for clan in &clans_info {
        // println!(
        //     "Added {:?}: {:?} into clan lookup hashmap",
        //     RoleId(clan.platform_id.parse::<u64>()?),
        //     clan
        // );
        clan_lookup.insert(RoleId(clan.platform_id.parse::<u64>()?), clan);
    }
    // println!("Clan lookup found {} clans", clan_lookup.len());

    let guild_id = msg
        .guild_id
        .ok_or_else(|| "Failed to get GuildID from Message.".to_string())?;
    let member = {
        match cache.member(guild_id, msg.author.id).await {
            Some(member) => member,
            None => return Err(RongError::Custom("Error finding member data".to_string())),
        }
    };

    let mut clan_info = &clans_info[0];
    let mut has_clan = false;
    for role in &member.roles {
        // println!("Checking {:?}", role);
        if clan_lookup.contains_key(role) {
            clan_info = clan_lookup[role];
            has_clan = true;
            break;
        }
    }
    if !has_clan {
        return Err(RongError::Custom(format!(
            "You do not have the correct role for {}.",
            clan_info.clan_name
        )));
    }

    Ok((clan_info.clan_id, clan_info.clan_name.to_owned()))
}

pub async fn get_user_id(ctx: &Context, msg: &Message) -> Result<i32, RongError> {
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();
    match sqlx::query!(
        "SELECT id FROM rong_user WHERE platform_id = $1;",
        msg.author.id.to_string()
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

macro_rules! result_or_say_why {
    ($expression:expr, $ctx:ident, $msg:ident) => {
        match $expression.await {
            Ok(info) => info,
            Err(why) => {
                $msg.channel_id.say($ctx, why).await?;
                return Ok(());
            }
        }
    }
}
pub(crate) use result_or_say_why;
