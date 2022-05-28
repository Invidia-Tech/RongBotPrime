use chrono::Utc;
use serenity::{
    builder::CreateEmbed,
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

#[command("atc_check_fc")]
#[aliases("cfc", "check_fc")]
#[description("Check if people have used their FC or not.")]
async fn check_fc(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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

    let all_clanmember_ign_map =
        result_or_say_why!(get_all_clanmember_ign_map(ctx, &clan_id), ctx, msg);

    let (mut member_id, mut user_id);
    let mut search_ids: Vec<i32> = Vec::new();
    if args.is_empty() {
        match sqlx::query!(
            "SELECT id FROM rong_clanmember WHERE clan_id = $1 AND active = true ORDER BY ign;",
            clan_id
        )
        .fetch_all(&pool)
        .await
        {
            Ok(ids) => {
                for id in ids {
                    search_ids.push(id.id);
                }
            }
            Err(_) => {
                msg.reply(ctx, "Your clan has no users, or something is wrong.")
                    .await?;
                return Ok(());
            }
        }
    } else {
        while !args.is_empty() {
            let ign = args.single::<String>().unwrap();
            (member_id, user_id) =
                result_or_say_why!(get_clan_member_id_by_ign(ctx, &clan_id, &ign), ctx, msg);
            search_ids.push(member_id);
        }
    }

    let mut fc_results: Vec<(String, bool)> = Vec::new();
    for id in search_ids {
        let used_fc: i64 = match sqlx::query!(
            "SELECT COUNT (*) as count FROM rongbot.force_quit
                     WHERE clanmember_id = $1 AND
                           cb_id = $2 AND
                           day_used = $3;",
            &id,
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
        let default_no_ign = "No IGN".to_string();
        fc_results.push((
            (&all_clanmember_ign_map.get(&id).unwrap_or(&default_no_ign)).to_string(),
            used_fc > 0,
        ));
    }

    fc_results.sort_unstable_by(|a, b| a.0.cmp(&b.0));

    let mut out_msg = "".to_string();
    let mut fc_result_page = CreateEmbed::default();
    fc_result_page
        .title(format!(
            "Checked FC usage for {} - {} on day {}:",
            &clan_name, &cb_info.name, &cb_day
        ))
        .timestamp(chrono::Utc::now().to_rfc3339());

    let single_row_limit = 15;
    let mut row_count = 0;
    for (ign, used_fc) in fc_results {
        row_count += 1;
        out_msg += format!("{} - {}\n", if used_fc { "❌" } else { "✅" }, ign).as_str();
        if row_count >= single_row_limit {
            fc_result_page.field("FCs", &out_msg, true);
            row_count = 0;
            out_msg = "".to_string();
        }
    }
    if row_count != 0 {
        fc_result_page.field("FCs", &out_msg, true);
    }

    msg.channel_id
        .send_message(ctx, |m| m.set_embed(fc_result_page))
        .await?;

    Ok(())
}
