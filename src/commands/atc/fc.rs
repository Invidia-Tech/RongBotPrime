use chrono::Utc;
use serenity::{
    client::Context,
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    model::channel::Message,
};

use crate::{
    data::{
        CbStatus,
        ChannelPersona,
        DatabasePool,
    },
    utils::{
        clan::*,
        macros::*,
    },
};

#[command("atc_fc")]
#[aliases("fc", "reset")]
#[description("Marks a person to have used force quit.")]
async fn force_quit(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // Only allows this command within CB marked channels.
    let (clan_id, clan_name) = result_or_say_why!(
        get_clan_from_channel_context(ctx, msg, ChannelPersona::Cb),
        ctx,
        msg
    );

    let (cb_info, cb_status) =
        result_or_say_why!(get_latest_cb(ctx, &clan_id, &clan_name), ctx, msg);

    match cb_status {
        CbStatus::Future => {
            msg.channel_id
                .say(
                    ctx,
                    format!(
                        "You cannot call FC commands without an active CB.\n\
                        {clan_name} - {name} has not started yet. \
                        {name} will start <t:{start_epoch}:R> and end <t:{end_epoch}:R>.",
                        clan_name = clan_name,
                        name = cb_info.name,
                        start_epoch = cb_info.start_time.unwrap().timestamp(),
                        end_epoch = cb_info.end_time.unwrap().timestamp()
                    ),
                )
                .await?;
            return Ok(());
        }
        CbStatus::Past => {
            msg.channel_id
                .say(
                    ctx,
                    format!(
                        "You cannot call FC commands without an active CB.\n\
                        {clan_name} - {name} is already over. \
                        {name} started <t:{start_epoch}:R> and ended <t:{end_epoch}:R>.",
                        clan_name = clan_name,
                        name = cb_info.name,
                        start_epoch = cb_info.start_time.unwrap().timestamp(),
                        end_epoch = cb_info.end_time.unwrap().timestamp()
                    ),
                )
                .await?;
            return Ok(());
        }
        _ => (),
    };

    let cb_start_epoch = cb_info.start_time.unwrap().timestamp();

    let epoch_now = Utc::now().timestamp();
    let cb_day: i32 = ((epoch_now - cb_start_epoch) / 86400 + 1)
        .try_into()
        .unwrap();

    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();

    let (member_id, _user_id);
    match args.len() {
        0 | 1 => {
            if !args.is_empty() {
                let ign = args.single::<String>().unwrap();
                (member_id, _user_id) =
                    result_or_say_why!(get_clan_member_id_by_ign(ctx, &clan_id, &ign), ctx, msg);
            } else {
                member_id = match sqlx::query!(
                    "SELECT cm.id
                     FROM rong_clanmember cm
                     JOIN rong_user cu ON
                          cu.id = cm.user_id
                     WHERE cu.platform_id = $1 AND
                           cm.clan_id = $2;",
                    &msg.author.id.to_string(),
                    &clan_id
                )
                .fetch_one(&pool)
                .await
                {
                    Ok(row) => row.id,
                    Err(_) => {
                        msg.channel_id
                            .say(ctx, "Who are you? I don't know you!")
                            .await?;
                        return Ok(());
                    }
                };
            }
        }
        _ => {
            msg.channel_id
                .say(ctx, "You can only FC yourself or another player")
                .await?;
            return Ok(());
        }
    }
    let used_fc: i64 = match sqlx::query!(
        "SELECT COUNT (*) as count FROM rongbot.force_quit
                     WHERE clanmember_id = $1 AND
                           cb_id = $2 AND
                           day_used = $3;",
        &member_id,
        &cb_info.id,
        &cb_day
    )
    .fetch_one(&pool)
    .await
    {
        Ok(row) => row.count.unwrap_or(0),
        Err(_) => {
            msg.channel_id
                .say(ctx, "There is a problem getting FCs.")
                .await?;
            return Ok(());
        }
    };

    if used_fc > 0 {
        msg.reply_ping(ctx, "**WARNING, THIS FC HAS ALREADY BEEN USED.**")
            .await?;
    } else if used_fc == 0 {
        sqlx::query!(
            "INSERT INTO rongbot.force_quit (clanmember_id, cb_id, day_used, time_used)
                 VALUES ($1, $2, $3, now())",
            &member_id,
            &cb_info.id,
            &cb_day
        )
        .execute(&pool)
        .await?;
        msg.react(ctx, '✅').await?;
    } else {
        msg.reply(ctx, "Something went wrong... You've used... negative FCs?")
            .await?;
    }

    Ok(())
}
