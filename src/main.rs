/*!
Here lies Rong, reborn, better than before.
:KannaBurn:
*/

mod checks;
mod commands;
mod data;
mod error;
mod listeners;
mod utils;

use commands::{
    admin::help_yon::*,
    atc::{
        alert::*, award::*, call_sign::*, check_fc::*, crash::*, end::*, fc::*, hounds::*,
        motto::*, note::*, start::*, status::*, summary::*, unfc::*,
    },
    cb::{carry_over_calc::*, status::*},
    config::{set_channel::*, superadmin::*},
    fun::{
        kyouka::*, ping_add::*, ping_rarity_update::*, ping_remove::*, ping_roll::*, ping_table::*,
        shadow_ping::*,
    },
    general::{debug::*, general::*, threads::*},
    help::help::*,
};

use listeners::{handlers::basic::*, hooks::general::*};

use std::{
    collections::{HashMap, HashSet},
    env,
    error::Error,
    sync::Arc,
};

use serenity::{
    framework::standard::{buckets::LimitedFor, macros::group, StandardFramework},
    model::gateway::GatewayIntents,
    prelude::*,
};

use sqlx::postgres::PgPoolOptions;

use crate::data::*;

#[group]
#[commands(
    say,
    latency,
    debug_args,
    make_threads,
    force_quit,
    check_fc,
    add_superadmin,
    remove_superadmin,
    cot_calc_time,
    cot_calc_dmg,
    cot_old_calc_time,
    cot_old_calc_dmg,
    kyouka,
    shadow_ping,
    undo_force_quit
)]
struct General;

// Admin group for admin stuffs
#[group]
#[description = "Admins only please"]
#[summary = "Admin secret stuffs"]
#[commands(help_yon)]
struct Admin;

// Rong ATC (Air Traffic Control)
#[group]
#[only_in(guilds)]
#[prefixes("atc", "flight")]
#[description = "These commands helps us to know the status of pilots, current flights, and logins."]
#[summary = "Rong ATC (Air Traffic Control)"]
#[commands(
    flight_status,
    flight_summary,
    flight_end,
    flight_crash,
    flight_start,
    set_alert_channel,
    set_motto,
    set_note,
    set_call_sign,
    atc_award,
    check_fc,
    force_quit,
    undo_force_quit,
    atc_hounds
)]
struct ATC;

// Rong Clan Battle utilities
#[group]
#[only_in(guilds)]
#[prefixes("cb")]
#[description = "These commands help with clan battle utilities, status, hit submission, etc."]
#[summary = "Rong Clan Battle utilities."]
#[commands(
    cb_status,
    cot_calc_time,
    cot_calc_dmg,
    cot_old_calc_time,
    cot_old_calc_dmg
)]
//#[default_command(cb_status)]
struct CB;

// Rong Clan Battle utilities
#[group]
#[only_in(guilds)]
#[prefixes("ping")]
#[description = "These commands make up the Rong ping gacha!"]
#[summary = ">ping to start pinging"]
#[commands(
    ping_roll,
    ping_rarity_update,
    ping_drop_table,
    ping_add_loot,
    ping_remove_loot
)]
#[default_command(ping_roll)]
struct PING;

// Rong configs
#[group]
#[only_in(guilds)]
#[prefixes("config")]
#[description = "These commands help configure rong's internal workings."]
#[summary = "Configure Rong's inner workings."]
#[commands(set_channel)]
struct Config;

// Rong super fun
// #[group]
// #[only_in(guilds)]
// #[summary = "KyoukaSmile"]
// struct Kyouka;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let token = env::var("DISCORD_TOKEN").expect("Expect DISCORD_TOKEN in environment.");
    let dburl = env::var("DATABASE_URL").expect("Expect DATABASE_URL in environment.");

    let http = serenity::http::client::Http::new(&token);

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
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| {
            c.with_whitespace(true)
                .on_mention(Some(bot_id))
                .prefix(">")
                .delimiters(vec![", ", " ", ","])
                .case_insensitivity(true)
                .owners(owners)
        })
        .before(before)
        .after(after)
        .unrecognised_command(unknown_command)
        .normal_message(normal_message)
        .on_dispatch_error(dispatch_error)
        .bucket("general", |b| b.delay(3))
        .await
        .bucket("atc", |b| b.delay(3))
        .await
        // Can't be used more than 2 times per 30 seconds, with a 5 second delay applying per channel.
        // Optionally `await_ratelimits` will delay until the command can be executed instead of
        // cancelling the command invocation.
        .bucket("cb", |b| {
            b.limit(5)
                .time_span(10)
                .delay(5)
                // The target each bucket will apply to.
                .limit_for(LimitedFor::Channel)
                // The maximum amount of command invocations that can be delayed per target.
                // Setting this to 0 (default) will never await/delay commands and cancel the invocation.
                .await_ratelimits(1)
                // A function to call when a rate limit leads to a delay.
                .delay_action(delay_action)
        })
        .await
        .bucket("config", |b| b.delay(3))
        .await
        .bucket("kyouka", |b| {
            b.limit(1)
                .time_span(1800)
                .limit_for(LimitedFor::User)
                .delay_action(kyouka_delay)
        })
        .await
        .bucket("atc_hounds", |b| {
            b.limit(1).time_span(30).limit_for(LimitedFor::Channel)
        })
        .await
        .help(&MY_HELP)
        .group(&ADMIN_GROUP)
        .group(&GENERAL_GROUP)
        .group(&ATC_GROUP)
        .group(&CB_GROUP)
        .group(&PING_GROUP)
        .group(&CONFIG_GROUP);

    let intents =
        GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .intents(GatewayIntents::all())
        .type_map_insert::<CommandCounter>(HashMap::default())
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        let pgpool = PgPoolOptions::new()
            .max_connections(50)
            .connect(&dburl)
            .await?;
        println!("Rong database is connected.");
        // Run migrations, which updates the database's schema to the latest version.
        sqlx::migrate!("./migrations")
            .run(&pgpool)
            .await
            .expect("Couldn't run database migrations");
        println!("Database migrations complete.");
        data.insert::<DatabasePool>(pgpool);
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }
    if let Err(why) = client.start_autosharded().await {
        println!("Client error: {:?}", why);
    }

    Ok(())
}
