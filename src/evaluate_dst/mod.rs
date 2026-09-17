//! Evaluate Stage — Node stream → CssNode stream
//!
//! flat_map(eval_node) 处理每个 Node,展开为 0..N CssNode.

use crate::ast::{CssNode, Node};

pub fn eval_node_vec(node: Node) -> Vec<CssNode> {
    fn go(node: Node) -> Vec<CssNode> {
        match &node {
            Node::Rule { selector, body } => {
                let body_nodes: Vec<CssNode> =
                    body.iter().cloned().flat_map(go).collect();
                vec![CssNode::Rule {
                    selector: selector.clone(),
                    body: body_nodes,
                }]
            }
            Node::Declaration { prop, value } => {
                let substituted = substitute_variables(value);
                vec![CssNode::Declaration {
                    prop: prop.clone(),
                    value: substituted,
                }]
            }
            Node::Comment(c) => vec![CssNode::Comment(c.clone())],
            Node::Text(t) => vec![CssNode::Text(t.clone())],
            _ => Vec::new(),
        }
    }
    go(node)
}

fn substitute_variables(input: &str) -> String {
    input.to_string()
}
