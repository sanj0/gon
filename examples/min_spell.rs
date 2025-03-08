use gon::*;

fn main() -> Result<(), String> {
    let mut input = String::new();
    loop {
        std::io::stdin().read_line(&mut input);
        let value_res = parse(input.trim().chars());
        match value_res {
            Ok(Some(value)) => {
                println!("{}", value.min_spell());
            }
            Ok(None) => (),
            Err(e) => eprintln!("{e}"),
        }
        input.clear();
    }
}
