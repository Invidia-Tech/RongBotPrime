use serenity::{
    client::Context,
    framework::standard::{
        Args,
        CommandResult,
        macros::command,
    },
    model::channel::Message,
};

#[command("end")]
#[description("End a flight.")]
#[bucket = "atc"]
async fn flight_end(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {

    Ok(())
}
