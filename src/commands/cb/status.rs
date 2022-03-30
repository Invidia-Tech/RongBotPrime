use crate::data::DatabasePool;

use chrono::TimeZone;
use serenity::{
    client::Context,
    framework::standard::{
        Args,
        CommandResult,
        macros::command,
    },
    model::channel::Message,
    utils::MessageBuilder,
};

#[command("status")]
#[description("This shows the status of the current active clan battle.")]
async fn cb_status(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let cache = &ctx.cache;
    // The message builder allows for creating a message by
    // mentioning users dynamically, pushing "safe" versions of
    // content (such as bolding normalized content), displaying
    // emojis, and more.
    let response = MessageBuilder::new()
            .push("User ")
            .push_bold_safe(&msg.author.name)
            .push(" used the 'cb status' command in the ")
            .mention(&msg.channel_id.to_channel_cached(cache).await.unwrap())
            .push(" channel")
            .build();

    if let Err(why) = msg.channel_id.say(&ctx.http, &response).await {
        println!("Error sending message: {:?}", why);
    }

    let pool = ctx.data.read().await.get::<DatabasePool>().cloned().unwrap();
    //let clans_info =
    //     sqlx::query("SELECT id, platform_id FROM rong_clan WHERE platform_id != 'unset';")
    //         .fetch_all(&pool)
    //         .await?;

    // let mut clan_lookup:HashMap<String, i32> = HashMap::new();
    // for clan in clans_info {
    //     println!("Added {}: {} into clan lookup hashmap", clan.get::<String, _>("platform_id"), clan.get::<i32, _>("id"));
    //     clan_lookup.insert(clan.get("platform_id"), clan.get("id"));
    // }
    // println!("Clan lookup found {} clans", clan_lookup.len());

    let clan_info =
        match sqlx::query!(
            "SELECT clan_id, name, platform_id
             FROM rongbot.channel_type
             NATURAL JOIN public.rong_clan
             WHERE persona = 'cb'
                   AND channel_id = $1
                   AND platform_id != 'unset';",
            msg.channel_id.to_string()
        )
        .fetch_one(&pool)
        .await {
            Ok(row) => row,
            Err(_) => {
                msg.channel_id.say(ctx, "This channel does not allow cb commands.").await?;
                return Ok(());
            }
        };
    let guild_id = msg.guild_id.ok_or("Failed to get GuildID from Message.")?;
    let member = {
        match cache.member(guild_id, msg.author.id).await {
            Some(member) => member,
            None => {
                if let Err(why) = msg.channel_id.say(&ctx.http, "Error finding member data").await {
                    println!("Error sending message: {:?}", why);
                }
                return Ok(());
            },
        }
    };
    let required_role =
        cache.role(guild_id, clan_info.platform_id.parse::<u64>()?).await.ok_or("Error getting cached role.").unwrap();
    // let mut user_roles = vec![];
    // for role in &member.roles {
    //     user_roles.push(role);
    // }
    // msg.channel_id.say(ctx, format!("Looking for role: {:} for guild: {:}",
    //                                 required_role.name,
    //                                 clan_info.name)).await?;
    // msg.channel_id.say(ctx, format!("User has roles: {:?}", user_roles)).await?;
    if !member.roles.contains(&required_role.id) {
        msg.channel_id.say(ctx, format!("You do not have the correct role for {:?}.", clan_info.name)).await?;
        return Ok(());
    }


    msg.channel_id.say(ctx, format!("Clan you're in is: {:?}", clan_info.name)).await?;

    let closest_cb =
        match sqlx::query!(
            "SELECT cb.id, clan.name
            FROM rong_clanbattle AS cb
            JOIN rong_clan AS clan
            ON cb.clan_id = clan.id
            WHERE start_time = (SELECT start_time
                                FROM public.rong_clanbattle AS cb
                                JOIN rong_clan AS clan
                                ON cb.clan_id = clan.id
                                WHERE clan.id = $1
                                ORDER BY abs(EXTRACT(EPOCH FROM start_time) - EXTRACT(EPOCH FROM now()))
                                LIMIT 1)
                  AND clan.id = $1
            LIMIT 1;",
            clan_info.clan_id
        )
        .fetch_one(&pool)
        .await {
            Ok(row) => row,
            Err(_) => {
                msg.channel_id.say(ctx, format!("There are no available clan battles for {:}", clan_info.name)).await?;
                return Ok(());
            }
        };

    msg.channel_id.say(ctx, format!("Current closest CB is {:?}", closest_cb.name)).await?;

    let cb_info =
        match sqlx::query!(
            "SELECT start_time, end_time, current_boss, current_hp, current_lap
             FROM public.rong_clanbattle
             WHERE id = $1;",
            closest_cb.id
        )
        .fetch_one(&pool)
        .await {
            Ok(row) => row,
            Err(_) => {
                msg.channel_id.say(ctx, format!("There are no clan battle info for {:}", closest_cb.name)).await?;
                return Ok(());
            }
        };

    msg.channel_id.say(ctx, format!("CB info: start: <t:{:}:f>, end: <t:{:}:f>, {:}, {:}, {:}",
                                    cb_info.start_time.unwrap().unix_timestamp(), cb_info.end_time.unwrap().unix_timestamp(),
                                    cb_info.current_boss.unwrap(), cb_info.current_hp.unwrap(), cb_info.current_lap.unwrap())).await?;

    /*

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
    */
    Ok(())
}
