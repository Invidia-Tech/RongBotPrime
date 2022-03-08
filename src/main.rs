/*!
Here lies Rong, reborn, better than before.
:KannaBurn:
*/

mod utils;
mod data;
mod check;
mod commands;

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

use std::{
    collections::{HashMap, HashSet},
    env,
    error::Error,
    sync::Arc,
};

use serenity::prelude::*;
use serenity::{
    async_trait,
    client::bridge::gateway::GatewayIntents,
    framework::standard::{
        buckets::LimitedFor,
        help_commands,
        macros::{check, command, group, help, hook},
        Args,
        CommandGroup,
        CommandOptions,
        CommandResult,
        DispatchError,
        HelpOptions,
        Reason,
        StandardFramework,
    },
    http::Http,
    model::{
        channel::Message,
        gateway::Ready,
        id::UserId,
    }
};

use sqlx::postgres::PgPoolOptions;

use crate::data::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

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

#[hook]
async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!("Got command '{}' by user '{}'", command_name, msg.author.name);

    // Increment the number of times this command has been run once. If
    // the command's name does not exist in the counter, add a default
    // value of 0.
    let mut data = ctx.data.write().await;
    let counter = data.get_mut::<CommandCounter>().expect("Expected CommandCounter in TypeMap.");
    let entry = counter.entry(command_name.to_string()).or_insert(0);
    *entry += 1;

    true // if `before` returns false, command processing doesn't happen.
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => println!("Processed command '{}'", command_name),
        Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
    }
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named '{}'", unknown_command_name);
}

#[hook]
async fn normal_message(_ctx: &Context, msg: &Message) {
    println!("Normal message '{}#{}: {}'", msg.author.name, msg.author.discriminator, msg.content);
}

#[hook]
async fn delay_action(ctx: &Context, msg: &Message) {
    // You may want to handle a Discord rate limit if this fails.
    let _ = msg.react(ctx, '⏱').await;
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    if let DispatchError::Ratelimited(info) = error {
        // We notify them only once.
        if info.is_first_try {
            let _ = msg
            .channel_id
            .say(&ctx.http, &format!("Try this again in {} seconds.", info.as_secs()))
            .await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let token = env::var("DISCORD_TOKEN").expect("Expect DISCORD_TOKEN in environment.");
    let dburl = env::var("DB_URL").expect("Expect DB_URL in environment.");

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
            // In this case, if "," would be first, a message would never
            // be delimited at ", ", forcing you to trim your arguments if you
            // want to avoid whitespaces at the start of each.
            .delimiters(vec![" ", ", ", ","])
            // Sets the bot's owners. These will be used for commands that
            // are owners only.
            .owners(owners))

        // Set a function to be called prior to each command execution. This
        // provides the context of the command, the message that was received,
        // and the full name of the command that will be called.
        //
        // Avoid using this to determine whether a specific command should be
        // executed. Instead, prefer using the `#[check]` macro which
        // gives you this functionality.
        //
        // **Note**: Async closures are unstable, you may use them in your
        // application if you are fine using nightly Rust.
        // If not, we need to provide the function identifiers to the
        // hook-functions (before, after, normal, ...).
        .before(before)
        // Similar to `before`, except will be called directly _after_
        // command execution.
        .after(after)
        // Set a function that's called whenever an attempted command-call's
        // command could not be found.
        .unrecognised_command(unknown_command)
        // Set a function that's called whenever a message is not a command.
        .normal_message(normal_message)
        // Set a function that's called whenever a command's execution didn't complete for one
        // reason or another. For example, when a user has exceeded a rate-limit or a command
        // can only be performed by the bot owner.
        .on_dispatch_error(dispatch_error)
        // Can't be used more than once per 5 seconds:
        .bucket("emoji", |b| b.delay(5)).await
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
        // The `#[group]` macro generates `static` instances of the options set for the group.
        // They're made in the pattern: `#name_GROUP` for the group instance and `#name_GROUP_OPTIONS`.
        // #name is turned all uppercase
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        //.group(&EMOJI_GROUP)
        //.group(&MATH_GROUP)
        .group(&ATC_GROUP)
        .group(&CB_GROUP);
        //.group(&OWNER_GROUP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        // For this example to run properly, the "Presence Intent" and "Server Members Intent" 
        // options need to be enabled.
        // These are needed so the `required_permissions` macro works on the commands that need to
        // use it.
        // You will need to enable these 2 options on the bot application, and possibly wait up to 5
        // minutes.
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

