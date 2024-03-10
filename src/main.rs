
use std::io::{self};

use serde_json::Value;
use anyhow::{Result};

fn main() -> Result<()> {
    let mut buffer = String::new();

    let stdin = io::stdin();

    stdin.read_line(&mut buffer)?;

    while stdin.read_line(&mut buffer).is_ok() {
        match serde_json::from_str::<Value>(&buffer) {
            Ok(v) => {
                println!("{}", v);
            }
            Err(e) => { eprintln!("Failed to parse json {}", e);}
        }
        buffer.clear();
    }
    Ok(())
}
