use st::errors::StError;

#[test]
fn test_branch_not_tracked_error() {
    let err = StError::BranchNotTracked("feature-branch".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("feature-branch"));
    assert!(msg.contains("not tracked"));
}

#[test]
fn test_branch_already_tracked_error() {
    let err = StError::BranchAlreadyTracked("existing-branch".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("existing-branch"));
    assert!(msg.contains("already tracked"));
}

#[test]
fn test_cannot_delete_trunk_branch() {
    let err = StError::CannotDeleteTrunkBranch;
    let msg = format!("{}", err);
    assert!(msg.contains("Cannot delete the trunk branch"));
}

#[test]
fn test_needs_restack_error() {
    let err = StError::NeedsRestack("feature-1".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("feature-1"));
    assert!(msg.contains("restack"));
}

#[test]
fn test_working_tree_dirty() {
    let err = StError::WorkingTreeDirty;
    let msg = format!("{}", err);
    assert!(msg.contains("Working tree is dirty"));
}

#[test]
fn test_base_branch_not_on_remote() {
    let err = StError::BaseBranchNotOnRemote("main".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("main"));
    assert!(msg.contains("does not exist on remote"));
    assert!(msg.contains("git push origin"));
}

#[test]
fn test_remote_not_found_error() {
    let err = StError::RemoteNotFound("origin".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("origin"));
    assert!(msg.contains("not found"));
}

#[test]
fn test_pull_request_not_found() {
    let err = StError::PullRequestNotFound;
    let msg = format!("{}", err);
    assert!(msg.contains("Remote pull request not found"));
}

#[test]
fn test_not_a_git_repository() {
    let err = StError::NotAGitRepository;
    let msg = format!("{}", err);
    assert!(msg.contains("must be used within a git repository"));
}

#[test]
fn test_commit_message_required() {
    let err = StError::CommitMessageRequired;
    let msg = format!("{}", err);
    assert!(msg.contains("Commit message is required"));
}

#[test]
fn test_missing_parent_oid_cache() {
    let err = StError::MissingParentOidCache;
    let msg = format!("{}", err);
    assert!(msg.contains("Parent's"));
    assert!(msg.contains("cache is missing"));
}

#[test]
fn test_decoding_error() {
    let err = StError::DecodingError("invalid UTF-8".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("Decoding error"));
    assert!(msg.contains("invalid UTF-8"));
}

#[test]
fn test_git_repository_root_not_found() {
    let err = StError::GitRepositoryRootNotFound;
    let msg = format!("{}", err);
    assert!(msg.contains("Git repository root could not be found"));
}

#[test]
fn test_branch_unavailable() {
    let err = StError::BranchUnavailable;
    let msg = format!("{}", err);
    assert!(msg.contains("Branch was not found"));
}

#[test]
fn test_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let st_err: StError = io_err.into();
    let msg = format!("{}", st_err);
    assert!(msg.contains("IO error"));
}

#[test]
fn test_error_from_fmt() {
    let fmt_err = std::fmt::Error;
    let st_err: StError = fmt_err.into();
    let msg = format!("{}", st_err);
    assert!(msg.contains("write error"));
}
