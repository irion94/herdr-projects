//! The Lead's life (fork, `lead.agent`): started when the task's agent starts, in a tab of its own (`Lead`) in the
//! task's own folder, its program fenced off from edits and outward actions (agents/fence.rs), its first prompt handed
//! once it is up; its turn ends counted, never taken for the agent's (turn.rs); closed with its task, as the reviewer
//! is. A Lead that cannot start leaves the task as upstream has it (its waits go to you) and says so in
//! `fallbacks.lead`. Its hook has a token slot of its own (store/marks.rs).
use super::brief::first_prompt;
use crate::agents::{fence::lead_fence, installed, kinds::program, start};
use crate::chat_only::is_chat;
use crate::clock::now;
use crate::delegate::{agent_name, launch_extras};
use crate::err::{Error, Res};
use crate::flow::{self, tabs::Opened};
use crate::herdr::Herdr;
use crate::json::*;
use crate::runner::Guard;
use crate::store::files::dir_of;
use crate::store::settings::lookup::work;
use crate::store::tasks::{get, one_line, update_wait, Change, Task};

/// Its task is over: archived, done, or stopped (a stop at a limit waits for you and keeps its Lead).
pub fn ended(t: &Obj) -> bool { t.b("archivedAt") || t.s("stage") == "done" || (t.s("stage") == "stopped" && !t.b("limitStop")) }

/// Start the task's Lead, when `lead.agent` names one and it has none running. Only an abort of the job fails it: a
/// Lead that cannot start is a fallback, never the task's failure.
pub fn start_lead(g: &mut Guard, t: Task, api: &Herdr) -> Res<Task> {
    let w = work(Some(&t));
    let who = start::role_of("lead", &|n| w.at(n).clone());
    if who.kind.is_empty() || t.b("attemptOf") || is_chat(&t) { return Ok(t); }
    // Still running: an agent in its pane, in the terminal it was started in. herdr gives a closed pane's id to the next
    // pane of its space, so an agent there in another terminal is someone else's, and a fresh Lead starts.
    let l = t.at("link").at("lead");
    if l.b("pane") && api.agent_get(l.s("pane")).ok().flatten().is_some_and(|a| flow::pane_owner::leads_pane(&t, &a.t("terminal_id"))) { return Ok(t); }
    if t.arr("attempts").len() > 1 { return g.set(|t| { t.obj_mut("fallbacks").put("lead", "no Lead: a task done several times at once has none in this release"); }); }
    let mut made: Option<Opened> = None;
    let name = match launch(&t, api, &who, &mut made) {
        Ok(name) => name,
        Err(e) => {
            if let Some(o) = &made { flow::tabs::close_reviewer_tab(api, &json!({ "tab": o.tab, "pane": o.pane })); }
            let line = format!("no Lead: it could not start ({}); its waits go to you", one_line(&e.msg, 200));
            return g.set(|t| { t.obj_mut("fallbacks").put("lead", line); if t.at("link").is_object() { t.obj_mut("link").del("lead"); } }); // never a link to a pane that is not its Lead
        }
    };
    let o = made.unwrap_or(Opened { tab: String::new(), pane: String::new(), terminal: String::new() });
    let first = first_prompt(&t);
    let t = g.set(|t| {
        t.obj_mut("link").put("lead", json!({ "pane": o.pane, "tab": o.tab, "terminal": o.terminal, "agent": name, "kind": who.kind }));
        let l = t.obj_mut("lead"); l.put("startedAt", now()); l.put("unsent", first);
        t.obj_mut("fallbacks").del("lead");
    })?;
    hand_unsent(&t);
    Ok(get(t.s("id")).unwrap_or(t))
}
/// Its tab, its program with the deny list and its hook, waited for until it has drawn its screen; its agent's name.
fn launch(t: &Task, api: &Herdr, who: &start::Role, made: &mut Option<Opened>) -> Res<String> {
    let kind = who.kind.as_str();
    let mut extra = lead_fence(kind).ok_or("lead.agent has no deny list")?;
    let dir = dir_of(t.s("id")).join("lead");
    std::fs::create_dir_all(&dir)?;
    installed::pre_trust(kind, &dir, &who.access);
    let o = flow::tabs::open_tab(api, t, flow::tabs::LEAD_TAB, &dir.display().to_string(), &crate::temp::task_env(t), "the Lead")?;
    let pane = o.pane.clone();
    *made = Some(o);
    let taken: Vec<String> = api.agents().unwrap_or_default().iter().map(|a| a.t("name")).filter(|n| !n.is_empty()).collect();
    let mut name: String = format!("{}-l", agent_name(t, &taken)).chars().take(32).collect();
    if taken.contains(&name) { name = format!("{}-{:04}", name.chars().take(27).collect::<String>(), now() % 10000); }
    let x = launch_extras(t, "lead", kind, 0, "", None)?;
    extra.extend(x.hook.clone());
    let opts = start::StartOpts { model: who.model.clone(), effort: start::effort_for(kind, &who.model, &who.effort), fast: Some(who.fast), access: who.access.clone(), session: x.session.clone().unwrap_or_default(), extra, ..Default::default() };
    let args = start::start_args(kind, &opts)?;
    match api.agent_start(&name, kind, &pane, &args, 60000) { Err(e) if e.code != "agent_not_ready" => return Err(Error::new(e.message)), _ => {} }
    crate::submit::settle(api, &pane, &std::iter::once(program(kind).to_string()).chain(args).collect::<Vec<_>>().join(" "));
    Ok(name)
}
/// Hand the Lead the text that waits for it (`lead.unsent`); true when it went in. Kept for the next try otherwise.
pub fn hand_unsent(t: &Obj) -> bool {
    let (pane, text) = (t.at("link").at("lead").t("pane"), t.at("lead").t("unsent"));
    if pane.is_empty() || text.is_empty() || crate::submit::hand(&flow::task::h(t), &pane, &text).is_err() { return false; }
    let _ = update_wait(t.s("id"), 2000, |c| { if c.at("lead").t("unsent") != text { return Ok(Change::Keep); } c.obj_mut("lead").del("unsent"); Ok(Change::Write) });
    true
}
/// Close the Lead's tab and forget it, as the reviewer's (finish/review.rs: close_reviewer).
pub fn close_lead(id: &str) {
    let Some(t) = get(id) else { return };
    if !t.at("link").at("lead").is_object() { return; }
    flow::tabs::close_reviewer_tab(&flow::task::h(&t), t.at("link").at("lead"));
    let _ = update_wait(id, 10000, |t| { if t.at("link").is_object() { t.obj_mut("link").del("lead"); } Ok(Change::Write) });
}
/// The Lead's own turn-end report: counted on the task, nothing else. Never an event, so the finish runner never
/// takes it for the working agent's turn end.
pub fn turn_ended(id: &str) {
    let _ = update_wait(id, 2000, |t| { let n = t.at("lead").i("turns") + 1; let l = t.obj_mut("lead"); l.put("turns", n); l.put("endedAt", now()); Ok(Change::Write) });
}
