# Branching Dialog Language (BDL) Specification
Version 1.3

## 1. File Structure

### 1.1 Metadata Header
Every BDL file must begin with metadata in the following format:
```
# Topic: <title>
# Description: <description>
# Author: <author_name>
# Version: <version_number>
# Required: <comma_separated_dependencies>
```
All fields are required except `Required`.

### 1.2 Variable Declarations
Variables can be declared at file scope:
```
# Global variables (main.bdl only)
$global_vars: {
    user_name: "",
    score: 0,
    completed_modules: {}
}

# Local variables (any file)
$local_vars: {
    attempts: 0,
    current_progress: 0
}
```
Rules:
- Global variables can ONLY be declared in main.bdl
- Local variables can be declared in any file
- Variable names must be unique within their scope
- Values can be strings, numbers, booleans, or empty
- Non-main files cannot declare or modify $global_vars

### 1.3 Nodes
Nodes are the basic building blocks, defined by:
```
@node_name
content
options
```
Node names must:
- Start with '@'
- Contain only letters, numbers, and underscores
- Be unique within a file

## 2. Content Elements

### 2.1 Text Content
- Plain text is rendered as-is
- Supports multiple lines
- Empty lines are preserved
- Lines starting with '#' are comments

### 2.2 Variable Interpolation
Variables are referenced using: ${variable_name}
- Can appear anywhere in text content
- Can be used in function results
- Can be used in file transfers

### 2.3 Function Calls
Function calls follow the format:
```
!{function_name} : ~{var1} ~{var2}
```
Where:
- !{} indicates a function call
- ~{} indicates variables to store results
- First variable typically stores message text
- Second variable typically stores next node name

Example:
```
!{analyzePassword} : ~{result} ~{next}
${result}
-> ${next}
```

## 3. Flow Control

### 3.1 Basic Branching
Options are defined using:
```
{keyword1, keyword2, ...} -> destination
```
Where:
- Keywords are comma-separated
- Destination is either a node name or file transfer
- Numbers can be included as keywords (e.g., "1", "2")

### 3.2 Simple Conditions
Conditions use the format:
```
?{variable} -> destination
```
Checks if variable:
- Exists and is not empty
- Is not "false" or "0"
- For more complex conditions, use function calls

### 3.3 File Transfers
File transfers use the format:
```
[filename.bdl:node_name]
```
Or with variables:
```
[${module_name}:${node_name}]
```

## 4. Special Commands

### 4.1 Exit Command
```
{exit}
```
Terminates the current session

### 4.2 Return Command
```
[main.bdl:return_from_module]
```
Standard pattern for returning to main module

## 5. Comments
Single line comments start with '#'
- Must be on their own line
- Are ignored by the parser
- Used for documentation and metadata

## 6. Variable Scope

### 6.1 Global Variables
- Declared ONLY in main.bdl using $global_vars
- Accessible across all files
- Persist throughout session
- Cannot be redeclared in other files
- Should be used for:
  * User information
  * Progress tracking
  * Session state
  * Cross-module data

### 6.2 Local Variables
- Declared in $local_vars
- Scope limited to current file
- Reset when leaving file
- Can be used for:
  * Temporary calculations
  * Module-specific state
  * Function results
  * Flow control

## 7. Function Results

### 7.1 Result Format
Functions must return:
- A message string (displayed to user)
- A destination node name
These are captured using: `: ~{message} ~{next_node}`

### 7.2 Using Results
Typical pattern:
```
!{functionName} : ~{message} ~{next}
${message}
-> ${next}
```

## 8. Error Handling

### 8.1 Required Handling
- Missing required files should error
- Invalid node references should error
- Syntax errors should prevent execution

### 8.2 Runtime Handling
- Undefined variables should return empty string
- Failed function calls should have default handling
- Invalid node references should return to main menu

## 9. Best Practices

### 9.1 File Organization
- One topic per file
- Clear node naming
- Consistent return points
- Document dependencies

### 9.2 Content Structure
- Clear user instructions
- Consistent option patterns
- Helpful error messages
- Progress tracking

### 9.3 Flow Control
- Always provide back option
- Clear navigation paths
- Avoid dead ends
- Track completion

### 9.4 Function Usage
- Build complete messages in functions
- Use clear, descriptive variable names
- Handle all error cases
- Provide helpful feedback

## 10. Syntax Summary

### 10.1 Special Characters
- `@` : Node definition
- `->` : Navigation/branching
- `:` : Assignment operator
- `!{func}` : Function call
- `?{var}` : Condition check
- `~{var}` : Assignment target
- `${var}` : Variable reference
- `#` : Comment
- `[]` : File transfer
- `{}` : Content block (keywords, variables, etc.) 