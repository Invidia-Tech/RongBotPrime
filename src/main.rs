/*!
Here lies Rong, reborn, better than before.
:KannaBurn:
*/

mod utils;
mod data;

use std::{
    collections::{HashMap, HashSet},
    env,
    error::Error,
    fmt::Write,
    sync::Arc,
};

use serenity::prelude::*;
use serenity::{
    async_trait,
    client::bridge::gateway::{GatewayIntents, ShardId, ShardManager},
    framework::standard::{
        buckets::{LimitedFor, RevertBucket},
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
        channel::{Channel, Message},
        gateway::Ready,
        id::UserId,
        permissions::Permissions,
    },
    utils::{content_safe, ContentSafeOptions, MessageBuilder},
};

use tokio::sync::Mutex;

use sqlx::postgres::PgPoolOptions;

use crate::data::DatabasePool;

// This allows data to be shared across the shard, so that all frameworks
// and handlers can see the same data as long as they have a copy of the
// `data` Arc. Arc is an atomic reference counter btw. It's a thread safe
// way to share immutable data.
struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct CommandCounter;

impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[group]
#[commands(/*about, am_i_admin, say, commands, ping, */latency,
    debug_args/*, upper_command*/)]
struct General;

#[group]
// Sets multiple prefixes for a group.
// This requires us to call commands in this group
// via `~emoji` (or `~em`) instead of just `~`.
#[prefixes("emoji", "em")]
// Set a description to appear if a user wants to display a single group
// e.g. via help using the group-name or one of its prefixes.
#[description = "A group with commands providing an emoji as response."]
// Summary only appears when listing multiple groups.
#[summary = "Do emoji fun!"]
// Sets a command that will be executed if only a group-prefix was passed.
#[default_command(bird)]
#[commands(cat, dog)]
struct Emoji;

#[group]
// Sets a single prefix for this group.
// So one has to call commands in this group
// via `~math` instead of just `~`.
#[prefix = "math"]
#[commands(multiply)]
struct Math;

#[group]
#[owners_only]
// Limit all commands to be guild-restricted.
#[only_in(guilds)]
// Summary only appears when listing multiple groups.
#[summary = "Commands for server owners"]
#[commands(slow_mode)]
struct Owner;

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

// The framework provides two built-in help commands for you to use.
// But you can also make your own customized help command that forwards
// to the behaviour of either of them.
#[help]
// This replaces the information that a user can pass
// a command-name as argument to gain specific information about it.
#[individual_command_tip = "Welcome to Rong Prime!\n\nTo learn more about a command, run help with the command's name."]
// Some arguments require a `{}` in order to replace it with contextual information.
// In this case our `{}` refers to a command's name.
#[command_not_found_text = "Could not find: `{}`."]
// Define the maximum Levenshtein-distance between a searched command-name
// and commands. If the distance is lower than or equal the set distance,
// it will be displayed as a suggestion.
// Setting the distance to 0 will disable suggestions.
#[max_levenshtein_distance(3)]
// When you use sub-groups, Serenity will use the `indention_prefix` to indicate
// how deeply an item is indented.
// The default value is "-", it will be changed to "+".
#[indention_prefix = "+"]
// On another note, you can set up the help-menu-filter-behaviour.
// Here are all possible settings shown on all possible options.
// First case is if a user lacks permissions for a command, we can hide the command.
#[lacking_permissions = "Hide"]
// If the user is nothing but lacking a certain role, we just display it hence our variant is `Nothing`.
#[lacking_role = "Nothing"]
// The last `enum`-variant is `Strike`, which ~~strikes~~ a command.
#[wrong_channel = "Strike"]
// Serenity will automatically analyse and generate a hint/tip explaining the possible
// cases of ~~strikethrough-commands~~, but only if
// `strikethrough_commands_tip_in_{dm, guild}` aren't specified.
// If you pass in a value, it will be displayed instead.
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

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

// You can construct a hook without the use of a macro, too.
// This requires some boilerplate though and the following additional import.
use serenity::{futures::future::BoxFuture, FutureExt};
fn _dispatch_error_no_macro<'fut>(
    ctx: &'fut mut Context,
    msg: &'fut Message,
    error: DispatchError,
) -> BoxFuture<'fut, ()> {
    async move {
        if let DispatchError::Ratelimited(info) = error {
            if info.is_first_try {
                let _ = msg
                .channel_id
                .say(&ctx.http, &format!("Try this again in {} seconds.", info.as_secs()))
                .await;
            }
        };
    }
    .boxed()
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
            .prefix("~")
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

// Commands can be created via the attribute `#[command]` macro.
#[command]
// Options are passed via subsequent attributes.
// Make this command use the "complicated" bucket.
#[bucket = "complicated"]
async fn commands(ctx: &Context, msg: &Message) -> CommandResult {
    let mut contents = "Commands used:\n".to_string();

    let data = ctx.data.read().await;
    let counter = data.get::<CommandCounter>().expect("Expected CommandCounter in TypeMap.");

    for (k, v) in counter {
        writeln!(contents, "- {name}: {amount}", name = k, amount = v)?;
    }

    msg.channel_id.say(&ctx.http, &contents).await?;

    Ok(())
}

// Repeats what the user passed as argument but ensures that user and role
// mentions are replaced with a safe textual alternative.
// In this example channel mentions are excluded via the `ContentSafeOptions`.
#[command]
async fn say(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let settings = if let Some(guild_id) = msg.guild_id {
        // By default roles, users, and channel mentions are cleaned.
        ContentSafeOptions::default()
            // We do not want to clean channal mentions as they
            // do not ping users.
            .clean_channel(false)
            // If it's a guild channel, we want mentioned users to be displayed
            // as their display name.
            .display_as_member_from(guild_id)
    } else {
        ContentSafeOptions::default().clean_channel(false).clean_role(false)
    };

    let content = content_safe(&ctx.cache, &args.rest(), &settings).await;

    msg.channel_id.say(&ctx.http, &content).await?;

    Ok(())
}

// A function which acts as a "check", to determine whether to call a command.
//
// In this case, this command checks to ensure you are the owner of the message
// in order for the command to be executed. If the check fails, the command is
// not called.
#[check]
#[name = "Owner"]
async fn owner_check(
    _: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> Result<(), Reason> {
    // Replace 7 with your ID to make this check pass.
    //
    // 1. If you want to pass a reason alongside failure you can do:
    // `Reason::User("Lacked admin permission.".to_string())`,
    //
    // 2. If you want to mark it as something you want to log only:
    // `Reason::Log("User lacked admin permission.".to_string())`,
    //
    // 3. If the check's failure origin is unknown you can mark it as such:
    // `Reason::Unknown`
    //
    // 4. If you want log for your system and for the user, use:
    // `Reason::UserAndLog { user, log }`
    if msg.author.id != 162034086066520064 {
        return Err(Reason::User("Lacked owner permission".to_string()));
    }

    Ok(())
}

#[command]
#[checks(Owner)]
async fn debug_args(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id.say(&ctx.http, &format!("Arguments: {:?}", args.rest())).await?;

    Ok(())
}

#[command]
// Limits the usage of this command to roles named:
#[allowed_roles("mods", "ultimate neko")]
async fn about_role(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let potential_role_name = args.rest();

    if let Some(guild) = msg.guild(&ctx.cache).await {
        // `role_by_name()` allows us to attempt attaining a reference to a role
        // via its name.
        if let Some(role) = guild.role_by_name(potential_role_name) {
            if let Err(why) = msg.channel_id.say(&ctx.http, &format!("Role-ID: {}", role.id)).await
            {
                println!("Error sending message: {:?}", why);
            }

            return Ok(());
        }
    }

    msg.channel_id
        .say(&ctx.http, format!("Could not find role named: {:?}", potential_role_name))
        .await?;

    Ok(())
}

#[command]
// Lets us also call `~math *` instead of just `~math multiply`.
#[aliases("*")]
async fn multiply(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    println!("{:?}", args.current());
    let first = args.single::<f64>()?;
    let second = args.single::<f64>()?;

    let res = first * second;

    msg.channel_id.say(&ctx.http, &res.to_string()).await?;

    Ok(())
}

#[command]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "This is a small test-bot! : )").await?;

    Ok(())
}

#[command]
async fn latency(ctx: &Context, msg: &Message) -> CommandResult {
    // The shard manager is an interface for mutating, stopping, restarting, and
    // retrieving information about shards.
    let data = ctx.data.read().await;

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            msg.reply(ctx, "There was a problem getting the shard manager").await?;

            return Ok(());
        },
    };

    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;

    // Shards are backed by a "shard runner" responsible for processing events
    // over the shard, so we'll get the information about the shard runner for
    // the shard this command was sent over.
    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            msg.reply(ctx, "No shard found").await?;

            return Ok(());
        },
    };

    println!("Latency is: {:?}", runner.latency);
    match runner.latency {
        Some(dur) => {
            msg.reply(ctx, &format!("This shard's latency is {:?}", dur)).await?;
        },
        None => {msg.reply(ctx, "Error retriving latency for this shard. Or it's not ready yet.").await?;},
    };

    Ok(())
}

#[command]
// Limit command usage to guilds.
#[only_in(guilds)]
// #[checks(Owner)]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong! : )").await?;

    Ok(())
}

#[command]
// Adds multiple aliases
#[aliases("kitty", "neko")]
// Make this command use the "emoji" bucket.
#[bucket = "emoji"]
// Allow only administrators to call this:
#[required_permissions("ADMINISTRATOR")]
async fn cat(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, ":cat:").await?;

    // We can return one ticket to the bucket undoing the ratelimit.
    Err(RevertBucket.into())
}

#[command]
#[description = "Sends an emoji with a dog."]
#[bucket = "emoji"]
async fn dog(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, ":dog:").await?;

    Ok(())
}

#[command]
async fn bird(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let say_content = if args.is_empty() {
        ":bird: can find animals for you.".to_string()
    } else {
        format!(":bird: could not find animal named: `{}`.", args.rest())
    };

    msg.channel_id.say(&ctx.http, say_content).await?;

    Ok(())
}

// We could also use
// #[required_permissions(ADMINISTRATOR)]
// but that would not let us reply when it fails.
#[command]
async fn am_i_admin(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    if let Some(member) = &msg.member {
        for role in &member.roles {
            if role
                .to_role_cached(&ctx.cache)
                .await
                .map_or(false, |r| r.has_permission(Permissions::ADMINISTRATOR))
            {
                msg.channel_id.say(&ctx.http, "Yes, you are.").await?;

                return Ok(());
            }
        }
    }

    msg.channel_id.say(&ctx.http, "No, you are not.").await?;

    Ok(())
}

#[command]
async fn slow_mode(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let say_content = if let Ok(slow_mode_rate_seconds) = args.single::<u64>() {
        if let Err(why) =
            msg.channel_id.edit(&ctx.http, |c| c.slow_mode_rate(slow_mode_rate_seconds)).await
        {
            println!("Error setting channel's slow mode rate: {:?}", why);

            format!("Failed to set slow mode to `{}` seconds.", slow_mode_rate_seconds)
        } else {
            format!("Successfully set slow mode rate to `{}` seconds.", slow_mode_rate_seconds)
        }
    } else if let Some(Channel::Guild(channel)) = msg.channel_id.to_channel_cached(&ctx.cache).await
    {
        format!("Current slow mode rate is `{}` seconds.", channel.slow_mode_rate.unwrap_or(0))
    } else {
        "Failed to find channel in cache.".to_string()
    };

    msg.channel_id.say(&ctx.http, say_content).await?;

    Ok(())
}

// =========================== CB COMMANDS ================================
#[command("status")]
#[description("This shows the status of the current active clan battle.")]
async fn cb_status(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    // The message builder allows for creating a message by
    // mentioning users dynamically, pushing "safe" versions of
    // content (such as bolding normalized content), displaying
    // emojis, and more.
    let response = MessageBuilder::new()
            .push("User ")
            .push_bold_safe(&msg.author.name)
            .push(" used the 'cb status' command in the ")
            .mention(&msg.channel_id.to_channel_cached(&ctx.cache).await.unwrap())
            .push(" channel")
            .build();

    if let Err(why) = msg.channel_id.say(&ctx.http, &response).await {
        println!("Error sending message: {:?}", why);
    }

    let msg = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.content("Current CB: Test CB 13 (mocked)")
                .embed(|e| {
                    e.title("Clan: JEthereal")
                        .description("Current day: 2")
                        .image("attachment://KyoukaSmile.jpg")
                        .fields(vec![
                            ("Hits today:", "15", true),
                            ("Bosses killed today:", "3", true),
                            ("Dmg done today:", "89,204,302", true),
                            ("Overall RV%:", "98.3%", false)
                        ])
                        .field("Current boss:", "Kyouka", false)
                        .footer(|f| f.text("Days since last int: 0"))
                        // Add a timestamp for the current time
                        // This also accepts a rfc3339 Timestamp
                        .timestamp(chrono::Utc::now().to_rfc3339())
                })
                .add_file("./KyoukaSmile.jpg")
        })
        .await;

    if let Err(why) = msg {
        println!("Error sending message: {:?}", why);
    }
    Ok(())
}

// =========================== FLIGHT COMMANDS =====================================

#[command("status")]
#[description("This shows the status of current flights.")]
async fn flight_status(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Current flights: None").await?;
    // The message builder allows for creating a message by
    // mentioning users dynamically, pushing "safe" versions of
    // content (such as bolding normalized content), displaying
    // emojis, and more.
    let response = MessageBuilder::new()
            .push("User ")
            .push_bold_safe(&msg.author.name)
            .push(" used the 'atc status' command in the ")
            .mention(&msg.channel_id.to_channel_cached(&ctx.cache).await.unwrap())
            .push(" channel")
            .build();

    if let Err(why) = msg.channel_id.say(&ctx.http, &response).await {
        println!("Error sending message: {:?}", why);
    }

    let msg = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.content("Rong ATC (Air Traffic Control) Status")
                .embed(|e| {
                    e.title("Current Flights")
                        .description("These are the recent running/landed flights.")
                        //.image("attachment://KyoukaSmile.jpg")
                        .fields(vec![
                            ("🛫 __Flight DB 14002__", "Pilot: Dabomstew", false),
                            ("Current Status:", "**In Progress**", true),
                            ("Duration:", "37 Minutes", true),
                        ])
                        .fields(vec![
                            ("🛬 __Flight BN 14002__", "Pilot: Boon", false),
                            ("Current Status:", "**Landed**", true),
                            ("Duration:", "09 Minutes", true),
                        ])
                        .fields(vec![
                            ("💥 __Flight RG 14001__", "Pilot: Ring", false),
                            ("Current Status:", "**Crashed**", true),
                            ("Duration:", "23 Minutes", true),
                        ])
                        .field("Overall Flight Status", "Flights Today: 2", false)
                        .footer(|f| f.text("Days since last int: 0"))
                        // Add a timestamp for the current time
                        // This also accepts a rfc3339 Timestamp
                        .timestamp(chrono::Utc::now().to_rfc3339())
                })
                //.add_file("./KyoukaSmile.jpg")
        })
        .await;

    if let Err(why) = msg {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}

#[command("summary")]
#[description("Full summary of current flights.")]
#[bucket = "atc"]
async fn flight_summary(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Current flights: None").await?;
    // The message builder allows for creating a message by
    // mentioning users dynamically, pushing "safe" versions of
    // content (such as bolding normalized content), displaying
    // emojis, and more.
    let response = MessageBuilder::new()
            .push("User ")
            .push_bold_safe(&msg.author.name)
            .push(" used the 'atc status' command in the ")
            .mention(&msg.channel_id.to_channel_cached(&ctx.cache).await.unwrap())
            .push(" channel")
            .build();

    if let Err(why) = msg.channel_id.say(&ctx.http, &response).await {
        println!("Error sending message: {:?}", why);
    }

    let msg = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.content("Rong ATC (Air Traffic Control) Status")
                .embed(|e| {
                    e.title("Current Flights")
                        .description("These are the recent running/landed flights.")
                        //.image("attachment://KyoukaSmile.jpg")
                        .fields(vec![
                            ("🛫 __Flight DB 14002__", "Pilot: Dabomstew", false),
                            ("Current Status:", "**In Progress**", true),
                            ("Duration:", "37 Minutes", true),
                        ])
                        .fields(vec![
                            ("🛬 __Flight BN 14002__", "Pilot: Boon", false),
                            ("Current Status:", "**Landed**", true),
                            ("Duration:", "09 Minutes", true),
                        ])
                        .fields(vec![
                            ("💥 __Flight RG 14001__", "Pilot: Ring", false),
                            ("Current Status:", "**Crashed**", true),
                            ("Duration:", "23 Minutes", true),
                        ])
                        .field("Overall Flight Status", "Flights Today: 2", false)
                        .footer(|f| f.text("Days since last int: 0"))
                        // Add a timestamp for the current time
                        // This also accepts a rfc3339 Timestamp
                        .timestamp(chrono::Utc::now().to_rfc3339())
                })
                //.add_file("./KyoukaSmile.jpg")
        })
        .await;

    if let Err(why) = msg {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}

#[command("start")]
#[description("Start a flight.")]
#[bucket = "atc"]
async fn flight_start(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {

    Ok(())
}

#[command("end")]
#[description("End a flight.")]
#[bucket = "atc"]
async fn flight_end(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {

    Ok(())
}

// A command can have sub-commands, just like in command lines tools.
// Imagine `cargo help` and `cargo help run`.
#[command("upper")]
#[sub_commands(sub)]
async fn upper_command(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "This is the main function!").await?;

    Ok(())
}

// This will only be called if preceded by the `upper`-command.
#[command]
#[aliases("sub-command", "secret")]
#[description("This is `upper`'s sub-command.")]
async fn sub(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "This is a sub function!").await?;

    Ok(())
}
