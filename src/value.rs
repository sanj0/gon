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
    indent_amount: usize,
    indent_char: char,
    trailing_commas: bool,
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
                for (k, v) in m {
                    spelling.push_str(k);
                    spelling.push(':');
                    spelling.push_str(&v.min_spell());
                    spelling.push(',');
                }
                spelling.push('}');
                spelling
            }
            Self::List(xs) => {
                let mut spelling = String::from("[");
                for v in xs {
                    spelling.push_str(&v.min_spell());
                    spelling.push(',');
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
            Self::Str(s) => write!(buf, "{}", klex::Token::Str(s.clone()).spelling())?,
            Self::Num(s) => write!(buf, "{s}")?,
            Self::Bool(b) => write!(buf, "{b}")?,
            Self::Obj(obj) => {
                writeln!(buf, "{{")?;
                let new_indent = current_indent + config.indent_amount;
                for (i, (k, v)) in obj.iter().enumerate() {
                    apply_indent(buf, new_indent, config)?;
                    write!(buf, "{k}: ")?;
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
                    write!(buf, "[ ")?;
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
                    if config.trailing_commas || i < xs.len() - 1 {
                        write!(buf, ",")?;
                    }
                    if !oneline {
                        writeln!(buf, "")?;
                    }
                }
                apply_indent(buf, current_indent, config)?;
                write!(buf, "]")?;
            }
        }
        Ok(())
    }
}

fn apply_indent(buf: &mut String, amount: usize, config: &SpellConfig) -> std::fmt::Result {
    write!(buf, "{}", std::iter::repeat(config.indent_char).take(amount).collect::<String>())
}

impl Default for SpellConfig {
    fn default() -> Self {
        Self {
            indent_amount: 4,
            indent_char: ' ',
            trailing_commas: false,
        }
    }
}
