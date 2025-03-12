use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BdlError {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Variable error: {0}")]
    VariableError(String),
    #[error("Node error: {0}")]
    NodeError(String),
}

/// Represents a complete BDL document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Document metadata
    pub metadata: Metadata,
    /// Global variables (only valid in main.bdl)
    pub global_vars: Option<HashMap<String, Value>>,
    /// Local variables
    pub local_vars: HashMap<String, Value>,
    /// Nodes in the document
    pub nodes: HashMap<String, Node>,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub topic: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub required: Option<Vec<String>>,
}

/// Represents a node in the BDL document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Node name (without @ symbol)
    pub name: String,
    /// Node content (text, function calls, etc.)
    pub content: Vec<ContentElement>,
    /// Available options/branches from this node
    pub options: Vec<BranchOption>,
}

/// Represents different types of content within a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentElement {
    /// Plain text content
    Text(String),
    /// Variable interpolation: ${var_name}
    Variable(String),
    /// Function call: !{function_name}
    FunctionCall {
        name: String,
        result_vars: Vec<String>,
    },
}

/// Represents an option/branch from a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchOption {
    /// Keywords that trigger this option
    pub keywords: Vec<String>,
    /// Destination (node name or file transfer)
    pub destination: Destination,
    /// Optional condition
    pub condition: Option<Condition>,
}

/// Represents a destination for an option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Destination {
    /// Points to a node in the current file: @node_name
    Node(String),
    /// Points to a node in another file: [file.bdl:node_name]
    FileTransfer {
        file: String,
        node: String,
    },
    /// Special exit command
    Exit,
}

/// Represents a condition check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// Variable name to check
    pub variable: String,
}

/// Represents possible values for variables
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Empty,
}

impl Document {
    /// Creates a new empty document
    pub fn new(metadata: Metadata) -> Self {
        Self {
            metadata,
            global_vars: None,
            local_vars: HashMap::new(),
            nodes: HashMap::new(),
        }
    }

    /// Adds a node to the document
    pub fn add_node(&mut self, node: Node) -> Result<(), BdlError> {
        if self.nodes.contains_key(&node.name) {
            return Err(BdlError::NodeError(format!("Node '{}' already exists", node.name)));
        }
        self.nodes.insert(node.name.clone(), node);
        Ok(())
    }
}

impl Node {
    /// Creates a new node
    pub fn new(name: String) -> Self {
        Self {
            name,
            content: Vec::new(),
            options: Vec::new(),
        }
    }

    /// Adds content to the node
    pub fn add_content(&mut self, content: ContentElement) {
        self.content.push(content);
    }

    /// Adds an option to the node
    pub fn add_option(&mut self, option: BranchOption) {
        self.options.push(option);
    }
} 