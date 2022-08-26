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

        let status_channel = match ctx.cache.channel(1010093749969371187) {
            Some(channel) => channel,
            None => match ctx.http.get_channel(1010093749969371187).await {
                Ok(ch) => ch,
                Err(e) => {
                    println!(
                        "Cannot find the status channel to give an update to. Error: {:?}",
                        e
                    );
                    return;
                }
            },
        };

        if let Some(ch) = status_channel.guild() {
            match ch
                .send_message(ctx, |m| {
                    m.add_embed(|e| {
                        e.title("RongPrime is now online!")
                            .field("**Version**", VERSION, false)
                    })
                })
                .await
            {
                Ok(_) => {
                    println!("Updated status channel.");
                }
                Err(e) => {
                    println!("Could not update status channel due to {:?}", e);
                }
            };
        } else {
            println!("Somehow got a channel that's not a guild channel???");
        }
    }
}
