use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

#[command("start")]
#[description("Start a flight.")]
#[bucket = "atc"]
async fn flight_start(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {
    Ok(())
}
