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

#[command("atc_fc")]
#[aliases("fc")]
#[description("Marks a person to have used force quit.")]
async fn forcequit(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
	msg.channel_id
		.say(&ctx.http, "This command is not finished")
		.await?;

	Ok(())
}
