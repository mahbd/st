use st::config::{PrTemplate, StConfig};

#[test]
fn test_config_defaults() {
    let config = StConfig::default();
    assert_eq!(config.github_token, "");
    assert_eq!(config.editor, ""); // Default derive doesn't apply serde defaults
    assert_eq!(config.gemini_api_key, "");
    assert!(config.pr_templates.is_empty());
}

#[test]
fn test_config_serialization() {
    let config = StConfig {
        github_token: "test_token".to_string(),
        editor: "vim".to_string(),
        gemini_api_key: "test_gemini_key".to_string(),
        pr_templates: vec![],
    };
    
    let serialized = toml::to_string(&config).unwrap();
    let deserialized: StConfig = toml::from_str(&serialized).unwrap();
    
    assert_eq!(config, deserialized);
}

#[test]
fn test_config_validate_valid() {
    let valid_config = StConfig {
        github_token: "ghp_test123".to_string(),
        editor: "vim".to_string(),
        gemini_api_key: "".to_string(),
        pr_templates: vec![],
    };
    assert!(valid_config.validate().is_ok());
}

#[test]
fn test_config_validate_invalid() {
    let invalid_config = StConfig {
        github_token: "".to_string(),
        editor: "vim".to_string(),
        gemini_api_key: "test_key".to_string(),
        pr_templates: vec![],
    };
    assert!(invalid_config.validate().is_err());
}

#[test]
fn test_config_roundtrip() {
    let original = StConfig {
        github_token: "ghp_abc123".to_string(),
        editor: "emacs".to_string(),
        gemini_api_key: "gemini_test_key".to_string(),
        pr_templates: vec![
            PrTemplate {
                name: "feature".to_string(),
                content: "## Summary\nDescription here.".to_string(),
            },
        ],
    };
    
    let toml_str = toml::to_string_pretty(&original).unwrap();
    let parsed: StConfig = toml::from_str(&toml_str).unwrap();
    
    assert_eq!(original, parsed);
}

#[test]
fn test_config_partial_deserialization() {
    // Test that config can deserialize with missing optional fields
    let toml_str = r#"
        github_token = "test_token"
    "#;
    
    let config: StConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.github_token, "test_token");
    assert_eq!(config.editor, "nano"); // serde default applied during deserialization
    assert_eq!(config.gemini_api_key, ""); // skip_serializing_if means empty string default
    assert!(config.pr_templates.is_empty());
}

#[test]
fn test_config_all_fields() {
    let toml_str = r#"
        github_token = "ghp_test"
        editor = "code"
        gemini_api_key = "test_gemini_api_key"
    "#;
    
    let config: StConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.github_token, "ghp_test");
    assert_eq!(config.editor, "code");
    assert_eq!(config.gemini_api_key, "test_gemini_api_key");
}

#[test]
fn test_config_gemini_only() {
    let config = StConfig {
        github_token: "ghp_test".to_string(),
        editor: "nano".to_string(),
        gemini_api_key: "gemini_key_123".to_string(),
        pr_templates: vec![],
    };
    
    assert!(config.validate().is_ok());
    assert_eq!(config.gemini_api_key, "gemini_key_123");
}

#[test]
fn test_config_with_templates() {
    let config = StConfig {
        github_token: "ghp_test".to_string(),
        editor: "nano".to_string(),
        gemini_api_key: "".to_string(),
        pr_templates: vec![
            PrTemplate {
                name: "feature".to_string(),
                content: "## Summary\nFeature description.".to_string(),
            },
            PrTemplate {
                name: "bugfix".to_string(),
                content: "## Problem\nBug description.".to_string(),
            },
        ],
    };
    
    assert!(config.validate().is_ok());
    assert_eq!(config.template_names(), vec!["feature", "bugfix"]);
    assert!(config.get_template("feature").is_some());
    assert!(config.get_template("bugfix").is_some());
    assert!(config.get_template("nonexistent").is_none());
}

#[test]
fn test_config_template_serialization() {
    let config = StConfig {
        github_token: "ghp_test".to_string(),
        editor: "vim".to_string(),
        gemini_api_key: "gemini_key".to_string(),
        pr_templates: vec![
            PrTemplate {
                name: "refactor".to_string(),
                content: "## Motivation\nWhy refactor?\n\n## Changes\nWhat changed?".to_string(),
            },
        ],
    };
    
    let serialized = toml::to_string_pretty(&config).unwrap();
    let deserialized: StConfig = toml::from_str(&serialized).unwrap();
    
    assert_eq!(config, deserialized);
    assert_eq!(deserialized.pr_templates.len(), 1);
    assert_eq!(deserialized.pr_templates[0].name, "refactor");
}

#[test]
fn test_config_no_templates() {
    let config = StConfig {
        github_token: "ghp_test".to_string(),
        editor: "nano".to_string(),
        gemini_api_key: "key".to_string(),
        pr_templates: vec![],
    };
    
    assert!(config.template_names().is_empty());
    assert!(config.get_template("any").is_none());
}