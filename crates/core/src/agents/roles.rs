//! Which program can do which role, and why not: a setting that picks a program for a role it cannot do well is
//! refused with the reason (docs/agents.md). The reasons are the table's (kinds.rs `cannot`), and two follow from what
//! a program can be told at start: the Watcher needs a headless one-shot, and a model needs a way to give it.
use super::adapter::can_one_shot;
use super::kinds::{kind, Kind, KIND_NAMES};

/// The role a setting picks a program for: `worker.fallback` and `attempts.by` are the working agent's.
pub fn role_of_setting(name: &str) -> &str {
    match name.split('.').next().unwrap_or("") { "attempts" => "worker", r => r }
}
fn role_words(role: &str) -> &str {
    match role { "worker" => "work on a task", "reviewer" => "review", "coordinator" => "be the Coordinator", "plan" => "write a plan", "watcher" => "be the Watcher", "lead" => "be the Lead", r => r }
}
/// The programs that can do a role, for a refusal to name.
pub fn able(role: &str) -> Vec<&'static str> { KIND_NAMES.iter().copied().filter(|k| why_not(k, role, false).is_none()).collect() }
/// Why this program cannot do this role (with a model, when `model`), or None when it can.
pub fn why_not(k: &str, role: &str, model: bool) -> Option<String> {
    let a: &Kind = kind(k)?;
    if let Some((_, why)) = a.cannot.iter().find(|(r, _)| *r == role) { return Some(format!("{k} cannot {}: {why}.", role_words(role))); }
    // (fork) the Lead runs with a deny list of its tools, which the board has for claude only
    if role == "lead" && super::fence::lead_fence(k).is_none() { return Some(format!("{k} cannot be the Lead: the board starts the Lead with edits and outward actions denied, and has that deny list only for claude.")); }
    if role == "watcher" {
        if !can_one_shot(k) { return Some(format!("{k} has no headless one-shot mode, so it cannot be the Watcher.")); }
        // The three's own one-shot (adapter.rs) takes a model.
        if model && a.headless.as_ref().map(|h| h.model.is_empty()).unwrap_or(false) { return Some(format!("{k} takes no model as the Watcher: leave the model out.")); }
        return None;
    }
    if model && a.model.is_empty() { return Some(format!("{k} takes no model when the board starts it: leave the model out and choose it in {k}'s own settings.")); }
    None
}
/// (fork) Why a role cannot work under this access, whatever its program, or None. The Lead under `ask` would stop at its
/// first command with nobody to answer, and the board would type its wakes into that permission dialog.
pub fn why_not_access(role: &str, access: &str) -> Option<String> {
    (role == "lead" && access == "ask").then(|| "the Lead cannot work under \"ask\": nobody answers its permission dialogs, and the board's wakes would be typed into one. Keep lead.access at full or auto.".to_string())
}
/// The same, as a whole refusal of a setting's value, naming who can do it instead.
pub fn refusal(name: &str, k: &str, model: bool) -> Option<String> {
    let role = role_of_setting(name);
    let why = why_not(k, role, model)?;
    let others = able(role);
    Some(if others.contains(&k) { format!("{name}: {why}") } else { format!("{name}: {why} These can: {}.", others.join(", ")) })
}
