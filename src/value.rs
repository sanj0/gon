use std::fmt::Write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    None,
    Str(String),
    Num(String),
    Bool(bool),
    Obj(crate::MapT),
    List(Vec<Value>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Object {
    inner: crate::MapT,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct List {
    inner: Vec<Value>,
}

/// Configures how a `Value` should be [`spell`]ed
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpellConfig {
    pub indent_amount: usize,
    pub indent_char: char,
    pub trailing_commas: bool,
    /// Max width of string literals before they get wrapped.
    pub max_width: usize,
}

impl Value {
    pub fn as_f64(&self) -> Option<f64> {
        let Self::Num(num) = self else {
            return None;
        };
        num.parse().ok()
    }

    pub fn as_i128(&self) -> Option<i128> {
        let Self::Num(num) = self else {
            return None;
        };
        num.parse().ok()
    }

    /// Minimally spells this value
    pub fn min_spell(&self) -> String {
        match self {
            Self::None => "None".into(),
            Self::Str(s) => klex::Token::Str(s.into()).spelling(),
            Self::Num(s) => s.into(),
            Self::Bool(b) => if *b { "true".into() } else { "false".into() },
            Self::Obj(m) => {
                let mut spelling = String::from("{");
                for (i, (k, v)) in m.iter().enumerate() {
                    let key_needs_quotes = key_needs_quoting(k);
                    if key_needs_quotes {
                        spelling.push('"');
                    }
                    spelling.push_str(k);
                    if key_needs_quotes {
                        spelling.push('"');
                    }
                    spelling.push(':');
                    spelling.push_str(&v.min_spell());
                    if i != m.len() - 1 {
                        spelling.push(',');
                    }
                }
                spelling.push('}');
                spelling
            }
            Self::List(xs) => {
                let mut spelling = String::from("[");
                for (i, v) in xs.iter().enumerate() {
                    spelling.push_str(&v.min_spell());
                    if i != xs.len() - 1 {
                        spelling.push(',');
                    }
                }
                spelling.push(']');
                spelling
            }
        }
    }

    pub fn spell(&self, config: SpellConfig) -> Result<String, std::fmt::Error> {
        let mut buf = String::new();
        self.spell0(&mut buf, 0, &config)?;
        Ok(buf)
    }

    fn spell0(&self, buf: &mut String, current_indent: usize, config: &SpellConfig) -> std::fmt::Result {
        match self {
            Self::None => write!(buf, "None")?,
            Self::Str(s) => {
                if config.max_width == 0 {
                    write!(buf, "{}", klex::Token::Str(s.clone()).spelling())?;
                } else {
                    let raw = format!("{}", klex::Token::Str(s.clone()).spelling());
                    let raw = squash_whitespace(&raw);
                    let wrapped_lines = textwrap::wrap(&raw, textwrap::Options::new(config.max_width).subsequent_indent(&gen_indent(current_indent + config.indent_amount, config)));
                    for (i, line) in wrapped_lines.iter().enumerate() {
                        if i == wrapped_lines.len() - 1 {
                            write!(buf, "{line}")?;
                        } else {
                            writeln!(buf, "{line}")?;
                        }
                    }
                }
            }
            Self::Num(s) => write!(buf, "{s}")?,
            Self::Bool(b) => write!(buf, "{b}")?,
            Self::Obj(obj) => {
                writeln!(buf, "{{")?;
                let new_indent = current_indent + config.indent_amount;
                for (i, (k, v)) in obj.iter().enumerate() {
                    apply_indent(buf, new_indent, config)?;
                    if key_needs_quoting(k) {
                        write!(buf, "\"{k}\": ")?;
                    } else {
                        write!(buf, "{k}: ")?;
                    }
                    v.spell0(buf, new_indent, config)?;
                    if !config.trailing_commas && i == obj.len() - 1 {
                        writeln!(buf, "")?;
                    } else {
                        writeln!(buf, ",")?;
                    }
                }
                apply_indent(buf, current_indent, config)?;
                write!(buf, "}}")?;
            }
            Self::List(xs) => 'match_arm: {
                if xs.is_empty() {
                    write!(buf, "[]")?;
                    break 'match_arm;
                }
                let oneline = xs.len() <= 5 && xs.iter().find(|v| matches!(v, Self::List(_) | Self::Obj(_))).is_none();
                if oneline {
                    write!(buf, "[")?;
                } else {
                    writeln!(buf, "[")?;
                }
                for (i, x) in xs.iter().enumerate() {
                    if oneline {
                        x.spell0(buf, 0, config)?;
                    } else {
                        let new_indent = current_indent + config.indent_amount;
                        apply_indent(buf, new_indent, config)?;
                        x.spell0(buf, new_indent, config)?;
                    }
                    if oneline {
                        if i != xs.len() - 1 {
                            write!(buf, ", ")?;
                        }
                    } else {
                        if config.trailing_commas || i != xs.len() - 1 {
                            write!(buf, ",")?;
                        }
                        writeln!(buf, "")?;
                    }
                }
                if !oneline {
                    apply_indent(buf, current_indent, config)?;
                }
                write!(buf, "]")?;
            }
        }
        Ok(())
    }
}

fn squash_whitespace(input: &str) -> String {
    let re = regex::Regex::new(r"[ \t\r\n]{2,}").unwrap();
    re.replace_all(input, " ").into_owned()
}

fn apply_indent(buf: &mut String, amount: usize, config: &SpellConfig) -> std::fmt::Result {
    write!(buf, "{}", gen_indent(amount, config))
}

fn gen_indent(amount: usize, config: &SpellConfig) -> String {
    std::iter::repeat(config.indent_char).take(amount).collect::<String>()
}

fn key_needs_quoting(key: &str) -> bool {
    let lexer_result = klex::Lexer::new(key, 0).lex();
    match lexer_result {
        Ok(tokens) => tokens.len() > 1,
        _ => true,
    }
}

impl Default for SpellConfig {
    fn default() -> Self {
        Self {
            indent_amount: 4,
            indent_char: ' ',
            trailing_commas: false,
            max_width: 100,
        }
    }
}
