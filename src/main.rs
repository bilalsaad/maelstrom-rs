mod message;
mod node;

use std::io;

use anyhow::Result;
use node::Node;

fn init_reply(msg: message::Message, msg_id: u64) -> message::Message {
    let body = message::Body {
        typ: "init_ok".to_string(),
        msg_id,
        in_reply_to: msg.body.msg_id,
        ..Default::default()
    };

    message::Message {
        src: msg.dest,
        dest: msg.src,
        body,
    }
}

fn echo_reply(msg: message::Message, msg_id: u64) -> message::Message {
    let body = message::Body {
        typ: "echo_ok".to_string(),
        msg_id,
        in_reply_to: msg.body.msg_id,
        ..msg.body
    };

    message::Message {
        src: msg.dest,
        dest: msg.src,
        body,
    }
}

fn main() -> Result<()> {
    eprintln!("Node starting...");

    let mut buffer = String::new();
    let stdin = io::stdin();

    let mut node;
    let mut msg_id = 0;
    while stdin.read_line(&mut buffer).is_ok() {
        match serde_json::from_str::<message::Message>(&buffer) {
            Ok(msg) => {
                eprintln!("Recieved msg: {:?}", msg);
                if msg.body.typ == "init" {
                    // handle init
                    if let Ok(n) = Node::new(&msg.body) {
                        node = n;
                        eprintln!("initialized node {:?}", node);
                        println!(
                            "{}",
                            serde_json::to_string(&init_reply(msg.clone(), msg_id))
                                .expect("deserializing init_reply..")
                        );
                        msg_id += 1;
                    }
                }
                if msg.body.typ == "echo" {
                    println!(
                        "{}",
                        serde_json::to_string(&echo_reply(msg.clone(), msg_id))
                            .expect("deserializing init_reply..")
                    );
                    msg_id += 1;
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
