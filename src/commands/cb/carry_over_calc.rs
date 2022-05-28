use crate::{
    data::{
        CbStatus,
        ChannelPersona,
        DatabasePool,
    },
    utils::{
        clan::*,
        macros::*,
    },
};

use serenity::{
    client::Context,
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    model::channel::Message,
    utils::Color,
};

#[command("cot_calc")]
#[aliases("cot", "ovk")]
#[description(
    "Calculates carry over time based on damage. \
     The first number is always the boss HP left. \
     The rest of the numbers are each damage value you're \
     thinking about sending into the boss. \
     Feel free to write the number in any denomination.\n\n\
     Examples:\n\
     \t`>cot 4000000`\n\
     \t`>ovk 4.2 3.7 3.8 2.7`"
)]
async fn carry_over_calc(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        msg.reply(
            ctx,
            "You gotta at least give me something to work with here!",
        )
        .await?;

        return Ok(());
    }

    // let boss_hp_left: f64 = args.parse
    Ok(())
}
