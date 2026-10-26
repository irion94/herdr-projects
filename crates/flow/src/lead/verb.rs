//! The Lead's verb (fork, `lead.agent`, spec 4.3): `lead <id> answer|plan-ok|round|escalate`, from the task's Lead tab
//! only, on the wait it was woken for and still open. Its words go to the working agent as the board's own turn
//! (send_own), never as a reply of yours: the task's round counters stay as they are. A `round` is one more pass for
//! its class (counts.rs: leadRounds), never the merge. Each verb is counted first (counts.rs: acted), then done.
use super::{counts::acted, escalate::{escalate, line}, wake::{deciding, wait_of}};
use crate::actions::{finish_line, plan::{approve_plan_by, plan_reply, sent_back}, shared::{say, Said}};
use crate::clock::now;
use crate::err::Res;
use crate::flow::send::send_own;
use crate::gate;
use crate::json::*;
use crate::store::failures::{bound_detail, failure_patch};
use crate::store::settings::{lookup::setting_text, values::ms};
use crate::store::tasks::{find, get, update, Change};

const VERBS: [&str; 4] = ["answer", "plan-ok", "round", "escalate"];
/// Does verb `act` fit a wait of `class`? escalate fits every one but quiet, which has no wait to hand over.
fn fits(act: &str, class: &str) -> bool {
    match act {
        "answer" => ["question", "plan", "quiet"].contains(&class),
        "plan-ok" => class == "plan",
        "round" => ["review", "check", "finish"].contains(&class),
        _ => class != "quiet",
    }
}
/// Why `escalate` does not fit a quiet wake, and what the Lead does instead.
pub const QUIET_ESCALATE: &str = "A quiet wake has no wait to hand over: answer the agent to stop and ask with report --state blocked; that question can be escalated.";
/// The verbs that fit a wait of `class`, as the Lead types them.
fn fitting(id: &str, class: &str) -> String {
    VERBS.iter().filter(|a| fits(a, class)).map(|a| format!("`lead {id} {a}`")).collect::<Vec<_>>().join(", ")
}

/// `lead <id> <act> <words>`. `me`: who runs it (cli/mod.rs: caller); only the task's own Lead may.
pub fn lead(id: &str, act: &str, words: &str, me: Option<(&Obj, &str)>) -> Res<Said> {
    let cur = find(id)?;
    let id = cur.t("id");
    if !me.is_some_and(|(t, role)| role == "lead" && t.s("id") == id) {
        crate::fail!("lead {id} is the Lead's own command: run it from the task's Lead tab. You answer with tell, approve, review or retry as ever.");
    }
    if !VERBS.contains(&act) { crate::fail!("lead {id} takes answer <text>, plan-ok, round <note> or escalate <why>."); }
    let words = words.trim().to_string();
    let (mut class, mut key) = (String::new(), String::new());
    // Checked in the write itself: a wait that moved on (you answered, another came) takes no verb of the Lead.
    let t = update(&id, |t| {
        let wake = t.at("lead").at("wake").clone();
        let open = wait_of(t, now(), ms(&setting_text("lead.quiet", Some(t)))).map(|w| w.key);
        if !deciding(t) || open.as_deref() != Some(wake.s("key")) { crate::fail!("Nothing waits for the Lead on {id}."); }
        let c = wake.t("class");
        if act == "escalate" && c == "quiet" { crate::fail!("{QUIET_ESCALATE}"); }
        if !fits(act, &c) { crate::fail!("{act} does not fit the {c} wait on {id}: the Lead may {} there.", fitting(&id, &c)); }
        if act != "plan-ok" && words.is_empty() { crate::fail!("lead {id} {act} needs the words: herdr-projects lead {id} {act} <text>."); }
        acted(t, &c, act, wake.s("key"));
        if act == "round" && c == "finish" { t.obj_mut("lead").put("note", words.as_str()); }
        (class, key) = (c, wake.t("key"));
        Ok(Change::Write)
    })?;
    let keep = t.b("mergeIntent");
    let msg = match (act, class.as_str()) {
        ("answer", "question") => { send_own(&id, &format!("The task's Lead answers:\n\n{words}"), "lead", keep, obj(json!({ "kind": null, "selfBlocked": null, "blocked": null })), 0)?; "the Lead answered" }
        ("answer", "plan") => {
            update(&id, |t| { sent_back(t); Ok(Change::Write) })?;
            send_own(&id, &plan_reply(&t, &words), "lead", false, Obj::new(), 0)?;
            "the Lead answered"
        }
        ("answer", _) => { send_own(&id, &format!("The task's Lead says:\n\n{words}"), "lead", keep, Obj::new(), 0)?; "the Lead answered" }
        ("plan-ok", _) => { approve_plan_by(&id, "lead")?; "plan approved by the Lead, the build starts" }
        ("round", "review") => {
            let rv = t.at("review").clone();
            let mut stale = rv.as_object().cloned().unwrap_or_default(); stale.put("stale", true);
            let text = format!("{}\n\nThe task's Lead adds:\n{words}", gate::review_message(&rv, false));
            send_own(&id, &text, "review", keep, obj(json!({ "review": stale, "reviewRounds": t.i("reviewRounds") + 1 })), 0)?;
            "one more round, from the Lead"
        }
        ("round", "check") => {
            let f = t.o("failure").cloned().unwrap_or_default();
            let text = format!("The check failed, so this is not done yet.\n\n{}\n\nThe task's Lead adds:\n{words}\n\nFix it and finish again.", bound_detail(f.s("detail")));
            let mut patch = failure_patch(&f);
            patch.obj_mut("failure").put("sentAt", now());
            send_own(&id, &text, "check", keep, patch, 0)?; // checkFails stays: the Lead's round is counted in leadRounds
            "one more round, from the Lead"
        }
        ("round", _) => {
            // The failed step again, nothing re-armed. Only from `failed`, in the same read: retry at `approval` would be
            // the merge's yes.
            finish_line::retry_failed(&id)?;
            "one more round, from the Lead"
        }
        _ => { escalate(&id, &key, &class, &line(&words))?; "handed to you" }
    };
    Ok(say(get(&id).unwrap_or(t), format!("{id}: {msg}")))
}
