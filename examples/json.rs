use serde_json::Value as JsonValue;

use gon::*;

fn main() -> Result<(), String> {
    let mut input = String::new();
    loop {
        std::io::stdin().read_line(&mut input);
        let value_res = parse(input.trim().chars());
        match value_res {
            Ok(value) => {
                dbg!(JsonValue::from(value));
            }
            Err(e) => eprintln!("{e}"),
        }
        input.clear();
    }
}
