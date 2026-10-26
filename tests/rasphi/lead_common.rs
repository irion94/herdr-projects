//! What the Lead's tests share (tests/rasphi/lead_*.rs): a world with a Lead, its pane and prompts, one pass of the
//! board's housekeeping, a question from the working agent, and a task whose agent is at work.
#![allow(dead_code)] // the helpers are used by the Lead's later test files
pub use crate::common::*;
use serde_json::{json, Value};
use sw::json::*;

/// A world with `lead.agent claude` and the given settings, the clock fixed, and every program still counting as
/// installed however far the test moves the clock (as `pilot_world_with` does).
pub fn lead_world(settings: Value) -> World {
    let mut s = json!({ "lead.agent": "claude" });
    for (k, v) in settings.as_object().cloned().unwrap_or_default() { s[k] = v; }
    let w = world_with(Opts { repo: true, settings: s, ..Default::default() });
    sw::clock::set(Some(1_800_000_000_000));
    let file = sw::store::files::data_dir().join("versions.json");
    let mut v: Value = serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
    for (_, k) in v.as_object_mut().unwrap() { k["at"] = json!(4_000_000_000_000i64); }
    std::fs::write(&file, v.to_string()).unwrap();
    w
}
/// The pane of the task's Lead ("" when it has none).
pub fn lead_pane(id: &str) -> String { task(id).at("link").at("lead").t("pane") }
/// The prompts the Lead was given that start with `Lead woke:`.
pub fn wakes(w: &World, id: &str) -> Vec<String> { prompts(w, &lead_pane(id)).into_iter().filter(|p| p.starts_with("Lead woke:")).collect() }
/// One pass of the board's own timer for the task: settle, the housekeeping, settle.
pub fn pass(w: &World, id: &str) -> Obj {
    settle(id);
    sw::housekeeping::housekeeping(&w.socket, sw::housekeeping::Force::Yes);
    settle(id)
}
/// The working agent asks a question, from its own pane.
pub fn ask(id: &str, msg: &str) {
    sw::actions::filing::report(id, sw::actions::filing::Report { state: "blocked".into(), kind: "question".into(), msg: msg.into(), own: "agent", ..Default::default() }).unwrap();
}
/// A task added and delegated whose agent keeps working for `steps` steps; its id.
pub fn at_work(w: &World, steps: usize) -> String {
    w.script(json!({ "worker": (0..steps).map(|_| json!({ "status": "working" })).collect::<Vec<_>>() }));
    let id = w.add("At work");
    delegate(&id);
    settle(&id);
    id
}
