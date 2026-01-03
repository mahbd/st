use st::tree::{StackTree, TrackedBranch, RemoteMetadata};
use std::collections::HashMap;

#[test]
fn test_stack_tree_new() {
    let tree = StackTree::new("main".to_string());
    assert_eq!(tree.active_trunk, "main");
    assert_eq!(tree.list_trunks(), vec!["main"]);
    assert!(tree.get("main").is_some());
}

#[test]
fn test_add_trunk() {
    let mut tree = StackTree::new("main".to_string());
    tree.add_trunk("dev".to_string());
    
    let trunks = tree.list_trunks();
    assert!(trunks.contains(&"main".to_string()));
    assert!(trunks.contains(&"dev".to_string()));
}

#[test]
fn test_switch_trunk() {
    let mut tree = StackTree::new("main".to_string());
    tree.add_trunk("dev".to_string());
    
    assert!(tree.switch_trunk("dev").is_ok());
    assert_eq!(tree.trunk_name(), "dev");
    
    assert!(tree.switch_trunk("nonexistent").is_err());
}

#[test]
fn test_remove_trunk() {
    let mut tree = StackTree::new("main".to_string());
    tree.add_trunk("dev".to_string());
    tree.add_trunk("staging".to_string());
    
    assert!(tree.remove_trunk("main").is_err()); // Can't remove active trunk
    assert!(tree.switch_trunk("dev").is_ok());
    assert!(tree.remove_trunk("staging").is_ok());
    assert!(!tree.list_trunks().contains(&"staging".to_string()));
}

#[test]
fn test_insert_branch() {
    let mut tree = StackTree::new("main".to_string());
    let result = tree.insert("main", "abc123", "feature-1");
    
    assert!(result.is_ok());
    assert!(tree.get("feature-1").is_some());
    
    let branch = tree.get("feature-1").unwrap();
    assert_eq!(branch.parent.as_ref().unwrap(), "main");
}

#[test]
fn test_delete_branch() {
    let mut tree = StackTree::new("main".to_string());
    tree.insert("main", "abc123", "feature-1").unwrap();
    tree.insert("feature-1", "def456", "feature-2").unwrap();
    
    let result = tree.delete("feature-1");
    assert!(result.is_ok());
    assert!(tree.get("feature-1").is_none());
    
    // feature-2 should now have main as parent
    let feature2 = tree.get("feature-2").unwrap();
    assert_eq!(feature2.parent.as_ref().unwrap(), "main");
}

#[test]
fn test_branches_method() {
    let mut tree = StackTree::new("main".to_string());
    tree.insert("main", "abc123", "feature-1").unwrap();
    tree.insert("feature-1", "def456", "feature-2").unwrap();
    
    let branches = tree.branches().unwrap();
    assert_eq!(branches.len(), 3); // main, feature-1, feature-2
}

#[test]
fn test_migration_from_legacy_format() {
    let mut tree = StackTree {
        active_trunk: String::new(),
        trunk_name: Some("master".to_string()),
        trunks: HashMap::new(),
        branches: Some(HashMap::from([(
            "master".to_string(),
            TrackedBranch::new("master".to_string(), None, None),
        )])),
    };
    
    tree.migrate_if_needed();
    
    assert_eq!(tree.active_trunk, "master");
    assert!(tree.trunk_name.is_none());
    assert!(tree.branches.is_none());
    assert!(tree.trunks.contains_key("master"));
}

#[test]
fn test_tracked_branch_new() {
    let branch = TrackedBranch::new(
        "feature".to_string(),
        Some("main".to_string()),
        Some("abc123".to_string()),
    );
    
    assert_eq!(branch.name, "feature");
    assert_eq!(branch.parent.unwrap(), "main");
    assert_eq!(branch.parent_oid_cache.unwrap(), "abc123");
    assert!(branch.remote.is_none());
}

#[test]
fn test_remote_metadata() {
    let metadata = RemoteMetadata::new(42);
    assert_eq!(metadata.pr_number, 42);
    assert!(metadata.comment_id.is_none());
}

#[test]
fn test_trunk_isolation() {
    let mut tree = StackTree::new("main".to_string());
    tree.insert("main", "abc", "main-feature").unwrap();
    
    tree.add_trunk("dev".to_string());
    tree.switch_trunk("dev").unwrap();
    tree.insert("dev", "def", "dev-feature").unwrap();
    
    // dev-feature should not be visible from main trunk
    tree.switch_trunk("main").unwrap();
    assert!(tree.get("dev-feature").is_none());
    assert!(tree.get("main-feature").is_some());
    
    // main-feature should not be visible from dev trunk
    tree.switch_trunk("dev").unwrap();
    assert!(tree.get("main-feature").is_none());
    assert!(tree.get("dev-feature").is_some());
}

#[test]
fn test_multiple_children() {
    let mut tree = StackTree::new("main".to_string());
    tree.insert("main", "abc", "feature-1").unwrap();
    tree.insert("main", "abc", "feature-2").unwrap();
    tree.insert("feature-1", "def", "feature-1a").unwrap();
    tree.insert("feature-1", "def", "feature-1b").unwrap();
    
    let branches = tree.branches().unwrap();
    assert_eq!(branches.len(), 5);
    
    // Both children should have feature-1 as parent
    assert_eq!(tree.get("feature-1a").unwrap().parent.as_ref().unwrap(), "feature-1");
    assert_eq!(tree.get("feature-1b").unwrap().parent.as_ref().unwrap(), "feature-1");
}

#[test]
fn test_delete_nonexistent_branch() {
    let mut tree = StackTree::new("main".to_string());
    let result = tree.delete("nonexistent");
    assert!(result.is_err());
}
