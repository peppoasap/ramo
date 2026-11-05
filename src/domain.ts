export interface Repository {
  id: string;
  name: string;
  path: string;
  trusted: boolean;
}
export interface FileChange {
  path: string;
  original: string | null;
  index: string;
  worktree: string;
}
export interface Branch {
  name: string;
  oid: string;
  remote: boolean;
}
export interface Worktree {
  path: string;
  branch: string;
  locked: boolean;
  primary: boolean;
  head: string;
}
export interface Commit {
  oid: string;
  parents: string[];
  author: string;
  date: string;
  subject: string;
  refs: string;
}
export interface Stash {
  index: number;
  oid: string;
  reference: string;
  message: string;
}
export interface StashDetails {
  stash: Stash;
  tracked: FileChange[];
  untracked: FileChange[];
}
export interface Snapshot {
  repo: Repository;
  branch: string;
  head: string | null;
  upstream: string | null;
  ahead: number | null;
  behind: number | null;
  files: FileChange[];
  branches: Branch[];
  worktrees: Worktree[];
  stashes: Stash[];
  fingerprint: string;
  operation: string | null;
}
export type Action =
  | { type: "worktreeLock"; path: string; locked: boolean }
  | { type: "removeWorktree"; path: string }
  | { type: "restoreStashFile"; index: number; oid: string; path: string }
  | {
      type: "createWorktree";
      selectionId: string;
      directory: string;
      branch: string;
      createBranch: boolean;
    }
  | {
      type: "hunk";
      path: string;
      index: number;
      unstage: boolean;
      patchHash: string;
    }
  | { type: "stage" | "unstage"; paths: string[] }
  | { type: "commit"; message: string }
  | { type: "amend"; head: string; message: string }
  | { type: "configureRemote"; name: string; url: string; existing: boolean }
  | { type: "markResolved"; path: string }
  | { type: "resolveFile"; path: string; content: string | null }
  | { type: "abortMerge" }
  | {
      type: "sequence";
      operation: "cherryPick" | "revert";
      step: "continue" | "abort" | "skip";
    }
  | { type: "switch" | "createBranch" | "deleteBranch" | "merge"; name: string }
  | { type: "createBranchAt" | "createTag"; name: string; target: string }
  | { type: "renameBranch"; name: string; newName: string }
  | { type: "trackBranch"; name: string; remoteBranch: string }
  | { type: "checkoutCommit" | "cherryPick" | "revert"; target: string }
  | { type: "fetch" | "pull" | "push" }
  | { type: "publish"; remote: string }
  | { type: "stash"; message: string }
  | {
      type: "applyStash" | "popStash" | "dropStash";
      index: number;
      oid: string;
    };
export interface AiInput {
  id: string;
  repoId: string;
  fingerprint: string;
  kind: string;
  paths: string[];
  excluded: string[];
  text: string;
}
export interface AiConfig {
  provider: string;
  model: string;
}
export interface Provider {
  id: string;
  name: string;
  available: boolean;
  detail: string;
}

export type SelectedFile = { path: string; staged: boolean };
export type Form = {
  repoId?: string;
  fingerprint?: string;
  kind:
    | "createBranch"
    | "stash"
    | "publish"
    | "createBranchAt"
    | "createTag"
    | "renameBranch"
    | "trackBranch";
  target?: string;
  title: string;
  value: string;
};
export type Confirmation = {
  action: Action;
  title: string;
  description: string;
  repoId?: string;
  fingerprint?: string;
};
export const isConflicted = (file: FileChange) =>
  file.index === "U" ||
  file.worktree === "U" ||
  (file.index === "A" && file.worktree === "A") ||
  (file.index === "D" && file.worktree === "D");
export const isStaged = (file: FileChange) =>
  !isConflicted(file) && file.index !== " " && file.index !== "?";
export const canCommit = (snapshot: Snapshot) =>
  !snapshot.files.some(isConflicted) &&
  (!snapshot.operation || snapshot.operation === "Merge in progress") &&
  (snapshot.files.some(isStaged) || snapshot.operation === "Merge in progress");

export type ComparisonKind = "trees" | "mergeBase";
export interface Comparison {
  from: string;
  to: string;
  effectiveFrom: string;
  kind: ComparisonKind;
  files: FileChange[];
}
