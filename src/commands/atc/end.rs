use serenity::{
    client::Context,
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    model::channel::Message,
};

#[command("end")]
#[description("End a flight.")]
#[bucket = "atc"]
async fn flight_end(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult { Ok(()) }
