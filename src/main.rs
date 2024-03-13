use anyhow::Result;
use message::Body;
use std::{collections::HashMap, io};

mod message;

#[derive(Debug, Default)]
struct Node {
    id: String,
    other_nodes: Vec<String>,
}

impl Node {
    fn new(body: &Body) -> Result<Self> {
        assert!(
            body.typ == "init",
            "Can only create a Node with an init body"
        );

        let id = body
            .extra
            .get("node_id")
            .and_then(|n| Some(n.to_string()))
            .ok_or(anyhow::anyhow!(
                "can't init node if body has no node_id field"
            ))?;
        let other_nodes: Vec<String> = body
            .extra
            .get("node_ids")
            .and_then(|v| v.as_array())
            .ok_or(anyhow::anyhow!(
                "node_ids must be an array of node names..."
            ))?
            .into_iter()
            .map(|n| n.to_string())
            .collect();

        Ok(Node {
            id: id,
            other_nodes: other_nodes,
        })
    }
}

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
