
use std::io::{self, Read};
use serde_json::{self, Value};
use anyhow::{Ok, Result};

fn main() -> Result<()> {
    let mut buffer = String::new();

    let mut stdin = io::stdin();

    stdin.read_line(&mut buffer)?;

    let data: Value = serde_json::from_str(&buffer)?;
    println!("Parsed JSON: {}", data);

    Ok(())
}
