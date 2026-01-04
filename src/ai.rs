//! AI-powered utilities using local Ollama models and Google Gemini API.

use crate::errors::StResult;
use ollama_rs::{
    generation::completion::request::GenerationRequest, Ollama,
};
use std::process::Command;

/// Checks if Ollama is installed and running.
pub async fn is_ollama_available() -> bool {
    // Check if ollama command exists
    if Command::new("ollama").arg("--version").output().is_err() {
        return false;
    }

    // Try to connect to ollama API
    let ollama = Ollama::default();
    ollama.list_local_models().await.is_ok()
}

/// Lists available Ollama models.
pub async fn list_models() -> StResult<Vec<String>> {
    let ollama = Ollama::default();
    let models = ollama.list_local_models().await?;
    Ok(models.iter().map(|m| m.name.clone()).collect())
}

/// Generates a PR description using the specified Ollama model.
///
/// ## Takes
/// - `model` - The name of the Ollama model to use
/// - `title` - The PR title
/// - `branch_name` - The name of the branch
/// - `parent_name` - The name of the parent branch
/// - `commits` - The commit messages in the branch
/// - `diff` - The git diff between the branches
///
/// ## Returns
/// - `Result<String>` - The generated PR description
pub async fn generate_pr_description(
    model: &str,
    title: &str,
    branch_name: &str,
    parent_name: &str,
    commits: &[String],
    diff: &str,
) -> StResult<String> {
    let ollama = Ollama::default();

    let commits_section = if !commits.is_empty() {
        let commit_list = commits
            .iter()
            .map(|c| format!("- {}", c))
            .collect::<Vec<_>>()
            .join("\n");
        format!("\n\nCommit messages:\n{}\n", commit_list)
    } else {
        String::new()
    };

    let prompt = format!(
        r#"You are a technical writer creating a pull request description.

PR Title: {}
Branch: {} -> {}{}

Git diff:
```
{}
```

Write a concise pull request description in markdown format. Requirements:
- Maximum 300 words
- Use markdown formatting (headers, lists, code blocks, etc.)
- The description should align with and expand upon the PR title
- Focus on WHAT changed and WHY (use commit messages for context)
- Include: summary, key changes, and any relevant context
- Be specific and technical
- Do NOT repeat the PR title in the description

Generate the description now:"#,
        title, branch_name, parent_name, commits_section, diff
    );

    let request = GenerationRequest::new(model.to_string(), prompt);
    let response = ollama.generate(request).await?;

    Ok(response.response.trim().to_string())
}

/// Generates a PR description using Google Gemini API.
///
/// ## Takes
/// - `api_key` - The Gemini API key
/// - `title` - The PR title
/// - `branch_name` - The name of the branch
/// - `parent_name` - The name of the parent branch
/// - `commits` - The commit messages in the branch
/// - `diff` - The git diff between the branches
///
/// ## Returns
/// - `Result<String>` - The generated PR description
pub async fn generate_pr_description_with_gemini(
    api_key: &str,
    title: &str,
    branch_name: &str,
    parent_name: &str,
    commits: &[String],
    diff: &str,
) -> StResult<String> {
    let commits_section = if !commits.is_empty() {
        let commit_list = commits
            .iter()
            .map(|c| format!("- {}", c))
            .collect::<Vec<_>>()
            .join("\n");
        format!("\n\nCommit messages:\n{}\n", commit_list)
    } else {
        String::new()
    };

    let prompt = format!(
        r#"You are a technical writer creating a pull request description.

PR Title: {}
Branch: {} -> {}{}

Git diff:
```
{}
```

Write a concise pull request description in markdown format. Requirements:
- Maximum 300 words
- Use markdown formatting (headers, lists, code blocks, etc.)
- The description should align with and expand upon the PR title
- Focus on WHAT changed and WHY (use commit messages for context)
- Include: summary, key changes, and any relevant context
- Be specific and technical
- Do NOT repeat the PR title in the description

Generate the description now:"#,
        title, branch_name, parent_name, commits_section, diff
    );

    // Build the request body for Gemini API
    let request_body = serde_json::json!({
        "contents": [{
            "role": "user",
            "parts": [{
                "text": prompt
            }]
        }],
        "generationConfig": {
            "thinkingConfig": {
                "thinkingBudget": 0
            }
        }
    });

    // Call Gemini API
    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-flash-lite-latest:generateContent?key={}",
        api_key
    );

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    let response_json: serde_json::Value = response.json().await?;

    // Extract the text from the response
    let text = response_json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No text in Gemini response"
        ))?;

    Ok(text.trim().to_string())
}

