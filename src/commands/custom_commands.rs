use sqlite;
use std::fmt::Write;
use serenity::model::Message;
use chrono::prelude::*;
use std::env;
use serde_json;
use serde_json::Value;
use serde_json::Map;

fn home_guild() -> u64 {
  env::var("HOME_GUILD_ID")
    .expect("Expected a home guild ID in environment")
    .parse::<u64>().unwrap()
}

fn has_permission(msg: &Message) -> bool {
  let guild = match msg.guild() {
      Some(guild) => guild,
      None => {
          warn!("Couldn't get message guild!");

          return false;
      }
  };
  let guild = guild.read().unwrap();

  if guild.id.0 != home_guild() {
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

  let mut contents = "```Available Commands:\n".to_string();
  for cmd in commands {
      let _ = write!(contents, "{}\n", cmd.name);
  }

  let _ = write!(contents, "```");

  let _ = msg.channel_id.say(&contents);
});

command!(top(ctx, msg, _args) {
  let mut data = ctx.data.lock();
  let db = data.get_mut::<sqlite::Database>().unwrap();

  let commands = try!(db.top());

  let mut contents = "```Top 10 Most Used Commands:\n".to_string();

  let commands = commands.iter().take(10);

  for cmd in commands {
      let _ = write!(contents, "{} - {}\n", cmd.stat, cmd.name);
  }

  let _ = write!(contents, "```");

  let _ = msg.channel_id.say(&contents);
});

command!(add(ctx, msg, args) {
  // limit command adding to the main guild
  if let Some(guild_id) = msg.guild_id() {
    if guild_id.0 != home_guild() {
      let _ = msg.channel_id.say("Commands can be only added in the BLACKPINK server!");
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

  let timestamp = Utc.timestamp(cmd.created as i64, 0).format("%Y-%m-%d %H:%M:%S").to_string();

  let _ = msg.channel_id.send_message(|m| m
      .embed(|e| e
        .title(format!("Stats for {}", name))
        .field(|f| f
          .name("Url")
          .value(&cmd.url)
          .inline(false)
        )
        .field(|f| f
          .name("Times used")
          .value(&cmd.stat)
        )
        .field(|f| f
          .name("Added on")
          .value(timestamp)
        )
        .field(|f| f
          .name("Added by")
          .value(format!("<@{}>", &cmd.owner)
        ))));
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

  let mut contents = "```Search Results:\n".to_string();
  for cmd in results {
      let _ = write!(contents, "- {}\n", name=cmd.name);
  }

  let _ = write!(contents, "```");

  let _ = msg.channel_id.say(&contents);
});


#[derive(Serialize, Deserialize)]
struct Command {
  commands: Map<String, Value>,
}

command!(import(ctx, msg, args) {
  let mut raw_json = args.full();

  // try reading file
  if raw_json.is_empty() && msg.attachments.len() > 0 {
    let json_bytes = match msg.attachments[0].download() {
      Ok(content) => content,
      Err(why) => {
        println!("Error downloading attachment: {:?}", why);
        let _ = msg.channel_id.say("Error downloading attachment.");

        return Ok(());
      },
    };

    raw_json = match String::from_utf8(json_bytes) {
      Ok(content) => content,
      Err(why) => {
        let _ = msg.channel_id.say(&format!("Invalid UTF-8 sequence: {}", why));

        return Ok(());
      }
    };
  }

  let imported: Command = match serde_json::from_str(&raw_json) {
    Ok(val) => val,
    Err(why) => {
      let _ = msg.channel_id.say(&format!("Error parsing JSON: {}", why));
      return Ok(());
    }
  };

  let mut data = ctx.data.lock();
  let db = data.get_mut::<sqlite::Database>().unwrap();

  let _ = msg.channel_id.say(&format!("Importing commands {}...", &imported.commands.len()));

  for (key, value) in imported.commands.iter() {
    if !try!(db.is_command(&key)) {
      try!(db.add(&key, &value.as_str().unwrap().to_string(), 0000));
    } else {
      let _ = msg.channel_id.say(&format!("Error when importing `{}`: Command already exists.", &key));
    }
  }

  let _ = msg.channel_id.say("Finished import.");
});