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

#[command("atc_call_sign")]
#[aliases("callsign", "call_sign")]
#[description("Sets your pilot call sign.")]
#[bucket = "atc"]
async fn call_sign(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
	msg.channel_id
		.say(&ctx.http, "This command is not finished")
		.await?;

	Ok(())
}
