//! workspace.push: how a task's work leaves the machine. `own` is upstream's (its own branch pushed, a pull request
//! opened and merged); `branch` lands on the branch it started from (`workspace.from colleague/count-lines`): no push
//! and no pull request at the finish, the review runs on the local branch, and once it may merge one push of the task's
//! branch onto that branch at origin, fast-forward only. A push git refuses waits for you, never forced.
use crate::common::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use sw::actions::finish_line;
use sw::json::*;

const FROM: &str = "colleague/count-lines";
fn set(k: &str, v: &str) { unsafe { std::env::set_var(k, v) } }
fn sh(p: &Path, body: &str) { std::fs::write(p, body).unwrap(); use std::os::unix::fs::PermissionsExt; std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o755)).unwrap(); }
/// git on PATH logs each `push` it is asked for, then runs the real git. A `race` file is run once before the first
/// push it sees: someone else pushing in between.
struct Pushes { log: PathBuf, race: PathBuf, real: String, old: Option<String> }
impl Drop for Pushes { fn drop(&mut self) { match &self.old { Some(p) => set("PATH", p), None => unsafe { std::env::remove_var("PATH") } } } }
impl Pushes {
    fn lines(&self) -> Vec<String> { read(&self.log).lines().map(String::from).collect() }
}
fn logged_pushes(w: &World) -> Pushes {
    let real = String::from_utf8(std::process::Command::new("sh").args(["-c", "command -v git"]).output().unwrap().stdout).unwrap().trim().to_string();
    let dir = w.base.join("gitlog");
    std::fs::create_dir_all(&dir).unwrap();
    let (log, race) = (dir.join("pushes.log"), dir.join("race.sh"));
    sh(&dir.join("git"), &format!("#!/bin/sh\ncase \"$1\" in push) echo \"$*\" >> '{}'; if [ -f '{1}' ]; then sh '{1}'; rm -f '{1}'; fi;; esac\nexec '{real}' \"$@\"\n", log.display(), race.display()));
    let old = std::env::var("PATH").ok();
    set("PATH", &format!("{}:{}", dir.display(), old.clone().unwrap_or_default()));
    Pushes { log, race, real, old }
}
/// A colleague's clone with `file` committed on FROM and pushed: the clone and its commit.
fn colleague(w: &World, file: &str) -> (PathBuf, String) {
    let other = w.base.join("colleague");
    if !other.exists() {
        git(&w.base, &["clone", "-q", &w.base.join("origin.git").display().to_string(), &other.display().to_string()]);
        git(&other, &["checkout", "-q", "-b", FROM]);
    }
    std::fs::write(other.join(file), "theirs\n").unwrap(); git(&other, &["add", "-A"]); git(&other, &["commit", "-qm", file]); git(&other, &["push", "-q", "origin", FROM]);
    let sha = git(&other, &["rev-parse", "HEAD"]);
    (other, sha)
}
fn at_origin(w: &World) -> String { git(&w.root, &["ls-remote", "origin", &format!("refs/heads/{FROM}")]).split_whitespace().next().unwrap_or("").to_string() }
fn on_branch(settings: Value) -> World {
    let mut s = json!({ "workspace.from": FROM, "workspace.push": "branch" });
    for (k, v) in settings.as_object().cloned().unwrap_or_default() { s[k] = v; }
    let w = world_with(Opts { repo: true, origin: true, settings: s, ..Default::default() });
    colleague(&w, "base.txt");
    w.script(json!({ "worker": [{ "files": { "count.txt": "3\n" }, "commit": "count lines", "note": "Counted." }], "reviewer": [{ "verdict": { "meets": true, "text": "Looked at it.", "findings": [] } }] }));
    w
}

#[test]
fn workspace_push_is_a_work_choice_built_in_own() {
    let e = sw::store::settings_list::entry_of("workspace.push").expect("workspace.push is a setting");
    assert_eq!((e.kind.name(), e.builtin.clone()), ("work", json!("own")));
    assert_eq!(e.values, vec!["own", "branch"]);
}

#[test]
fn branch_is_refused_unless_workspace_from_names_a_branch_that_is_not_origins_main() {
    let w = on_branch(json!({ "workspace.from": "main" }));
    let refused = |title: &str| {
        w.script(json!({ "worker": [{ "status": "working" }] }));
        let id = w.add(title);
        delegate(&id);
        let t = settle(&id);
        assert_eq!((t.s("stage"), t.at("failure").s("title")), ("failed", "Could not start from that branch"), "{}", t.at("failure"));
        assert!(t.at("failure").s("detail").contains("workspace.push branch needs workspace.from <branch>"), "{}", t.at("failure"));
        assert!(!t.b("worktree"));
    };
    refused("From main");
    // a branch origin calls its own main is main by another name
    sw::store::settings::change::set_value("workspace.from", &json!(FROM), &sw::store::settings::lookup::Scope::you()).unwrap();
    git(&w.root, &["fetch", "-q", "origin"]);
    git(&w.root, &["symbolic-ref", "refs/remotes/origin/HEAD", &format!("refs/remotes/origin/{FROM}")]);
    refused("From origin's main by name");
}

#[test]
fn branch_finishes_with_no_push_and_no_pull_request_and_is_reviewed_on_the_local_branch() {
    let w = on_branch(json!({}));
    let p = logged_pushes(&w);
    let id = w.add("Count lines");
    delegate(&id);
    let t = settle(&id);
    assert_eq!(t.s("stage"), "approval", "{}", t.at("failure"));
    assert!(t.b("review"), "reviewed on the local branch");
    assert_eq!(p.lines(), Vec::<String>::new(), "nothing pushed");
    assert!(!gh_log(&w).contains("pr create"), "{}", gh_log(&w));
    assert!(git(&w.root, &["ls-remote", "--heads", "origin", t.s("branch")]).is_empty(), "its branch is not at origin");
    let d = finish_line::details(&id).unwrap();
    assert_eq!((d.s("branch"), d.at("pr")), (t.s("branch"), &Value::Null));
    assert!(t.s("branch").starts_with("work/"), "{}", t.s("branch"));
}

#[test]
fn branch_lands_with_one_fast_forward_push_onto_that_branch_never_through_github() {
    let w = on_branch(json!({ "merge.by": "itself" }));
    let p = logged_pushes(&w);
    let before = at_origin(&w);
    let id = w.add("Count lines");
    delegate(&id);
    let t = settle(&id);
    assert_eq!(t.s("stage"), "done", "{}", t.at("failure"));
    let branch = t.t("branch");
    assert_eq!(p.lines(), vec![format!("push origin {branch}:refs/heads/{FROM}")]);
    assert!(!p.lines().iter().any(|l| l.contains("--force") || l.contains("--delete")), "{:?}", p.lines());
    let now = at_origin(&w);
    assert_eq!(now, t.s("mergeCommit"), "{}", t.at("msg"));
    assert!(git(&w.root, &["merge-base", "--is-ancestor", &before, &now]).is_empty(), "a fast-forward of what was there");
    assert!(git(&w.root, &["show", &format!("{now}:count.txt")]).contains('3'));
    let log = gh_log(&w);
    assert!(!log.contains("pr create") && !log.contains("pr ready") && !log.contains("pr merge"), "{log}");
    // the pull requests' watch has nothing of it: the colleague's branch is never tidied or deleted
    sw::housekeeping::poll_prs();
    sw::housekeeping::housekeeping(&w.socket, sw::housekeeping::Force::Yes);
    assert_eq!(p.lines().len(), 1, "{:?}", p.lines());
    assert_eq!(at_origin(&w), now, "the colleague's branch stays");
}

#[test]
fn a_push_git_refuses_as_not_a_fast_forward_waits_for_you_and_is_never_forced() {
    let w = on_branch(json!({ "merge.by": "itself" }));
    let p = logged_pushes(&w);
    // the colleague pushes again between the board's last look and its push
    sh(&p.race, &format!("cd '{}' && echo late > late.txt && '{r}' add -A && '{r}' commit -qm late && '{r}' push -q origin {FROM}\n", w.base.join("colleague").display(), r = p.real));
    let id = w.add("Count lines");
    delegate(&id);
    let t = settle(&id);
    let theirs = git(&w.base.join("colleague"), &["rev-parse", "HEAD"]);
    assert_eq!(at_origin(&w), theirs, "origin keeps the colleague's commit");
    assert_ne!(t.s("stage"), "done");
    assert!(t.b("mergeFailed"), "{t:?}");
    assert_eq!(t.at("failure").s("title"), format!("Could not push onto {FROM}"), "{}", t.at("failure"));
    assert!(t.at("failure").s("detail").contains("rejected"), "{}", t.at("failure"));
    // the board's own passes do not try again, with force or without
    sw::housekeeping::housekeeping(&w.socket, sw::housekeeping::Force::Yes);
    let t = settle(&id);
    assert_ne!(t.s("stage"), "done");
    assert_eq!(p.lines().len(), 1, "{:?}", p.lines());
    assert!(!p.lines().iter().any(|l| l.contains("--force")), "{:?}", p.lines());
}
