/* * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * *
 * Copyright (C) 2022 Jonas Møller <jonas@moesys.no>             *
 *                                                               *
 * This program is free software: you can redistribute it and/or *
 * modify it under the terms of the GNU General Public License   *
 * as published by the Free Software Foundation, version 3.      *
 *                                                               *
 * This program is distributed in the hope that it will be       *
 * useful, but WITHOUT ANY WARRANTY; without even the implied    *
 * warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR       *
 * PURPOSE. See the GNU General Public License for more details. *
 *                                                               *
 * You should have received a copy of the GNU General Public     *
 * License along with this program. If not, see                  *
 * <https://www.gnu.org/licenses/>                               *
 * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * */

use core::fmt;
use std::{env, fmt::Display, fs::File, io::{self, BufReader, Read}, process, str::FromStr};
use toml::Value;

#[derive(Debug)]
struct Opts {
    pattern: String,
    input: Option<String>,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("No such key: {key}")]
    NoSuchKey { key: String },
    #[error("IOError: {source}")]
    IOError { #[from] source: io::Error },
}

struct ExportSpec {
    path: Vec<String>,
}

impl FromStr for ExportSpec {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path =  s.split('.').map(|s| s.to_string()).collect();
        Ok(ExportSpec { path })
    }
}

fn get_path<'a, S>(mut obj: &'a Value, path: &[S]) -> Result<&'a Value, Error>
    where S: AsRef<str>
{
    for part in path.iter() {
        obj = obj.get(part.as_ref()).ok_or_else(|| {
            Error::NoSuchKey {
                key: path.iter().map(|s| s.as_ref()).collect::<Vec<_>>().join(".")
            }
        })?;
    }
    Ok(obj)
}

fn is_atomic(obj: &Value) -> bool {
    use Value::*;
    matches!(obj, String(_) | Boolean(_) | Integer(_) | Datetime(_) | Float(_))
}

fn write_atom(f: &mut fmt::Formatter<'_>, obj: &Value) -> fmt::Result {
    match obj {
        Value::String(s) => write!(f, "{}", snailquote::unescape(s).unwrap())?,
        Value::Integer(i) => write!(f, "{i}")?,
        Value::Boolean(b) => write!(f, "{b}")?,
        Value::Float(fl) => write!(f, "{fl}")?,
        Value::Datetime(dt) => write!(f, "{dt}")?,
        _ => unreachable!(),
    }
    Ok(())
}

struct FmtBash<'a> {
    value: &'a toml::Value,
}

impl<'a> Display for FmtBash<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            Value::Array(arr) => {
                let mut had_one = false;
                for elem in arr.iter() {
                    if !is_atomic(elem) { continue; }

                    if had_one {
                        write!(f, " ")?;
                    } else {
                        had_one = true;
                    }

                    write_atom(f, elem)?;
                }
            }
            Value::Table(tbl) => {
                let mut had_one = false;
                for (key, value) in tbl.iter() {
                    if !is_atomic(value) { continue; }

                    if had_one {
                        write!(f, " ")?;
                    } else {
                        had_one = true;
                    }

                    write!(f, r#"[{}]="#, snailquote::unescape(key).unwrap())?;
                    write_atom(f, value)?;
                }
            }
            Value::Integer(i) => writeln!(f, r#"{i}"#)?,
            x => {
                write_atom(f, x)?;
            }
        }
        Ok(())
    }
}

fn doit(opts: Opts) -> Result<(), Box<dyn std::error::Error>> {
    let mut input_file = BufReader::new(
        opts.input.map(|f| -> Result<Box<dyn Read>, io::Error> {
            Ok(Box::new(File::open(f)?))
        }).unwrap_or_else(|| Ok(Box::new(io::stdin())))?);
    let mut input = String::new();
    input_file.read_to_string(&mut input)?;
    let obj: toml::Value = toml::from_str(&input)?;
    let ExportSpec { path } = opts.pattern.parse()?;
    let value = get_path(&obj, &path)?;
    print!("{}", FmtBash { value });
    Ok(())
}

fn main() {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => {
            eprintln!("{name} - command line TOML processor [version {version}] \n");
            eprintln!("{}", include_str!("../docs/header.txt"));
            eprintln!("{}", include_str!("../docs/opt-help.txt"));
        }
        2 => { // read input from stdin
            let arg = &args[1];
            if arg == "--help" || arg == "-h" {
                eprintln!("{name} {version}\n");
                eprintln!("{}", include_str!("../docs/opt-help.txt"));
                return;
            }
            if arg == "--version" || arg == "-V" {
                eprintln!("{name} {version}");
                return;
            }
            let input = None;
            let opts = Opts { pattern: arg.clone(), input };
            if let Err(e) = doit(opts) {
                eprintln!("{e}");
                process::exit(1);
            }
        }
        3 => { // read input from file
            eprintln!("{}", include_str!("../docs/opt-help.txt"));
        }
        4 => { // read input from file
            {
                let shorthand = &args[1];
                if !(shorthand == "--file" || shorthand == "-f") {
                    eprintln!("{}", include_str!("../docs/opt-help.txt"));
                    return;
                }
            }

            let filename =  &args[2];
            let pattern =  &args[3];

            let opts = Opts { pattern: pattern.clone(), input: Option::from(filename.clone()) };
            if let Err(e) = doit(opts) {
                eprintln!("{e}");
                process::exit(1);
            }
        }
        _ => {
            eprintln!("too lot of arguments");
            eprintln!("{}", include_str!("../docs/opt-help.txt"));
        }
    }
}
