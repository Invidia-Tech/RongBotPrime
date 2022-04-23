use crate::checks::owner::*;

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

#[command]
#[checks(Owner)]
async fn debug_args(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    println!("Arugments: {:?}", args.rest());
    msg.channel_id
        .say(&ctx.http, &format!("Check logs!"))
        .await?;

    Ok(())
}
