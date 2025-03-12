use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod parser;

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
    #[serde(default)]
    pub metadata: Metadata,
    /// Global variables (only valid in main.bdl)
    pub global_vars: Option<HashMap<String, Value>>,
    /// Local variables
    pub local_vars: HashMap<String, Value>,
    /// Nodes in the document
    pub nodes: HashMap<String, Node>,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Metadata {
    pub topic: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
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
    pub fn new(metadata: Option<Metadata>) -> Self {
        Self {
            metadata: metadata.unwrap_or_default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        // Test empty document
        let doc = Document::new(None);
        assert!(doc.metadata.topic.is_none());
        assert!(doc.metadata.description.is_none());
        assert!(doc.metadata.author.is_none());
        assert!(doc.metadata.version.is_none());
        assert!(doc.metadata.required.is_none());
        assert!(doc.global_vars.is_none());
        assert!(doc.local_vars.is_empty());
        assert!(doc.nodes.is_empty());

        // Test document with metadata
        let metadata = Metadata {
            topic: Some("Test Topic".to_string()),
            description: Some("Test Description".to_string()),
            author: Some("Test Author".to_string()),
            version: Some("1.0".to_string()),
            required: Some(vec!["dep1.bdl".to_string()]),
        };
        let doc = Document::new(Some(metadata.clone()));
        assert_eq!(doc.metadata.topic, Some("Test Topic".to_string()));
        assert_eq!(doc.metadata.description, Some("Test Description".to_string()));
        assert_eq!(doc.metadata.author, Some("Test Author".to_string()));
        assert_eq!(doc.metadata.version, Some("1.0".to_string()));
        assert_eq!(doc.metadata.required, Some(vec!["dep1.bdl".to_string()]));
    }

    #[test]
    fn test_node_management() {
        let mut doc = Document::new(None);
        
        // Test adding a node
        let mut node = Node::new("test_node".to_string());
        node.add_content(ContentElement::Text("Hello".to_string()));
        node.add_content(ContentElement::Variable("user".to_string()));
        node.add_option(BranchOption {
            keywords: vec!["next".to_string()],
            destination: Destination::Node("next_node".to_string()),
            condition: None,
        });

        assert!(doc.add_node(node.clone()).is_ok());
        
        // Test duplicate node error
        assert!(matches!(
            doc.add_node(node),
            Err(BdlError::NodeError(_))
        ));
    }

    #[test]
    fn test_node_content() {
        let mut node = Node::new("test".to_string());
        
        // Test adding different types of content
        node.add_content(ContentElement::Text("Hello ".to_string()));
        node.add_content(ContentElement::Variable("name".to_string()));
        node.add_content(ContentElement::FunctionCall {
            name: "getTime".to_string(),
            result_vars: vec!["time".to_string()],
        });

        assert_eq!(node.content.len(), 3);
        
        // Verify content types
        assert!(matches!(node.content[0], ContentElement::Text(_)));
        assert!(matches!(node.content[1], ContentElement::Variable(_)));
        assert!(matches!(node.content[2], ContentElement::FunctionCall { .. }));
    }

    #[test]
    fn test_branch_options() {
        let mut node = Node::new("test".to_string());
        
        // Test node destination
        node.add_option(BranchOption {
            keywords: vec!["next".to_string()],
            destination: Destination::Node("next_node".to_string()),
            condition: None,
        });

        // Test file transfer destination
        node.add_option(BranchOption {
            keywords: vec!["goto".to_string()],
            destination: Destination::FileTransfer {
                file: "other.bdl".to_string(),
                node: "start".to_string(),
            },
            condition: None,
        });

        // Test exit destination
        node.add_option(BranchOption {
            keywords: vec!["quit".to_string()],
            destination: Destination::Exit,
            condition: Some(Condition {
                variable: "can_exit".to_string(),
            }),
        });

        assert_eq!(node.options.len(), 3);
        
        // Verify destinations
        assert!(matches!(node.options[0].destination, Destination::Node(_)));
        assert!(matches!(node.options[1].destination, Destination::FileTransfer { .. }));
        assert!(matches!(node.options[2].destination, Destination::Exit));
    }

    #[test]
    fn test_value_types() {
        let mut vars = HashMap::new();
        
        // Test different value types
        vars.insert("string".to_string(), Value::String("hello".to_string()));
        vars.insert("number".to_string(), Value::Number(42.0));
        vars.insert("boolean".to_string(), Value::Boolean(true));
        vars.insert("empty".to_string(), Value::Empty);

        assert!(matches!(vars.get("string"), Some(Value::String(_))));
        assert!(matches!(vars.get("number"), Some(Value::Number(_))));
        assert!(matches!(vars.get("boolean"), Some(Value::Boolean(_))));
        assert!(matches!(vars.get("empty"), Some(Value::Empty)));
    }
} 