use serenity::{
    client::Context,
    framework::standard::{
        Args,
        CommandResult,
        macros::command,
    },
    model::channel::Message,
};

#[command("start")]
#[description("Start a flight.")]
#[bucket = "atc"]
async fn flight_start(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {

    Ok(())
}
