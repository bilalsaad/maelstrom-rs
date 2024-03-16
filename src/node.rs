use crate::message::Body;
use anyhow::Result;

#[derive(Debug, Default)]
pub struct Node {
    id: String,
    other_nodes: Vec<String>,
}

impl Node {
    pub fn new(body: &Body) -> Result<Self> {
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
