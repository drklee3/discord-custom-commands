use std::path::Path;
use std::sync::Mutex;
use chrono::prelude::*;
use rusqlite::{Connection, Error};

pub struct CustomCommand {
    pub name: String,
    pub url: String,
    pub owner: i64,
    pub stat: u32,
    pub created: u32
}

impl CustomCommand {
    pub fn is_owner(&self, id: u64) -> bool {
        id == self.owner as u64
    }
}

const DB_PATH: &'static str = "database.sqlite3";

pub fn connect() -> Result<Database, Error> {
    let conn = try!(Connection::open(Path::new(DB_PATH)));

    try!(conn.execute("CREATE TABLE IF NOT EXISTS commands (
                      id              INTEGER PRIMARY KEY,
                      name            TEXT NOT NULL,
                      url             TEXT NOT NULL,
                      owner           INTEGER,
                      stat            INTEGER,
                      created         INTEGER
                      )", &[]));

    try!(conn.execute("CREATE TABLE IF NOT EXISTS servers (
                      id              INTEGER PRIMARY KEY,
                      prefix          TEXT NOT NULL
                      )", &[]));
    
    let db = Database {conn: Mutex::new(conn)};

    Ok(db)
}

pub struct Database {
    conn: Mutex<Connection>
}

impl Database {
    pub fn is_command(&self, name: &String) -> Result<bool, Error> {
        let conn = &self.conn.lock().unwrap();
        let mut stmt = try!(conn.prepare_cached("SELECT * FROM commands WHERE name = ?"));
        stmt.exists(&[name])
    }

    pub fn increment(&self, command: &CustomCommand) -> Result<(), Error> {
        let conn = &self.conn.lock().unwrap();
        let mut stmt = try!(conn.prepare_cached("UPDATE commands SET stat = ?1 WHERE name = ?2"));

        let new_stat = command.stat + 1;
        try!(stmt.execute(&[&new_stat, &command.name]));

        Ok(())
    }

    pub fn all(&self) -> Result<Vec<CustomCommand>, Error> {
        let conn = &self.conn.lock().unwrap();
        let mut stmt = try!(conn.prepare_cached("SELECT * FROM commands ORDER BY LOWER(name)"));
        let mut rows = try!(stmt.query(&[]));

        let mut commands = Vec::new();
        while let Some(result_row) = rows.next() {
            let row = try!(result_row);

            let cmd = CustomCommand {
                name: row.get(1),
                url: row.get(2),
                owner: row.get(3),
                stat: row.get(4),
                created: row.get(5)
            };

            commands.push(cmd);
        }

        Ok(commands)
    }

    pub fn top(&self) -> Result<Vec<CustomCommand>, Error> {
        let conn = &self.conn.lock().unwrap();
        let mut stmt = try!(conn.prepare_cached("SELECT * FROM commands ORDER BY stat DESC"));
        let mut rows = try!(stmt.query(&[]));

        let mut commands = Vec::new();
        while let Some(result_row) = rows.next() {
            let row = try!(result_row);

            let cmd = CustomCommand {
                name: row.get(1),
                url: row.get(2),
                owner: row.get(3),
                stat: row.get(4),
                created: row.get(5)
            };

            commands.push(cmd);
        }

        Ok(commands)
    }

    pub fn get(&self, name: &String) -> Result<CustomCommand, Error> {
        let conn = &self.conn.lock().unwrap();
        let mut stmt = try!(conn.prepare_cached("SELECT * FROM commands WHERE name = ?"));
        let row = try!(stmt.query_row(&[name], |row| CustomCommand {
            name: row.get(1),
            url: row.get(2),
            owner: row.get(3),
            stat: row.get(4),
            created: row.get(5)
        }));

        Ok(row)
    }


    pub fn search(&self, search: &String) -> Result<Vec<CustomCommand>, Error> {
        let conn = &self.conn.lock().unwrap();
        let mut stmt = try!(conn.prepare_cached("SELECT * FROM commands WHERE name LIKE ?1 \
                                                 or name LIKE ?2 ORDER BY LOWER(name)"));
        let search_loose = format!("%{}%", search);
        let mut rows = try!(stmt.query(&[search, &search_loose]));

        let mut commands = Vec::new();
        while let Some(result_row) = rows.next() {
            let row = try!(result_row);

            let cmd = CustomCommand {
                name: row.get(1),
                url: row.get(2),
                owner: row.get(3),
                stat: row.get(4),
                created: row.get(5)
            };

            commands.push(cmd);
        }

        Ok(commands)
    }

    pub fn add(&self, name: &String, url: &String, owner: u64) -> Result<(), Error> {
        let conn = &self.conn.lock().unwrap();
        let mut stmt = try!(conn.prepare_cached("INSERT INTO commands (name, url, owner, stat, created) \
                                                      VALUES (:name, :url, :owner, :stat, :created)"));

        let current_time = Utc::now().timestamp();

        let owner = owner as i64;

        try!(stmt.execute_named(&[(":name", name), (":url", url), (":owner", &owner),
                                  (":stat", &0), (":created", &current_time)]));

        Ok(())
    }

    pub fn delete(&self, name: &String) -> Result<(), Error> {
        let conn = &self.conn.lock().unwrap();
        let mut stmt = try!(conn.prepare_cached("DELETE FROM commands WHERE name = ?"));
        try!(stmt.execute(&[name]));

        Ok(())
    }

    pub fn delete_all(&self) -> Result<(), Error> {
        let conn = &self.conn.lock().unwrap();
        let mut stmt = try!(conn.prepare_cached("DELETE FROM commands"));
        try!(stmt.execute(&[]));

        Ok(())
    }

    pub fn edit(&self, name: &String, new_name: &String, new_url: &String) -> Result<(), Error> {
        let conn = &self.conn.lock().unwrap();
        let mut stmt = try!(conn.prepare_cached("UPDATE commands SET name = :new_name, url = :new_url WHERE name = :name"));
        try!(stmt.execute_named(&[(":new_name", new_name), (":new_url", new_url), (":name", name)]));

        Ok(())
    }

}

