command!(latency(ctx, msg) {
    let latency = ctx.shard.lock()
        .latency()
        .map_or_else(|| "N/A".to_string(), |s| {
            format!("{}.{}s", s.as_secs(), s.subsec_nanos())
        });

    let _ = msg.channel_id.say(latency);
});

command!(shutdown(ctx, msg) {
    match ctx.quit() {
        Ok(()) => {
            let _ = msg.reply("Shutting down. :wave:");
        },
        Err(why) => {
            let _ = msg.reply(&format!("Failed to shutdown: {:?}", why));
        }
    }
});
