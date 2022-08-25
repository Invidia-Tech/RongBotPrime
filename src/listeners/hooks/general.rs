use std::time::Duration;

use humantime::format_duration;
use serenity::{
    client::Context,
    framework::standard::{macros::hook, CommandResult, DispatchError, Reason},
    model::channel::Message,
};

use crate::data::CommandCounter;

#[hook]
pub async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!(
        "Got command '{}' by user '{}'",
        command_name, msg.author.name
    );

    // Increment the number of times this command has been run once. If
    // the command's name does not exist in the counter, add a default
    // value of 0.
    let mut data = ctx.data.write().await;
    let counter = data
        .get_mut::<CommandCounter>()
        .expect("Expected CommandCounter in TypeMap.");
    let entry = counter.entry(command_name.to_string()).or_insert(0);
    *entry += 1;

    true // if `before` returns false, command processing doesn't happen.
}

#[hook]
pub async fn after(
    ctx: &Context,
    msg: &Message,
    command_name: &str,
    command_result: CommandResult,
) {
    match command_result {
        Ok(()) => println!("Processed command '{}'", command_name),
        Err(why) => {
            let _ = msg
                .channel_id
                .say(ctx, "An unknown error occured! AHHHHHHH")
                .await;
            println!("Command '{}' returned error {:?}", command_name, why);
        }
    }
}

#[hook]
pub async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named '{}'", unknown_command_name);
}

#[hook]
pub async fn normal_message(_ctx: &Context, _msg: &Message) {
    // println!(
    //     "Normal message '{}#{}: {}'",
    //     msg.author.name, msg.author.discriminator, msg.content
    // );
}

#[hook]
pub async fn delay_action(ctx: &Context, msg: &Message) {
    // You may want to handle a Discord rate limit if this fails.
    let _ = msg.react(ctx, '⏱').await;
}

#[hook]
pub async fn dispatch_error(
    ctx: &Context,
    msg: &Message,
    error: DispatchError,
    command_name: &str,
) {
    let error_response: String;
    match error {
        DispatchError::NotEnoughArguments { min, given } => {
            error_response = format!("Need {} arguments, but only got {}.", min, given);
        }
        DispatchError::TooManyArguments { max, given } => {
            error_response = format!("Max arguments allowed is {}, but got {}.", max, given);
        }
        DispatchError::Ratelimited(secs) => {
            error_response = format!(
                "Please use in moderation! Try again in {}. <:Angry:964436597909127169>",
                format_duration(Duration::from_secs(secs.as_secs()))
            );
        }
        DispatchError::CheckFailed(check, reason) => match reason {
            Reason::User(why) => error_response = format!("User error: {}. {}", check, why),
            _ => {
                error_response = "Unknown error, oh god <@162034086066520064> help! <:YuiCry:924146816201654293>".to_string();
                println!(
                    "Unhandled reason type within CheckFailed: {:?} on command {}",
                    reason, command_name
                );
            }
        },
        _ => {
            error_response =
                "Unknown error, oh god <@162034086066520064> help! <:YuiCry:924146816201654293>"
                    .to_string();
            println!(
                "Unhandled Dispatch error: {:?} on command {}",
                error, command_name
            );
        }
    }
    let _ = msg.channel_id.say(ctx, error_response).await;
}

#[hook]
pub async fn kyouka_delay(ctx: &Context, msg: &Message) {
    let _ = msg.reply(ctx, "Please use >kyouka in moderation.").await;
}
