use serenity::{
    client::{
        bridge::gateway::ShardId,
        Context,
    },
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    model::{
        channel::Message,
        id::UserId,
    },
    utils::{
        content_safe,
        ContentSafeOptions,
    },
};

use crate::{
    checks::rong_admin::*,
    data::DatabasePool,
};

#[command("ping_drop_table")]
// Limit command usage to guilds.
#[only_in(guilds)]
#[description("Check the full drop table of pings.")]
#[aliases("table")]
#[checks(RongAdmin)]
async fn ping_drop_table(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();

    let guild_id = match msg.guild_id {
        None => {
            msg.reply(ctx, "Unable to obtain guild id.").await?;
            return Ok(());
        }
        Some(id) => id.0,
    };

    let ping_list = match sqlx::query!(
        "SELECT
            user_id, weight, rarity_rank,
            dm.nickname as disc_nick,
            dt.nickname as ping_nick
         FROM rongbot.ping_droptable dt
         JOIN rongbot.discord_members dm
            ON dt.user_id = dm.member_id
         WHERE server=$1
         ORDER BY rarity_rank;",
        &guild_id.to_string(),
    )
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => {
            if rows.is_empty() {
                msg.reply(
                    ctx,
                    "The droptable is empty! <:KasumiDerp:988300507091189760>",
                )
                .await?;
                return Ok(());
            }
            rows
        }
        Err(_e) => {
            msg.reply(ctx, "There was an error in getting the drop table!")
                .await?;
            return Ok(());
        }
    };

    let rarity_text: Vec<&str> = vec!["[N]", "[R]", "[SR]", "[SSR]", "[UR]"];
    let mut weights_total: Vec<i32> = vec![0, 0, 0, 0, 0];
    for l in &ping_list {
        weights_total[(l.rarity_rank - 1) as usize] += l.weight
    }

    let mut out_msg = "The current loot table:".to_string();
    for l in &ping_list {
        let name_from_guild;
        if l.user_id == "self" {
            name_from_guild = "Self Ping".to_string();
        } else {
            let user_id = l.user_id.parse::<u64>().unwrap();
            if let Some(u) = ctx.cache.user(user_id) {
                name_from_guild = match u.nick_in(ctx, guild_id).await {
                    Some(nick) => nick,
                    None => u.name,
                };
            } else if let Ok(u) = UserId(user_id).to_user(ctx).await {
                name_from_guild = match u.nick_in(ctx, guild_id).await {
                    Some(nick) => nick,
                    None => u.name,
                };
            } else {
                name_from_guild = format!("Unknown name. {}", l.user_id);
            }
        }
        out_msg.push_str(&format!(
            "\n\t- {} {} ({:.2}%)",
            name_from_guild,
            rarity_text[(l.rarity_rank - 1) as usize],
            (l.weight as f64 / weights_total[(l.rarity_rank - 1) as usize] as f64) * 100.0
        ));
    }

    msg.reply(ctx, out_msg).await?;
    Ok(())
}
