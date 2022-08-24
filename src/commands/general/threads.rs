use crate::checks::owner::*;

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::{ChannelType, Message},
};

#[command("make_threads")]
#[checks(Owner)]
async fn make_threads(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.len() < 2 {
        msg.reply(ctx, "Give me names.").await?;
        return Ok(());
    }

    let inner_message = args.single_quoted::<String>()?;

    for arg in args.iter::<String>() {
        let name_err = "NAMING ERROR".to_string();
        // Zero troubles, zero worries.
        let name = arg.unwrap_or(name_err);
        if name == "NAMING ERROR" {
            continue;
        }
        let thread = msg
            .channel_id
            .create_private_thread(ctx, |t| {
                t.name(name)
                    .auto_archive_duration(10080)
                    .kind(ChannelType::PublicThread)
            })
            .await?;
        thread.say(ctx, &inner_message).await?;
    }

    msg.delete(ctx).await?;

    Ok(())
}
