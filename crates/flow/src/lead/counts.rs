//! What the Lead has used of a task (fork, `lead.agent`): its tries per class of wait (`lead.tries[class]`, against
//! `lead.tries`) and the extra rounds it asked for (`leadRounds[class]`), which raise that class's cap by one each, and
//! what a verb of the Lead writes of them (acted). A round also keeps the wait it was made on (`lead.roundKey[class]`):
//! the next wait of that class after it goes to the human (lead/pass.rs), and the mark goes once the work is past it.
use crate::json::*;
use crate::store::settings::lookup::setting;

/// How many times the Lead acted on this class of wait in the task's whole life.
pub fn tries_used(t: &Obj, class: &str) -> i64 { t.at("lead").at("tries").i(class) }
/// `lead.tries`: how many times the Lead may act on one class of wait.
pub fn tries(t: &Obj) -> i64 { number(&setting("lead.tries", Some(t))) as i64 }
/// How many it has left of `lead.tries` for this class.
pub fn tries_left(t: &Obj, class: &str) -> i64 { (tries(t) - tries_used(t, class)).max(0) }
/// The extra rounds the Lead asked for in this class (review, check).
pub fn lead_rounds(t: &Obj, class: &str) -> i64 { t.at("leadRounds").i(class) }
/// What one verb of the Lead (`act`) on the wait `key` of `class` writes: a try used (an escalation is none), an extra
/// round counted for `round` with the wait it was made on, the verb as the class's last, and the wake closed.
pub fn acted(t: &mut Obj, class: &str, act: &str, key: &str) {
    let (tries, rounds) = (tries_used(t, class), lead_rounds(t, class));
    if act == "round" { t.obj_mut("leadRounds").put(class, rounds + 1); }
    let l = t.obj_mut("lead");
    if act != "escalate" { l.obj_mut("tries").put(class, tries + 1); }
    if act == "round" { l.obj_mut("roundKey").put(class, key); }
    l.obj_mut("last").put(class, act);
    l.put("wake", Value::Null);
}
/// The wait the Lead's last round on this class was made on ("" when none is open).
pub fn round_key(t: &Obj, class: &str) -> String { t.at("lead").at("roundKey").t(class) }
/// Forget the round marks of the classes the work is past (at approval: all but the class of a wait that stands there).
/// True when one went.
pub fn rounds_past(t: &mut Obj, standing: Option<&str>) -> bool {
    if t.s("stage") != "approval" || !t.at("lead").at("roundKey").is_object() { return false; }
    let r = t.obj_mut("lead").obj_mut("roundKey");
    let n = r.len();
    r.retain(|k, _| Some(k.as_str()) == standing);
    r.len() != n
}
