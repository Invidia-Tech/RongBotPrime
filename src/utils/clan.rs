use crate::{
    data::{CbInfo, CbStatus, ChannelPersona, DatabasePool},
    error::RongError,
};

use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

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
        return Err(RongError::Custom(format!(
            "This channel does not allow {} commands.",
            persona
        )));
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
        match cache.member(guild_id, msg.author.id) {
            Some(member) => member,
            None => msg.member(ctx).await?,
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

pub async fn is_channel_persona(
    ctx: &Context,
    msg: &Message,
    persona: ChannelPersona,
) -> Result<(), RongError> {
    let _cache = &ctx.cache;
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
        return Err(RongError::Custom(format!(
            "This channel does not allow {} commands.",
            persona
        )));
    }

    Ok(())
}

pub async fn get_latest_cb(
    ctx: &Context,
    clan_id: &i32,
    clan_name: &String,
) -> Result<(CbInfo, CbStatus), RongError> {
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();

    let closest_cb =
        match sqlx::query!(
            "SELECT cb.id, cb.name AS cb_name
            FROM rong_clanbattle AS cb
            JOIN rong_clan AS clan
            ON cb.clan_id = clan.id
            WHERE start_time = (SELECT start_time
                                FROM public.rong_clanbattle AS cb
                                JOIN rong_clan AS clan
                                ON cb.clan_id = clan.id
                                WHERE clan.id = $1
                                ORDER BY abs(EXTRACT(EPOCH FROM start_time) - EXTRACT(EPOCH FROM now()))
                                LIMIT 1)
                  AND clan.id = $1
            LIMIT 1;",
            clan_id
        )
        .fetch_one(&pool)
        .await {
            Ok(row) => row,
            Err(_) =>
                return Err(RongError::Custom(format!("There are no available clan battles for {}", clan_name)))
        };

    let cb_info = match sqlx::query_as!(
        CbInfo,
        "SELECT id, name, clan_id, start_time, end_time,
                current_boss, current_hp, current_lap
         FROM public.rong_clanbattle
         WHERE id = $1;",
        closest_cb.id
    )
    .fetch_one(&pool)
    .await
    {
        Ok(info) => info,
        Err(_) => {
            return Err(RongError::Custom(format!(
                "There are no clan battle info for {:}",
                closest_cb.cb_name
            )))
        }
    };

    let cb_start_epoch = cb_info.start_time.unwrap().timestamp();
    let cb_end_epoch = cb_info.end_time.unwrap().timestamp();

    // msg.channel_id.say(ctx, format!("CB info: start: <t:{:}:f>, end: <t:{:}:f>, {:}, {:}, {:}",
    //                                 cb_start_epoch, cb_end_epoch,
    //                                 cb_info.current_boss.unwrap(), cb_info.current_hp.unwrap(), cb_info.current_lap.unwrap())).await?;

    let epoch_now = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(t) => t.as_secs() as i64,
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };
    let cb_status: CbStatus;
    if epoch_now <= cb_end_epoch {
        if epoch_now >= cb_start_epoch {
            cb_status = CbStatus::Active;
        } else {
            cb_status = CbStatus::Future;
        }
    } else {
        cb_status = CbStatus::Past;
    }
    Ok((cb_info, cb_status))
}

pub async fn get_clan_member_id_by_ign(
    ctx: &Context,
    clan_id: &i32,
    ign: &String,
) -> Result<(i32, i32), RongError> {
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();
    match sqlx::query!(
        "SELECT id, user_id FROM rong_clanmember
         WHERE  clan_id = $1
            AND ign ilike $2
            AND active = True",
        clan_id,
        ign
    )
    .fetch_one(&pool)
    .await
    {
        Ok(row) => Ok((row.id, row.user_id.unwrap_or(0))),
        Err(_) => Err(RongError::Custom(format!(
            "Who is {}? <:ReiThink:924146816151351366>, or maybe {} is not an active member!",
            ign, ign
        ))),
    }
}

pub async fn get_all_clanmember_ign_map(
    ctx: &Context,
    clan_id: &i32,
) -> Result<HashMap<i32, String>, RongError> {
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();
    match sqlx::query!(
        "SELECT id, ign
         FROM rong_clanmember
         WHERE clan_id = $1
            AND active = TRUE;",
        clan_id
    )
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => {
            let mut names_hashmap: HashMap<i32, String> = HashMap::default();
            for row in rows {
                names_hashmap.insert(row.id, row.ign);
            }
            Ok(names_hashmap)
        }
        Err(e) => Err(RongError::Database(e)),
    }
}

pub async fn get_clanmember_mention_from_id(
    ctx: &Context,
    clanmember_id: &i32,
) -> Result<Option<u64>, RongError> {
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();
    match sqlx::query!(
        "SELECT platform_id
         FROM rong_clanmember cm
         JOIN rong_user u ON user_id = u.id
         WHERE cm.id = $1;",
        clanmember_id
    )
    .fetch_one(&pool)
    .await
    {
        Ok(row) => Ok(Some(row.platform_id.parse::<u64>()?)),
        Err(_) => Ok(None),
    }
}
