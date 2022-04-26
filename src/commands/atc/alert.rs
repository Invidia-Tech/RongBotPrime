use serenity::{
	client::Context,
	framework::standard::{
		macros::command,
		Args,
		CommandResult,
	},
	model::channel::Message,
};

use crate::{
	data::{
		CbStatus,
		ChannelPersona,
		DatabasePool,
	},
	utils::{
		atc::*,
		clan::*,
		macros::*,
		rong::*,
	},
};

#[command("atc_alert")]
#[aliases("alert", "set_alert")]
#[description("Changes the alert channel.")]
#[bucket = "atc"]
async fn flight_summary(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
	msg.channel_id
		.say(&ctx.http, "This command is not finished")
		.await?;

	Ok(())
}
