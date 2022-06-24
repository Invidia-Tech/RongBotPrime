



use serenity::{
    client::Context,
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    model::channel::Message,
};

fn required_dmg_full_cot(mut out_msg: String, boss_hp_left: f32, max_num_hits: i32) -> String {
    for i in 0..max_num_hits {
        let mut dmg_needed = boss_hp_left / (i as f32 + (11.0 / 90.0));
        dmg_needed = (dmg_needed * 1000.0 + 1.0).ceil() / 1000.0;
        out_msg.push_str(&format!("\n {} hit(s) avg dmg: {}", i + 1, dmg_needed));
    }
    out_msg
}

fn required_dmg_target_cot(
    mut out_msg: String,
    boss_hp_left: f32,
    cot_target: u32,
    max_num_hits: i32,
) -> String {
    for i in 1..=max_num_hits {
        // =CEILING(I39/(I40-(I41-11)/90))
        let mut dmg_needed = boss_hp_left / (i as f32 - ((cot_target - 11) as f32) / 90.0);
        dmg_needed = (dmg_needed * 1000.0 + 1.0).ceil() / 1000.0;
        out_msg.push_str(&format!("\n\t{} hit(s) avg dmg: {}", i, dmg_needed));
    }
    out_msg
}

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

    let mut boss_hp_left = match args.single::<f32>() {
        Ok(hp) => hp,
        _ => {
            msg.reply(
                ctx,
                "Pure numbers only please! I did not recognize your boss hp number.",
            )
            .await?;
            return Ok(());
        }
    };
    let mut triaged_dmgs: Vec<f32> = Vec::new();
    let max_num_hits = 3;

    let mut out_msg = "Rong's recommendations for full 90s COT:".to_string();
    if args.is_empty() {
        out_msg = required_dmg_full_cot(out_msg, boss_hp_left, max_num_hits);
    } else {
        for arg in args.iter::<f32>() {
            // Zero troubles, zero worries.
            let new_dmg = arg.unwrap_or(0.0);
            if new_dmg == 0.0 {
                continue;
            }
            triaged_dmgs.push(new_dmg);
        }
        if triaged_dmgs.is_empty() {
            msg.reply(ctx, "I don't recognize any dmg numbers you sent")
                .await?;
            return Ok(());
        }

        triaged_dmgs.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap());

        out_msg.push_str("\nTriaged damage: ");
        let mut triaged_count = 0;
        for dmg in &triaged_dmgs {
            if dmg > &boss_hp_left {
                if triaged_count != 0 {
                    out_msg.push_str("-> ");
                }
                out_msg.push_str(&format!("**{}**", &dmg));

                let mut cot = (((dmg - boss_hp_left) / dmg * 90.0) + 10.0).ceil() as u32;
                if cot > 90 {
                    cot = 90;
                }
                out_msg.push_str(&format!(
                    "\nReleased in this order, your COT is: **{}s**",
                    cot
                ));
                if cot < 90 {
                    out_msg.push_str("\nFull COT not achieved, Rong recommends:");
                    out_msg = required_dmg_full_cot(out_msg, boss_hp_left, max_num_hits);
                }
                break;
            } else {
                if triaged_count != 0 {
                    out_msg.push_str("-> ");
                }
                if triaged_count + 1 >= triaged_dmgs.len() {
                    out_msg.push_str(&format!("**{}**", &dmg));
                } else {
                    out_msg.push_str(&format!("{} ", &dmg));
                }
                triaged_count += 1;
                boss_hp_left -= dmg;
            }
        }

        if triaged_count == triaged_dmgs.len() {
            out_msg.push_str(&format!(
                "\nNot enough total dmg to kill. Rong recommends:\n\
                 Remaining boss HP - **{:.3}**",
                &boss_hp_left
            ));
            out_msg = required_dmg_full_cot(out_msg, boss_hp_left, max_num_hits);
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

    let mut boss_hp_left = match args.single::<f32>() {
        Ok(hp) => hp,
        _ => {
            msg.reply(
                ctx,
                "Pure numbers only please! I did not recognize your boss hp number.",
            )
            .await?;
            return Ok(());
        }
    };
    let mut cot_target = args.single::<u32>()?;
    let mut triaged_dmgs: Vec<f32> = Vec::new();

    if cot_target > 90 {
        cot_target = 90;
    }
    let max_num_hits = 3;
    let mut out_msg = format!("**Target COT: {}s**", cot_target);
    if args.is_empty() {
        out_msg = required_dmg_target_cot(out_msg, boss_hp_left, cot_target, max_num_hits);
    } else {
        for arg in args.iter::<f32>() {
            // Zero troubles, zero worries.
            let new_dmg = arg.unwrap_or(0.0);
            if new_dmg == 0.0 {
                continue;
            }
            triaged_dmgs.push(new_dmg);
        }
        if triaged_dmgs.is_empty() {
            msg.reply(ctx, "I don't recognize any dmg numbers you sent")
                .await?;
            return Ok(());
        }

        triaged_dmgs.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap());

        out_msg.push_str("\nTriaged damage: ");
        let mut triaged_count = 0;
        for dmg in &triaged_dmgs {
            if dmg > &boss_hp_left {
                if triaged_count != 0 {
                    out_msg.push_str("-> ");
                }
                out_msg.push_str(&format!("**{}**", &dmg));

                let mut cot = (((dmg - boss_hp_left) / dmg * 90.0) + 10.0).ceil() as u32;
                if cot > 90 {
                    cot = 90;
                }
                out_msg.push_str(&format!(
                    "\nReleased in this order, your COT is: **{}s**",
                    cot
                ));
                if cot < cot_target {
                    out_msg.push_str(&format!(
                        "\nCOT target (**{}s**) not reached, Rong recommends:",
                        &cot_target
                    ));
                    out_msg =
                        required_dmg_target_cot(out_msg, boss_hp_left, cot_target, max_num_hits);
                }
                break;
            } else {
                if triaged_count != 0 {
                    out_msg.push_str("-> ");
                }
                if triaged_count + 1 >= triaged_dmgs.len() {
                    out_msg.push_str(&format!("**{}**", &dmg));
                } else {
                    out_msg.push_str(&format!("{} ", &dmg));
                }
                triaged_count += 1;
                boss_hp_left -= dmg;
            }
        }

        if triaged_count == triaged_dmgs.len() {
            out_msg.push_str(&format!(
                "\nNot enough total dmg to kill. Rong recommends:\n\
                 Remaining boss HP - **{:.3}**",
                &boss_hp_left
            ));
            out_msg = required_dmg_target_cot(out_msg, boss_hp_left, cot_target, max_num_hits);
        }
    }

    msg.reply(ctx, out_msg).await?;

    // let boss_hp_left: f64 = args.parse
    Ok(())
}
