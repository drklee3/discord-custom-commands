command!(add(_ctx, msg, args) {
  let url = match args.single::<String>() {
    Ok(val) => val,
    Err(why) => {
      let _ = msg.channel_id.say(&format!("Error: {}", why));
      return Ok(());
    },
  };

  let _ = msg.channel_id.say(url);

});
