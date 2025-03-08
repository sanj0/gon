use std::error::Error;
use std::path::PathBuf;
use std::io::Read;
use std::fs::File;

use clap::Parser;
use serde_json::Value as JsonValue;

use gon::*;

#[derive(Parser)]
#[command(name = "gon", version, about = "CLI-utility for working with GON data", long_about = None)]
struct Args {
    /// What can I do for you?
    verb: Verb,
    /// How many characters to indent formatted output with?
    /// Only works with the `fmt` and `from` verbs.
    #[arg(long, short = 'w', default_value_t = 4)]
    indent_width: usize,
    /// What characters to indent formatted output with?
    /// Only works with the `fmt` and `into` verbs.
    #[arg(long, short = 'c', default_value_t = ' ')]
    indent_char: char,
    /// Put commas after last entries in lists and objects in formatted output?
    /// Only works with the `fmt` verb.
    #[arg(long, short, action)]
    trailing_commas: bool,
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
    Into,
    /// Convert json input to gon
    From,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    match args.verb {
        Verb::Min => {
            let Some(value) = get_gon_input(args.file)? else {
                return Ok(());
            };
            println!("{}", value.min_spell());
        },
        Verb::Fmt => {
            let Some(value) = get_gon_input(args.file)? else {
                return Ok(());
            };
            let spell_config = SpellConfig {
                indent_amount: args.indent_width,
                indent_char: args.indent_char,
                trailing_commas: args.trailing_commas,
            };
            println!("{}", value.spell(spell_config)?);
        }
        Verb::Into => {
            let Some(value) = get_gon_input(args.file)? else {
                return Ok(());
            };
            println!("{}", serde_json::to_string_pretty(&serde_json::Value::from(value)).map_err(|e| Box::new(e))?);
        }
        Verb::From => {
            let json = get_json_input(args.file)?;
            let spell_config = SpellConfig {
                indent_amount: args.indent_width,
                indent_char: args.indent_char,
                trailing_commas: args.trailing_commas,
            };
            println!("{}", Value::from(json).spell(spell_config)?);
        }
    }
    Ok(())
}

fn get_src(file: Option<PathBuf>) -> Result<String, Box<dyn Error>> {
    let src = if let Some(file) = file {
        let file = File::open(file).map_err(|e| Box::new(e))?;
        std::io::read_to_string(file).map_err(|e| Box::new(e))?
    } else {
        let mut input = Vec::new();
        std::io::stdin().read_to_end(&mut input).map_err(|e| Box::new(e))?;
        String::from_utf8(input).map_err(|e| Box::new(e))?
    };
    Ok(src)
}

fn get_json_input(file: Option<PathBuf>) -> Result<JsonValue, Box<dyn Error>> {
    let src = get_src(file)?;
    serde_json::from_str(&src).map_err(|e| e.into())
}

fn get_gon_input(file: Option<PathBuf>) -> Result<Option<Value>, Box<dyn Error>> {
    let src = get_src(file)?;
    parse_str(&src).map_err(|e| e.into())
}
