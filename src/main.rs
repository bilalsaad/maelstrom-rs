
use std::io::{self};
use anyhow::Result;

mod message;

fn main() -> Result<()> {

    eprintln!("Node starting...");

    let mut buffer = String::new();
    let stdin = io::stdin();

    while stdin.read_line(&mut buffer).is_ok() {
        match serde_json::from_str::<message::Message>(&buffer) {
            Ok(msg) => { eprintln!("Recieved msg: {:?}", msg);
                if msg.body.typ == "init" {
                    // handle init
                }
                if msg.body.typ == "echo" {
                    // handle init
                }
            }
            Err(e) => { eprintln!("Failed to parse json {}", e);}
        }
        buffer.clear();
    }
    Ok(())
}
