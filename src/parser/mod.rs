use crate::{Metadata, BdlError};

pub struct Parser {
    content: String,
}

impl Parser {
    pub fn new(content: String) -> Self {
        Self { content }
    }

    /// Parse metadata from the beginning of the file
    pub fn parse_metadata(&self) -> Result<Metadata, BdlError> {
        let mut metadata = Metadata::default();
        
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_metadata() {
        let parser = Parser::new("".to_string());
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

        let parser = Parser::new(content.to_string());
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

        let parser = Parser::new(content.to_string());
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

        let parser = Parser::new(content.to_string());
        let metadata = parser.parse_metadata().unwrap();
        
        assert_eq!(metadata.topic, Some("With Comments".to_string()));
        assert_eq!(metadata.description, Some("Test Description".to_string()));
        assert_eq!(metadata.author, Some("Test Author".to_string()));
        assert!(metadata.version.is_none());
        assert!(metadata.required.is_none());
    }
} 