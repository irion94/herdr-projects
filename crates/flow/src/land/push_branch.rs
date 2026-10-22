//! `workspace.push branch` (fork, irion94/herdr-projects): a task started from a named branch at the remote lands on
//! that branch by one push of its reviewed branch, a fast-forward only. No pull request is opened or merged, nothing is
//! forced, and the branch it lands on is never deleted: it is someone else's. A push git refuses waits for you.
use super::merge::{fail, landed, no_fix};
use crate::err::Res;
use crate::finish::auto_merge::called_off;
use crate::json::*;
use crate::runner::{is_aborted, Guard};
use crate::store::repos::git_slow;
use crate::store::settings::lookup::setting_text;
use crate::store::settings::values::ms;
use crate::store::tasks::Task;
use crate::sync;
use std::path::Path;

pub const NEEDS_FROM: &str = "workspace.push branch needs workspace.from <branch>";
/// The task lands by a push onto the branch it started from. One that has a pull request already goes on through it.
pub fn lands_on_base(t: &Obj) -> bool { setting_text("workspace.push", Some(t)) == "branch" && !t.b("pr") }
/// At the start: the branch named is not origin's main, which this would push onto without a pull request.
pub fn not_main(root: &Path, b: &str, push: bool) -> Res<()> {
    if push && b == sync::default_branch(root) { crate::refuse!("{NEEDS_FROM}: {b} is origin's main branch, which takes work through a pull request. Set workspace.from to another branch, or workspace.push to own."); }
    Ok(())
}
/// Before anything else of the merge: the branch it lands on, or None when it may not land (said on the task).
pub(super) fn target(g: &mut Guard, root: &Path, t: &Task) -> Res<Option<String>> {
    if let Some(b) = sync::named_base(root, t) { return Ok(Some(b)); }
    fail(g, NEEDS_FROM, &format!("This task lands by a push onto the branch it started from (workspace.push branch), and it started from {}, not a branch of its own at the remote.\n\nNothing was pushed.", sync::main_branch(root, t)),
        &format!("Set workspace.push to own for this task (herdr-projects set workspace.push own --task {}), then approve again.", t.s("id")), no_fix())?;
    Ok(None)
}
/// The push itself, after the last catch-up with the branch: `git push origin <task branch>:refs/heads/<b>`.
pub(super) fn land(id: &str, g: &mut Guard, t: &Task, tree: &str, b: &str) -> Res<()> {
    let tp = Path::new(tree);
    let head = { let v = t.at("mergeIntent").t("verifiedHead"); if v.is_empty() { crate::flow::git::head_of(tp) } else { v } };
    if called_off(g) { return Ok(()); }
    let most = match ms(&setting_text("merge.pushTimeout", Some(t))) { 0 => u64::MAX, l => l as u64 };
    match git_slow(tp, &["push", "origin", &format!("{}:refs/heads/{b}", t.s("branch"))], most) {
        Ok(_) => landed(id, g, t, &format!("Pushed onto {b}"), &head, &head),
        Err(e) if is_aborted(&e) => Err(e),
        Err(e) => fail(g, &format!("Could not push onto {b}"), &format!("Branch: {}\nLands on: {b} at origin\n\ngit push:\n{}\n\nNothing was forced: {b} at origin is as it was.", t.s("branch"), e.msg),
            &format!("{b} has commits at origin this task does not have, or origin refused the push. Approve again: the board brings {b} in, checks and reviews the work again, then pushes."), no_fix()),
    }
}
