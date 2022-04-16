macro_rules! result_or_say_why {
    ($expression:expr, $ctx:ident, $msg:ident) => {
        match $expression.await {
            Ok(info) => info,
            Err(why) => {
                $msg.channel_id.say($ctx, why).await?;
                return Ok(());
            }
        }
    };
}
pub(crate) use result_or_say_why;
