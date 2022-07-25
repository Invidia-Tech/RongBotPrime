use serenity::{
    client::Context,
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    model::channel::Message,
};

fn required_dmg_full_cot(mut out_msg: String, boss_hp_left: f64, max_num_hits: i32) -> String {
    for i in 0..max_num_hits {
        let mut dmg_needed = boss_hp_left / (i as f64 + (11.0 / 90.0));
        dmg_needed = (dmg_needed * 1000.0 + 1.0).ceil() / 1000.0;
        out_msg.push_str(&format!("\n {} hit(s) avg dmg: {}", i + 1, dmg_needed));
    }
    out_msg
}

fn avg_dmg_needed_full_cot(boss_hp_left: f64, max_num_hits: i32) -> Vec<f64> {
    let mut v: Vec<f64> = Vec::new();
    for i in 0..max_num_hits {
        let mut dmg_needed = boss_hp_left / (i as f64 + (11.0 / 90.0));
        dmg_needed = (dmg_needed * 1000.0 + 1.0).ceil() / 1000.0;
        v.push(dmg_needed)
    }
    v
}

fn dmg_for_full_cot_in_n(boss_hp_left: f64, hits: f64) -> f64 {
    let dmg_needed = boss_hp_left / (hits - 1.0 + (11.0 / 90.0));
    (dmg_needed * 1000.0 + 1.0).ceil() / 1000.0
}

fn avg_dmg_for_given_hits(boss_hp_left: f64, dmg_already_in: f64, hits_needed: f64) -> f64 {
    (boss_hp_left - dmg_already_in / 8.2) / hits_needed
}

fn calc_cot(mut boss_hp_left: f64, triaged_dmg: &Vec<f64>) -> i32 {
    let mut cot = 0;
    for dmg in triaged_dmg {
        if dmg > &boss_hp_left {
            cot = (((dmg - boss_hp_left) / dmg * 90.0) + 10.0).ceil() as i32;
            if cot > 90 {
                cot = 90;
            }
            if cot < 0 {
                cot = 0;
            }
            return cot;
        } else {
            boss_hp_left -= dmg;
        }
    }
    cot
}

fn output_dmg_triage(mut boss_hp_left: f64, triaged_dmg: &Vec<f64>, mut out_msg: String) -> String {
    let mut triaged_count = 0;
    for dmg in triaged_dmg {
        if triaged_count != 0 {
            out_msg.push_str("-> ");
        }

        if dmg > &boss_hp_left {
            out_msg.push_str(&format!("**{}**", &dmg));

            let mut cot = (((dmg - boss_hp_left) / dmg * 90.0) + 10.0).ceil() as u64;
            if cot > 90 {
                cot = 90;
            }
            out_msg.push_str(&format!(
                "\nReleased in this order, your COT is: **{}s**",
                cot
            ));
            break;
        } else {
            if triaged_count + 1 >= triaged_dmg.len() {
                out_msg.push_str(&format!("**{}**", &dmg));
            } else {
                out_msg.push_str(&format!("{} ", &dmg));
            }
            triaged_count += 1;
            boss_hp_left -= dmg;
        }
    }
    out_msg
}

fn calculate_best_new_hits_needed(
    triaged_dmg: &Vec<f64>,
    boss_hp: f64,
    boss_hp_left: f64,
    new_hits_needed: f64,
) -> f64 {
    let avg_dmg: f64 = triaged_dmg.iter().sum::<f64>() / triaged_dmg.len() as f64;
    let triaged_dmg_sum = triaged_dmg.iter().sum::<f64>();
    let triaged_hits: f64 = triaged_dmg.len() as f64;
    let avg_hits_needed = (boss_hp / avg_dmg).ceil();
    let avg_dmg_needed = boss_hp * 90.0 / ((avg_hits_needed - 1.0) * 90.0 + 11.0);

    let mut avg_new_dmg_per_hit =
        (avg_dmg_needed * avg_hits_needed - triaged_dmg_sum) / new_hits_needed;
    avg_new_dmg_per_hit *= 1000.0;
    avg_new_dmg_per_hit = avg_new_dmg_per_hit.round();
    avg_new_dmg_per_hit /= 1000.0;

    // Compare against a potential smaller hit if all other hits are larger.
    let potential_smaller_hit = dmg_for_full_cot_in_n(boss_hp_left, new_hits_needed);
    if potential_smaller_hit < avg_new_dmg_per_hit {
        avg_new_dmg_per_hit = potential_smaller_hit;
    }

    // Calculate case when using the lowest triaged hit as the scam janny hit.
    let mut triaged_dmg_copy = triaged_dmg.clone();
    let last_hit_dmg = triaged_dmg_copy.pop().unwrap_or(0.0);
    let mut boss_hp_left_after_triage = boss_hp;
    for n in triaged_dmg_copy {
        boss_hp_left_after_triage -= n;
    }
    let max_front_hits = 3;
    if new_hits_needed > max_front_hits as f64 {
        return 0.0;
    }
    let mut front_dmg: f64 = 0.0;
    for i in new_hits_needed as usize..=max_front_hits {
        let potential_front_dmg =
            avg_dmg_for_given_hits(boss_hp_left_after_triage, last_hit_dmg, i as f64);
        if potential_front_dmg < last_hit_dmg {
            break;
        }
        front_dmg = potential_front_dmg;
    }

    if front_dmg != 0.0 && front_dmg < avg_new_dmg_per_hit {
        avg_new_dmg_per_hit = front_dmg;
    }

    avg_new_dmg_per_hit
}

fn required_dmg_target_cot(
    mut out_msg: String,
    boss_hp_left: f64,
    cot_target: u64,
    max_num_hits: i64,
) -> String {
    for i in 1..=max_num_hits {
        // =CEILING(I39/(I40-(I41-11)/90))
        let mut dmg_needed = boss_hp_left / (i as f64 - ((cot_target - 11) as f64) / 90.0);
        dmg_needed = (dmg_needed * 1000.0 + 1.0).ceil() / 1000.0;
        out_msg.push_str(&format!("\n\t{} hit(s) avg dmg: {}", i, dmg_needed));
    }
    out_msg
}

fn reach_target_cot(boss_hp_left: f64, cot_target: i32) -> f64 {
    let dmg_needed = boss_hp_left / (1.0 - ((cot_target - 11) as f64) / 90.0);
    (dmg_needed * 10000.0 + 1.0).ceil() / 10000.0
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

    let boss_hp = match args.single::<f64>() {
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

    let mut boss_hp_left = boss_hp;
    let mut dmg_inputs: Vec<f64> = Vec::new();
    let max_num_hits = 3;

    let mut out_msg = "Rong's recommendations for full 90s COT:".to_string();
    if args.is_empty() {
        out_msg = required_dmg_full_cot(out_msg, boss_hp_left, max_num_hits);
        msg.reply(ctx, out_msg).await?;
        return Ok(());
    }

    for arg in args.iter::<f64>() {
        // Zero troubles, zero worries.
        let new_dmg = arg.unwrap_or(0.0);
        if new_dmg == 0.0 {
            continue;
        }
        dmg_inputs.push(new_dmg);
    }
    if dmg_inputs.is_empty() {
        msg.reply(ctx, "I don't recognize any dmg numbers you sent")
            .await?;
        return Ok(());
    }

    dmg_inputs.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap());

    out_msg.push_str("\nTriaged damage: ");
    let mut triaged_count = 0;
    let mut triaged_dmg: Vec<f64> = Vec::new();
    let mut cot_reached = false;
    let mut cot = 0;
    for dmg in &dmg_inputs {
        triaged_dmg.push(*dmg);
        if triaged_count != 0 {
            out_msg.push_str("-> ");
        }

        if dmg > &boss_hp_left {
            out_msg.push_str(&format!("**{}**", &dmg));

            cot = (((dmg - boss_hp_left) / dmg * 90.0) + 10.0).ceil() as i32;
            if cot > 90 {
                cot = 90;
            }
            if cot < 0 {
                cot = 0;
            }
            out_msg.push_str(&format!(
                "\nReleased in this order, your COT is: **{}s**",
                cot
            ));
            cot_reached = true;
            break;
        } else {
            if triaged_count + 1 >= dmg_inputs.len() {
                out_msg.push_str(&format!("**{}**", &dmg));
            } else {
                out_msg.push_str(&format!("{} ", &dmg));
            }
            triaged_count += 1;
            boss_hp_left -= dmg;
        }
    }

    let avg_dmg: f64 = triaged_dmg.iter().sum::<f64>() / triaged_dmg.len() as f64;
    let triaged_dmg_sum = triaged_dmg.iter().sum::<f64>();
    let triaged_hits: f64 = triaged_dmg.len() as f64;
    let avg_hits_needed = (boss_hp / avg_dmg).ceil();
    let avg_dmg_needed = boss_hp * 90.0 / ((avg_hits_needed - 1.0) * 90.0 + 11.0);
    let new_hits_needed = avg_hits_needed - triaged_hits;

    if triaged_count == dmg_inputs.len() {
        out_msg.push_str(&format!(
            "\nNot enough total dmg to kill. Rong recommends:\n\
                 Remaining boss HP - **{:.3}**",
            &boss_hp_left
        ));

        // [BOSS HP] * 90 / ([Hits Desired] x 90 + 10) = [Average Damage needed]
        // out_msg.push_str(&format!("\nNew avg dmg needed: {:.3}", &avg_dmg_needed));
        println!(
            "Triaged_dmg_sum: {}, avg_hits_needed: {}, triaged_hits: {} new_hits_needed: {}",
            &triaged_dmg_sum, &avg_hits_needed, &triaged_hits, &new_hits_needed
        );

        // out_msg.push_str("\nThe average damage of triaged hits is lacking.\n");
        // Calculate underflow, need hits that is larger on average.
        let mut avg_new_dmg_per_hit =
            (avg_dmg_needed * avg_hits_needed - triaged_dmg_sum) / new_hits_needed;

        // Compare against a potential smaller hit if all other hits are larger.
        let potential_smaller_hit = dmg_for_full_cot_in_n(boss_hp_left, new_hits_needed);
        if potential_smaller_hit < avg_new_dmg_per_hit {
            avg_new_dmg_per_hit = potential_smaller_hit;
        }

        // Calculate case when using the lowest triaged hit as the scam janny hit.
        let mut triaged_dmg_copy = triaged_dmg.clone();
        let last_hit_dmg = triaged_dmg_copy.pop().unwrap();
        let mut boss_hp_left_after_triage = boss_hp;
        for n in triaged_dmg_copy {
            boss_hp_left_after_triage -= n;
        }
        let max_front_hits = 3;
        if new_hits_needed > max_front_hits as f64 {
            msg.reply(
                ctx,
                "The required number of hits is above what Rong is willing to calculate.",
            )
            .await?;
            return Ok(());
        }
        let mut front_dmg: f64 = 0.0;
        for i in new_hits_needed as usize..=max_front_hits {
            let potential_front_dmg =
                avg_dmg_for_given_hits(boss_hp_left_after_triage, last_hit_dmg, i as f64);
            if potential_front_dmg < last_hit_dmg {
                break;
            }
            front_dmg = potential_front_dmg;
        }

        if front_dmg != 0.0 && front_dmg < avg_new_dmg_per_hit {
            avg_new_dmg_per_hit = front_dmg;
        }

        // for (i, n) in front_dmg {
        //     let mut new_triaged_dmg = triaged_dmg.clone();
        //     for _ in 0..i {
        //         new_triaged_dmg.push(n);
        //     }
        //     new_triaged_dmg.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap());
        //     out_msg.push_str("\nAssuming new hits in the front:\n");
        //     out_msg = output_dmg_triage(boss_hp, &new_triaged_dmg, out_msg);
        // }

        // let mut filled_triaged_dmg = triaged_dmg.clone();
        // This means that we got some giga weird dmg and we're no longer getting 90s.
        // if (boss_hp / avg_new_dmg_per_hit).ceil() < avg_hits_needed {
        //     for _ in 0..(new_hits_needed as usize) {
        //         filled_triaged_dmg.push(avg_dmg);
        //     }
        // }

        // Consider n hits replacement
        let max_replacement = 2;
        let mut replacement_needs: Vec<(usize, f64)> = Vec::new();
        for i in 1..=max_replacement {
            if i <= new_hits_needed as usize {
                continue;
            }
            let mut triaged_dmg_copy = triaged_dmg.clone();
            for _ in (0)..(i - new_hits_needed as usize) {
                println!("Replacing {} hits", i);
                triaged_dmg_copy.pop();
            }
            let total_dmg_kept = triaged_dmg_copy.iter().sum();
            println!(
                "Boss_hp_left: {}, dmg_not_replaced: {}, hits_needed: {}",
                &boss_hp, &total_dmg_kept, i as f64
            );
            println!(
                "Answer was: {}",
                avg_dmg_for_given_hits(boss_hp, total_dmg_kept, i as f64)
            );
            let mut avg_dmg_for_replacement =
                avg_dmg_for_given_hits(boss_hp, total_dmg_kept, i as f64);
            avg_dmg_for_replacement *= 10000.0;
            avg_dmg_for_replacement = avg_dmg_for_replacement.round();
            avg_dmg_for_replacement /= 10000.0;
            replacement_needs.push((i, avg_dmg_for_replacement));
        }

        // let mut dmg_not_replaced = boss_hp;
        // let mut replacement_needs: Vec<f64> = Vec::new();
        // for (i, n) in filled_triaged_dmg.iter().rev().enumerate() {
        //     if i >= max_replacement {
        //         break;
        //     }
        //     dmg_not_replaced -= n;
        //     println!(
        //         "Boss_hp_left: {}, dmg_not_replaced: {}, hits_needed: {}",
        //         &boss_hp,
        //         &dmg_not_replaced,
        //         i as f64 + 1.0
        //     );
        //     println!(
        //         "Answer was: {}",
        //         avg_dmg_for_given_hits(boss_hp, dmg_not_replaced, i as f64 + 1.0)
        //     );
        //     replacement_needs.push(avg_dmg_for_given_hits(
        //         boss_hp,
        //         dmg_not_replaced,
        //         i as f64 + 1.0,
        //     ));
        // }

        println!("Replacement needs: {:?}", &replacement_needs);
        // let l = filled_triaged_dmg.len() - 1;
        // for (i, n) in replacement_needs.iter().enumerate() {
        //     if i + 1 < new_hits_needed as usize {
        //         continue;
        //     }
        //     let mut replacement_triaged_dmg = filled_triaged_dmg.clone();
        //     for j in 0..(i + 1) {
        //         replacement_triaged_dmg[l - j] = *n;
        //     }
        //     replacement_triaged_dmg.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap());
        //     out_msg.push_str(&format!("\nReplacing {} low hit, results in:\n", i + 1));
        //     out_msg = output_dmg_triage(boss_hp, &replacement_triaged_dmg, out_msg);
        // }

        for (i, n) in replacement_needs {
            let mut triaged_dmg_copy = triaged_dmg.clone();
            for _ in (0)..(i - new_hits_needed as usize) {
                println!("Replacing {} hits", i);
                triaged_dmg_copy.pop();
            }
            for _ in 0..i {
                triaged_dmg_copy.push(n);
            }
            triaged_dmg_copy.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap());
            if calc_cot(boss_hp, &triaged_dmg_copy) != 90 {
                // Rong is stupid
                continue;
            }
            if new_hits_needed > 0.0 {
                out_msg.push_str(&format!(
                    "\nReplacing {} low hit and adding {} new hit, results in:\n",
                    i - 1,
                    new_hits_needed
                ));
            } else {
                out_msg.push_str(&format!("\nReplacing {} low hit, results in:\n", i));
            }

            out_msg = output_dmg_triage(boss_hp, &triaged_dmg_copy, out_msg);
        }

        avg_new_dmg_per_hit *= 10000.0;
        avg_new_dmg_per_hit = avg_new_dmg_per_hit.round();
        avg_new_dmg_per_hit /= 10000.0;
        out_msg.push_str(&format!(
            "\nAdding {} new hits: *{}*",
            &new_hits_needed, &avg_new_dmg_per_hit
        ));

        for _ in 0..(new_hits_needed as usize) {
            triaged_dmg.push(avg_new_dmg_per_hit);
        }
        triaged_dmg.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap());

        if calc_cot(boss_hp, &triaged_dmg) == 90 {
            println!("new triaged dmg: {:?}", triaged_dmg);
            out_msg.push_str("\nNew Triaged dmg: ");
            out_msg = output_dmg_triage(boss_hp, &triaged_dmg, out_msg);
        }
    } else {
        // Calculate hits replacement.
        let max_replacement = 2;
        let mut hit_replacement = false;
        let mut replacement_needs: Vec<(usize, f64)> = Vec::new();
        for i in 1..=max_replacement {
            if i <= new_hits_needed as usize {
                continue;
            }
            let mut triaged_dmg_copy = triaged_dmg.clone();
            println!(
                "new hits needed: {} and i is {}",
                new_hits_needed as usize, i
            );
            println!("Replacing {} hits in calcs", i - new_hits_needed as usize);
            for _ in (0)..(i - new_hits_needed as usize) {
                triaged_dmg_copy.pop();
            }
            let total_dmg_kept = triaged_dmg_copy.iter().sum();
            println!(
                "Boss_hp_left: {}, dmg_not_replaced: {}, hits_needed: {}",
                &boss_hp, &total_dmg_kept, i as f64
            );
            println!(
                "Answer was: {}",
                avg_dmg_for_given_hits(boss_hp, total_dmg_kept, i as f64)
            );
            let mut avg_new_dmg_per_hit = avg_dmg_for_given_hits(boss_hp, total_dmg_kept, i as f64);
            let boss_hp_left = boss_hp - triaged_dmg_copy.iter().sum::<f64>();
            let potential_smaller_avg =
                calculate_best_new_hits_needed(&triaged_dmg_copy, boss_hp, boss_hp_left, i as f64);
            if potential_smaller_avg < avg_new_dmg_per_hit {
                avg_new_dmg_per_hit = potential_smaller_avg;
            }

            avg_new_dmg_per_hit *= 10000.0;
            avg_new_dmg_per_hit = avg_new_dmg_per_hit.round();
            avg_new_dmg_per_hit /= 10000.0;
            replacement_needs.push((i, avg_new_dmg_per_hit));
        }
        println!("Replacement needs: {:?}", &replacement_needs);

        for (i, n) in replacement_needs {
            let mut triaged_dmg_copy = triaged_dmg.clone();
            println!("Replacing {} hits in output", i - new_hits_needed as usize);
            for _ in (0)..(i - new_hits_needed as usize) {
                triaged_dmg_copy.pop();
            }
            for _ in 0..i {
                triaged_dmg_copy.push(n);
            }
            triaged_dmg_copy.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap());
            if calc_cot(boss_hp, &triaged_dmg_copy) != 90 {
                // Rong is stupid
                continue;
            }
            if new_hits_needed > 0.0 {
                out_msg.push_str(&format!(
                    "\n\nReplacing {} low hit and adding {} new hit, results in:\n",
                    i - 1,
                    new_hits_needed
                ));
            } else {
                out_msg.push_str(&format!("\n\nReplacing {} low hit, results in:\n", i));
            }

            out_msg = output_dmg_triage(boss_hp, &triaged_dmg_copy, out_msg);
        }
    }

    if cot_reached && cot < 89 {
        out_msg.push_str(&format!("\n\nTo reach {}s COT:", cot + 1));
        let mut triaged_dmg_copy = triaged_dmg.clone();
        let lowest_dmg = triaged_dmg_copy.pop().unwrap_or(0.0);
        let required_dmg =
            reach_target_cot(boss_hp - triaged_dmg_copy.iter().sum::<f64>(), cot + 1);
        out_msg.push_str(&format!(
            "\nReplacing the last hit: need an additional {:.4} dmg, making it **{}**.",
            required_dmg - lowest_dmg,
            required_dmg
        ));

        if triaged_dmg.len() > 1 {
            let len = triaged_dmg.len();
            // Calculate desired hp remaining.
            let hp_remaining_before_last_2 =
                boss_hp - (triaged_dmg[..(len - 2)].iter().sum::<f64>());
            // last_hit_dmg - last_hit_dmg(cot_target-10)/90
            let want_hp_remaining = lowest_dmg - lowest_dmg * (cot - 10) as f64 / 90.0;
            let mut second_to_last_hit = hp_remaining_before_last_2 - want_hp_remaining;
            second_to_last_hit *= 10000.0;
            second_to_last_hit = second_to_last_hit.ceil();
            second_to_last_hit /= 10000.0;
            out_msg.push_str(&format!(
                "\nReplacing the 2nd to last hit: need an additional {:.4} dmg, making it **{}**.",
                second_to_last_hit - triaged_dmg[len - 2],
                second_to_last_hit
            ));
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

    let mut boss_hp_left = match args.single::<f64>() {
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
    let mut cot_target = args.single::<u64>()?;
    let mut dmg_inputs: Vec<f64> = Vec::new();

    if cot_target > 90 {
        cot_target = 90;
    }
    let max_num_hits = 3;
    let mut out_msg = format!("**Target COT: {}s**", cot_target);
    if args.is_empty() {
        out_msg = required_dmg_target_cot(out_msg, boss_hp_left, cot_target, max_num_hits);
    } else {
        for arg in args.iter::<f64>() {
            // Zero troubles, zero worries.
            let new_dmg = arg.unwrap_or(0.0);
            if new_dmg == 0.0 {
                continue;
            }
            dmg_inputs.push(new_dmg);
        }
        if dmg_inputs.is_empty() {
            msg.reply(ctx, "I don't recognize any dmg numbers you sent")
                .await?;
            return Ok(());
        }

        dmg_inputs.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap());

        out_msg.push_str("\nTriaged damage: ");
        let mut triaged_count = 0;
        for dmg in &dmg_inputs {
            if dmg > &boss_hp_left {
                if triaged_count != 0 {
                    out_msg.push_str("-> ");
                }
                out_msg.push_str(&format!("**{}**", &dmg));

                let mut cot = (((dmg - boss_hp_left) / dmg * 90.0) + 10.0).ceil() as u64;
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
                if triaged_count + 1 >= dmg_inputs.len() {
                    out_msg.push_str(&format!("**{}**", &dmg));
                } else {
                    out_msg.push_str(&format!("{} ", &dmg));
                }
                triaged_count += 1;
                boss_hp_left -= dmg;
            }
        }

        if triaged_count == dmg_inputs.len() {
            out_msg.push_str(&format!(
                "\nNot enough total dmg to kill. Rong recommends:\n\
                 Remaining boss HP - **{:.3}**",
                &boss_hp_left
            ));
            out_msg = required_dmg_target_cot(out_msg, boss_hp_left, cot_target, max_num_hits);
        }
    }

    msg.reply(ctx, out_msg).await?;

    Ok(())
}
