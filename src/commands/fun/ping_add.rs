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
        user::User,
    },
    utils::{
        content_safe,
        parse_username,
        ContentSafeOptions,
    },
};

use crate::{
    checks::rong_admin::*,
    data::DatabasePool,
};

#[command("ping_add_loot")]
// Limit command usage to guilds.
#[only_in(guilds)]
#[description(
    "Add someone to the ping loot table! \
     >ping add <@Name> <rarity> <weight>\n\
     Example: >ping add @Ring 3 1\n\
     This will add ring to [SR] with a weight of 1."
)]
#[aliases("add", "set")]
#[checks(RongAdmin)]
async fn ping_add_loot(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.len() != 3 {
        msg.channel_id
            .say(
                ctx,
                "Invalid command usage. Please use the help command on `ping add`.",
            )
            .await?;
        return Ok(());
    }

    let guild_id = match msg.guild_id {
        None => {
            msg.reply(ctx, "Unable to obtain guild id.").await?;
            return Ok(());
        }
        Some(id) => id.0,
    };

    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();

    let first_arg = args.single::<String>().unwrap();
    let mentioned_user: String;
    let mut mentioned_user_id = 0;

    if first_arg != "self" {
        mentioned_user = match parse_username(&first_arg) {
            Some(user) => {
                mentioned_user_id = user;
                user.to_string()
            }
            _ => {
                msg.reply(ctx, "Please mention an actual user! Ping them ping them!")
                    .await?;
                return Ok(());
            }
        };
    } else {
        mentioned_user = first_arg;
    }

    let rarity = args.single::<u32>().unwrap_or(0);
    if rarity == 0 {
        msg.reply(ctx, "Please enter a valid rarity! ([N] 1 - 5 [UR])")
            .await?;
        return Ok(());
    }

    let weight = args.single::<u32>().unwrap_or(0);
    if weight == 0 {
        msg.reply(ctx, "Please enter a valid weight! Anything above 0.")
            .await?;
        return Ok(());
    }

    sqlx::query!(
        "INSERT INTO rongbot.ping_droptable (server, user_id, rarity_rank, weight)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (server, user_id)
         DO UPDATE SET rarity_rank = $3, weight = $4;",
        guild_id.to_string(),
        mentioned_user,
        rarity as i32,
        weight as i32
    )
    .execute(&pool)
    .await?;

    let user;
    let rarity_text: Vec<&str> = vec!["[N]", "[R]", "[SR]", "[SSR]", "[UR]"];

    if let Some(u) = ctx.cache.user(mentioned_user_id) {
        user = u;
    } else if let Ok(u) = UserId(mentioned_user_id).to_user(ctx).await {
        user = u;
    } else {
        user = User::default();
    }

    let username = match user.nick_in(ctx, guild_id).await {
        Some(nick) => nick,
        None => user.name,
    };
    msg.channel_id
        .say(
            ctx,
            &format!(
                "{} has been updated to rarity: {} and weight: {}.",
                &username,
                rarity_text[(rarity - 1) as usize],
                weight
            ),
        )
        .await?;

    Ok(())
}
