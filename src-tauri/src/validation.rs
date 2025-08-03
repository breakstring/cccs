// JSON validation framework with extensible validation rules
use crate::{AppResult, AppError, ValidationResult, ValidationError};
use serde_json;
use std::collections::HashMap;

pub trait Validator {
    fn validate(&self, content: &str) -> AppResult<ValidationResult>;
    fn get_validator_name(&self) -> &'static str;
}

pub struct JsonValidator {
    rules: Vec<Box<dyn ValidationRule>>,
}

pub trait ValidationRule: Send + Sync {
    fn validate(&self, json_value: &serde_json::Value) -> Vec<ValidationError>;
    fn get_rule_name(&self) -> &'static str;
}

// JSON format validator
pub struct JsonFormatValidator;

impl Validator for JsonFormatValidator {
    fn validate(&self, content: &str) -> AppResult<ValidationResult> {
        let mut errors = Vec::new();
        
        // Try to parse as JSON
        match serde_json::from_str::<serde_json::Value>(content) {
            Ok(_) => {
                // JSON is valid
            }
            Err(e) => {
                errors.push(ValidationError {
                    line: e.line(),
                    column: e.column(),
                    message: e.to_string(),
                    error_type: "syntax".to_string(),
                });
            }
        }
        
        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
        })
    }
    
    fn get_validator_name(&self) -> &'static str {
        "json_format"
    }
}

// Rule: Must be a JSON object
pub struct ObjectRule;

impl ValidationRule for ObjectRule {
    fn validate(&self, json_value: &serde_json::Value) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        
        if !json_value.is_object() {
            errors.push(ValidationError {
                line: 1,
                column: 1,
                message: "Configuration must be a JSON object".to_string(),
                error_type: "semantic".to_string(),
            });
        }
        
        errors
    }
    
    fn get_rule_name(&self) -> &'static str {
        "object_rule"
    }
}

// Rule: Check for required fields (extensible)
pub struct RequiredFieldsRule {
    required_fields: Vec<String>,
}

impl RequiredFieldsRule {
    pub fn new(fields: Vec<String>) -> Self {
        Self {
            required_fields: fields,
        }
    }
}

impl ValidationRule for RequiredFieldsRule {
    fn validate(&self, json_value: &serde_json::Value) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        
        if let Some(obj) = json_value.as_object() {
            for field in &self.required_fields {
                if !obj.contains_key(field) {
                    errors.push(ValidationError {
                        line: 1,
                        column: 1,
                        message: format!("Required field '{}' is missing", field),
                        error_type: "semantic".to_string(),
                    });
                }
            }
        }
        
        errors
    }
    
    fn get_rule_name(&self) -> &'static str {
        "required_fields"
    }
}

// Rule: Check field types (extensible)
pub struct FieldTypeRule {
    field_types: HashMap<String, FieldType>,
}

#[derive(Clone)]
pub enum FieldType {
    String,
    Number,
    Boolean,
    Array,
    Object,
}

impl FieldTypeRule {
    pub fn new() -> Self {
        Self {
            field_types: HashMap::new(),
        }
    }
    
    pub fn add_field_type(mut self, field_name: String, field_type: FieldType) -> Self {
        self.field_types.insert(field_name, field_type);
        self
    }
}

impl ValidationRule for FieldTypeRule {
    fn validate(&self, json_value: &serde_json::Value) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        
        if let Some(obj) = json_value.as_object() {
            for (field_name, expected_type) in &self.field_types {
                if let Some(field_value) = obj.get(field_name) {
                    let type_matches = match expected_type {
                        FieldType::String => field_value.is_string(),
                        FieldType::Number => field_value.is_number(),
                        FieldType::Boolean => field_value.is_boolean(),
                        FieldType::Array => field_value.is_array(),
                        FieldType::Object => field_value.is_object(),
                    };
                    
                    if !type_matches {
                        errors.push(ValidationError {
                            line: 1,
                            column: 1,
                            message: format!("Field '{}' has incorrect type", field_name),
                            error_type: "semantic".to_string(),
                        });
                    }
                }
            }
        }
        
        errors
    }
    
    fn get_rule_name(&self) -> &'static str {
        "field_type"
    }
}

impl JsonValidator {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }
    
    pub fn with_basic_rules() -> Self {
        let mut validator = Self::new();
        validator.add_rule(Box::new(ObjectRule));
        validator
    }
    
    pub fn add_rule(&mut self, rule: Box<dyn ValidationRule>) {
        self.rules.push(rule);
    }
    
    pub fn validate(&self, content: &str) -> AppResult<ValidationResult> {
        let mut all_errors = Vec::new();
        
        // First validate JSON format
        let format_validator = JsonFormatValidator;
        let format_result = format_validator.validate(content)?;
        all_errors.extend(format_result.errors);
        
        // If JSON format is invalid, don't run semantic rules
        if !format_result.is_valid {
            return Ok(ValidationResult {
                is_valid: false,
                errors: all_errors,
            });
        }
        
        // Parse JSON for semantic validation
        let json_value = serde_json::from_str::<serde_json::Value>(content)
            .map_err(|e| AppError::ConfigError(format!("Failed to parse JSON: {}", e)))?;
        
        // Run all semantic rules
        for rule in &self.rules {
            let rule_errors = rule.validate(&json_value);
            all_errors.extend(rule_errors);
        }
        
        Ok(ValidationResult {
            is_valid: all_errors.is_empty(),
            errors: all_errors,
        })
    }
    
    pub fn get_rule_names(&self) -> Vec<&'static str> {
        self.rules.iter().map(|rule| rule.get_rule_name()).collect()
    }
}

impl Default for JsonValidator {
    fn default() -> Self {
        Self::with_basic_rules()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_format_validator_valid() {
        let validator = JsonFormatValidator;
        let result = validator.validate(r#"{"key": "value"}"#).unwrap();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_json_format_validator_invalid() {
        let validator = JsonFormatValidator;
        let result = validator.validate(r#"{"key": "value""#).unwrap();
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
        assert_eq!(result.errors[0].error_type, "syntax");
    }

    #[test]
    fn test_object_rule_valid() {
        let rule = ObjectRule;
        let json = serde_json::json!({"key": "value"});
        let errors = rule.validate(&json);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_object_rule_invalid() {
        let rule = ObjectRule;
        let json = serde_json::json!("not an object");
        let errors = rule.validate(&json);
        assert!(!errors.is_empty());
        assert_eq!(errors[0].error_type, "semantic");
    }

    #[test]
    fn test_required_fields_rule() {
        let rule = RequiredFieldsRule::new(vec!["name".to_string(), "version".to_string()]);
        
        // Valid case
        let json = serde_json::json!({"name": "test", "version": "1.0"});
        let errors = rule.validate(&json);
        assert!(errors.is_empty());
        
        // Missing field case
        let json = serde_json::json!({"name": "test"});
        let errors = rule.validate(&json);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("version"));
    }

    #[test]
    fn test_field_type_rule() {
        let rule = FieldTypeRule::new()
            .add_field_type("name".to_string(), FieldType::String)
            .add_field_type("count".to_string(), FieldType::Number);
        
        // Valid case
        let json = serde_json::json!({"name": "test", "count": 42});
        let errors = rule.validate(&json);
        assert!(errors.is_empty());
        
        // Invalid type case
        let json = serde_json::json!({"name": 123, "count": "not a number"});
        let errors = rule.validate(&json);
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_json_validator_with_rules() {
        let mut validator = JsonValidator::new();
        validator.add_rule(Box::new(ObjectRule));
        validator.add_rule(Box::new(RequiredFieldsRule::new(vec!["theme".to_string()])));
        
        // Valid case
        let result = validator.validate(r#"{"theme": "dark", "lang": "en"}"#).unwrap();
        assert!(result.is_valid);
        
        // Invalid case - missing required field
        let result = validator.validate(r#"{"lang": "en"}"#).unwrap();
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_json_validator_syntax_error() {
        let validator = JsonValidator::with_basic_rules();
        let result = validator.validate(r#"{"invalid": json"#).unwrap();
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
        assert_eq!(result.errors[0].error_type, "syntax");
    }
}