use crate::data::DatabasePool;

use serenity::{client::Context, framework::standard::Reason, model::channel::Message};

macro_rules! run_check {
    ($expression:expr, $ctx:ident, $msg:ident) => {
        match $expression($ctx, $msg).await {
            Ok(_) => Ok(()),
            Err(why) => Err(why),
        }
    };
}
pub(crate) use run_check;

pub async fn dabo_ring(msg: &Message) -> Result<(), Reason> {
    // Ring/Dabo admin aboose
    match msg.author.id.0 {
        162034086066520064 | 79515100536385536 => Ok(()),
        _ => Err(Reason::User("You are not Ring or Dabo.".to_string())),
    }
}

pub async fn rong_admin(ctx: &Context, msg: &Message) -> Result<(), Reason> {
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .cloned()
        .unwrap();

    let user_admin_status = match sqlx::query!(
        "SELECT is_superadmin
         FROM public.rong_user
         WHERE platform_id = $1;",
        msg.author.id.to_string()
    )
    .fetch_one(&pool)
    .await
    {
        Ok(row) => row.is_superadmin,
        Err(_) => {
            return Err(Reason::User(
                "You're definitely not a Rong superadmin! <:Suzunaaaaaaaaa:914426187378466836>"
                    .to_string(),
            ));
        }
    };

    if !user_admin_status {
        return Err(Reason::User(
            "Nice try, but you are not a rong superadmin! <:YuiBat:964435494509346897>".to_string(),
        ));
    }

    Ok(())
}
