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
    model::channel::Message,
    utils::{
        content_safe,
        ContentSafeOptions,
    },
};

#[command]
// Limit command usage to guilds.
#[only_in(guilds)]
// #[checks(Owner)]
async fn ping_roll(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong! : )").await?;

    Ok(())
}
