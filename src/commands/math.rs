command!(multiply(_ctx, msg, args) {
  let two = match args.single::<f64>() {
    Ok(val) => val,
    Err(why) => {
      let _ = msg.channel_id.say(&format!("Error: {}", why));
      return Ok(());
    },
  };

  let one = match args.single::<f64>() {
    Ok(val) => val,
    Err(why) => {
      let _ = msg.channel_id.say(&format!("Error: {}", why));
      return Ok(());
    },
  };

  let product = one * two;

  let _ = msg.channel_id.say(product);
});
