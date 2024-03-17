mod message;
mod node;

use std::{collections::HashMap, io};

use anyhow::Result;
use node::Node;

fn echo_reply(msg: message::Message, msg_id: u64) -> Result<message::Message> {
    let body = message::Body {
        typ: "echo_ok".to_string(),
        msg_id,
        in_reply_to: msg.body.msg_id,
        ..msg.body
    };

    Ok(message::Message {
        src: msg.dest,
        dest: msg.src,
        body,
    })
}

fn main() -> Result<()> {
    eprintln!("Node starting...");

    let mut buffer = String::new();
    let stdin = io::stdin();

    let node = Node::new(HashMap::from([(
        "echo".to_string(),
        echo_reply as node::Handler,
    )]))?;
    while stdin.read_line(&mut buffer).is_ok() {
        eprintln!("Recieved msg: {}", buffer);
        match serde_json::from_str::<message::Message>(&buffer) {
            Ok(msg) => {
                if let Ok(reply) = node.handle(msg) {
                    println!(
                        "{}",
                        serde_json::to_string(&reply).expect("deserializing reply.")
                    );
                }
            }
            Err(e) => {
                eprintln!("Failed to parse json {}", e);
            }
        }
        buffer.clear();
    }
    Ok(())
}
