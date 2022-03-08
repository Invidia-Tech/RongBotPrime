use crate::checks::owner::*;

use serenity::{
    client::Context,
    framework::standard::{
        Args,
        CommandResult,
        macros::command,
    },
    model::channel::Message,
};

#[command]
#[checks(Owner)]
async fn debug_args(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id.say(&ctx.http, &format!("Arguments: {:?}", args.rest())).await?;

    Ok(())
}
