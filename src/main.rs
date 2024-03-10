
use std::io::{self};

use serde_json::{Value};
use anyhow::Result;

mod message;



// Maelstrom RPC msg
#[derive(Debug)]
struct Message {
    // ID of node sending this.
    src: String, 
    // ID of node this message is for.
    dest: String,
    // Body of the message.
    // {
    //   "type":        (mandatory) A string identifying the type of message this is
    //   "msg_id":      (optional)  A unique integer identifier
    //    "in_reply_to": (optional)  For req/response, the msg_id of the request
    //  }
    body: Value,
}


enum Type {
    // Unknown, string is the type
    Unknown(String),
    // Init RPC
    Init,
}


impl Message {
    fn new(v: Value) -> Result<Self> {
        let src: String = v.get("src").expect("todo fix src in msg").as_str().expect("sheet").to_string();
        let dest: String = v.get("dest").expect("todo fix dest in msg").as_str().expect("sheeet dest").to_string();
        let body: Value = v.get("body").expect("shit todo no body in msg").clone();

        Ok(Self{
            src,
            dest,
            body
        })
    }

    fn msg_type(&self) -> Type {
        match self.body.get("type") {
            Some(v) => {
                if v.as_str().is_some_and(|x| x == "init") {
                    return Type::Init
                }
                return Type::Unknown(v.as_str().unwrap_or("unknown type").to_string())
            }
            None => {panic!("Message with no type in body...");}
        }
    }
}

fn main() -> Result<()> {

    eprintln!("Node starting...");

    let mut buffer = String::new();
    let stdin = io::stdin();

    while stdin.read_line(&mut buffer).is_ok() {
        match serde_json::from_str::<message::Message>(&buffer) {
            Ok(msg) => {
                eprintln!("Recieved msg: {:?}", msg);
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
