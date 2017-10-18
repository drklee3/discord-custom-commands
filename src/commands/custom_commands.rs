use sqlite;
use std::fmt::Write;
use serenity::model::Message;

const HOME_GUILD_ID: u64 = 167058919611564043;

fn has_permission(msg: &Message) -> bool {
  let guild = match msg.guild() {
      Some(guild) => guild,
      None => {
          warn!("Couldn't get message guild!");

          return false;
      }
  };
  let guild = guild.read().unwrap();

  if guild.id.0 != HOME_GUILD_ID {
    return false;
  }

  // fetch member
  let member = match guild.members.get(&msg.author.id) {
      Some(member) => member,
      None => return false
  };
  // check if has perm
  if let Ok(permissions) = member.permissions() {
    return permissions.manage_guild();
  } else {
    return false;
  }
}

command!(commands(ctx, msg, _args) {
  let mut data = ctx.data.lock();
  let db = data.get_mut::<sqlite::Database>().unwrap();

  let commands = try!(db.all());

  let mut contents = "Custom Commands:\n".to_string();
  for cmd in commands {
      let _ = write!(contents, "- {}\n", name=cmd.name);
  }

  let _ = msg.channel_id.say(&contents);
});

command!(add(ctx, msg, args) {
  // limit command adding to the main guild
  if let Some(guild_id) = msg.guild_id() {
    if guild_id != HOME_GUILD_ID {
      return Ok(());
    }
  } else { // return if no guild found
    return Ok(()); 
  }

  let name = match args.single::<String>() {
    Ok(val) => val,
    Err(why) => {
      let _ = msg.channel_id.say(&format!("Error: {}", why));
      return Ok(());
    },
  };

  let url = match args.single::<String>() {
    Ok(val) => val,
    Err(why) => {
      let _ = msg.channel_id.say(&format!("Error: {}", why));
      return Ok(());
    },
  };

  let mut data = ctx.data.lock();
  let db = data.get_mut::<sqlite::Database>().unwrap();

  if !try!(db.is_command(&name)) {
    try!(db.add(&name, &url, msg.author.id.0));
    let _ = msg.channel_id.say(&format!("The command `{}` has been added with the response `{}`", name, url));
  } else {
    let _ = msg.channel_id.say(&format!("The command `{}` already exists!", name));
  }
});


command!(delete(ctx, msg, args) {
  let name = match args.single::<String>() {
    Ok(val) => val,
    Err(why) => {
      let _ = msg.channel_id.say(&format!("Error: {}", why));
      return Ok(());
    },
  };

  let mut data = ctx.data.lock();
  let db = data.get_mut::<sqlite::Database>().unwrap();

  if try!(db.is_command(&name)) {
    let cmd = try!(db.get(&name));

    if cmd.is_owner(msg.author.id.0) || has_permission(msg) {
      try!(db.delete(&name));
    }
  } else {
    let _ = msg.channel_id.say(&format!("The command `{}` was not found.", name));
  }  
});

command!(edit(ctx, msg, args) {
  let name = match args.single::<String>() {
    Ok(val) => val,
    Err(why) => {
      let _ = msg.channel_id.say(&format!("Error: {}", why));
      return Ok(());
    },
  };

  let new_name = match args.single::<String>() {
    Ok(val) => val,
    Err(why) => {
      let _ = msg.channel_id.say(&format!("Error: {}", why));
      return Ok(());
    },
  };

  let new_url = match args.single::<String>() {
    Ok(val) => val,
    Err(why) => {
      let _ = msg.channel_id.say(&format!("Error: {}", why));
      return Ok(());
    },
  };

  let mut data = ctx.data.lock();
  let db = data.get_mut::<sqlite::Database>().unwrap();

  if try!(db.is_command(&name)) {
    let cmd = try!(db.get(&name));

    if cmd.is_owner(msg.author.id.0) || has_permission(msg) {
      try!(db.edit(&name, &new_name, &new_url));
      let _ = msg.channel_id.say(&format!("The command `{}` has been updated with the 
                            name `{}` and response `{}`.", name, new_name, new_url));
    }
  } else {
    let _ = msg.channel_id.say(&format!("The command `{}` was not found.", name));
  }  
});

command!(stat(ctx, msg, args) {
  let name = match args.single::<String>() {
    Ok(val) => val,
    Err(why) => {
      let _ = msg.channel_id.say(&format!("Error: {}", why));
      return Ok(());
    },
  };

  let mut data = ctx.data.lock();
  let db = data.get_mut::<sqlite::Database>().unwrap();

  let cmd = match db.get(&name) {
    Ok(val) => val,
    Err(e) => {
      let _ = msg.channel_id.say(&format!("Error: {}", e));
      return Ok(());
    }
  };

  let _ = msg.channel_id.say(&format!("Stats for `{}`:\nUrl: <{}> \nUsed {} times\nAdded on {}\n \
    Added by <@{}>", name, cmd.url, cmd.stat, cmd.created, cmd.owner));
});

command!(search(ctx, msg, args) {
  let search = match args.single::<String>() {
    Ok(val) => val,
    Err(why) => {
      let _ = msg.channel_id.say(&format!("Error: {}", why));
      return Ok(());
    },
  };

  let mut data = ctx.data.lock();
  let db = data.get_mut::<sqlite::Database>().unwrap();

  let results = try!(db.search(&search));

  let mut contents = "Search Results:\n".to_string();
  for cmd in results {
      let _ = write!(contents, "- {}\n", name=cmd.name);
  }

  let _ = msg.channel_id.say(&contents);
});
