use serenity::{
	client::Context,
	framework::standard::{
		macros::command,
		Args,
		CommandResult,
	},
	model::channel::Message,
	utils::MessageBuilder,
};

#[command("atc_my_status")]
#[aliases("ms", "mys", "my_status")]
#[description("Full summary of your flights.")]
#[bucket = "atc"]
async fn flight_summary(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
	msg.channel_id
		.say(&ctx.http, "This command is not finished")
		.await?;

	Ok(())
}
