use crate::{
    data::{
        DatabasePool,
    },
};

use crate::checks::owner::*;



use serenity::{
    client::Context,
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    model::channel::Message,
    utils::{
        parse_username,
    },
};

#[command("add_superadmin")]
#[description("Adds a new rong superadmin.")]
#[checks(Owner)]
async fn add_superadmin(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.len() != 1 {
        msg.channel_id
            .say(ctx, "Please mention a person to make superadmin")
            .await?;
        return Ok(());
    }

    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();

    let mentioned_user = match parse_username(args.single::<String>().unwrap()) {
        Some(user) => user.to_string(),
        _ => {
            msg.reply(ctx, "Please mention an actual user! Ping them ping them!")
                .await?;
            return Ok(());
        }
    };

    sqlx::query!(
        "UPDATE rong_user
         SET is_superadmin=True
         WHERE platform_id=$1;",
        mentioned_user
    )
    .execute(&pool)
    .await?;

    msg.reply(ctx, "Added new rong superadmin!").await?;

    Ok(())
}

#[command("remove_superadmin")]
#[description("Removes a person from being rong superadmin.")]
#[checks(Owner)]
async fn remove_superadmin(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.len() != 1 {
        msg.channel_id
            .say(ctx, "Please mention a person to remove from superadmin")
            .await?;
        return Ok(());
    }

    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();

    let mentioned_user = match parse_username(args.single::<String>().unwrap()) {
        Some(user) => user.to_string(),
        _ => {
            msg.reply(ctx, "Please mention an actual user! Ping them ping them!")
                .await?;
            return Ok(());
        }
    };

    sqlx::query!(
        "UPDATE rong_user
         SET is_superadmin=False
         WHERE platform_id=$1;",
        mentioned_user
    )
    .execute(&pool)
    .await?;

    msg.reply(ctx, "They are no longer a rong superadmin.")
        .await?;

    Ok(())
}
