use crate::model::RepoId;
use gitcomet_core::conflict_session::ConflictSession;
use gitcomet_core::domain::*;
use gitcomet_core::error::Error;
use gitcomet_core::services::GitRepository;
use gitcomet_core::services::{CommandOutput, ConflictSide, PullMode, RemoteUrlKind, ResetMode};
use std::path::PathBuf;
use std::sync::Arc;

use super::repo_command_kind::RepoCommandKind;
use super::repo_external_change::RepoExternalChange;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConflictAutosolveMode {
    Safe,
    Regex,
    History,
}

impl ConflictAutosolveMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Regex => "regex",
            Self::History => "history",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConflictBulkChoice {
    Base,
    Ours,
    Theirs,
    Both,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConflictRegionChoice {
    Base,
    Ours,
    Theirs,
    Both,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConflictRegionResolutionUpdate {
    pub region_index: usize,
    pub resolution: gitcomet_core::conflict_session::ConflictRegionResolution,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ConflictAutosolveStats {
    pub pass1: usize,
    pub pass2_split: usize,
    pub pass1_after_split: usize,
    pub regex: usize,
    pub history: usize,
}

impl ConflictAutosolveStats {
    pub fn total_resolved(self) -> usize {
        self.pass1 + self.pass2_split + self.pass1_after_split + self.regex + self.history
    }
}

pub enum Msg {
    OpenRepo(PathBuf),
    RestoreSession {
        open_repos: Vec<PathBuf>,
        active_repo: Option<PathBuf>,
    },
    CloseRepo {
        repo_id: RepoId,
    },
    DismissRepoError {
        repo_id: RepoId,
    },
    SetActiveRepo {
        repo_id: RepoId,
    },
    ReorderRepoTabs {
        repo_id: RepoId,
        insert_before: Option<RepoId>,
    },
    ReloadRepo {
        repo_id: RepoId,
    },
    RepoExternallyChanged {
        repo_id: RepoId,
        change: RepoExternalChange,
    },
    SetHistoryScope {
        repo_id: RepoId,
        scope: LogScope,
    },
    SetFetchPruneDeletedRemoteTrackingBranches {
        repo_id: RepoId,
        enabled: bool,
    },
    LoadMoreHistory {
        repo_id: RepoId,
    },
    SelectCommit {
        repo_id: RepoId,
        commit_id: CommitId,
    },
    ClearCommitSelection {
        repo_id: RepoId,
    },
    SelectDiff {
        repo_id: RepoId,
        target: DiffTarget,
    },
    ClearDiffSelection {
        repo_id: RepoId,
    },
    LoadStashes {
        repo_id: RepoId,
    },
    LoadConflictFile {
        repo_id: RepoId,
        path: PathBuf,
    },
    LoadReflog {
        repo_id: RepoId,
    },
    LoadFileHistory {
        repo_id: RepoId,
        path: PathBuf,
        limit: usize,
    },
    LoadBlame {
        repo_id: RepoId,
        path: PathBuf,
        rev: Option<String>,
    },
    LoadWorktrees {
        repo_id: RepoId,
    },
    LoadSubmodules {
        repo_id: RepoId,
    },
    RefreshBranches {
        repo_id: RepoId,
    },
    StageHunk {
        repo_id: RepoId,
        patch: String,
    },
    UnstageHunk {
        repo_id: RepoId,
        patch: String,
    },
    ApplyWorktreePatch {
        repo_id: RepoId,
        patch: String,
        reverse: bool,
    },
    CheckoutBranch {
        repo_id: RepoId,
        name: String,
    },
    CheckoutRemoteBranch {
        repo_id: RepoId,
        remote: String,
        branch: String,
        local_branch: String,
    },
    CheckoutCommit {
        repo_id: RepoId,
        commit_id: CommitId,
    },
    CherryPickCommit {
        repo_id: RepoId,
        commit_id: CommitId,
    },
    RevertCommit {
        repo_id: RepoId,
        commit_id: CommitId,
    },
    CreateBranch {
        repo_id: RepoId,
        name: String,
    },
    CreateBranchAndCheckout {
        repo_id: RepoId,
        name: String,
    },
    DeleteBranch {
        repo_id: RepoId,
        name: String,
    },
    ForceDeleteBranch {
        repo_id: RepoId,
        name: String,
    },
    CloneRepo {
        url: String,
        dest: PathBuf,
    },
    CloneRepoProgress {
        dest: PathBuf,
        line: String,
    },
    CloneRepoFinished {
        url: String,
        dest: PathBuf,
        result: Result<CommandOutput, Error>,
    },
    ExportPatch {
        repo_id: RepoId,
        commit_id: CommitId,
        dest: PathBuf,
    },
    ApplyPatch {
        repo_id: RepoId,
        patch: PathBuf,
    },
    AddWorktree {
        repo_id: RepoId,
        path: PathBuf,
        reference: Option<String>,
    },
    RemoveWorktree {
        repo_id: RepoId,
        path: PathBuf,
    },
    AddSubmodule {
        repo_id: RepoId,
        url: String,
        path: PathBuf,
    },
    UpdateSubmodules {
        repo_id: RepoId,
    },
    RemoveSubmodule {
        repo_id: RepoId,
        path: PathBuf,
    },
    StagePath {
        repo_id: RepoId,
        path: PathBuf,
    },
    StagePaths {
        repo_id: RepoId,
        paths: Vec<PathBuf>,
    },
    UnstagePath {
        repo_id: RepoId,
        path: PathBuf,
    },
    UnstagePaths {
        repo_id: RepoId,
        paths: Vec<PathBuf>,
    },
    DiscardWorktreeChangesPath {
        repo_id: RepoId,
        path: PathBuf,
    },
    DiscardWorktreeChangesPaths {
        repo_id: RepoId,
        paths: Vec<PathBuf>,
    },
    SaveWorktreeFile {
        repo_id: RepoId,
        path: PathBuf,
        contents: String,
        stage: bool,
    },
    Commit {
        repo_id: RepoId,
        message: String,
    },
    CommitAmend {
        repo_id: RepoId,
        message: String,
    },
    FetchAll {
        repo_id: RepoId,
    },
    PruneMergedBranches {
        repo_id: RepoId,
    },
    PruneLocalTags {
        repo_id: RepoId,
    },
    Pull {
        repo_id: RepoId,
        mode: PullMode,
    },
    PullBranch {
        repo_id: RepoId,
        remote: String,
        branch: String,
    },
    MergeRef {
        repo_id: RepoId,
        reference: String,
    },
    Push {
        repo_id: RepoId,
    },
    ForcePush {
        repo_id: RepoId,
    },
    PushSetUpstream {
        repo_id: RepoId,
        remote: String,
        branch: String,
    },
    DeleteRemoteBranch {
        repo_id: RepoId,
        remote: String,
        branch: String,
    },
    Reset {
        repo_id: RepoId,
        target: String,
        mode: ResetMode,
    },
    Rebase {
        repo_id: RepoId,
        onto: String,
    },
    RebaseContinue {
        repo_id: RepoId,
    },
    RebaseAbort {
        repo_id: RepoId,
    },
    MergeAbort {
        repo_id: RepoId,
    },
    CreateTag {
        repo_id: RepoId,
        name: String,
        target: String,
    },
    DeleteTag {
        repo_id: RepoId,
        name: String,
    },
    PushTag {
        repo_id: RepoId,
        remote: String,
        name: String,
    },
    DeleteRemoteTag {
        repo_id: RepoId,
        remote: String,
        name: String,
    },
    AddRemote {
        repo_id: RepoId,
        name: String,
        url: String,
    },
    RemoveRemote {
        repo_id: RepoId,
        name: String,
    },
    SetRemoteUrl {
        repo_id: RepoId,
        name: String,
        url: String,
        kind: RemoteUrlKind,
    },
    CheckoutConflictSide {
        repo_id: RepoId,
        path: PathBuf,
        side: ConflictSide,
    },
    AcceptConflictDeletion {
        repo_id: RepoId,
        path: PathBuf,
    },
    CheckoutConflictBase {
        repo_id: RepoId,
        path: PathBuf,
    },
    LaunchMergetool {
        repo_id: RepoId,
        path: PathBuf,
    },
    RecordConflictAutosolveTelemetry {
        repo_id: RepoId,
        path: Option<PathBuf>,
        mode: ConflictAutosolveMode,
        total_conflicts_before: usize,
        total_conflicts_after: usize,
        unresolved_before: usize,
        unresolved_after: usize,
        stats: ConflictAutosolveStats,
    },
    ConflictSetHideResolved {
        repo_id: RepoId,
        path: PathBuf,
        hide_resolved: bool,
    },
    ConflictApplyBulkChoice {
        repo_id: RepoId,
        path: PathBuf,
        choice: ConflictBulkChoice,
    },
    ConflictSetRegionChoice {
        repo_id: RepoId,
        path: PathBuf,
        region_index: usize,
        choice: ConflictRegionChoice,
    },
    ConflictSyncRegionResolutions {
        repo_id: RepoId,
        path: PathBuf,
        updates: Vec<ConflictRegionResolutionUpdate>,
    },
    ConflictApplyAutosolve {
        repo_id: RepoId,
        path: PathBuf,
        mode: ConflictAutosolveMode,
        whitespace_normalize: bool,
    },
    ConflictResetResolutions {
        repo_id: RepoId,
        path: PathBuf,
    },
    Stash {
        repo_id: RepoId,
        message: String,
        include_untracked: bool,
    },
    ApplyStash {
        repo_id: RepoId,
        index: usize,
    },
    PopStash {
        repo_id: RepoId,
        index: usize,
    },
    DropStash {
        repo_id: RepoId,
        index: usize,
    },

    RepoOpenedOk {
        repo_id: RepoId,
        spec: RepoSpec,
        repo: Arc<dyn GitRepository>,
    },
    RepoOpenedErr {
        repo_id: RepoId,
        spec: RepoSpec,
        error: Error,
    },

    BranchesLoaded {
        repo_id: RepoId,
        result: Result<Vec<Branch>, Error>,
    },
    RemotesLoaded {
        repo_id: RepoId,
        result: Result<Vec<Remote>, Error>,
    },
    RemoteBranchesLoaded {
        repo_id: RepoId,
        result: Result<Vec<RemoteBranch>, Error>,
    },
    StatusLoaded {
        repo_id: RepoId,
        result: Result<RepoStatus, Error>,
    },
    HeadBranchLoaded {
        repo_id: RepoId,
        result: Result<String, Error>,
    },
    UpstreamDivergenceLoaded {
        repo_id: RepoId,
        result: Result<Option<UpstreamDivergence>, Error>,
    },
    LogLoaded {
        repo_id: RepoId,
        scope: LogScope,
        cursor: Option<LogCursor>,
        result: Result<LogPage, Error>,
    },
    TagsLoaded {
        repo_id: RepoId,
        result: Result<Vec<Tag>, Error>,
    },
    RemoteTagsLoaded {
        repo_id: RepoId,
        result: Result<Vec<RemoteTag>, Error>,
    },
    StashesLoaded {
        repo_id: RepoId,
        result: Result<Vec<StashEntry>, Error>,
    },
    ReflogLoaded {
        repo_id: RepoId,
        result: Result<Vec<ReflogEntry>, Error>,
    },
    RebaseStateLoaded {
        repo_id: RepoId,
        result: Result<bool, Error>,
    },
    MergeCommitMessageLoaded {
        repo_id: RepoId,
        result: Result<Option<String>, Error>,
    },
    FileHistoryLoaded {
        repo_id: RepoId,
        path: PathBuf,
        result: Result<LogPage, Error>,
    },
    BlameLoaded {
        repo_id: RepoId,
        path: PathBuf,
        rev: Option<String>,
        result: Result<Vec<gitcomet_core::services::BlameLine>, Error>,
    },
    ConflictFileLoaded {
        repo_id: RepoId,
        path: PathBuf,
        result: Box<Result<Option<crate::model::ConflictFile>, Error>>,
        conflict_session: Option<ConflictSession>,
    },
    WorktreesLoaded {
        repo_id: RepoId,
        result: Result<Vec<Worktree>, Error>,
    },
    SubmodulesLoaded {
        repo_id: RepoId,
        result: Result<Vec<Submodule>, Error>,
    },

    CommitDetailsLoaded {
        repo_id: RepoId,
        commit_id: CommitId,
        result: Result<CommitDetails, Error>,
    },

    DiffLoaded {
        repo_id: RepoId,
        target: DiffTarget,
        result: Result<Diff, Error>,
    },
    DiffFileLoaded {
        repo_id: RepoId,
        target: DiffTarget,
        result: Result<Option<FileDiffText>, Error>,
    },
    DiffFileImageLoaded {
        repo_id: RepoId,
        target: DiffTarget,
        result: Result<Option<FileDiffImage>, Error>,
    },

    RepoActionFinished {
        repo_id: RepoId,
        result: Result<(), Error>,
    },
    CommitFinished {
        repo_id: RepoId,
        result: Result<(), Error>,
    },
    CommitAmendFinished {
        repo_id: RepoId,
        result: Result<(), Error>,
    },

    RepoCommandFinished {
        repo_id: RepoId,
        command: RepoCommandKind,
        result: Result<CommandOutput, Error>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitcomet_core::conflict_session::ConflictRegionResolution;
    use gitcomet_core::error::ErrorKind;

    fn repo_id() -> RepoId {
        RepoId(7)
    }

    fn path(value: &str) -> PathBuf {
        PathBuf::from(value)
    }

    fn commit(value: &str) -> CommitId {
        CommitId(value.to_string())
    }

    fn backend_error(value: &str) -> Error {
        Error::new(ErrorKind::Backend(value.to_string()))
    }

    fn working_tree_target(path_value: &str) -> DiffTarget {
        DiffTarget::WorkingTree {
            path: path(path_value),
            area: DiffArea::Unstaged,
        }
    }

    macro_rules! assert_msg_debug_contains {
        ($msg:expr, $needle:expr) => {{
            let rendered = format!("{:?}", $msg);
            assert!(
                rendered.contains($needle),
                "expected {rendered:?} to contain {:?}",
                $needle
            );
        }};
    }

    #[test]
    fn msg_debug_formats_repo_command_and_ui_action_variants() {
        let cases: Vec<(Msg, &str)> = vec![
            (Msg::OpenRepo(path("repo")), "OpenRepo"),
            (
                Msg::RestoreSession {
                    open_repos: vec![path("repo-a"), path("repo-b")],
                    active_repo: Some(path("repo-b")),
                },
                "RestoreSession",
            ),
            (Msg::CloseRepo { repo_id: repo_id() }, "CloseRepo"),
            (
                Msg::DismissRepoError { repo_id: repo_id() },
                "DismissRepoError",
            ),
            (Msg::SetActiveRepo { repo_id: repo_id() }, "SetActiveRepo"),
            (
                Msg::ReorderRepoTabs {
                    repo_id: repo_id(),
                    insert_before: Some(RepoId(9)),
                },
                "ReorderRepoTabs",
            ),
            (Msg::ReloadRepo { repo_id: repo_id() }, "ReloadRepo"),
            (
                Msg::RepoExternallyChanged {
                    repo_id: repo_id(),
                    change: RepoExternalChange::Both,
                },
                "RepoExternallyChanged",
            ),
            (
                Msg::SetHistoryScope {
                    repo_id: repo_id(),
                    scope: LogScope::AllBranches,
                },
                "SetHistoryScope",
            ),
            (
                Msg::SetFetchPruneDeletedRemoteTrackingBranches {
                    repo_id: repo_id(),
                    enabled: true,
                },
                "SetFetchPruneDeletedRemoteTrackingBranches",
            ),
            (
                Msg::LoadMoreHistory { repo_id: repo_id() },
                "LoadMoreHistory",
            ),
            (
                Msg::SelectCommit {
                    repo_id: repo_id(),
                    commit_id: commit("abc123"),
                },
                "SelectCommit",
            ),
            (
                Msg::ClearCommitSelection { repo_id: repo_id() },
                "ClearCommitSelection",
            ),
            (
                Msg::SelectDiff {
                    repo_id: repo_id(),
                    target: working_tree_target("src/lib.rs"),
                },
                "SelectDiff",
            ),
            (
                Msg::ClearDiffSelection { repo_id: repo_id() },
                "ClearDiffSelection",
            ),
            (Msg::LoadStashes { repo_id: repo_id() }, "LoadStashes"),
            (
                Msg::LoadConflictFile {
                    repo_id: repo_id(),
                    path: path("src/conflicted.rs"),
                },
                "LoadConflictFile",
            ),
            (Msg::LoadReflog { repo_id: repo_id() }, "LoadReflog"),
            (
                Msg::LoadFileHistory {
                    repo_id: repo_id(),
                    path: path("src/file.rs"),
                    limit: 25,
                },
                "LoadFileHistory",
            ),
            (
                Msg::LoadBlame {
                    repo_id: repo_id(),
                    path: path("src/file.rs"),
                    rev: Some("HEAD~1".to_string()),
                },
                "LoadBlame",
            ),
            (Msg::LoadWorktrees { repo_id: repo_id() }, "LoadWorktrees"),
            (Msg::LoadSubmodules { repo_id: repo_id() }, "LoadSubmodules"),
            (
                Msg::RefreshBranches { repo_id: repo_id() },
                "RefreshBranches",
            ),
            (
                Msg::StageHunk {
                    repo_id: repo_id(),
                    patch: "@@ -1 +1 @@\n-old\n+new\n".to_string(),
                },
                "StageHunk",
            ),
            (
                Msg::UnstageHunk {
                    repo_id: repo_id(),
                    patch: "@@ -1 +1 @@\n-old\n+new\n".to_string(),
                },
                "UnstageHunk",
            ),
            (
                Msg::ApplyWorktreePatch {
                    repo_id: repo_id(),
                    patch: "@@ -1 +1 @@\n-old\n+new\n".to_string(),
                    reverse: true,
                },
                "ApplyWorktreePatch",
            ),
            (
                Msg::CheckoutBranch {
                    repo_id: repo_id(),
                    name: "main".to_string(),
                },
                "CheckoutBranch",
            ),
            (
                Msg::CheckoutRemoteBranch {
                    repo_id: repo_id(),
                    remote: "origin".to_string(),
                    branch: "feature".to_string(),
                    local_branch: "feature".to_string(),
                },
                "CheckoutRemoteBranch",
            ),
            (
                Msg::CheckoutCommit {
                    repo_id: repo_id(),
                    commit_id: commit("deadbeef"),
                },
                "CheckoutCommit",
            ),
            (
                Msg::CherryPickCommit {
                    repo_id: repo_id(),
                    commit_id: commit("deadbeef"),
                },
                "CherryPickCommit",
            ),
            (
                Msg::RevertCommit {
                    repo_id: repo_id(),
                    commit_id: commit("deadbeef"),
                },
                "RevertCommit",
            ),
            (
                Msg::CreateBranch {
                    repo_id: repo_id(),
                    name: "topic".to_string(),
                },
                "CreateBranch",
            ),
            (
                Msg::CreateBranchAndCheckout {
                    repo_id: repo_id(),
                    name: "topic".to_string(),
                },
                "CreateBranchAndCheckout",
            ),
            (
                Msg::DeleteBranch {
                    repo_id: repo_id(),
                    name: "old-topic".to_string(),
                },
                "DeleteBranch",
            ),
            (
                Msg::ForceDeleteBranch {
                    repo_id: repo_id(),
                    name: "old-topic".to_string(),
                },
                "ForceDeleteBranch",
            ),
            (
                Msg::CloneRepo {
                    url: "https://example.test/repo.git".to_string(),
                    dest: path("/tmp/repo"),
                },
                "CloneRepo",
            ),
            (
                Msg::CloneRepoProgress {
                    dest: path("/tmp/repo"),
                    line: "cloning".to_string(),
                },
                "CloneRepoProgress",
            ),
            (
                Msg::CloneRepoFinished {
                    url: "https://example.test/repo.git".to_string(),
                    dest: path("/tmp/repo"),
                    result: Ok(CommandOutput::empty_success("git clone")),
                },
                "CloneRepoFinished",
            ),
            (
                Msg::ExportPatch {
                    repo_id: repo_id(),
                    commit_id: commit("beaded"),
                    dest: path("/tmp/change.patch"),
                },
                "ExportPatch",
            ),
            (
                Msg::ApplyPatch {
                    repo_id: repo_id(),
                    patch: path("/tmp/change.patch"),
                },
                "ApplyPatch",
            ),
            (
                Msg::AddWorktree {
                    repo_id: repo_id(),
                    path: path("../worktree-topic"),
                    reference: Some("topic".to_string()),
                },
                "AddWorktree",
            ),
            (
                Msg::RemoveWorktree {
                    repo_id: repo_id(),
                    path: path("../worktree-topic"),
                },
                "RemoveWorktree",
            ),
            (
                Msg::AddSubmodule {
                    repo_id: repo_id(),
                    url: "https://example.test/submodule.git".to_string(),
                    path: path("vendor/submodule"),
                },
                "AddSubmodule",
            ),
            (
                Msg::UpdateSubmodules { repo_id: repo_id() },
                "UpdateSubmodules",
            ),
            (
                Msg::RemoveSubmodule {
                    repo_id: repo_id(),
                    path: path("vendor/submodule"),
                },
                "RemoveSubmodule",
            ),
            (
                Msg::StagePath {
                    repo_id: repo_id(),
                    path: path("src/a.rs"),
                },
                "StagePath",
            ),
            (
                Msg::StagePaths {
                    repo_id: repo_id(),
                    paths: vec![path("src/a.rs"), path("src/b.rs")],
                },
                "StagePaths",
            ),
            (
                Msg::UnstagePath {
                    repo_id: repo_id(),
                    path: path("src/a.rs"),
                },
                "UnstagePath",
            ),
            (
                Msg::UnstagePaths {
                    repo_id: repo_id(),
                    paths: vec![path("src/a.rs"), path("src/b.rs")],
                },
                "UnstagePaths",
            ),
            (
                Msg::DiscardWorktreeChangesPath {
                    repo_id: repo_id(),
                    path: path("src/a.rs"),
                },
                "DiscardWorktreeChangesPath",
            ),
            (
                Msg::DiscardWorktreeChangesPaths {
                    repo_id: repo_id(),
                    paths: vec![path("src/a.rs"), path("src/b.rs")],
                },
                "DiscardWorktreeChangesPaths",
            ),
            (
                Msg::SaveWorktreeFile {
                    repo_id: repo_id(),
                    path: path("src/a.rs"),
                    contents: "new file contents".to_string(),
                    stage: true,
                },
                "SaveWorktreeFile",
            ),
            (
                Msg::Commit {
                    repo_id: repo_id(),
                    message: "feat: commit".to_string(),
                },
                "Commit",
            ),
            (
                Msg::CommitAmend {
                    repo_id: repo_id(),
                    message: "fixup".to_string(),
                },
                "CommitAmend",
            ),
            (Msg::FetchAll { repo_id: repo_id() }, "FetchAll"),
            (
                Msg::PruneMergedBranches { repo_id: repo_id() },
                "PruneMergedBranches",
            ),
            (Msg::PruneLocalTags { repo_id: repo_id() }, "PruneLocalTags"),
            (
                Msg::Pull {
                    repo_id: repo_id(),
                    mode: PullMode::Rebase,
                },
                "Pull",
            ),
            (
                Msg::PullBranch {
                    repo_id: repo_id(),
                    remote: "origin".to_string(),
                    branch: "main".to_string(),
                },
                "PullBranch",
            ),
            (
                Msg::MergeRef {
                    repo_id: repo_id(),
                    reference: "origin/main".to_string(),
                },
                "MergeRef",
            ),
            (Msg::Push { repo_id: repo_id() }, "Push"),
            (Msg::ForcePush { repo_id: repo_id() }, "ForcePush"),
            (
                Msg::PushSetUpstream {
                    repo_id: repo_id(),
                    remote: "origin".to_string(),
                    branch: "topic".to_string(),
                },
                "PushSetUpstream",
            ),
            (
                Msg::DeleteRemoteBranch {
                    repo_id: repo_id(),
                    remote: "origin".to_string(),
                    branch: "topic".to_string(),
                },
                "DeleteRemoteBranch",
            ),
            (
                Msg::Reset {
                    repo_id: repo_id(),
                    target: "HEAD~1".to_string(),
                    mode: ResetMode::Hard,
                },
                "Reset",
            ),
            (
                Msg::Rebase {
                    repo_id: repo_id(),
                    onto: "origin/main".to_string(),
                },
                "Rebase",
            ),
            (Msg::RebaseContinue { repo_id: repo_id() }, "RebaseContinue"),
            (Msg::RebaseAbort { repo_id: repo_id() }, "RebaseAbort"),
            (Msg::MergeAbort { repo_id: repo_id() }, "MergeAbort"),
            (
                Msg::CreateTag {
                    repo_id: repo_id(),
                    name: "v1.0.0".to_string(),
                    target: "HEAD".to_string(),
                },
                "CreateTag",
            ),
            (
                Msg::DeleteTag {
                    repo_id: repo_id(),
                    name: "v1.0.0".to_string(),
                },
                "DeleteTag",
            ),
            (
                Msg::PushTag {
                    repo_id: repo_id(),
                    remote: "origin".to_string(),
                    name: "v1.0.0".to_string(),
                },
                "PushTag",
            ),
            (
                Msg::DeleteRemoteTag {
                    repo_id: repo_id(),
                    remote: "origin".to_string(),
                    name: "v1.0.0".to_string(),
                },
                "DeleteRemoteTag",
            ),
            (
                Msg::AddRemote {
                    repo_id: repo_id(),
                    name: "origin".to_string(),
                    url: "https://example.test/repo.git".to_string(),
                },
                "AddRemote",
            ),
            (
                Msg::RemoveRemote {
                    repo_id: repo_id(),
                    name: "origin".to_string(),
                },
                "RemoveRemote",
            ),
            (
                Msg::SetRemoteUrl {
                    repo_id: repo_id(),
                    name: "origin".to_string(),
                    url: "ssh://example.test/repo.git".to_string(),
                    kind: RemoteUrlKind::Push,
                },
                "SetRemoteUrl",
            ),
            (
                Msg::CheckoutConflictSide {
                    repo_id: repo_id(),
                    path: path("src/conflict.rs"),
                    side: ConflictSide::Ours,
                },
                "CheckoutConflictSide",
            ),
            (
                Msg::AcceptConflictDeletion {
                    repo_id: repo_id(),
                    path: path("src/deleted.rs"),
                },
                "AcceptConflictDeletion",
            ),
            (
                Msg::CheckoutConflictBase {
                    repo_id: repo_id(),
                    path: path("src/conflict.rs"),
                },
                "CheckoutConflictBase",
            ),
            (
                Msg::LaunchMergetool {
                    repo_id: repo_id(),
                    path: path("src/conflict.rs"),
                },
                "LaunchMergetool",
            ),
            (
                Msg::RecordConflictAutosolveTelemetry {
                    repo_id: repo_id(),
                    path: Some(path("src/conflict.rs")),
                    mode: ConflictAutosolveMode::Safe,
                    total_conflicts_before: 5,
                    total_conflicts_after: 2,
                    unresolved_before: 3,
                    unresolved_after: 1,
                    stats: ConflictAutosolveStats {
                        pass1: 1,
                        pass2_split: 2,
                        pass1_after_split: 0,
                        regex: 0,
                        history: 0,
                    },
                },
                "RecordConflictAutosolveTelemetry",
            ),
            (
                Msg::ConflictSetHideResolved {
                    repo_id: repo_id(),
                    path: path("src/conflict.rs"),
                    hide_resolved: true,
                },
                "ConflictSetHideResolved",
            ),
            (
                Msg::ConflictApplyBulkChoice {
                    repo_id: repo_id(),
                    path: path("src/conflict.rs"),
                    choice: ConflictBulkChoice::Theirs,
                },
                "ConflictApplyBulkChoice",
            ),
            (
                Msg::ConflictSetRegionChoice {
                    repo_id: repo_id(),
                    path: path("src/conflict.rs"),
                    region_index: 2,
                    choice: ConflictRegionChoice::Both,
                },
                "ConflictSetRegionChoice",
            ),
            (
                Msg::ConflictSyncRegionResolutions {
                    repo_id: repo_id(),
                    path: path("src/conflict.rs"),
                    updates: vec![ConflictRegionResolutionUpdate {
                        region_index: 2,
                        resolution: ConflictRegionResolution::PickOurs,
                    }],
                },
                "ConflictSyncRegionResolutions",
            ),
            (
                Msg::ConflictApplyAutosolve {
                    repo_id: repo_id(),
                    path: path("src/conflict.rs"),
                    mode: ConflictAutosolveMode::Regex,
                    whitespace_normalize: true,
                },
                "ConflictApplyAutosolve",
            ),
            (
                Msg::ConflictResetResolutions {
                    repo_id: repo_id(),
                    path: path("src/conflict.rs"),
                },
                "ConflictResetResolutions",
            ),
            (
                Msg::Stash {
                    repo_id: repo_id(),
                    message: "wip".to_string(),
                    include_untracked: true,
                },
                "Stash",
            ),
            (
                Msg::ApplyStash {
                    repo_id: repo_id(),
                    index: 1,
                },
                "ApplyStash",
            ),
            (
                Msg::PopStash {
                    repo_id: repo_id(),
                    index: 2,
                },
                "PopStash",
            ),
            (
                Msg::DropStash {
                    repo_id: repo_id(),
                    index: 3,
                },
                "DropStash",
            ),
        ];

        for (msg, expected) in cases {
            assert_msg_debug_contains!(msg, expected);
        }
    }

    #[test]
    fn msg_debug_formats_loaded_and_finished_variants() {
        let cases: Vec<(Msg, &str)> = vec![
            (
                Msg::RepoOpenedErr {
                    repo_id: repo_id(),
                    spec: RepoSpec {
                        workdir: path("/tmp/repo"),
                    },
                    error: backend_error("open failed"),
                },
                "RepoOpenedErr",
            ),
            (
                Msg::BranchesLoaded {
                    repo_id: repo_id(),
                    result: Err(backend_error("branches failed")),
                },
                "BranchesLoaded",
            ),
            (
                Msg::RemotesLoaded {
                    repo_id: repo_id(),
                    result: Err(backend_error("remotes failed")),
                },
                "RemotesLoaded",
            ),
            (
                Msg::RemoteBranchesLoaded {
                    repo_id: repo_id(),
                    result: Err(backend_error("remote branches failed")),
                },
                "RemoteBranchesLoaded",
            ),
            (
                Msg::StatusLoaded {
                    repo_id: repo_id(),
                    result: Err(backend_error("status failed")),
                },
                "StatusLoaded",
            ),
            (
                Msg::HeadBranchLoaded {
                    repo_id: repo_id(),
                    result: Err(backend_error("head branch failed")),
                },
                "HeadBranchLoaded",
            ),
            (
                Msg::UpstreamDivergenceLoaded {
                    repo_id: repo_id(),
                    result: Err(backend_error("divergence failed")),
                },
                "UpstreamDivergenceLoaded",
            ),
            (
                Msg::LogLoaded {
                    repo_id: repo_id(),
                    scope: LogScope::CurrentBranch,
                    cursor: None,
                    result: Err(backend_error("log failed")),
                },
                "LogLoaded",
            ),
            (
                Msg::TagsLoaded {
                    repo_id: repo_id(),
                    result: Err(backend_error("tags failed")),
                },
                "TagsLoaded",
            ),
            (
                Msg::RemoteTagsLoaded {
                    repo_id: repo_id(),
                    result: Err(backend_error("remote tags failed")),
                },
                "RemoteTagsLoaded",
            ),
            (
                Msg::StashesLoaded {
                    repo_id: repo_id(),
                    result: Err(backend_error("stashes failed")),
                },
                "StashesLoaded",
            ),
            (
                Msg::ReflogLoaded {
                    repo_id: repo_id(),
                    result: Err(backend_error("reflog failed")),
                },
                "ReflogLoaded",
            ),
            (
                Msg::RebaseStateLoaded {
                    repo_id: repo_id(),
                    result: Err(backend_error("rebase state failed")),
                },
                "RebaseStateLoaded",
            ),
            (
                Msg::MergeCommitMessageLoaded {
                    repo_id: repo_id(),
                    result: Err(backend_error("merge message failed")),
                },
                "MergeCommitMessageLoaded",
            ),
            (
                Msg::FileHistoryLoaded {
                    repo_id: repo_id(),
                    path: path("src/history.rs"),
                    result: Err(backend_error("file history failed")),
                },
                "FileHistoryLoaded",
            ),
            (
                Msg::BlameLoaded {
                    repo_id: repo_id(),
                    path: path("src/blame.rs"),
                    rev: Some("HEAD~2".to_string()),
                    result: Err(backend_error("blame failed")),
                },
                "BlameLoaded",
            ),
            (
                Msg::ConflictFileLoaded {
                    repo_id: repo_id(),
                    path: path("src/conflict.rs"),
                    result: Box::new(Err(backend_error("conflict file failed"))),
                    conflict_session: None,
                },
                "ConflictFileLoaded",
            ),
            (
                Msg::WorktreesLoaded {
                    repo_id: repo_id(),
                    result: Err(backend_error("worktrees failed")),
                },
                "WorktreesLoaded",
            ),
            (
                Msg::SubmodulesLoaded {
                    repo_id: repo_id(),
                    result: Err(backend_error("submodules failed")),
                },
                "SubmodulesLoaded",
            ),
            (
                Msg::CommitDetailsLoaded {
                    repo_id: repo_id(),
                    commit_id: commit("1234abcd"),
                    result: Err(backend_error("commit details failed")),
                },
                "CommitDetailsLoaded",
            ),
            (
                Msg::DiffLoaded {
                    repo_id: repo_id(),
                    target: working_tree_target("src/lib.rs"),
                    result: Err(backend_error("diff failed")),
                },
                "DiffLoaded",
            ),
            (
                Msg::DiffFileLoaded {
                    repo_id: repo_id(),
                    target: working_tree_target("src/lib.rs"),
                    result: Err(backend_error("file diff failed")),
                },
                "DiffFileLoaded",
            ),
            (
                Msg::DiffFileImageLoaded {
                    repo_id: repo_id(),
                    target: working_tree_target("assets/icon.png"),
                    result: Err(backend_error("image diff failed")),
                },
                "DiffFileImageLoaded",
            ),
            (
                Msg::RepoActionFinished {
                    repo_id: repo_id(),
                    result: Err(backend_error("repo action failed")),
                },
                "RepoActionFinished",
            ),
            (
                Msg::CommitFinished {
                    repo_id: repo_id(),
                    result: Err(backend_error("commit failed")),
                },
                "CommitFinished",
            ),
            (
                Msg::CommitAmendFinished {
                    repo_id: repo_id(),
                    result: Err(backend_error("amend failed")),
                },
                "CommitAmendFinished",
            ),
            (
                Msg::RepoCommandFinished {
                    repo_id: repo_id(),
                    command: RepoCommandKind::FetchAll,
                    result: Err(backend_error("command failed")),
                },
                "RepoCommandFinished",
            ),
        ];

        for (msg, expected) in cases {
            assert_msg_debug_contains!(msg, expected);
        }
    }

    #[test]
    fn msg_debug_redacts_large_text_fields_to_lengths() {
        let stage = format!(
            "{:?}",
            Msg::StageHunk {
                repo_id: repo_id(),
                patch: "sensitive patch body".to_string(),
            }
        );
        assert!(stage.contains("patch_len"));
        assert!(!stage.contains("sensitive patch body"));

        let save = format!(
            "{:?}",
            Msg::SaveWorktreeFile {
                repo_id: repo_id(),
                path: path("src/file.rs"),
                contents: "secret data".to_string(),
                stage: false,
            }
        );
        assert!(save.contains("contents_len"));
        assert!(!save.contains("secret data"));

        let clone_finished = format!(
            "{:?}",
            Msg::CloneRepoFinished {
                url: "https://example.test/repo.git".to_string(),
                dest: path("/tmp/repo"),
                result: Err(Error::new(ErrorKind::Backend("network down".to_string()))),
            }
        );
        assert!(clone_finished.contains("ok: false"));
        assert!(!clone_finished.contains("network down"));
    }
}
