use st::config::StConfig;

#[test]
fn test_config_defaults() {
    let config = StConfig::default();
    assert_eq!(config.github_token, "");
    assert_eq!(config.editor, ""); // Default derive doesn't apply serde defaults
    assert_eq!(config.ollama_model, "");
    assert_eq!(config.gemini_api_key, "");
}

#[test]
fn test_config_serialization() {
    let config = StConfig {
        github_token: "test_token".to_string(),
        editor: "vim".to_string(),
        ollama_model: "codellama".to_string(),
        gemini_api_key: "test_gemini_key".to_string(),
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
        ollama_model: "llama2".to_string(),
        gemini_api_key: "".to_string(),
    };
    assert!(valid_config.validate().is_ok());
}

#[test]
fn test_config_validate_invalid() {
    let invalid_config = StConfig {
        github_token: "".to_string(),
        editor: "vim".to_string(),
        ollama_model: "llama2".to_string(),
        gemini_api_key: "test_key".to_string(),
    };
    assert!(invalid_config.validate().is_err());
}

#[test]
fn test_config_roundtrip() {
    let original = StConfig {
        github_token: "ghp_abc123".to_string(),
        editor: "emacs".to_string(),
        ollama_model: "mistral".to_string(),
        gemini_api_key: "gemini_test_key".to_string(),
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
    assert_eq!(config.ollama_model, ""); // skip_serializing_if means empty string default
    assert_eq!(config.gemini_api_key, ""); // skip_serializing_if means empty string default
}

#[test]
fn test_config_all_fields() {
    let toml_str = r#"
        github_token = "ghp_test"
        editor = "code"
        ollama_model = "codellama"
        gemini_api_key = "test_gemini_api_key"
    "#;
    
    let config: StConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.github_token, "ghp_test");
    assert_eq!(config.editor, "code");
    assert_eq!(config.ollama_model, "codellama");
    assert_eq!(config.gemini_api_key, "test_gemini_api_key");
}

#[test]
fn test_config_gemini_only() {
    let config = StConfig {
        github_token: "ghp_test".to_string(),
        editor: "nano".to_string(),
        ollama_model: "".to_string(),
        gemini_api_key: "gemini_key_123".to_string(),
    };
    
    assert!(config.validate().is_ok());
    assert_eq!(config.gemini_api_key, "gemini_key_123");
}

#[test]
fn test_config_ollama_only() {
    let config = StConfig {
        github_token: "ghp_test".to_string(),
        editor: "nano".to_string(),
        ollama_model: "llama2".to_string(),
        gemini_api_key: "".to_string(),
    };
    
    assert!(config.validate().is_ok());
    assert_eq!(config.ollama_model, "llama2");
}

#[test]
fn test_config_both_ai_services() {
    let config = StConfig {
        github_token: "ghp_test".to_string(),
        editor: "vim".to_string(),
        ollama_model: "codellama".to_string(),
        gemini_api_key: "gemini_key".to_string(),
    };
    
    assert!(config.validate().is_ok());
    assert_eq!(config.ollama_model, "codellama");
    assert_eq!(config.gemini_api_key, "gemini_key");
}
