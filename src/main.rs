use std::error::Error;
use std::path::PathBuf;
use std::io::Read;
use std::fs::File;

use clap::Parser;

use gon::*;

#[derive(Parser)]
#[command(name = "gon", version, about = "CLI-utility for working with GON data", long_about = None)]
struct Args {
    /// What can I do for you?
    verb: Verb,
    /// The input file. Leave empty for stdin.
    file: Option<PathBuf>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum Verb {
    /// Minify the input
    Min,
    /// Format the input
    Fmt,
    /// Convert input to json
    Json,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let Some(input) = get_input(args.file)? else {
        return Ok(());
    };
    match args.verb {
        Verb::Min => println!("{}", input.min_spell()),
        Verb::Fmt => {
            println!("{}", input.spell(SpellConfig::default())?);
        }
        Verb::Json => println!("{}", serde_json::to_string_pretty(&serde_json::Value::from(input)).map_err(|e| Box::new(e))?),
    }
    Ok(())
}

fn get_input(file: Option<PathBuf>) -> Result<Option<Value>, Box<dyn Error>> {
    let src = if let Some(file) = file {
        let file = File::open(file).map_err(|e| Box::new(e))?;
        std::io::read_to_string(file).map_err(|e| Box::new(e))?
    } else {
        let mut input = Vec::new();
        std::io::stdin().read_to_end(&mut input).map_err(|e| Box::new(e))?;
        String::from_utf8(input).map_err(|e| Box::new(e))?
    };
    parse_str(&src).map_err(|e| e.into())
}
