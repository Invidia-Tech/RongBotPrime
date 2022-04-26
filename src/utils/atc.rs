use crate::{
    data::{
        DatabasePool,
        Flight,
        RongPilot,
    },
    error::RongError,
};

use std::collections::HashMap;

use serenity::client::Context;

pub async fn update_pilot_info(ctx: &Context, pilot_info: &RongPilot) -> Result<(), RongError> {
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();
    match sqlx::query!(
        "UPDATE rongbot.pilot SET (nickname, motto, code) = ($1, $2, $3)
         WHERE id = $4;",
        pilot_info.nickname,
        pilot_info.motto,
        pilot_info.code,
        pilot_info.id
    )
    .execute(&pool)
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(RongError::Database(e)),
    }
}

pub async fn get_all_pilot_info_map(
    ctx: &Context,
    clan_id: &i32,
) -> Result<HashMap<i32, RongPilot>, RongError> {
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();
    match sqlx::query_as!(
        RongPilot,
        "SELECT * FROM rongbot.pilot WHERE clan_id = $1;",
        clan_id
    )
    .fetch_all(&pool)
    .await
    {
        Ok(pilots) => {
            let mut pilot_hashmap: HashMap<i32, RongPilot> = HashMap::default();
            for pilot in pilots {
                pilot_hashmap.insert(pilot.id, pilot);
            }
            Ok(pilot_hashmap)
        }
        Err(e) => Err(RongError::Database(e)),
    }
}

pub async fn get_all_pilot_ign_map(
    ctx: &Context,
    clan_id: &i32,
) -> Result<HashMap<i32, String>, RongError> {
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();
    match sqlx::query!(
        "SELECT p.id, name, ign
         FROM rongbot.pilot p
         JOIN rong_user u
            ON p.user_id=u.id
         JOIN rong_clanmember cm
            ON cm.user_id = u.id
         WHERE cm.clan_id = $1;",
        clan_id
    )
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => {
            let mut names_hashmap: HashMap<i32, String> = HashMap::default();
            for row in rows {
                names_hashmap.insert(row.id, row.ign);
            }
            Ok(names_hashmap)
        }
        Err(e) => Err(RongError::Database(e)),
    }
}

pub async fn get_pilot_info_or_create_new(
    ctx: &Context,
    user_id: &i32,
    clan_id: &i32,
) -> Result<RongPilot, RongError> {
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();
    match sqlx::query_as!(
        RongPilot,
        "SELECT * FROM rongbot.pilot WHERE user_id = $1 AND clan_id = $2;",
        user_id,
        clan_id
    )
    .fetch_one(&pool)
    .await
    {
        Ok(row) => Ok(row),
        Err(_) => {
            match sqlx::query_as!(
                RongPilot,
                "INSERT INTO rongbot.pilot (user_id, clan_id)
                 VALUES ($1, $2) RETURNING *",
                user_id,
                clan_id
            )
            .fetch_one(&pool)
            .await
            {
                Ok(row) => Ok(row),
                Err(e) => Err(RongError::Database(e)),
            }
        }
    }
}

pub async fn get_pilot_ongoing_flights(
    ctx: &Context,
    pilot_id: &i32,
    clan_id: &i32,
    cb_id: &i32,
) -> Result<Vec<Flight>, RongError> {
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();
    match sqlx::query_as_unchecked!(
        Flight,
        "SELECT * FROM rongbot.flight
         WHERE  pilot_id = $1
            AND clan_id  = $2
            AND cb_id    = $3
            AND status   = 'in flight'",
        pilot_id,
        clan_id,
        cb_id
    )
    .fetch_all(&pool)
    .await
    {
        Ok(flights) => Ok(flights),
        Err(_) => Err(RongError::Custom(
            "There is a problem getting all of your flights.".to_string(),
        )),
    }
}

pub async fn get_all_flights(
    ctx: &Context,
    clan_id: &i32,
    cb_id: &i32,
) -> Result<Vec<Flight>, RongError> {
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();
    match sqlx::query_as_unchecked!(
        Flight,
        "SELECT * FROM rongbot.flight
         WHERE  clan_id  = $1
            AND cb_id    = $2
         ORDER BY CASE status
                    WHEN 'in flight' THEN 0
                  END,
            start_time DESC;",
        clan_id,
        cb_id
    )
    .fetch_all(&pool)
    .await
    {
        Ok(flights) => Ok(flights),
        Err(_) => Err(RongError::Custom(
            "There is a problem getting all of your flights.".to_string(),
        )),
    }
}

pub async fn get_all_pilot_flights(
    ctx: &Context,
    pilot_id: &i32,
    clan_id: &i32,
    cb_id: &i32,
) -> Result<Vec<Flight>, RongError> {
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();
    match sqlx::query_as_unchecked!(
        Flight,
        "SELECT * FROM rongbot.flight
         WHERE  clan_id  = $1
            AND cb_id    = $2
            AND pilot_id = $3
         ORDER BY CASE status
                    WHEN 'in flight' THEN 0
                    WHEN 'landed' THEN 1
                  END,
            start_time;",
        clan_id,
        cb_id,
        pilot_id
    )
    .fetch_all(&pool)
    .await
    {
        Ok(flights) => Ok(flights),
        Err(_) => Err(RongError::Custom(
            "There is a problem getting all of your flights.".to_string(),
        )),
    }
}
