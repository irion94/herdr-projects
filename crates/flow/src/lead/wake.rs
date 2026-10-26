//! What wakes the Lead (fork, `lead.agent`, spec 4.3): the waits it may decide, each a class, read from where the task
//! stands, plus `lead.quiet`; each wait has a key so it fires once. While a wake is open, or a wait it may decide has
//! not reached it yet, the wait is held: the board says `lead decides` and the human is not told (notify.rs).
use crate::chat_only::is_chat;
use crate::clock::now;
use crate::json::*;
use crate::store::settings::{lookup::setting_text, values::ms};

/// The classes of wait, in the order of spec 4.3.
pub const CLASSES: [&str; 6] = ["question", "plan", "review", "check", "finish", "quiet"];
/// How long a wait the Lead may decide is held before its wake is written (by the Lead's pass of a live herdr session):
/// past it, the human is told as upstream tells them, so a wait no pass reaches is never hidden for good.
pub const GRACE_MS: i64 = 120_000;
/// A wait that wakes the Lead: its class and its key (one wake per key).
#[derive(Clone, Debug, PartialEq)]
pub struct Wait { pub class: &'static str, pub key: String }

/// The task has a Lead to wake: its pane is known, and it is no chat and not archived.
pub fn on(t: &Obj) -> bool { t.at("link").at("lead").b("pane") && !is_chat(t) && !t.b("archivedAt") }
/// The class of the wait the task stands at, when the Lead may decide it (no quiet: that one needs the clock). A failed
/// review at `approval` is read from the verdict: the finish line clears `failure` before it lands there.
pub fn class_of(t: &Obj) -> Option<&'static str> {
    if !on(t) || t.b("attemptOf") { return None; }
    let f = t.at("failure");
    match t.s("stage") {
        "asks" if t.s("kind") == "question" => Some("question"),
        "plan" if crate::actions::plan::awaits_plan(t) => Some("plan"),
        "approval" if crate::status_words::review_failed(t) && !t.b("mergeFailed") && !t.b("syncBlocked") && !t.b("prFailure") && !f.is_object() => Some("review"),
        "failed" if !f.b("auth") && f.s("stage") == "check" => Some("check"),
        "failed" if !f.b("auth") && ["workflow", "push", "pr"].contains(&f.s("stage")) => Some("finish"),
        _ => None,
    }
}
/// The wait that wakes the Lead now: a class of `class_of`, or a working agent quiet for `quiet_ms` (0: never).
pub fn wait_of(t: &Obj, now: i64, quiet_ms: i64) -> Option<Wait> {
    if let Some(c) = class_of(t) { return Some(Wait { class: c, key: key_of(c, t) }); }
    (on(t) && !t.b("attemptOf") && quiet_ms > 0 && t.s("stage") == "working" && now - t.i("armedAt") >= quiet_ms).then(|| Wait { class: "quiet", key: key_of("quiet", t) })
}
/// One key per wait: its class and the turn it came in (a new turn of the agent is a new wait). A failed check or finish
/// step also names its failure: the finish line runs a step again in the same turn, and a second failure is a new wait.
pub fn key_of(class: &str, t: &Obj) -> String {
    if ["check", "finish"].contains(&class) { return format!("{class}@{}:{}", t.i("armedAt"), t.at("failure").i("at")); }
    format!("{class}@{}", t.i("armedAt"))
}
/// A wake is open: the Lead was woken and has not acted, escalated or timed out.
pub fn deciding(t: &Obj) -> bool { t.at("lead").at("wake").is_object() }
/// The human's notice waits: a wake is open and within `lead.timeout`, or a wait the Lead may decide has not been
/// through it yet and came less than `GRACE_MS` ago (`at`: when the task reached that stage). A wait it was woken for,
/// that was escalated or that it acted on is not held. Each hold ends by itself, whether or not a pass comes.
pub fn held(t: &Obj) -> bool {
    if !on(t) { return false; } // an ended task's Lead is closed: a wake left open on it holds nothing
    let n = now();
    if deciding(t) { return n - t.at("lead").at("wake").i("at") < timeout_ms(t); }
    class_of(t).map(|c| t.at("lead").s("lastKey") != key_of(c, t) && n - t.i("at") < GRACE_MS).unwrap_or(false)
}
/// The wait the task stands at went to the human through the Lead (it escalated, or the board did for it): `list --json`
/// says `"lead": "escalated"` while it stands.
pub fn escalated(t: &Obj) -> bool { class_of(t).is_some_and(|c| t.at("lead").at("escalated").s("key") == key_of(c, t)) }
/// `lead.timeout` in milliseconds.
pub fn timeout_ms(t: &Obj) -> i64 { ms(&setting_text("lead.timeout", Some(t))) }
/// The class the Lead decides now: the open wake's, or the wait's.
pub fn class_now(t: &Obj) -> String {
    let w = t.at("lead").at("wake");
    if w.b("class") { return w.t("class"); }
    class_of(t).unwrap_or("").to_string()
}
