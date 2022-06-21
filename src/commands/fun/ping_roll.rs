use std::collections::HashMap;

use serenity::{
    client::{
        bridge::gateway::ShardId,
        Context,
    },
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    model::channel::Message,
    utils::{
        content_safe,
        ContentSafeOptions,
    },
};

use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

use crate::data::DatabasePool;

#[command("ping_roll")]
// Limit command usage to guilds.
#[aliases("roll")]
#[only_in(guilds)]
// #[checks(Owner)]
async fn ping_roll(ctx: &Context, msg: &Message) -> CommandResult {
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();

    let guild_id = match msg.guild_id {
        None => {
            msg.reply(ctx, "Unable to obtain guild id.").await?;
            return Ok(());
        }
        Some(id) => id.0,
    };

    let weights = match sqlx::query!(
        "SELECT rank, weight FROM rongbot.ping_rarity
         WHERE server=$1
         ORDER BY rank;",
        &guild_id.to_string()
    )
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => {
            if rows.is_empty() {
                msg.reply(ctx, "This guild does not have a drop table configured!")
                    .await?;
                return Ok(());
            }
            rows
        }
        Err(_e) => {
            msg.reply(ctx, "There was an error in getting the drop table!")
                .await?;
            return Ok(());
        }
    };

    // Select the correct rank based on weight.
    let weight_total: i32 = weights.iter().clone().map(|x| x.weight).sum();
    // let mut rng = thread_rng();
    let mut rng = ChaCha20Rng::from_entropy();

    let rarity_roll: i32 = rng.gen_range(1..=weight_total);
    println!("Rolled: {}", rarity_roll);
    let mut running_total = 0;
    let mut rank_roll = 1;
    for w in &weights {
        running_total += w.weight;
        if rarity_roll <= running_total {
            rank_roll = w.rank;
            break;
        }
    }

    let rarity_text: Vec<&str> = vec!["[N]", "[R]", "[SR]", "[SSR]", "[UR]"];
    let ping_list = match sqlx::query!(
        "SELECT user_id, weight, nickname
         FROM rongbot.ping_droptable dt
         WHERE server=$1 AND rarity_rank=$2;",
        &guild_id.to_string(),
        &rank_roll
    )
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => {
            if rows.is_empty() {
                msg.reply(
                    ctx,
                    format!(
                        "You rolled a {} but I can't find anyone to ping... \
                         <:KasumiDerp:988300507091189760>",
                        &rarity_text[(rank_roll - 1) as usize]
                    ),
                )
                .await?;
                return Ok(());
            }
            rows
        }
        Err(_e) => {
            msg.reply(ctx, "There was an error in getting the drop table!")
                .await?;
            return Ok(());
        }
    };

    let weight_total: i32 = ping_list.iter().clone().map(|x| x.weight).sum();
    let ping_roll: i32 = rng.gen_range(1..=weight_total);
    let mut running_total = 0;
    let mut chosen_ping = "".to_string();
    for w in ping_list {
        running_total += w.weight;
        if ping_roll <= running_total {
            chosen_ping = w.user_id;
            break;
        }
    }

    let rarity_text: HashMap<i32, &str> = HashMap::from([
        (1, "[N] Sure, I'll ping you! <@{}>"),
        (2, "[R] A rare <@{}> ping!"),
        (3, "[SR] Wow! A super rare <@{}> ping!!"),
        (
            4,
            "[SSR] Incredible!! A super duper rare <@{}> ping! Congrats.",
        ),
        (
            5,
            "[UR] No way! An ultra rare <@{}> ping... Your gacha luck has been used up today.",
        ),
    ]);

    msg.reply(ctx, rarity_text[&rank_roll].replace("{}", &chosen_ping))
        .await?;

    Ok(())
}
