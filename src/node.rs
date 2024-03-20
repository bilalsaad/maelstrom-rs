use std::{
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
#[derive(Debug, Default, Clone, PartialEq)]
enum State {
    // Node has yet to recieve its init message
    #[default]
    Start,
    // Node has recieved its init message
    Initialized(InitializedNode),
}

/// Represents an initialized, has the nodes ID and the information from the init method.
#[derive(Debug, Default, Clone, PartialEq)]
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

        if *self.state.borrow() == State::Start {
            return Err(anyhow!(
                "Not Ready: recieved message {:?} before init message cannot handle.",
                msg
            ));
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
            .and_then(|n| Some(n.to_string().replace("\"", "")))
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
            .map(|n| n.to_string().replace("\"", ""))
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

mod test {
    use std::collections::HashMap;

    use crate::message::Message;
    use crate::node::{InitializedNode, State};
    use crate::Node;

    use super::Handler;

    fn init_msg() -> Message {
        let msg = r#"{
            "src":"c1", "dest":"n1",
            "body":{ 
                "type":"init",
                "node_id":"n1",
                "node_ids":["n1", "n2"],
                "msg_id":1}
        }"#;

        serde_json::from_str::<Message>(&msg).expect("invalid init json.")
    }

    #[test]
    fn node_inital_state() -> anyhow::Result<()> {
        // Tests that the initial state of a node is in the "Start" state
        let node = Node::new(HashMap::new())?;
        assert_eq!(
            *node.state.borrow(),
            State::Start,
            "msg_id should start as Start, got {:?}",
            node.state
        );

        Ok(())
    }

    #[test]
    fn node_initializes_after_init() -> anyhow::Result<()> {
        // Test that node transitions into InializedNode state after recieving init msg.
        let node = Node::new(HashMap::new())?;

        node.handle(init_msg())?;

        let expected_state = State::Initialized(InitializedNode {
            id: "n1".into(),
            other_nodes: vec!["n1".into(), "n2".into()],
        });
        assert_eq!(
            *node.state.borrow(),
            expected_state,
            "node should transition into InitializedNode with id n1 and neighbor n2 got: {:?}",
            node.state
        );

        Ok(())
    }

    #[test]
    fn init_reply_is_valid() -> anyhow::Result<()> {
        // Tests that the reply for the first init message meets the Maelstrom spec from
        // https://github.com/jepsen-io/maelstrom/blob/main/doc/protocol.md#initialization
        let node = Node::new(HashMap::new())?;

        let reply = node.handle(init_msg())?;

        // Note that we expect that the first reply will have a message_id of 0 from us.
        let expected = r#"{
            "src":"n1", "dest":"c1",
            "body": { 
                "type":"init_ok",
                "in_reply_to": 1,
                "msg_id":0
                }
        }"#;

        let expected = serde_json::from_str::<Message>(&expected)?;

        assert_eq!(reply, expected);
        Ok(())
    }

    fn identity_handler(msg: Message, _: u64) -> anyhow::Result<Message> {
        Ok(msg)
    }

    #[test]
    fn cannot_create_node_with_init_handler() -> anyhow::Result<()> {
        // Test that creating node with a handler for "init" fails.
        let node = Node::new(HashMap::from([(
            "init".to_string(),
            identity_handler as Handler,
        )]));
        assert!(
            node.is_err(),
            "Creating a node with a handler for init is forbidden {:?}",
            node.unwrap()
        );
        Ok(())
    }

    #[test]
    fn multiple_init_messages_idempontent() -> anyhow::Result<()> {
        // Tests that multiple init messages are valid.
        let node = Node::new(HashMap::new())?;

        node.handle(init_msg())?;
        let expected_state = State::Initialized(InitializedNode {
            id: "n1".into(),
            other_nodes: vec!["n1".into(), "n2".into()],
        });
        assert_eq!(*node.state.borrow(), expected_state);
        node.handle(init_msg())?;
        assert_eq!(*node.state.borrow(), expected_state);
        node.handle(init_msg())?;
        assert_eq!(*node.state.borrow(), expected_state);
        node.handle(init_msg())?;
        assert_eq!(*node.state.borrow(), expected_state);

        Ok(())
    }

    #[test]
    fn reply_id_goes_up() -> anyhow::Result<()> {
        // T
        Ok(())
    }

    #[test]
    fn unimplemented_type_returns_error_after_init() -> anyhow::Result<()> {
        // Tests that an unknown message returns an error after init.
        let node = Node::new(HashMap::new())?;

        // Init
        node.handle(init_msg())?;

        // Known msg
        let msg = {
            let mut msg = init_msg();
            msg.body.typ = "unknown...".into();
            msg
        };

        let result = node.handle(msg);

        assert!(
            result.as_ref().is_err_and(|e| e
                .to_string()
                .contains("No handler for message type unknown...")),
            "expected failure with unknown handler, got {:?}",
            result
        );
        Ok(())
    }

    #[test]
    fn unknown_message_before_init_returns_error() -> anyhow::Result<()> {
        // Tests that an unknown message returns an error before init.
        let node = Node::new(HashMap::new())?;

        let msg = {
            let mut msg = init_msg();
            msg.body.typ = "unknown...".into();
            msg
        };

        let result = node.handle(msg);

        assert!(
            result
                .as_ref()
                .is_err_and(|e| e.to_string().contains("Not Ready")),
            "expected failure with unknown handler, got {:?}",
            result
        );
        Ok(())
    }

    fn message_before_init_returns_error() -> anyhow::Result<()> {
        // Tests that a message returns an error before init.
        let node = Node::new(HashMap::from([(
            "id".to_string(),
            identity_handler as Handler,
        )]))?;

        let msg = {
            let mut msg = init_msg();
            msg.body.typ = "id".into();
            msg
        };

        let result = node.handle(msg);

        assert!(
            result
                .as_ref()
                .is_err_and(|e| e.to_string().contains("Not Ready")),
            "expected failure with unknown handler, got {:?}",
            result
        );
        Ok(())
    }

    #[test]
    fn node_propagates_handler_error() -> anyhow::Result<()> {
        // Tests handler errors are propagated correctly.
        let handler: Handler = |_, _| Err(anyhow::anyhow!("error from handler"));
        let node = Node::new(HashMap::from([("id".to_string(), handler)]))?;

        node.handle(init_msg())?;

        let msg = {
            let mut msg = init_msg();
            msg.body.typ = "id".into();
            msg
        };
        let result = node.handle(msg);

        assert!(
            result
                .as_ref()
                .is_err_and(|e| e.to_string().contains("error from handler")),
            "expected failure from handler, got {:?}",
            result
        );
        Ok(())
    }
}
