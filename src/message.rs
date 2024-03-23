use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

// Maelstrom Message.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Default)]
pub struct Message {
    // ID of node sending message.
    pub src: String,
    // ID of node message is sent to.
    pub dest: String,
    // Body of the message.
    pub body: Body,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Default)]
pub struct Body {
    // Type of message
    #[serde(rename = "type", default, skip_serializing_if = "String::is_empty")]
    pub typ: String,

    // Optional. Message identifier that is unique to the source node.
    #[serde(default)]
    pub msg_id: u64,

    // Optional. For request/response, the msg_id of the request.
    #[serde(default)]
    pub in_reply_to: u64,

    // Per msg fields.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    use crate::message::Message;
    use crate::message::Body;

    #[test]
    fn parse_message() -> Result<()> {
        let echo = r#"{ "src": "c1", "dest": "n1", "body": { "type": "echo", "msg_id": 1, "echo": "Please echo 35" }}"#;

        let msg = serde_json::from_str::<Message>(&echo)?;
        let mut expected = Message {
            src: "c1".to_string(),
            dest: "n1".to_string(),
            body: Body::default(),
        };
        expected.body.typ = "echo".into();
        expected.body.msg_id = 1;
        expected
            .body
            .extra
            .insert("echo".into(), "Please echo 35".into());

        assert_eq!(msg, expected);
        Ok(())
    }

    #[test]
    fn parse_empty_message_fails() -> anyhow::Result<()> {
        let echo = "";

        let msg = serde_json::from_str::<Message>(&echo);

        assert!(msg.is_err(), "parsing empty message should fail.");
        Ok(())
    }

    #[test]
    fn parse_fails_when_no_src() -> anyhow::Result<()> {
        let echo =
            r#"{ "dest": "n1", "body": { "type": "echo", "msg_id": 1, "echo": "Please echo 35" }}"#;

        let msg = serde_json::from_str::<Message>(&echo);

        assert!(msg.is_err(), "parse should fail if src1");
        Ok(())
    }

    #[test]
    fn parse_fails_when_no_dst() -> anyhow::Result<()> {
        let echo =
            r#"{ "src": "c1",  "body": { "type": "echo", "msg_id": 1, "echo": "Please echo 35" }}"#;

        let msg = serde_json::from_str::<Message>(&echo);

        assert!(msg.is_err(), "parse should fail when no dst.");
        Ok(())
    }

    #[test]
    fn parse_fails_when_no_body() -> anyhow::Result<()> {
        let echo = r#"{ "src": "c1", "dest": "n1" }"#;

        let msg = serde_json::from_str::<Message>(&echo);

        assert!(msg.is_err(), "parse should fail when no body {:?}.", msg);
        Ok(())
    }
}
