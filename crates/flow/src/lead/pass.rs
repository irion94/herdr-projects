//! The Lead's part of the housekeeping (fork, `lead.agent`), before the human is told anything (notify.rs): an ended
//! task's Lead is closed, a prompt that did not go in is handed again (to the Lead's own terminal only), and each wait
//! the Lead may decide wakes it once (spec 4.3). An open wake that the wait moved on from is closed; one that ran past `lead.timeout` goes to the human,
//! and so does a wait whose class used up `lead.tries`, a second failure after the Lead's own round, or a Lead that is
//! not running (spec 4.4).
use super::{brief::wake_text, counts::{tries, tries_used}, escalate::{escalate, line}, life, wake::{deciding, timeout_ms, wait_of, Wait}};
use crate::clock::now;
use crate::flow::{pane_owner::leads_pane, task::h};
use crate::json::*;
use crate::store::settings::{lookup::setting_text, values::ms};
use crate::store::tasks::{get, update, Change};

pub fn lead_pass(tasks: &[Obj], socket: &str) {
    for seen in tasks {
        if seen.at("link").s("socket") != socket || !seen.at("link").at("lead").is_object() { continue; }
        let Some(t) = get(seen.s("id")) else { continue };
        if life::ended(&t) { life::close_lead(t.s("id")); continue; }
        if running(&t) == Some(true) { life::hand_unsent(&t); } // never into a pane herdr gave to someone else
        one(&get(t.s("id")).unwrap_or(t));
    }
}
/// One task: its open wake looked after, or its wait woken once.
fn one(t: &Obj) {
    let (id, n) = (t.s("id"), now());
    let w = wait_of(t, n, ms(&setting_text("lead.quiet", Some(t))));
    let key = w.as_ref().map(|w| w.key.as_str()).unwrap_or("");
    if deciding(t) {
        let wake = t.at("lead").at("wake");
        if key != wake.s("key") {
            // The wait moved on (answered, or another came): the wake is closed, and a new wait goes on below.
            let k = wake.t("key");
            let _ = update(id, |c| { if c.at("lead").at("wake").s("key") != k { return Ok(Change::Keep); } c.obj_mut("lead").del("wake"); Ok(Change::Write) });
        } else {
            let limit = setting_text("lead.timeout", Some(t));
            if n - wake.i("at") >= timeout_ms(t) { let _ = escalate(id, key, wake.s("class"), &line(&format!("did not decide in {limit} (lead.timeout)"))); }
            else if !wake.b("sent") { if let Some(w) = w { hand(t, &w); } }
            return;
        }
    }
    let Some(w) = w else { return };
    if t.at("lead").s("lastKey") == w.key { return; } // fired once already
    let c = w.class;
    let tries = tries(t);
    if tries_used(t, c) >= tries { let _ = escalate(id, &w.key, c, &line(&format!("used its {tries} tries on {c} (lead.tries)"))); return; }
    if ["review", "check", "finish"].contains(&c) && t.at("lead").at("last").s(c) == "round" { let _ = escalate(id, &w.key, c, &line(&format!("its extra round did not fix the {c}"))); return; }
    // herdr cannot say (None): the wake is written unsent all the same, so `lead.timeout` runs and the pass hands it
    // once herdr answers, or hands the wait to you.
    if running(t) == Some(false) { let _ = escalate(id, &w.key, c, &line("is not running")); return; }
    let Ok(t) = update(id, |c| {
        if deciding(c) || c.at("lead").s("lastKey") == w.key || wait_of(c, n, ms(&setting_text("lead.quiet", Some(c)))).is_none_or(|x| x.key != w.key) { return Ok(Change::Keep); }
        let armed = c.i("armedAt");
        let l = c.obj_mut("lead");
        l.put("wake", json!({ "key": w.key, "class": w.class, "at": n, "sent": false }));
        l.put("lastKey", w.key.clone());
        if w.class == "quiet" { l.put("quietFor", armed); }
        Ok(Change::Write)
    }) else { return };
    if t.at("lead").at("wake").s("key") == w.key && !t.at("lead").at("wake").b("sent") { hand(&t, &w); }
}
/// Is the Lead's agent in its pane, in its own terminal? None when herdr cannot say.
fn running(t: &Obj) -> Option<bool> {
    match h(t).agent_get(t.at("link").at("lead").s("pane")) {
        Ok(Some(a)) => Some(leads_pane(t, &a.t("terminal_id"))),
        Ok(None) => Some(false),
        Err(_) => None,
    }
}
/// Type the wake into the Lead's pane (its own terminal only), and mark it sent when it went in.
fn hand(t: &Obj, w: &Wait) {
    if running(t) != Some(true) { return; }
    if crate::submit::hand(&h(t), t.at("link").at("lead").s("pane"), &wake_text(t, w)).is_err() { return; }
    let _ = update(t.s("id"), |c| {
        if c.at("lead").at("wake").s("key") != w.key { return Ok(Change::Keep); }
        c.obj_mut("lead").obj_mut("wake").put("sent", true);
        Ok(Change::Write)
    });
}
