/*!
Here lies Rong, reborn, better than before.
:KannaBurn:
*/

mod utils;
mod data;
mod checks;
mod commands;
mod listeners;

use commands::{
    cb::status::*,
    atc::{
        status::*,
        summary::*,
        start::*,
        end::*
    },
    general::{
        debug::*,
        general::*,
    },
    help::help::*,
};

use listeners::{
    hooks::general::*,
    handlers::basic::*,
};

use std::{
    collections::{HashMap, HashSet},
    env,
    error::Error,
    sync::Arc,
};

use serenity::prelude::*;
use serenity::{
    client::bridge::gateway::GatewayIntents,
    framework::standard::{
        buckets::LimitedFor,
        macros::group,
        StandardFramework,
    },
    http::Http,
};

use sqlx::postgres::PgPoolOptions;

use crate::data::*;

#[group]
#[commands(say, latency, debug_args)]
struct General;

// Rong ATC (Air Traffic Control)
#[group]
#[only_in(guilds)]
#[prefixes("atc", "flight")]
#[description = "These commands helps us to know the status of pilots, current flights, and logins."]
#[summary = "Rong ATC (Air Traffic Control)"]
#[commands(flight_status, flight_summary, flight_end, flight_start)]
#[default_command(flight_status)]
struct ATC;

// Rong Clan Battle utilities
#[group]
#[only_in(guilds)]
#[prefixes("cb")]
#[description = "These commands help with clan battle utilities, status, hit submission, etc."]
#[summary = "Rong Clan Battle utilities."]
#[commands(cb_status)]
struct CB;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let token = env::var("DISCORD_TOKEN").expect("Expect DISCORD_TOKEN in environment.");
    let dburl = env::var("DATABASE_URL").expect("Expect DATABASE_URL in environment.");

    let http = Http::new_with_token(&token);

    // We will fetch your bot's owners and id
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c
            .with_whitespace(true)
            .on_mention(Some(bot_id))
            .prefix(">")
            .delimiters(vec![" ", ", ", ","])
            .owners(owners))

        .before(before)
        .after(after)
        .unrecognised_command(unknown_command)
        .normal_message(normal_message)
        .on_dispatch_error(dispatch_error)
        // Can't be used more than once per 5 seconds:
        .bucket("general", |b| b.delay(1)).await
        .bucket("atc", |b| b.delay(5)).await
        // Can't be used more than 2 times per 30 seconds, with a 5 second delay applying per channel.
        // Optionally `await_ratelimits` will delay until the command can be executed instead of
        // cancelling the command invocation.
        .bucket("complicated", |b| b.limit(2).time_span(30).delay(5)
        // The target each bucket will apply to.
        .limit_for(LimitedFor::Channel)
        // The maximum amount of command invocations that can be delayed per target.
        // Setting this to 0 (default) will never await/delay commands and cancel the invocation.
        .await_ratelimits(1)
        // A function to call when a rate limit leads to a delay.
        .delay_action(delay_action)).await
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        .group(&ATC_GROUP)
        .group(&CB_GROUP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .intents(GatewayIntents::all())
        .type_map_insert::<CommandCounter>(HashMap::default())
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        let pgpool = PgPoolOptions::new().max_connections(20).connect(&dburl).await?;
        data.insert::<DatabasePool>(pgpool);
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    if let Err(why) = client.start_autosharded().await {
        println!("Client error: {:?}", why);
    }

    Ok(())
}

