use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    utils::MessageBuilder,
};

#[command("status")]
#[description("This shows the status of current flights.")]
async fn flight_status(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.channel_id
        .say(&ctx.http, "Current flights: None")
        .await?;
    // The message builder allows for creating a message by
    // mentioning users dynamically, pushing "safe" versions of
    // content (such as bolding normalized content), displaying
    // emojis, and more.
    let response = MessageBuilder::new()
        .push("User ")
        .push_bold_safe(&msg.author.name)
        .push(" used the 'atc status' command in the ")
        .mention(&msg.channel_id.to_channel_cached(&ctx.cache).await.unwrap())
        .push(" channel")
        .build();

    if let Err(why) = msg.channel_id.say(&ctx.http, &response).await {
        println!("Error sending message: {:?}", why);
    }

    let msg = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.content("Rong ATC (Air Traffic Control) Status")
                .embed(|e| {
                    e.title("Current Flights")
                        .description("These are the recent running/landed flights.")
                        //.image("attachment://KyoukaSmile.jpg")
                        .fields(vec![
                            ("🛫 __Flight DB 14002__", "Pilot: Dabomstew", false),
                            ("Current Status:", "**In Progress**", true),
                            ("Duration:", "37 Minutes", true),
                        ])
                        .fields(vec![
                            ("🛬 __Flight BN 14002__", "Pilot: Boon", false),
                            ("Current Status:", "**Landed**", true),
                            ("Duration:", "09 Minutes", true),
                        ])
                        .fields(vec![
                            ("💥 __Flight RG 14001__", "Pilot: Ring", false),
                            ("Current Status:", "**Crashed**", true),
                            ("Duration:", "23 Minutes", true),
                        ])
                        .field("Overall Flight Status", "Flights Today: 2", false)
                        .footer(|f| f.text("Days since last int: 0"))
                        // Add a timestamp for the current time
                        // This also accepts a rfc3339 Timestamp
                        .timestamp(chrono::Utc::now().to_rfc3339())
                })
            //.add_file("./KyoukaSmile.jpg")
        })
        .await;

    if let Err(why) = msg {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}
