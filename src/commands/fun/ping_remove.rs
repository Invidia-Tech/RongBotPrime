use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::{channel::Message, id::UserId, user::User},
    utils::parse_username,
};

use crate::{checks::rong_admin::*, data::DatabasePool};

#[command("ping_remove_loot")]
// Limit command usage to guilds.
#[only_in(guilds)]
#[description(
    "Delete someone from the ping table! \n\
     >ping remove @Ring"
)]
#[aliases("remove", "rm", "delete")]
#[checks(RongAdmin)]
async fn ping_remove_loot(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.len() != 1 {
        msg.channel_id
            .say(
                ctx,
                "Invalid command usage. Please use the help command on `config channel`.",
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

    let remove_name = args.single::<String>().unwrap();
    let mentioned_user;
    let mentioned_user_id;
    if remove_name == "self" {
        mentioned_user = remove_name;
        mentioned_user_id = 0;
    } else {
        mentioned_user = match parse_username(remove_name) {
            Some(user) => {
                mentioned_user_id = user;
                user.to_string()
            }
            _ => {
                msg.channel_id.say(&ctx.http, "Please ping a user!").await?;
                return Ok(());
            }
        };
    }

    sqlx::query!(
        "DELETE FROM rongbot.ping_droptable WHERE server=$1 AND user_id=$2;",
        guild_id.to_string(),
        mentioned_user.to_string(),
    )
    .execute(&pool)
    .await?;

    let username;
    let user;

    if mentioned_user == "self" {
        username = "**Self Ping**".to_string();
    } else {
        if let Some(u) = ctx.cache.user(mentioned_user_id) {
            user = u;
        } else if let Ok(u) = UserId(mentioned_user_id).to_user(ctx).await {
            user = u;
        } else {
            user = User::default();
        }
        username = match user.nick_in(ctx, guild_id).await {
            Some(nick) => nick,
            None => user.name,
        };
    }
    msg.channel_id
        .say(
            ctx,
            &format!("{} has been removed from the drop table!", &username),
        )
        .await?;

    Ok(())
}
