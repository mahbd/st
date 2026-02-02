//! AI-powered utilities using Google Gemini API.

use crate::config::PrTemplate;
use crate::errors::StResult;

/// Builds the commit section string for prompts.
fn build_commits_section(commits: &[String]) -> String {
    if !commits.is_empty() {
        let commit_list = commits
            .iter()
            .map(|c| format!("- {}", c))
            .collect::<Vec<_>>()
            .join("\n");
        format!("\n\nCommit messages:\n{}\n", commit_list)
    } else {
        String::new()
    }
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
    let commits_section = build_commits_section(commits);

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
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "No text in Gemini response")
        })?;

    Ok(text.trim().to_string())
}

/// Generates a PR description using a template with Google Gemini API.
///
/// ## Takes
/// - `api_key` - The Gemini API key
/// - `template` - The PR template to use
/// - `title` - The PR title
/// - `branch_name` - The name of the branch
/// - `parent_name` - The name of the parent branch
/// - `commits` - The commit messages in the branch
/// - `diff` - The git diff between the branches
///
/// ## Returns
/// - `Result<String>` - The generated PR description
pub async fn generate_pr_description_with_template_gemini(
    api_key: &str,
    template: &PrTemplate,
    title: &str,
    branch_name: &str,
    parent_name: &str,
    commits: &[String],
    diff: &str,
) -> StResult<String> {
    let commits_section = build_commits_section(commits);

    let prompt = format!(
        r#"You are a technical writer creating a pull request description using a specific template.

PR Title: {}
Branch: {} -> {}{}

Git diff:
```
{}
```

Template to follow ("{}"):
```
{}
```

Write a pull request description following the template structure above. Requirements:
- Follow the template structure exactly
- Fill in each section with relevant content from the PR context
- Maximum 400 words
- Use markdown formatting
- The description should align with and expand upon the PR title
- Focus on WHAT changed and WHY (use commit messages for context)
- Be specific and technical
- Do NOT repeat the PR title in the description
- Do NOT include the template name in the output

Generate the description now:"#,
        title, branch_name, parent_name, commits_section, diff, template.name, template.content
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
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "No text in Gemini response")
        })?;

    Ok(text.trim().to_string())
}