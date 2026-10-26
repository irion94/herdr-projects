//! What the Lead has used of a task (fork, `lead.agent`): its tries per class of wait (`lead.tries[class]`, against
//! `lead.tries`) and the extra rounds it asked for (`leadRounds[class]`), which raise that class's cap by one each, and
//! what a verb of the Lead writes of them (acted).
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
/// What one verb of the Lead (`act`) on a wait of `class` writes: a try used (an escalation is none), an extra round
/// counted for `round`, the verb as the class's last, and the wake closed.
pub fn acted(t: &mut Obj, class: &str, act: &str) {
    let (tries, rounds) = (tries_used(t, class), lead_rounds(t, class));
    if act == "round" { t.obj_mut("leadRounds").put(class, rounds + 1); }
    let l = t.obj_mut("lead");
    if act != "escalate" { l.obj_mut("tries").put(class, tries + 1); }
    l.obj_mut("last").put(class, act);
    l.put("wake", Value::Null);
}
