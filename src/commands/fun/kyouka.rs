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
        (
            "You're gonna have a bad time",
            "https://i.imgur.com/vT22PQX.jpeg",
        ),
        ("Kyaru coming through!", "https://i.imgur.com/Ni9F4oh.png"),
        (
            "You know exactly what you've done",
            "https://i.imgur.com/W6MmjAv.jpeg",
        ),
        ("Y'all need Jesus", "https://i.imgur.com/GgAx2Na.png"),
        ("I told you she was sus", "https://i.imgur.com/fubDBqW.gif"),
        ("*Blink*", "https://i.imgur.com/955EgRB.gif"),
        ("Dressed to the 9s", "https://i.imgur.com/rVpR8YQ.jpeg"),
        (
            "SHEEEEESSSHHHHHHHHHHHH Kyouka's got the DRIP",
            "https://i.imgur.com/Tnocysp.png",
        ),
        ("Long Kyouka", "https://i.imgur.com/w8wXF0n.png"),
        (
            "KyoukaSmile... Sometimes",
            "https://i.imgur.com/O7jIL15.gif",
        ),
        ("**UGGGGHHHHHHHHHHH**", "https://i.imgur.com/BCP1QD8.gif"),
        ("Yeah... Sure...", "https://i.imgur.com/a6JbETW.png"),
        ("Pat moreeeee", "https://i.imgur.com/lzEm5vU.gif"),
        ("Rare pepe", "https://i.imgur.com/TqGBF5L.png"),
        (
            "Imagine looping when you can just",
            "https://i.imgur.com/KOVGkD2.png",
        ),
        ("ALL KYOUKA", "https://i.imgur.com/AhcEYqZ.png"),
        ("KYOUKA SMILE", "https://i.imgur.com/lyARZJU.png"),
        (
            "*sensual voice* k-kyoukaaa",
            "https://i.imgur.com/mjBaQf4.jpeg",
        ),
        ("Come here boy.", "https://i.imgur.com/cmOjXHm.png"),
        ("A nice cozy day", "https://i.imgur.com/0EeGQoQ.png"),
        (
            "Kyouka has bad trigger discipline",
            "https://i.imgur.com/lNUdq6E.png",
        ),
        ("Kyouka is inside", "https://i.imgur.com/o4yNSUV.png"),
        ("Seatbelts everyone", "https://i.imgur.com/cfhUAS3.png"),
        (
            "When you respond with :KyoukaSmile:",
            "https://i.imgur.com/qEuyGKu.png",
        ),
        ("Tea time", "https://i.imgur.com/2ZhGx8l.png"),
        (
            "Kyouka needs some alone time",
            "https://i.imgur.com/CncNGg8.png",
        ),
        ("The spark is worth it", "https://i.imgur.com/q8RHXiK.png"),
        ("KyoukaWide", "https://i.imgur.com/LLOESSR.png"),
        ("S-suzunaaaaa", "https://i.imgur.com/qUKh2X7.png"),
        ("Kyouka in the wild", "https://i.imgur.com/IH6WDMc.jpeg"),
        ("Seems good", "https://i.imgur.com/TEXFeYG.png"),
        ("Smile", "https://i.imgur.com/OtRoKYq.png"),
        (
            "You didn't get Kyouka... maybe try >kyouka later",
            "https://i.imgur.com/xnHMY84.png",
        ),
        (
            "Open your textbooks to pg.128",
            "https://i.imgur.com/Wi9HQGM.jpeg",
        ),
        ("Kyouka cute!", "https://i.imgur.com/3FyLxAE.png"),
        (
            "was this who you were thinking of?",
            "https://i.imgur.com/NEAOE1t.png",
        ),
        ("HKyouka dominance", "https://i.imgur.com/iGmRJ9a.jpeg"),
        (
            "Kyouka is coming to town",
            "https://i.imgur.com/N76EYE8.png",
        ),
        ("What will come out?", "https://i.imgur.com/rNx1rOW.png"),
        ("Give kyouka your DAs", "https://i.imgur.com/qqVF9SF.png"),
        (
            "Kyouka watching you rant",
            "https://i.imgur.com/gYixYZA.png",
        ),
        ("smugggg", "https://i.imgur.com/GHWTlJG.jpeg"),
        (
            "Kyouka visiting you at night",
            "https://i.imgur.com/5RJLKMB.png",
        ),
        ("Kyouka lickin'", "https://i.imgur.com/dqjLmNQ.gif"),
        ("Ho~ Ho~ Ho~", "https://i.imgur.com/5HgFBMb.png"),
        (
            "It's nothing personnel kid",
            "https://i.imgur.com/GBr0JXO.png",
        ),
        (
            "Wake me up when it's my turn to hit",
            "https://i.imgur.com/j9FJH2g.png",
        ),
        ("KyoukaS-Smile", "https://i.imgur.com/mOtURUZ.jpeg"),
        ("Kyouka is tilted", "https://i.imgur.com/ch0ekUS.png"),
        ("Happy holidays", "https://i.imgur.com/SVaBTKD.png"),
        ("You're already dead...", "https://i.imgur.com/eWau8lq.gif"),
        ("M'Kyouka", "https://i.imgur.com/VgB5uh5.png"),
        ("Oh hi there", "https://i.imgur.com/XlkSqbr.png"),
        (
            "Kyouka is a high class meme",
            "https://i.imgur.com/r443ctA.png",
        ),
        ("Yes. You.", "https://i.imgur.com/pGYguFZ.gif"),
    ];

    let chosen = kyoukas.choose(&mut thread_rng()).unwrap();

    msg.channel_id
        .send_message(&ctx, |m| {
            m.content(chosen.0).add_embed(|e| e.image(chosen.1))
        })
        .await?;
    Ok(())
}
