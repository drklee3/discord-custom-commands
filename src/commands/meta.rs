use CommandCounter;
use std::fmt::Write;

command!(latency(ctx, msg) {
    let latency = ctx.shard.lock()
        .latency()
        .map_or_else(|| "N/A".to_string(), |s| {
            format!("{}.{}s", s.as_secs(), s.subsec_nanos())
        });

    let _ = msg.channel_id.say(latency);
});

command!(ping(_ctx, msg) {
    let _ = msg.channel_id.say("Pong!");
});

command!(commands(ctx, msg, _args) {
    let mut contents = "Commands used:\n".to_string();

    let data = ctx.data.lock();
    let counter = data.get::<CommandCounter>().unwrap();

    for (k, v) in counter {
        let _ = write!(contents, "- {name}: {amount}\n", name=k, amount=v);
    }

    if let Err(why) = msg.channel_id.say(&contents) {
        error!("Error sending message: {:?}", why);
    }
});