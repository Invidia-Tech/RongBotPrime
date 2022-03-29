use crate::data::DatabasePool;

use serenity::{
    client::Context,
    framework::standard::{
        Args,
        CommandResult,
        macros::command,
    },
    model::channel::Message,
    utils::MessageBuilder,
};

#[command("status")]
#[description("This shows the status of the current active clan battle.")]
async fn cb_status(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    // The message builder allows for creating a message by
    // mentioning users dynamically, pushing "safe" versions of
    // content (such as bolding normalized content), displaying
    // emojis, and more.
    let response = MessageBuilder::new()
            .push("User ")
            .push_bold_safe(&msg.author.name)
            .push(" used the 'cb status' command in the ")
            .mention(&msg.channel_id.to_channel_cached(&ctx.cache).await.unwrap())
            .push(" channel")
            .build();

    if let Err(why) = msg.channel_id.say(&ctx.http, &response).await {
        println!("Error sending message: {:?}", why);
    }

    let pool = ctx.data.read().await.get::<DatabasePool>().cloned().unwrap();



    let msg = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.content("Current CB: Test CB 13 (mocked)")
                .embed(|e| {
                    e.title("Clan: JEthereal")
                        .description("Current day: 2")
                        .image("attachment://KyoukaSmile.jpg")
                        .fields(vec![
                            ("Hits today:", "15", true),
                            ("Bosses killed today:", "3", true),
                            ("Dmg done today:", "89,204,302", true),
                            ("Overall RV%:", "98.3%", false)
                        ])
                        .field("Current boss:", "Kyouka", false)
                        .footer(|f| f.text("Days since last int: 0"))
                        // Add a timestamp for the current time
                        // This also accepts a rfc3339 Timestamp
                        .timestamp(chrono::Utc::now().to_rfc3339())
                })
                .add_file("./KyoukaSmile.jpg")
        })
        .await;

    if let Err(why) = msg {
        println!("Error sending message: {:?}", why);
    }
    Ok(())
}
