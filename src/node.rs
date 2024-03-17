use std::{
    borrow::BorrowMut,
    cell::{Cell, RefCell},
    collections::HashMap,
};

use crate::message::{Body, Message};
use anyhow::{anyhow, Result};

#[derive(Debug, Default)]
/// A Maelstrom node, handles messages.
///
/// A node consumes maelstrom messages and returns replies to them.
///
/// After recieving an init message a node will its ID and topology.
/// Messages recieved before an init message cannot be handled.
pub struct Node {
    // State of the node,
    // -->Start(Init) --> Initiazlied (Final)
    // A node transitions into initialized after handling its first init message.
    state: RefCell<State>,
    // Running count for reply message ids.
    msg_id: Cell<u64>,

    // Incoming message handlers.
    handlers: HashMap<String, Handler>,
}

/// Functions that process incoming messages.
/// Args:
///     - 1st arg: Request Message.
///     - 2nd arg: The reply_id to use in the response.
/// TODO: Consider making this a trait or something.
pub type Handler = fn(Message, u64) -> Result<Message>;

/// Node states,
///   | state |   Start  |   Initialized |
///   | start |    *     |      0        |
///   | init_msg | 0     |      
///
///   State \ Event  |  init_msg    |   other_msg  |
///       Start      |  Initialized |   Start      |
///       Initialized | Initialized | Initialized  |
#[derive(Debug, Default, Clone)]
enum State {
    // Node has yet to recieve its init message
    #[default]
    Start,
    // Node has recieved its init message
    Initialized(InitializedNode),
}

/// Represents an initialized, has the nodes ID and the information from the init method.
#[derive(Debug, Default, Clone)]
struct InitializedNode {
    id: String,
    other_nodes: Vec<String>,
}

impl Node {
    /// Creates a new node with that will invoke the given handlers on incoming messages.
    /// Note that the node will only reply to messages after it transitions into the Initalized
    /// phase (after it recieves an init_message).
    ///
    /// Preconditions:
    ///  - Cannot have an "init" handler. The init handler is hard coded and it transitions the
    ///  node into the Initalized state.
    pub fn new(handlers: HashMap<String, Handler>) -> Result<Self> {
        if let Some(_) = handlers.get("init") {
            return Err(anyhow::anyhow!(
                "FailedPrecondition: Cannot create Node with an init handler."
            ));
        }

        Ok(Self {
            state: State::Start.into(),
            msg_id: 0.into(),
            handlers,
        })
    }

    fn reply_id(self: &Self) -> u64 {
        let id = self.msg_id.get();
        self.msg_id.set(id + 1);
        id
    }

    pub fn handle(self: &Self, msg: Message) -> Result<Message> {
        let msg_type = &msg.body.typ;
        // Handle init message.
        if msg_type == "init" {
            let state = { self.state.borrow().clone() };
            match state {
                State::Start => {
                    let initialized_node = InitializedNode::new(&msg.body)?;
                    *self.state.borrow_mut() = State::Initialized(initialized_node);
                    return Ok(init_reply(msg, self.reply_id()));
                }
                State::Initialized(node) => {
                    eprintln!(
                        "Ignoring init message {:?} recieved after node initialized {:?}",
                        msg, node
                    );
                    return Ok(init_reply(msg, self.reply_id()));
                }
            }
        }

        // Otherwise try to find a handler.
        if let Some(&handler) = self.handlers.get(msg_type) {
            return handler(msg, self.reply_id());
        }

        Err(anyhow!(
            "UnimplementedError: No handler for message type {}, message: {:?}",
            msg.body.typ,
            msg
        ))
    }
}

impl InitializedNode {
    fn new(body: &Body) -> Result<Self> {
        if body.typ != "init" {
            return Err(anyhow::anyhow!(
                "Can only initialze node with an init message, got {:?}",
                body
            ));
        }

        let id = body
            .extra
            .get("node_id")
            .and_then(|n| Some(n.to_string()))
            .ok_or(anyhow::anyhow!(
                "can't init node if body has no node_id field: {:?}",
                body
            ))?;
        let other_nodes: Vec<String> = body
            .extra
            .get("node_ids")
            .and_then(|v| v.as_array())
            .ok_or(anyhow::anyhow!(
                "node_ids must be an array of node names... got {:?}",
                body
            ))?
            .into_iter()
            .map(|n| n.to_string())
            .collect();

        Ok(Self { id, other_nodes })
    }
}

fn init_reply(msg: Message, msg_id: u64) -> Message {
    let body = Body {
        typ: "init_ok".to_string(),
        msg_id,
        in_reply_to: msg.body.msg_id,
        ..Default::default()
    };

    Message {
        src: msg.dest,
        dest: msg.src,
        body,
    }
}
