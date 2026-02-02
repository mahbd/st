<h1 align="center">
   <img src="./assets/banner.png" alt="st" width="35%" align="center">
</h1>

<h4 align="center">
   Yet another tool for managing stacked PRs locally and on GitHub, built on
   <a href="https://crates.io/crates/git2"><code>libgit2</code></a>
   and
   <a href="https://crates.io/crates/octocrab"><code>octocrab</code></a>.
</h4>

<p align="center">
  <a href="https://github.com/mahbd/st/actions/workflows/rust_ci.yaml"><img src="https://github.com/mahbd/st/actions/workflows/rust_ci.yaml/badge.svg?label=ci" alt="CI"></a>
  <img src="https://img.shields.io/badge/License-Beerware-green.svg?label=license&labelColor=2a2f35" alt="License">
</p>

<p align="center">
  <a href="#installation">Installation</a> •
  <a href="#what-are-stacked-prs">What are Stacked PRs?</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#configuration">Configuration</a> •
  <a href="#commands">Commands</a> •
  <a href="#workflows">Workflows</a> •
  <a href="#contributing">Contributing</a>
</p>

---

## Installation

> [!WARNING]
> `st` was written on a weekend for my own personal use, and may not be entirely stable. You're welcome to use it
> in its current state, though don't get mad at me if the tool messes up your local tree. I'll remove this warning once
> I feel that it's stable for my own usecase.

### From Source

```sh
git clone https://github.com/mahbd/st && \
   cd st && \
   cargo install --bin st --path . --force
```

### Prerequisites

- **Rust** (1.85 or later, 2024 edition)
- **Git** (2.0 or later)
- **GitHub Personal Access Token** with the following scopes:
  - `repo:status`
  - `repo:public_repo`
  - For private repositories: full `repo` scope

---

## What are Stacked PRs?

Stacked PRs (also known as stacked diffs or dependent PRs) are a workflow where you break down large changes into smaller, incremental pull requests that build on top of each other.

**Example:**
```
main
 └── feature/auth-base       (PR #1: Add authentication types)
      └── feature/auth-api   (PR #2: Add authentication API)
           └── feature/auth-ui (PR #3: Add login UI)
```

**Benefits:**
- **Faster reviews**: Smaller PRs are easier to review
- **Parallel work**: Continue working while waiting for reviews
- **Better history**: Each PR represents a logical unit of change
- **Reduced risk**: Smaller changes are less likely to introduce bugs

For more information, visit [stacking.dev](https://www.stacking.dev/).

---

## Quick Start

### 1. Initialize Configuration

Run `st` in any git repository to set up your configuration:

```sh
cd your-repo
st log
```

On first run, you'll be prompted to configure your GitHub token and preferences.

### 2. Create Your First Stack

```sh
# Start from main branch
git checkout main
git pull

# Create first branch in the stack
st create feature/step-1
# Make changes, commit them
git add .
git commit -m "Step 1: Add base functionality"

# Create second branch stacked on first
st create feature/step-2
# Make more changes
git add .
git commit -m "Step 2: Add API endpoints"

# Create third branch
st create feature/step-3
git add .
git commit -m "Step 3: Add UI components"
```

### 3. View Your Stack

```sh
st log
```

Output:
```
◯ main
├── ● feature/step-1
│   └── ● feature/step-2
│       └── ● feature/step-3 (current)
```

### 4. Submit to GitHub

```sh
st submit
```

This will:
1. Push all branches to GitHub
2. Create PRs for each branch (with proper base branches)
3. Add navigation comments to each PR

---

## Configuration

Configuration is stored in `~/.st.toml`. You can edit it directly or use `st config`.

### Full Configuration Example

```toml
# GitHub personal access token (required)
# Get one at: https://github.com/settings/tokens
github_token = "ghp_xxxxxxxxxxxxxxxxxxxx"

# Editor for commit messages and PR descriptions
# Common options: "vim", "emacs", "nano", "code --wait"
editor = "vim"

# Google Gemini API key for AI-generated PR descriptions (optional)
# Get one at: https://aistudio.google.com/app/apikey
gemini_api_key = "your-gemini-api-key"

# PR Templates (optional)
# Define templates for different types of changes
# When submitting a PR:
#   - If you have multiple templates, you'll be prompted to choose one
#   - If you have one template, it's used automatically
#   - The AI (if configured) will use the template to structure the description

[[pr_templates]]
name = "feature"
content = """
## Summary
Brief description of the feature.

## Changes
- Change 1
- Change 2

## Testing
How was this tested?

## Screenshots (if applicable)
"""

[[pr_templates]]
name = "bugfix"
content = """
## Problem
Description of the bug being fixed.

## Root Cause
What caused the bug?

## Solution
How was it fixed?

## Testing
How was the fix verified?
"""

[[pr_templates]]
name = "refactor"
content = """
## Motivation
Why is this refactor needed?

## Changes
What was refactored?

## Impact
What areas of the codebase are affected?

## Testing
How was this tested?
"""

[[pr_templates]]
name = "docs"
content = """
## Summary
What documentation was added/updated?

## Changes
- Change 1
- Change 2
"""
```

### Getting a GitHub Token

1. Go to [GitHub Settings > Developer settings > Personal access tokens](https://github.com/settings/tokens)
2. Click "Generate new token (classic)"
3. Select scopes:
   - `repo:status`
   - `public_repo` (for public repos)
   - `repo` (for private repos)
4. Copy the token and add it to your config

### Getting a Gemini API Key (Optional)

1. Go to [Google AI Studio](https://aistudio.google.com/app/apikey)
2. Create a new API key
3. Add it to your config as `gemini_api_key`

---

## Commands

### Overview

| Command | Aliases | Description |
|---------|---------|-------------|
| `st create <name>` | `c` | Create a new branch stacked on current |
| `st submit` | `s`, `ss` | Submit stack to GitHub |
| `st log` | `l`, `ls` | Show branch tree |
| `st checkout` | `co` | Checkout a tracked branch |
| `st restack` | `r`, `sr` | Rebase branches after changes |
| `st sync` | `rs`, `sy` | Sync with remote |
| `st status` | `st`, `stat` | Show PR status on GitHub |
| `st delete` | `d`, `del` | Delete a tracked branch |
| `st track` | `tr` | Track an existing branch |
| `st untrack` | `ut` | Untrack a branch |
| `st trunk` | `t` | Manage trunk branches |
| `st config` | `cfg` | Edit configuration |

### Detailed Command Reference

#### `st create <branch-name>`

Creates a new branch and tracks it as a child of the current branch.

```sh
# Create a branch stacked on current
st create feature/new-feature

# Short alias
st c feature/new-feature
```

#### `st submit`

Pushes branches and creates/updates PRs on GitHub.

```sh
# Submit current stack
st submit

# Force push (like git push --force)
st submit --force
st submit -f

# Submit all tracked branches, not just current stack
st submit --all
st submit -a
```

**PR Creation Flow:**
1. Enter PR title
2. Select template (if multiple templates configured)
3. AI generates description (if Gemini API key configured)
4. Edit description in your editor
5. Choose if PR is a draft
6. PR is created on GitHub

#### `st log`

Displays a tree view of all tracked branches.

```sh
st log
st l
st ls
```

**Example Output:**
```
◯ main
├── ● feature/auth-base
│   ├── ● feature/auth-api
│   │   └── ● feature/auth-ui (current)
│   └── ● feature/auth-tests
└── ● feature/unrelated-change
```

Legend:
- `◯` = Trunk branch
- `●` = Tracked branch
- `(current)` = Your current branch

#### `st checkout`

Interactively checkout a tracked branch.

```sh
st checkout
st co
```

You'll see a list of all tracked branches to choose from.

#### `st restack`

Rebases all branches in the current stack to ensure they're up-to-date with their parents.

```sh
st restack
st r
```

**When to use:**
- After making changes to a parent branch
- After pulling updates from remote
- When `st` tells you branches need restacking

#### `st sync`

Syncs local branches with remote, handling merged/closed PRs.

```sh
st sync
st sy
```

**What it does:**
- Fetches from remote
- Detects merged PRs
- Offers to delete merged branches
- Updates local tracking

#### `st status`

Shows the status of PRs in your current stack.

```sh
st status
st stat
```

**Example Output:**
```
┌─────────────────────┬────────┬─────────┬───────────┐
│ Branch              │ PR #   │ Status  │ Reviews   │
├─────────────────────┼────────┼─────────┼───────────┤
│ feature/auth-base   │ #123   │ Open    │ Approved  │
│ feature/auth-api    │ #124   │ Open    │ Pending   │
│ feature/auth-ui     │ #125   │ Draft   │ -         │
└─────────────────────┴────────┴─────────┴───────────┘
```

#### `st delete <branch-name>`

Deletes a tracked branch and re-links its children to its parent.

```sh
st delete feature/old-branch
st d feature/old-branch
```

#### `st track`

Tracks an existing branch on top of the current stack.

```sh
# Checkout an untracked branch, then track it
git checkout existing-branch
st track
```

#### `st untrack <branch-name>`

Removes a branch from `st` tracking without deleting it.

```sh
st untrack feature/some-branch
st ut feature/some-branch
```

#### `st trunk`

Manages trunk (base) branches for multi-trunk support.

```sh
# List all trunk branches
st trunk list
st trunk ls

# Switch to a different trunk
st trunk switch dev
st trunk sw dev

# Add a new trunk branch
st trunk add staging

# Remove a trunk branch
st trunk remove staging
st trunk rm staging
```

**Multi-trunk example:**
```
# Trunk: main
◯ main
└── ● feature/for-main

# Trunk: dev
◯ dev
└── ● feature/for-dev
```

#### `st config`

Opens your configuration file in your editor.

```sh
st config
st cfg
```

---

## Workflows

### Basic Stacking Workflow

```sh
# 1. Start from trunk
git checkout main && git pull

# 2. Create first branch
st create feature/part-1
# ... make changes ...
git add . && git commit -m "Part 1"

# 3. Stack another branch
st create feature/part-2
# ... make changes ...
git add . && git commit -m "Part 2"

# 4. Submit all to GitHub
st submit

# 5. Check status
st status
```

### Updating a Branch Mid-Stack

When you need to make changes to a branch that has children:

```sh
# 1. Checkout the branch to update
st checkout  # Select feature/part-1

# 2. Make your changes
git add . && git commit -m "Fix based on review"

# 3. Restack to update children
st restack

# 4. Submit updates
st submit --force
```

### Handling Merged PRs

After a PR is merged on GitHub:

```sh
# 1. Sync to detect merged PRs
st sync

# 2. Follow prompts to delete merged branches

# 3. Restack remaining branches
st restack

# 4. Submit updates (bases will be updated)
st submit
```

### Working with Multiple Trunks

For projects with multiple base branches (e.g., `main` and `develop`):

```sh
# Add develop as a trunk
st trunk add develop

# Switch to develop trunk
st trunk switch develop

# Create branches based on develop
st create feature/for-develop

# Switch back to main trunk
st trunk switch main

# Branches are isolated per trunk
st log  # Only shows main-based branches
```

### Branching Stacks (Siblings)

Create sibling branches for parallel features:

```sh
# Start with a base feature
git checkout main
st create feature/base

# Create first child
st create feature/child-a

# Go back to base and create sibling
st checkout  # Select feature/base
st create feature/child-b

# Result:
# ◯ main
# └── ● feature/base
#     ├── ● feature/child-a
#     └── ● feature/child-b
```

---

## Tips & Tricks

### Aliases

Add these to your shell config for even faster access:

```sh
# ~/.bashrc or ~/.zshrc
alias sc="st create"
alias ss="st submit"
alias sl="st log"
alias sr="st restack"
alias sco="st checkout"
```

### Editor Configuration

For VS Code users:
```toml
editor = "code --wait"
```

For Neovim users:
```toml
editor = "nvim"
```

### Force Push Safely

When force pushing after restack:
```sh
st submit -f
```

This is safe because `st` only pushes branches it tracks.

### Check Before Submit

Always run `st log` before `st submit` to verify your stack looks correct.

---

## Troubleshooting

### "Branch needs to be restacked"

Run `st restack` to update branch bases after changes.

### "Working tree is dirty"

Commit or stash your changes before running `st` commands:
```sh
git stash
st restack
git stash pop
```

### "Base branch does not exist on remote"

Push the parent branch first:
```sh
git push origin parent-branch-name
st submit
```

### PR base is wrong on GitHub

This can happen if branches were manually pushed. Fix with:
```sh
st submit  # Will update PR bases automatically
```

---

## Features Summary

| Feature | Description |
|---------|-------------|
| **Stacked PRs** | Create and manage dependent PRs easily |
| **Multi-Trunk Support** | Work with multiple base branches (main, dev, etc.) |
| **AI PR Descriptions** | Generate descriptions using Google Gemini |
| **PR Templates** | Consistent PR formatting with customizable templates |
| **Stack Navigation Comments** | Auto-generated comments linking related PRs |
| **Interactive Branch Selection** | Easy checkout with fuzzy finding |
| **Automatic Restacking** | Keep branches up-to-date with parents |
| **PR Status Tracking** | View PR status without leaving terminal |

---

## Contributing

Contributions are welcome! Feel free to:
- [Submit an issue](https://github.com/mahbd/st/issues/new) for bugs or feature requests
- Open a PR with your changes
- Improve documentation

---

## License

[Beerware License](./LICENSE.md) - If you like this tool and we meet someday, you can buy me a beer!