use rand::{seq::SliceRandom, thread_rng};

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

#[command("kyouka")]
#[bucket("kyouka")]
async fn kyouka(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let kyoukas = vec![
        ("KyoukaSmile", "https://i.imgur.com/Q1JSH4h.png"),
        ("SMH My Kyouka", "https://i.imgur.com/P0dakfH.gif"),
        ("Come to Kyouka", "https://i.imgur.com/wbJ5m6h.png"),
        ("Kyouka Giga Cursed", "https://i.imgur.com/V5P2t3I.gif"),
        ("Kyouka I5-3700", "https://i.imgur.com/pL95pZr.jpeg"),
        ("KyoukaArms", "https://i.imgur.com/aSIDf0T.gif"),
        ("Give kyouka your gems", "https://i.imgur.com/czbR041.png"),
        ("Pat the KyouPrism", "https://i.imgur.com/867idOR.gif"),
        ("POV: You lewded Kyouka", "https://i.imgur.com/zrj9U9D.png"),
        ("Are you winning son?", "https://i.imgur.com/GTSdAej.jpeg"),
        ("Kyouka loves you", "https://i.imgur.com/RhovUha.png"),
        ("Kyouka Drifting", "https://i.imgur.com/QclCmlK.png"),
        ("Kyouka didn't crit", "https://i.imgur.com/MM42Nb5.png"),
        ("Okay, just stop it.", "https://i.imgur.com/HvHosXe.png"),
        ("Tee hee~", "https://i.imgur.com/eJNcohK.jpeg"),
        (
            "May the powers of Kyouka compels you",
            "https://i.imgur.com/Oc68hab.png",
        ),
        ("Umm, can I crit too?", "https://i.imgur.com/L4WvfNb.png"),
        ("KyouDaddy", "https://i.imgur.com/JaaPaFe.png"),
        ("Happy Birthday", "https://i.imgur.com/IpR9nYR.png"),
        ("I've had enough", "https://i.imgur.com/UGsgNK3.png"),
        ("**Loud sipping noises**", "https://i.imgur.com/QtAqq5u.png"),
        ("Deserved.", "https://i.imgur.com/7dSJJEp.png"),
        ("Giga Kyouka Collage", "https://i.imgur.com/RtyZrQP.jpeg"),
        ("K-Kyouka S-Smile?", "https://i.imgur.com/UubKBr9.png"),
        ("Pat the Kyouka", "https://i.imgur.com/EMdkFu3.gif"),
        ("Big KyouPat", "https://i.imgur.com/aTvDl0Z.jpeg"),
        ("hehe ecks dee", "https://i.imgur.com/dXV5J5P.png"),
        ("hehe gl", "https://i.imgur.com/cn2IHxk.png"),
        ("KYOUKAAAAAAAAA", "https://i.imgur.com/Mlm83I9.gif"),
        ("POV: You Crit Ice Lance", "https://i.imgur.com/eSwFr5B.png"),
        ("hehe pat more", "https://i.imgur.com/Y9ClaNP.gif"),
        (
            "Kyouka has come to arrest you",
            "https://i.imgur.com/RUZIpAl.png",
        ),
        (
            "Dabo is sus, I saw him vent",
            "https://i.imgur.com/lwP9ZRd.png",
        ),
        ("Just Crit lmao", "https://i.imgur.com/3kUzOlE.jpeg"),
        (
            "I like some of you, don't come to CB next month",
            "https://i.imgur.com/Tc3NiX2.png",
        ),
        ("Why you no pull HKyouka", "https://i.imgur.com/2zSpg2e.gif"),
        (
            "I shouldn't have seen this",
            "https://i.imgur.com/lQ9kk8Y.png",
        ),
        ("Awwww yeahhhhhh", "https://i.imgur.com/8ugY9NB.jpeg"),
        ("Kyouka Chillin'", "https://i.imgur.com/nfpvL7P.png"),
        ("Not Kyouka", "https://i.imgur.com/hujUHDE.png"),
    ];

    let chosen = kyoukas.choose(&mut thread_rng()).unwrap();

    msg.channel_id
        .send_message(&ctx, |m| {
            m.content(chosen.0).add_embed(|e| e.image(chosen.1))
        })
        .await?;
    Ok(())
}
