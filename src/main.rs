
use std::io::{self};
use serde_json::{self, Value};
use anyhow::{Ok, Result};

fn main() -> Result<()> {
    let mut buffer = String::new();

    let stdin = io::stdin();

    stdin.read_line(&mut buffer)?;

    let data: Value = serde_json::from_str(&buffer)?;
    println!("Parsed JSON: {}", data);

    Ok(())
}
