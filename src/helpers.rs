use serde_json;
use serde_json::Value;
use serde_json::Map;
use std::fs::File;
use std::io::prelude::*;

lazy_static! {
  static ref LOCALE: Locale = Locale::new();
}

#[derive(Serialize, Deserialize)]
pub struct Locale {
    pub info: Map<String, Value>,
    pub error: Map<String, Value>,
}

impl Locale {
    pub fn new() -> Locale {
        let mut f = File::open("assets/locale.json").expect("Locale file not found.");

        let mut contents = String::new();
        f.read_to_string(&mut contents).expect(
            "Something went wrong reading the locale file.",
        );

        serde_json::from_str(&contents).expect("Failed to parse JSON")
    }
}

pub fn get_error(id: &str) -> String {
    match LOCALE.error.get(id) {
        Some(val) => val.as_str().unwrap_or("").to_string(),
        None => "".to_string(),
    }
}

pub fn get_info(id: &str) -> String {
    match LOCALE.info.get(id) {
        Some(val) => val.as_str().unwrap_or("").to_string(),
        None => "".to_string(),
    }
}

fn replace(text: String, replacements: &[&String]) -> String {
    let mut new_string = text;
    for repl in replacements {
        new_string = new_string.replacen("{}", repl, 1); // replace each single {} with replacements
    }

    new_string.to_string()
}

pub fn get_error_f(id: &str, replacements: &[&String]) -> String {
    let text = get_error(id);

    replace(text, &replacements)
}

pub fn get_info_f(id: &str, replacements: &[&String]) -> String {
    let text = get_info(id);

    replace(text, &replacements)
}
