use crate::data::{CbStatus, DatabasePool};

use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::{channel::Message, id::RoleId},
    utils::Color,
};

#[command("status")]
#[aliases("s")]
#[bucket = "cb"]
#[description("This shows the status of the current active clan battle.")]
async fn cb_status(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let cache = &ctx.cache;
    // The message builder allows for creating a message by
    // mentioning users dynamically, pushing "safe" versions of
    // content (such as bolding normalized content), displaying
    // emojis, and more.
    // let response = MessageBuilder::new()
    //         .push("User ")
    //         .push_bold_safe(&msg.author.name)
    //         .push(" used the 'cb status' command in the ")
    //         .mention(&msg.channel_id.to_channel_cached(cache).await.unwrap())
    //         .push(" channel")
    //         .build();

    // if let Err(why) = msg.channel_id.say(&ctx.http, &response).await {
    //     println!("Error sending message: {:?}", why);
    // }

    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();
    // let clans_info =
    //     sqlx::query("SELECT id, platform_id FROM rong_clan WHERE platform_id != 'unset';")
    //         .fetch_all(&pool)
    //         .await?;

    let clans_info = match sqlx::query!(
        "SELECT clan_id, name AS clan_name, platform_id
             FROM rongbot.channel_type channel
             JOIN public.rong_clan clan
               ON channel.clan_id = clan.id
             WHERE persona = 'cb'
                   AND channel_id = $1
                   AND platform_id != 'unset';",
        msg.channel_id.to_string()
    )
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => rows,
        Err(_) => {
            msg.channel_id
                .say(ctx, "There are no clans within Rong.")
                .await?;
            return Ok(());
        }
    };

    if clans_info.is_empty() {
        msg.channel_id
            .say(ctx, "This channel does not allow cb commands.")
            .await?;
        return Ok(());
    }

    let mut clan_lookup = HashMap::new();
    for clan in &clans_info {
        println!(
            "Added {:?}: {:?} into clan lookup hashmap",
            RoleId(clan.platform_id.parse::<u64>()?),
            clan
        );
        clan_lookup.insert(RoleId(clan.platform_id.parse::<u64>()?), clan);
    }
    println!("Clan lookup found {} clans", clan_lookup.len());

    let guild_id = msg.guild_id.ok_or("Failed to get GuildID from Message.")?;
    let member = {
        match cache.member(guild_id, msg.author.id).await {
            Some(member) => member,
            None => {
                if let Err(why) = msg
                    .channel_id
                    .say(&ctx.http, "Error finding member data")
                    .await
                {
                    println!("Error sending message: {:?}", why);
                }
                return Ok(());
            }
        }
    };

    let mut clan_info = &clans_info[0];
    let mut has_clan = false;
    for role in &member.roles {
        println!("Checking {:?}", role);
        if clan_lookup.contains_key(role) {
            clan_info = clan_lookup[&role];
            has_clan = true;
        }
    }
    if !has_clan {
        msg.channel_id
            .say(
                ctx,
                format!(
                    "You do not have the correct role for {:?}.",
                    clan_info.clan_name
                ),
            )
            .await?;
        return Ok(());
    }

    // let required_role =
    //     Role
    //     Id(clan_info.platform_id.parse::<u64>()?);
    // let mut user_roles = vec![];
    // for role in &member.roles {
    //     user_roles.push(role);
    // }
    // msg.channel_id.say(ctx, format!("Looking for role: {:} for guild: {:}",
    //                                 required_role.name,
    //                                 clan_info.name)).await?;
    // msg.channel_id.say(ctx, format!("User has roles: {:?}", user_roles)).await?;
    // if !member.roles.contains(&required_role) {
    //     msg.channel_id.say(ctx, format!("You do not have the correct role for {:?}.", clan_info.clan_name)).await?;
    //     return Ok(());
    // }

    msg.channel_id
        .say(ctx, format!("Clan you're in is: {:?}", clan_info.clan_name))
        .await?;

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
            clan_info.clan_id
        )
        .fetch_one(&pool)
        .await {
            Ok(row) => row,
            Err(_) => {
                msg.channel_id.say(ctx, format!("There are no available clan battles for {:}", clan_info.clan_name)).await?;
                return Ok(());
            }
        };

    msg.channel_id
        .say(
            ctx,
            format!("Current closest CB is {:?}", closest_cb.cb_name),
        )
        .await?;

    let cb_info = match sqlx::query!(
        "SELECT start_time, end_time, current_boss, current_hp, current_lap
             FROM public.rong_clanbattle
             WHERE id = $1;",
        closest_cb.id
    )
    .fetch_one(&pool)
    .await
    {
        Ok(row) => row,
        Err(_) => {
            msg.channel_id
                .say(
                    ctx,
                    format!("There are no clan battle info for {:}", closest_cb.cb_name),
                )
                .await?;
            return Ok(());
        }
    };

    let cb_start_epoch = cb_info.start_time.unwrap().unix_timestamp();
    let cb_end_epoch = cb_info.end_time.unwrap().unix_timestamp();

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

    match cb_status {
        CbStatus::Active => {
            let cb_day: i32 = ((epoch_now - cb_start_epoch) / 86400 + 1)
                .try_into()
                .unwrap();
            let hits_today =
                match sqlx::query!(
                    "SELECT
                        COALESCE(SUM(CASE WHEN \"hit_type\" = 'Normal' THEN 1 ELSE 0.5 END), 0) as hits
                     FROM public.rong_clanbattlescore
                     WHERE
                        clan_battle_id = $1
                        AND day = $2;",
                    closest_cb.id,
                    cb_day
                )
                .fetch_one(&pool)
                .await {
                    Ok(row) => row.hits.unwrap(),
                    Err(_) => {
                        msg.channel_id.say(ctx, format!("There are no clan battle info for {:}", closest_cb.cb_name)).await?;
                        return Ok(());
                    }
                };
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title(format!("{} - {}", clan_info.clan_name, closest_cb.cb_name))
                            .description(format!("Current day: {}", cb_day))
                            //.image("attachment://KyoukaSmile.jpg")
                            .fields(vec![
                                (
                                    "Status: Active",
                                    format!(
                                        "Current Lap: {}\n\
                                  Current Boss: {}\n\
                                  Current Hp: {}",
                                        cb_info.current_lap.unwrap(),
                                        cb_info.current_boss.unwrap(),
                                        cb_info.current_hp.unwrap()
                                    ),
                                    false,
                                ),
                                (
                                    "Hits Info",
                                    format!(
                                        "Hits today: {}/90\n\
                                      Carryovers left: WIP",
                                        hits_today
                                    ),
                                    true,
                                ),
                                // ("Bosses killed today:", format!("WIP"), true),
                                // ("Dmg done today:", format!("WIP"), true),
                                // ("Overall RV%:", format!("WIP"), false)
                            ])
                            .footer(|f| f.text("Days since last int: 0"))
                            // Add a timestamp for the current time
                            // This also accepts a rfc3339 Timestamp
                            .timestamp(chrono::Utc::now().to_rfc3339())
                            .color(Color::BLITZ_BLUE)
                    })
                    //.add_file("./KyoukaSmile.jpg")
                })
                .await?;
        }
        CbStatus::Past => {
            msg.channel_id
                .say(
                    ctx,
                    format!(
                        "{clan_name} - {name} is already over. \
                            {name} ended <t:{epoch}:R>.",
                        clan_name = clan_info.clan_name,
                        name = closest_cb.cb_name,
                        epoch = cb_end_epoch
                    ),
                )
                .await?;
        }
        CbStatus::Future => {
            msg.channel_id
                .say(
                    ctx,
                    format!(
                        "{clan_name} - {name} has not started yet, \
                            {name} will begin <t:{epoch}:R>.",
                        clan_name = clan_info.clan_name,
                        name = closest_cb.cb_name,
                        epoch = cb_start_epoch
                    ),
                )
                .await?;
        }
    }
    Ok(())
}
