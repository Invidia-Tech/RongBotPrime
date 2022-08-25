use std::time::Duration;

use crate::checks::owner::*;

use serenity::{
    client::Context,
    collector::MessageCollectorBuilder,
    framework::standard::{macros::command, Args, CommandResult},
    futures::stream::StreamExt,
    model::channel::Message,
};

#[command("shadow_ping")]
#[checks(Owner)]
async fn shadow_ping(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.reply(ctx, "Let's do this... Helping you to shadow ping Dabo.")
        .await?;
    // We can create a collector from scratch too using this builder future.
    let collector = MessageCollectorBuilder::new(&ctx)
        // Only collect messages by this user.
        .channel_id(msg.channel_id)
        .collect_limit(200u32)
        .timeout(Duration::from_secs(20))
        // Build the collector.
        .build();

    // Let's acquire borrow HTTP to send a message inside the `async move`.
    let http = &ctx.http;

    let collected: Vec<_> = collector
        .then(|msg| async move {
            // let _ = msg.reply(http, format!("I repeat: {}", msg.content)).await;
            if msg.content.contains("<@79515100536385536>") {
                msg.delete(http).await;
                ()
            }

            if msg.content.contains("You have pinged Dabo") {
                msg.delete(http).await;
            }
        })
        .collect()
        .await;

    msg.reply(ctx, format!("Cleaned up {} shadow pings.", collected.len()))
        .await?;

    Ok(())
}
