use crate::{BdlMetadata, BdlError, BdlValue, BdlDocument, BdlDestination, BdlNode, BdlContentElement, BdlBranchOption, BdlCondition};
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub struct BdlParser {
    content: String,
}

impl BdlParser {
    pub fn new(content: String) -> Self {
        Self { content }
    }

    /// Validate a dependency file name
    fn validate_dependency_file(&self, file: &str) -> Result<(), BdlError> {
        // Check file extension
        if !file.ends_with(".bdl") {
            return Err(BdlError::DependencyError(
                format!("Invalid dependency file extension: {}", file)
            ));
        }

        // Additional file name validation could go here
        // For example, checking for valid characters, path traversal, etc.

        Ok(())
    }

    /// Validate a list of dependencies
    fn validate_dependencies(&self, dependencies: &[String]) -> Result<HashSet<String>, BdlError> {
        let mut validated = HashSet::new();
        
        for dep in dependencies {
            self.validate_dependency_file(dep)?;
            
            if !validated.insert(dep.clone()) {
                return Err(BdlError::DependencyError(
                    format!("Duplicate dependency: {}", dep)
                ));
            }
        }

        Ok(validated)
    }

    /// Validate that a file transfer destination is allowed by dependencies
    pub fn validate_file_transfer(&self, file: &str, dependencies: &HashSet<String>) -> Result<(), BdlError> {
        self.validate_dependency_file(file)?;
        
        if !dependencies.contains(file) {
            return Err(BdlError::DependencyError(
                format!("Undeclared dependency: {}", file)
            ));
        }

        Ok(())
    }

    /// Parse metadata from the beginning of the file
    pub fn parse_metadata(&self) -> Result<BdlMetadata, BdlError> {
        let mut metadata = BdlMetadata::default();
        
        // Split content into lines and process each line
        for line in self.content.lines() {
            let line = line.trim();
            
            // Stop at first non-metadata line
            if !line.starts_with('#') || line.is_empty() {
                break;
            }

            // Skip comment lines that don't contain metadata
            if !line.contains(':') {
                continue;
            }

            // Parse metadata line
            let line = line.trim_start_matches('#').trim();
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();
                
                match key.to_lowercase().as_str() {
                    "topic" => metadata.topic = Some(value.to_string()),
                    "description" => metadata.description = Some(value.to_string()),
                    "author" => metadata.author = Some(value.to_string()),
                    "version" => metadata.version = Some(value.to_string()),
                    "required" => {
                        metadata.required = Some(
                            value.split(',')
                                .map(|s| s.trim().to_string())
                                .collect()
                        )
                    },
                    _ => {} // Ignore unknown metadata keys
                }
            }
        }

        Ok(metadata)
    }

    /// Parse variable declarations (both global and local)
    pub fn parse_variables(&self) -> Result<(Option<HashMap<String, BdlValue>>, HashMap<String, BdlValue>), BdlError> {
        let mut global_vars = None;
        let mut local_vars = HashMap::new();
        let mut in_vars_block = false;
        let mut current_block: Option<&mut HashMap<String, BdlValue>> = None;

        for line in self.content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || (line.starts_with('#') && !line.contains('$')) {
                continue;
            }

            // Check for variable block start
            if line.starts_with("$global_vars:") {
                if global_vars.is_some() {
                    return Err(BdlError::ParseError("Duplicate global variables declaration".to_string()));
                }
                global_vars = Some(HashMap::new());
                current_block = global_vars.as_mut();
                in_vars_block = true;
                continue;
            } else if line.starts_with("$local_vars:") {
                current_block = Some(&mut local_vars);
                in_vars_block = true;
                continue;
            }

            // Parse variables within a block
            if in_vars_block {
                if line == "}" {
                    in_vars_block = false;
                    current_block = None;
                    continue;
                }

                if let Some(block) = &mut current_block {
                    if let Some((key, value)) = parse_variable_line(line)? {
                        block.insert(key, value);
                    }
                }
            }
        }

        Ok((global_vars, local_vars))
    }

    /// Parse all nodes from the content
    pub fn parse_nodes(&self, dependencies: &HashSet<String>) -> Result<HashMap<String, BdlNode>, BdlError> {
        let mut nodes = HashMap::new();
        let mut current_node: Option<BdlNode> = None;
        let mut current_content = Vec::new();

        for line in self.content.lines() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Check for node start
            if line.starts_with('@') {
                // Save previous node if it exists
                if let Some(node) = current_node.take() {
                    nodes.insert(node.name.clone(), node);
                }

                // Start new node
                let name = line[1..].trim().to_string();
                if nodes.contains_key(&name) {
                    return Err(BdlError::NodeError(format!("Duplicate node name: {}", name)));
                }
                current_node = Some(BdlNode::new(name));
                current_content.clear();
                continue;
            }

            // Process node content if we're in a node
            if let Some(ref mut node) = current_node {
                if line.starts_with('{') || line.starts_with("?{") {
                    // Parse option line
                    let option = self.parse_option(line, dependencies)?;
                    node.options.push(option);
                } else {
                    // Add content line
                    current_content.push(line.to_string());
                    if !current_content.is_empty() {
                        node.content.push(BdlContentElement::Text(current_content.join("\n")));
                        current_content.clear();
                    }
                }
            }
        }

        // Save last node if it exists
        if let Some(node) = current_node {
            nodes.insert(node.name.clone(), node);
        }

        Ok(nodes)
    }

    /// Parse a single option line
    fn parse_option(&self, line: &str, dependencies: &HashSet<String>) -> Result<BdlBranchOption, BdlError> {
        // TODO: Implement option parsing
        unimplemented!("Option parsing not yet implemented")
    }
}

/// Parse a single variable declaration line
fn parse_variable_line(line: &str) -> Result<Option<(String, BdlValue)>, BdlError> {
    // Skip empty lines and closing braces
    if line.trim().is_empty() || line.trim() == "}" {
        return Ok(None);
    }

    // Split key and value
    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() != 2 {
        return Err(BdlError::ParseError(format!("Invalid variable declaration: {}", line)));
    }

    let key = parts[0].trim().to_string();
    let value = parts[1].trim().trim_matches(',').trim();

    // Parse the value based on its format
    let parsed_value = if value.starts_with('"') && value.ends_with('"') {
        BdlValue::String(value.trim_matches('"').to_string())
    } else if value == "true" {
        BdlValue::Boolean(true)
    } else if value == "false" {
        BdlValue::Boolean(false)
    } else if value.parse::<f64>().is_ok() {
        BdlValue::Number(value.parse::<f64>().unwrap())
    } else if value.is_empty() || value == "{}" {
        BdlValue::Empty
    } else {
        return Err(BdlError::ParseError(format!("Invalid value format: {}", value)));
    };

    Ok(Some((key, parsed_value)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_metadata() {
        let parser = BdlParser::new("".to_string());
        let metadata = parser.parse_metadata().unwrap();
        assert!(metadata.topic.is_none());
        assert!(metadata.description.is_none());
        assert!(metadata.author.is_none());
        assert!(metadata.version.is_none());
        assert!(metadata.required.is_none());
    }

    #[test]
    fn test_parse_complete_metadata() {
        let content = "\
# Topic: Test Dialog
# Description: A test dialog file
# Author: Test Author
# Version: 1.0
# Required: mod1.bdl, mod2.bdl

@start
Some content...";

        let parser = BdlParser::new(content.to_string());
        let metadata = parser.parse_metadata().unwrap();
        
        assert_eq!(metadata.topic, Some("Test Dialog".to_string()));
        assert_eq!(metadata.description, Some("A test dialog file".to_string()));
        assert_eq!(metadata.author, Some("Test Author".to_string()));
        assert_eq!(metadata.version, Some("1.0".to_string()));
        assert_eq!(metadata.required, Some(vec!["mod1.bdl".to_string(), "mod2.bdl".to_string()]));
    }

    #[test]
    fn test_parse_partial_metadata() {
        let content = "\
# Topic: Partial Test
# Author: Test Author

@start";

        let parser = BdlParser::new(content.to_string());
        let metadata = parser.parse_metadata().unwrap();
        
        assert_eq!(metadata.topic, Some("Partial Test".to_string()));
        assert!(metadata.description.is_none());
        assert_eq!(metadata.author, Some("Test Author".to_string()));
        assert!(metadata.version.is_none());
        assert!(metadata.required.is_none());
    }

    #[test]
    fn test_parse_metadata_with_comments() {
        let content = "\
# This is a comment
# Topic: With Comments
# Another comment
# Author: Test Author
# Description: Test Description
# Yet another comment

Content starts here...";

        let parser = BdlParser::new(content.to_string());
        let metadata = parser.parse_metadata().unwrap();
        
        assert_eq!(metadata.topic, Some("With Comments".to_string()));
        assert_eq!(metadata.description, Some("Test Description".to_string()));
        assert_eq!(metadata.author, Some("Test Author".to_string()));
        assert!(metadata.version.is_none());
        assert!(metadata.required.is_none());
    }

    #[test]
    fn test_parse_empty_variables() {
        let content = "# Empty file";
        let parser = BdlParser::new(content.to_string());
        let (global, local) = parser.parse_variables().unwrap();
        assert!(global.is_none());
        assert!(local.is_empty());
    }

    #[test]
    fn test_parse_global_variables() {
        let content = r#"
$global_vars: {
    user_name: "",
    score: 0,
    is_complete: false,
    high_score: 100.5,
    inventory: {}
}
"#;
        let parser = BdlParser::new(content.to_string());
        let (global, local) = parser.parse_variables().unwrap();
        
        let globals = global.unwrap();
        assert_eq!(globals.len(), 5);
        assert!(matches!(globals.get("user_name"), Some(BdlValue::String(s)) if s.is_empty()));
        assert!(matches!(globals.get("score"), Some(BdlValue::Number(n)) if *n == 0.0));
        assert!(matches!(globals.get("is_complete"), Some(BdlValue::Boolean(b)) if !b));
        assert!(matches!(globals.get("high_score"), Some(BdlValue::Number(n)) if *n == 100.5));
        assert!(matches!(globals.get("inventory"), Some(BdlValue::Empty)));
        assert!(local.is_empty());
    }

    #[test]
    fn test_parse_local_variables() {
        let content = r#"
$local_vars: {
    attempts: 0,
    current_progress: 50,
    has_key: true,
    player_name: "John"
}
"#;
        let parser = BdlParser::new(content.to_string());
        let (global, local) = parser.parse_variables().unwrap();
        
        assert!(global.is_none());
        assert_eq!(local.len(), 4);
        assert!(matches!(local.get("attempts"), Some(BdlValue::Number(n)) if *n == 0.0));
        assert!(matches!(local.get("current_progress"), Some(BdlValue::Number(n)) if *n == 50.0));
        assert!(matches!(local.get("has_key"), Some(BdlValue::Boolean(b)) if *b));
        assert!(matches!(local.get("player_name"), Some(BdlValue::String(s)) if s == "John"));
    }

    #[test]
    fn test_parse_both_variable_types() {
        let content = r#"
$global_vars: {
    score: 0
}

$local_vars: {
    attempts: 3
}
"#;
        let parser = BdlParser::new(content.to_string());
        let (global, local) = parser.parse_variables().unwrap();
        
        let globals = global.unwrap();
        assert_eq!(globals.len(), 1);
        assert!(matches!(globals.get("score"), Some(BdlValue::Number(n)) if *n == 0.0));
        
        assert_eq!(local.len(), 1);
        assert!(matches!(local.get("attempts"), Some(BdlValue::Number(n)) if *n == 3.0));
    }

    #[test]
    fn test_duplicate_global_vars() {
        let content = r#"
$global_vars: {
    score: 0
}

$global_vars: {
    lives: 3
}
"#;
        let parser = BdlParser::new(content.to_string());
        assert!(matches!(
            parser.parse_variables(),
            Err(BdlError::ParseError(_))
        ));
    }

    #[test]
    fn test_invalid_variable_format() {
        let content = r#"
$local_vars: {
    invalid_value: not_valid
}
"#;
        let parser = BdlParser::new(content.to_string());
        assert!(matches!(
            parser.parse_variables(),
            Err(BdlError::ParseError(_))
        ));
    }

    #[test]
    fn test_validate_dependency_file() {
        let parser = BdlParser::new(String::new());
        
        // Valid dependency
        assert!(parser.validate_dependency_file("module.bdl").is_ok());
        
        // Invalid extension
        assert!(matches!(
            parser.validate_dependency_file("module.txt"),
            Err(BdlError::DependencyError(_))
        ));
        
        // No extension
        assert!(matches!(
            parser.validate_dependency_file("module"),
            Err(BdlError::DependencyError(_))
        ));
    }

    #[test]
    fn test_validate_dependencies() {
        let parser = BdlParser::new(String::new());
        
        // Valid dependencies
        let deps = vec![
            "module1.bdl".to_string(),
            "module2.bdl".to_string(),
        ];
        let result = parser.validate_dependencies(&deps);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
        
        // Duplicate dependencies
        let deps = vec![
            "module1.bdl".to_string(),
            "module1.bdl".to_string(),
        ];
        assert!(matches!(
            parser.validate_dependencies(&deps),
            Err(BdlError::DependencyError(_))
        ));
        
        // Invalid extension
        let deps = vec![
            "module1.bdl".to_string(),
            "module2.txt".to_string(),
        ];
        assert!(matches!(
            parser.validate_dependencies(&deps),
            Err(BdlError::DependencyError(_))
        ));
    }

    #[test]
    fn test_validate_file_transfer() {
        let parser = BdlParser::new(String::new());
        
        let mut deps = HashSet::new();
        deps.insert("module1.bdl".to_string());
        deps.insert("module2.bdl".to_string());
        
        // Valid transfer
        assert!(parser.validate_file_transfer("module1.bdl", &deps).is_ok());
        
        // Undeclared dependency
        assert!(matches!(
            parser.validate_file_transfer("module3.bdl", &deps),
            Err(BdlError::DependencyError(_))
        ));
        
        // Invalid extension
        assert!(matches!(
            parser.validate_file_transfer("module1.txt", &deps),
            Err(BdlError::DependencyError(_))
        ));
    }

    /// Helper function for tests that need valid dependencies
    fn create_test_dependencies() -> HashSet<String> {
        let mut deps = HashSet::new();
        deps.insert("module1.bdl".to_string());
        deps.insert("module2.bdl".to_string());
        deps
    }

    #[test]
    fn test_parse_empty_node() {
        let content = "@empty_node";
        let parser = BdlParser::new(content.to_string());
        let deps = create_test_dependencies();
        
        let nodes = parser.parse_nodes(&deps).unwrap();
        assert_eq!(nodes.len(), 1);
        
        let node = nodes.get("empty_node").unwrap();
        assert_eq!(node.name, "empty_node");
        assert!(node.content.is_empty());
        assert!(node.options.is_empty());
    }

    #[test]
    fn test_parse_text_only_node() {
        let content = r#"
@greeting
Hello, this is some text content.
It can span multiple lines.
"#;
        let parser = BdlParser::new(content.to_string());
        let deps = create_test_dependencies();
        
        let nodes = parser.parse_nodes(&deps).unwrap();
        assert_eq!(nodes.len(), 1);
        
        let node = nodes.get("greeting").unwrap();
        assert_eq!(node.name, "greeting");
        assert_eq!(node.content.len(), 1);
        
        match &node.content[0] {
            BdlContentElement::Text(text) => {
                assert!(text.contains("Hello, this is some text content"));
                assert!(text.contains("It can span multiple lines"));
            },
            _ => panic!("Expected Text content"),
        }
    }

    #[test]
    fn test_duplicate_node_names() {
        let content = r#"
@node1
Some content

@node1
Other content
"#;
        let parser = BdlParser::new(content.to_string());
        let deps = create_test_dependencies();
        
        assert!(matches!(
            parser.parse_nodes(&deps),
            Err(BdlError::NodeError(_))
        ));
    }

    #[test]
    fn test_multiple_nodes() {
        let content = r#"
@node1
First node content

@node2
Second node content
"#;
        let parser = BdlParser::new(content.to_string());
        let deps = create_test_dependencies();
        
        let nodes = parser.parse_nodes(&deps).unwrap();
        assert_eq!(nodes.len(), 2);
        assert!(nodes.contains_key("node1"));
        assert!(nodes.contains_key("node2"));
    }
} 