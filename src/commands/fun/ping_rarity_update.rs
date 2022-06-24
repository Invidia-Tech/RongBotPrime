

use serenity::{
    client::{
        Context,
    },
    framework::standard::{
        macros::command,
        Args,
        CommandResult,
    },
    model::channel::Message,
};
use sqlx::{
    Postgres,
    QueryBuilder,
};

use crate::{
    checks::rong_admin::*,
    data::DatabasePool,
};

#[command("ping_rarity_update")]
// Limit command usage to guilds.
#[only_in(guilds)]
#[description(
    "Allows you to set weights on each of the ping rarity brackets. \
     Please note that the rarity of each bracket is r/total %. For \
     example, if your total weight is 100, a rarity of 51 would be \
     equal to 51%.\n\
     Example: >ping rates 51 35 10 3 1"
)]
#[aliases("rates")]
#[checks(RongAdmin)]
async fn ping_rarity_update(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();
    let rarity_text: Vec<&str> = vec!["[N]", "[R]", "[SR]", "[SSR]", "[UR]"];

    if args.is_empty() {
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

        let mut out_msg = "The current rates for this server are:".to_string();
        let weights_total: i32 = weights.iter().map(|x| x.weight).sum();
        for (i, w) in (&weights).iter().enumerate() {
            out_msg.push_str(&format!(
                "\n\t- {} ({:.2}%)",
                rarity_text[i],
                (w.weight as f64 / weights_total as f64) * 100.0
            ));
        }

        msg.reply(ctx, out_msg).await?;

        return Ok(());
    }

    if args.len() != 5 {
        msg.channel_id
            .say(
                ctx,
                "Invalid command usage. Please use the help command on `ping rates`.",
            )
            .await?;
        return Ok(());
    }

    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();

    let mut rates: Vec<u32> = Vec::new();
    let mut total = 0;
    for arg in args.iter::<u32>() {
        let arg = arg.unwrap_or(0);
        total += arg;
        rates.push(arg);
    }

    let mut out_msg = "Your rates has been set to:".to_string();
    for (i, r) in (&rates).iter().enumerate() {
        out_msg.push_str(&format!(
            "\n\t- {} ({:.2}%)",
            rarity_text[i],
            (*r as f64 / total as f64) * 100.0
        ));
    }

    let guild_id = match msg.guild_id {
        None => {
            msg.reply(ctx, "Unable to obtain guild id.").await?;
            return Ok(());
        }
        Some(id) => id.0,
    };

    sqlx::query!(
        "DELETE FROM rongbot.ping_rarity
         WHERE server=$1;",
        &guild_id.to_string()
    )
    .execute(&pool)
    .await?;

    let batch_in: Vec<(u64, i32, i32)> = rates
        .iter()
        .enumerate()
        .map(|(i, r)| (guild_id, (i + 1) as i32, *r as i32))
        .collect();

    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        // Note the trailing space; most calls to `QueryBuilder` don't automatically insert
        // spaces as that might interfere with identifiers or quoted strings where exact
        // values may matter.
        "INSERT INTO rongbot.ping_rarity(server, rank, weight) ",
    );

    query_builder.push_values(
        batch_in.into_iter().take(5),
        |mut b, (server, rank, weight)| {
            b.push_bind(server.to_string())
                .push_bind(rank)
                .push_bind(weight);
        },
    );

    query_builder.build().execute(&pool).await?;

    msg.reply(ctx, out_msg).await?;
    Ok(())
}
