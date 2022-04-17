use crate::data::{CbStatus, ChannelPersona, DatabasePool};
use crate::utils::{clan::*, macros::*};

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    utils::Color,
};

use chrono::Utc;

#[command("cb_status")]
#[aliases("s", "status")]
#[bucket = "cb"]
#[description("This shows the status of the current active clan battle.")]
async fn cb_status(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
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

    let (clan_id, clan_name) = result_or_say_why!(
        get_clan_from_channel_context(ctx, msg, ChannelPersona::Cb),
        ctx,
        msg
    );

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

    // msg.channel_id
    //     .say(ctx, format!("Clan you're in is: {:?}", clan_info.clan_name))
    //     .await?;

    let (cb_info, cb_status) =
        result_or_say_why!(get_latest_cb(ctx, &clan_id, &clan_name), ctx, msg);

    let cb_start_epoch = cb_info.start_time.unwrap().timestamp();
    let cb_end_epoch = cb_info.end_time.unwrap().timestamp();

    // msg.channel_id.say(ctx, format!("CB info: start: <t:{:}:f>, end: <t:{:}:f>, {:}, {:}, {:}",
    //                                 cb_start_epoch, cb_end_epoch,
    //                                 cb_info.current_boss.unwrap(), cb_info.current_hp.unwrap(), cb_info.current_lap.unwrap())).await?;

    let epoch_now = Utc::now().timestamp();

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
                    cb_info.id,
                    cb_day
                )
                .fetch_one(&pool)
                .await {
                    Ok(row) => row.hits.unwrap(),
                    Err(_) => {
                        msg.channel_id.say(ctx, format!("There are no clan battle info for {:}", cb_info.name)).await?;
                        return Ok(());
                    }
                };
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title(format!("{} - {}", clan_name, cb_info.name))
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
                        clan_name = clan_name,
                        name = cb_info.name,
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
                        clan_name = clan_name,
                        name = cb_info.name,
                        epoch = cb_start_epoch
                    ),
                )
                .await?;
        }
    }
    Ok(())
}
