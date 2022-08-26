use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::prelude::Ready,
};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected to Discord!", ready.user.name);
        const VERSION: &str = env!("CARGO_PKG_VERSION");

        let status_channel = match ctx.cache.guild_channel(1010093749969371187) {
            Some(channel) => channel,
            None => {
                println!("Cannot find the status channel to give an update to.");
                return;
            }
        };

        match status_channel
            .send_message(ctx, |m| {
                m.add_embed(|e| {
                    e.title("RongPrime is now online!")
                        .field("**Version**", VERSION, false)
                })
            })
            .await
        {
            Ok(_) => {
                println!("Updated status channel.")
            }
            Err(e) => {
                println!("Could not update status channel due to {:?}", e)
            }
        };
    }
}
