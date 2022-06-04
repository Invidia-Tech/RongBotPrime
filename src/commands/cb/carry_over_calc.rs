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

#[command("cot_calc_time")]
#[aliases("ct", "cot", "ovk", "co", "of")]
#[description(
    "Calculates carry over time based on damage. \
     The first number is always the boss HP left. \
     The rest of the numbers are each damage value you're \
     thinking about sending into the boss. \
     Feel free to write the number in any denomination.\n\n\
     Examples:\n\
     \t`>cot 4000000`\n\
     \t`>ct 4.2 3.7 3.8 2.7`"
)]
async fn cot_calc_time(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        msg.reply(
            ctx,
            "You gotta at least give me something to work with here!",
        )
        .await?;

        return Ok(());
    }

    let boss_hp_left = args.single::<f32>()?;
    let max_num_hits = 3;
    let mut out_msg = "Rong's recommendations".to_string();
    if args.is_empty() {
        for i in 0..max_num_hits {
            let dmg_needed = (boss_hp_left / (i as f32 + (11.0 / 90.0))).ceil();
            out_msg.push_str(&format!("\n {} hit(s) avg dmg: {}", i + 1, dmg_needed));
        }
    } else {
        for arg in args.iter::<f32>() {
            // Zero troubles, zero worries.
            let arg = arg.unwrap_or(0.0);
        }
    }

    msg.reply(ctx, out_msg).await?;

    // let boss_hp_left: f64 = args.parse
    Ok(())
}

#[command("cot_calc_dmg")]
#[aliases("cd", "cotdmg", "cdmg", "cod")]
#[description(
    "Calculates damage needed to hit a given carryover time. \
     Please give the boss hp first, then the time target. \
     Following that, enter 0 or as many dmg hits as you would like.\n\
     Examples:\n\
     \t`>cod 4000000 67`\n\
     \t`>cd 4.2 45 3.7 3.8 2.7`"
)]
async fn cot_calc_dmg(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.len() < 2 {
        msg.reply(ctx, "I need at least the boss hp and a COT target!")
            .await?;

        return Ok(());
    }

    let boss_hp_left = args.single::<f32>()?;
    let mut cot_target = args.single::<u32>()?;
    if cot_target > 90 {
        cot_target = 90;
    }
    let max_num_hits = 3;
    let mut out_msg = format!("**Target COT: {}s**", cot_target);
    if args.is_empty() {
        for i in 1..=max_num_hits {
            // =CEILING(I39/(I40-(I41-11)/90))
            let dmg_needed = (boss_hp_left / (i as f32 - ((cot_target - 11) as f32) / 90.0)).ceil();
            out_msg.push_str(&format!("\n {} hit(s) avg dmg: {}", i, dmg_needed));
        }
    } else {
        for arg in args.iter::<f32>() {
            // Zero troubles, zero worries.
            let arg = arg.unwrap_or(0.0);
        }
    }

    msg.reply(ctx, out_msg).await?;

    // let boss_hp_left: f64 = args.parse
    Ok(())
}
