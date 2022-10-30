use chrono::Utc;
use serenity::{
    builder::{CreateActionRow, CreateButton},
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::{application::component::ButtonStyle, channel::Message},
};

use core::fmt;
use std::{str::FromStr, time::Duration};

use crate::{
    data::{CbStatus, ChannelPersona, DatabasePool},
    utils::{clan::*, macros::*},
};

#[derive(Debug, strum_macros::EnumString)]
enum UnFCOptions {
    #[strum(ascii_case_insensitive)]
    Yes,
    #[strum(ascii_case_insensitive)]
    No,
}

impl fmt::Display for UnFCOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Yes => write!(f, "Yes"),
            Self::No => write!(f, "No"),
        }
    }
}

impl UnFCOptions {
    pub fn emoji(&self) -> char {
        match self {
            Self::Yes => '✅',
            Self::No => '❌',
        }
    }

    fn button(&self) -> CreateButton {
        let mut b = CreateButton::default();
        b.custom_id(self);
        b.emoji(self.emoji());
        b.label(self);
        b.style(ButtonStyle::Primary);
        b
    }

    pub fn action_row() -> CreateActionRow {
        let mut ar = CreateActionRow::default();
        // We can add up to 5 buttons per action row
        ar.add_button(UnFCOptions::Yes.button());
        ar.add_button(UnFCOptions::No.button());
        ar
    }
}

#[command("atc_unfc")]
#[aliases("unfc", "unreset")]
#[description(
    "Removes a false forcequit marking.
     Use with >unfc (your own account) or >unfc name"
)]
async fn undo_force_quit(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // Only allows this command within CB marked channels.
    let (clan_id, clan_name) = result_or_say_why!(
        get_clan_from_channel_context(ctx, msg, ChannelPersona::Cb),
        ctx,
        msg
    );

    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();

    let is_superadmin = match sqlx::query!(
        "SELECT is_superadmin
         FROM public.rong_user
         WHERE platform_id = $1;",
        msg.author.id.to_string()
    )
    .fetch_one(&pool)
    .await
    {
        Ok(row) => row.is_superadmin,
        Err(_) => false,
    };

    let is_guild_lead = match sqlx::query!(
        "SELECT is_lead
         FROM rong_clanmember cm
         JOIN rong_user u ON cm.user_id = u.id
         WHERE cm.active = true
           AND clan_id = $1
           AND platform_id = $2",
        clan_id,
        msg.author.id.to_string()
    )
    .fetch_one(&pool)
    .await
    {
        Ok(row) => row.is_lead,
        Err(_) => false,
    };

    if !(is_guild_lead || is_superadmin) {
        msg.reply_ping(
            ctx,
            format!("You are neither a lead of {} nor a Superadmin.", clan_name),
        )
        .await?;
        return Ok(());
    }

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

    let member_id;
    match args.len() {
        0 | 1 => {
            if !args.is_empty() {
                let ign = args.single::<String>().unwrap();
                (member_id, _) =
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
                .say(ctx, "You can only UnFC yourself or another player")
                .await?;
            return Ok(());
        }
    }
    let used_fc: i64 = match sqlx::query!(
        "SELECT COUNT (*) as count
         FROM rongbot.force_quit
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
        let m = msg
            .channel_id
            .send_message(&ctx, |m| {
                m.content("Are you **SURE** you want to undo this FC?")
                    .components(|c| c.add_action_row(UnFCOptions::action_row()))
            })
            .await
            .unwrap();

        // Wait for the user to make a selection
        let mci = match m
            .await_component_interaction(&ctx)
            .timeout(Duration::from_secs(20))
            .await
        {
            Some(ci) => ci,
            None => {
                msg.reply(&ctx, "Timed out.").await.unwrap();
                m.delete(&ctx).await.unwrap();
                return Ok(());
            }
        };

        let selected_option: UnFCOptions = UnFCOptions::from_str(&mci.data.custom_id)?;
        match selected_option {
            UnFCOptions::Yes => {
                sqlx::query!(
            "DELETE FROM rongbot.force_quit WHERE clanmember_id = $1 AND cb_id = $2 AND day_used = $3",
            &member_id,
            &cb_info.id,
            &cb_day
        )
                    .execute(&pool)
                    .await?;
                msg.react(ctx, '✅').await?;
                m.delete(ctx).await?;
                return Ok(());
            }
            UnFCOptions::No => {
                m.delete(ctx).await?;
                msg.reply(ctx, "Canceled, not clearing FC.").await?;
                return Ok(());
            }
        };
    } else if used_fc == 0 {
        msg.reply_ping(ctx, "**This FC doesn't seem to have been used yet.**")
            .await?;
    } else {
        msg.reply(ctx, "Something went wrong... You've used... negative FCs?")
            .await?;
    }

    Ok(())
}
