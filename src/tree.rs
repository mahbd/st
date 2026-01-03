//! Structured, [Serialize] + [Deserialize] representation of a stack of branches.

use crate::errors::{StError, StResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A simple n-nary tree of branches, with bidirectional references.
///
/// By itself, [StackTree] has no context of its relationship with the local repository. For this functionality,
/// [StContext] holds onto both the [StackTree] and the [Repository] to make informed decisions about the tree.
///
/// [StContext]: crate::ctx::StContext
/// [Repository]: git2::Repository
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StackTree {
    /// The name of the active trunk branch.
    #[serde(default)]
    pub active_trunk: String,
    /// Legacy field for backward compatibility. If present, migrated to trunks on load.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trunk_name: Option<String>,
    /// Map of trunk names to their branch trees.
    #[serde(default)]
    pub trunks: HashMap<String, TrunkBranches>,
    /// Legacy branches field for backward compatibility.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branches: Option<HashMap<String, TrackedBranch>>,
}

/// Branches associated with a specific trunk.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TrunkBranches {
    /// The trunk branch name.
    pub name: String,
    /// Map of branch names to tracked branches.
    pub branches: HashMap<String, TrackedBranch>,
}

impl Default for StackTree {
    fn default() -> Self {
        Self {
            active_trunk: String::new(),
            trunk_name: None,
            trunks: HashMap::new(),
            branches: None,
        }
    }
}

impl StackTree {
    /// Creates a new [StackTree] with the given trunk branch name.
    pub fn new(trunk_name: String) -> Self {
        let mut tree = Self::default();
        tree.add_trunk(trunk_name.clone());
        tree.active_trunk = trunk_name;
        tree
    }

    /// Migrates legacy format to new multi-trunk format.
    pub fn migrate_if_needed(&mut self) {
        if let (Some(trunk_name), Some(branches)) = (self.trunk_name.take(), self.branches.take()) {
            // Migrate from old format
            self.trunks.insert(
                trunk_name.clone(),
                TrunkBranches {
                    name: trunk_name.clone(),
                    branches,
                },
            );
            self.active_trunk = trunk_name;
        }
    }

    /// Adds a new trunk branch.
    pub fn add_trunk(&mut self, trunk_name: String) {
        if !self.trunks.contains_key(&trunk_name) {
            let branches = HashMap::from([(
                trunk_name.clone(),
                TrackedBranch::new(trunk_name.clone(), None, None),
            )]);
            self.trunks.insert(
                trunk_name.clone(),
                TrunkBranches {
                    name: trunk_name.clone(),
                    branches,
                },
            );
        }
    }

    /// Lists all trunk names.
    pub fn list_trunks(&self) -> Vec<String> {
        self.trunks.keys().cloned().collect()
    }

    /// Switches to a different trunk.
    pub fn switch_trunk(&mut self, trunk_name: &str) -> StResult<()> {
        if !self.trunks.contains_key(trunk_name) {
            return Err(StError::BranchNotTracked(format!(
                "Trunk '{}' not found",
                trunk_name
            )));
        }
        self.active_trunk = trunk_name.to_string();
        Ok(())
    }

    /// Removes a trunk and all its branches.
    pub fn remove_trunk(&mut self, trunk_name: &str) -> StResult<()> {
        if trunk_name == self.active_trunk {
            return Err(StError::BranchNotTracked(
                "Cannot remove active trunk. Switch to another trunk first.".to_string(),
            ));
        }
        if !self.trunks.contains_key(trunk_name) {
            return Err(StError::BranchNotTracked(format!(
                "Trunk '{}' not found",
                trunk_name
            )));
        }
        self.trunks.remove(trunk_name);
        Ok(())
    }

    /// Gets the current trunk name.
    pub fn trunk_name(&self) -> &str {
        &self.active_trunk
    }

    /// Gets branches for the active trunk.
    fn active_branches(&self) -> &HashMap<String, TrackedBranch> {
        use std::sync::OnceLock;
        static EMPTY: OnceLock<HashMap<String, TrackedBranch>> = OnceLock::new();
        
        self.trunks
            .get(&self.active_trunk)
            .map(|t| &t.branches)
            .unwrap_or_else(|| EMPTY.get_or_init(HashMap::new))
    }

    /// Gets mutable branches for the active trunk.
    fn active_branches_mut(&mut self) -> &mut HashMap<String, TrackedBranch> {
        self.trunks
            .get_mut(&self.active_trunk)
            .map(|t| &mut t.branches)
            .expect("Active trunk must exist")
    }

    /// Gets a branch by name from the stack graph.
    ///
    /// ## Takes
    /// - `branch_name` - The name of the branch to get.
    ///
    /// ## Returns
    /// - `Some(branch)` - The branch.
    /// - `None` - The branch by the name of `branch_name` was not found.
    pub fn get(&self, branch_name: &str) -> Option<&TrackedBranch> {
        self.active_branches().get(branch_name)
    }

    /// Gets a mutable branch by name from the stack graph.
    ///
    /// ## Takes
    /// - `branch_name` - The name of the branch to get.
    ///
    /// ## Returns
    /// - `Some(branch)` - The branch.
    /// - `None` - The branch by the name of `branch_name` was not found.
    pub fn get_mut(&mut self, branch_name: &str) -> Option<&mut TrackedBranch> {
        self.active_branches_mut().get_mut(branch_name)
    }

    /// Adds a child branch to the passed parent branch, if it exists.
    ///
    /// ## Takes
    /// - `parent` - The name of the parent branch.
    /// - `parent_oid_cache` - The [git2::Oid] cache of the parent branch.
    /// - `branch` - The name of the child branch.
    ///
    /// ## Returns
    /// - `Ok(()` if the child branch was successfully added.)`
    /// - `Err(_)` if the parent branch does not exist.
    pub fn insert(
        &mut self,
        parent_name: &str,
        parent_oid_cache: &str,
        branch_name: &str,
    ) -> StResult<()> {
        // Get the parent branch.
        let branches = self.active_branches_mut();
        let parent = branches
            .get_mut(parent_name)
            .ok_or_else(|| StError::BranchNotTracked(parent_name.to_string()))?;

        // Register the child branch with the parent.
        parent.children.insert(branch_name.to_string());

        // Create the child branch.
        let child = TrackedBranch::new(
            branch_name.to_string(),
            Some(parent_name.to_string()),
            Some(parent_oid_cache.to_string()),
        );
        
        let branches = self.active_branches_mut();
        branches.insert(branch_name.to_string(), child);

        Ok(())
    }

    /// Deletes a branch from the stack graph. If the branch does not exist, returns [None].
    ///
    /// ## Takes
    /// - `branch` - The name of the branch to delete.
    ///
    /// ## Returns
    /// - `Some(branch)` - The deleted branch.
    /// - `None` - The branch by the name of `branch` was not found.
    pub fn delete(&mut self, branch_name: &str) -> StResult<TrackedBranch> {
        // Remove the branch from the stack tree.
        let branches = self.active_branches_mut();
        let branch = branches
            .remove(branch_name)
            .ok_or_else(|| StError::BranchNotTracked(branch_name.to_string()))?;

        // Remove the child from the parent's children list.
        if let Some(ref parent_name) = branch.parent {
            let branches = self.active_branches_mut();
            let parent_branch = branches
                .get_mut(parent_name)
                .ok_or_else(|| StError::BranchNotTracked(parent_name.to_string()))?;
            parent_branch.children.remove(branch_name);

            // Re-link the children of the deleted branch to the parent.
            let children = branch.children.clone();
            for child_name in children.iter() {
                // Change the pointer of the child to the parent.
                let branches = self.active_branches_mut();
                let child = branches
                    .get_mut(child_name)
                    .ok_or_else(|| StError::BranchNotTracked(child_name.to_string()))?;
                child.parent = branch.parent.clone();

                // Add the child to the parent's children list.
                let branches = self.active_branches_mut();
                let parent = branches
                    .get_mut(parent_name)
                    .ok_or_else(|| StError::BranchNotTracked(parent_name.to_string()))?;
                parent.children.insert(child_name.clone());
            }
        }

        Ok(branch)
    }

    /// Returns a vector of branch names in the stack graph. The vector is filled recursively, meaning that children are
    /// guaranteed to be listed after their parents.
    pub fn branches(&self) -> StResult<Vec<String>> {
        let mut branch_names = Vec::new();
        self.fill_branches(self.trunk_name(), &mut branch_names)?;
        Ok(branch_names)
    }

    /// Fills a vector with the trunk branch and its children. The resulting vector is filled recursively, meaning that
    /// children are guaranteed to be listed after their parents.
    fn fill_branches(&self, name: &str, branch_names: &mut Vec<String>) -> StResult<()> {
        let current = self
            .active_branches()
            .get(name)
            .ok_or_else(|| StError::BranchNotTracked(name.to_string()))?;

        branch_names.push(current.name.clone());
        current
            .children
            .iter()
            .try_for_each(|child| self.fill_branches(child, branch_names))
    }
}

/// A local branch tracked by `st`.
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TrackedBranch {
    /// The branch name.
    pub name: String,
    /// The parent branch's [git2::Oid] cache, in string form.
    ///
    /// Invalid iff the parent branch's `HEAD` commit is not equal to the [git2::Oid] cache.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_oid_cache: Option<String>,
    /// The index of the parent branch in the stack graph.
    ///
    /// [None] if the branch is trunk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// The index of the child branches within the stack graph.
    pub children: HashSet<String>,
    /// The [RemoteMetadata] for the branch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<RemoteMetadata>,
}

impl TrackedBranch {
    /// Creates a new [TrackedBranch] with the given local metadata and parent branch name.
    ///
    /// Upon local instantiation, the branch has children or remote metadata.
    pub fn new(
        branch_name: String,
        parent: Option<String>,
        parent_oid_cache: Option<String>,
    ) -> Self {
        Self {
            name: branch_name,
            parent,
            parent_oid_cache,
            ..Default::default()
        }
    }
}

/// Remote metadata for a branch that is tracked by `st`.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RemoteMetadata {
    /// The number of the pull request on GitHub associated with the branch.
    pub(crate) pr_number: u64,
    /// The comment ID of the stack status comment on the pull request.
    ///
    /// This is used to update the comment with the latest stack status each time the stack
    /// is submitted.
    pub(crate) comment_id: Option<u64>,
}

impl RemoteMetadata {
    /// Creates a new [RemoteMetadata] with the given PR number and comment ID.
    pub fn new(pr_number: u64) -> Self {
        Self {
            pr_number,
            comment_id: None,
        }
    }
}
