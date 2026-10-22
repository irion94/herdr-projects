//! board.trust: whether the board marks a repo as trusted in Claude's ~/.claude.json before a full-access start.
//! Built in on, as upstream; off, the file is never touched and `doctor` names each known repo without the mark, to
//! be pre-trusted by hand.
use crate::common::*;
use serde_json::json;
use sw::json::*;

fn started(settings: serde_json::Value) -> World {
    let w = world_with(Opts { repo: true, settings, ..Default::default() });
    w.script(json!({ "worker": [{ "status": "working" }] }));
    let id = w.add("Add a greeting");
    delegate(&id);
    let t = settle(&id);
    assert_eq!(t.s("stage"), "working", "{}", t.at("failure"));
    w
}

#[test]
fn board_trust_is_a_board_switch_built_in_on() {
    let e = sw::store::settings_list::entry_of("board.trust").expect("board.trust is a setting");
    assert_eq!((e.kind.name(), e.builtin.clone()), ("board", json!(true)));
}

#[test]
fn built_in_a_full_access_start_writes_the_mark_and_doctor_says_nothing() {
    let w = started(json!({}));
    let j: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(w.base.join("claude.json")).unwrap()).unwrap();
    assert_eq!(j.at("projects").at(&w.root_str()).at("hasTrustDialogAccepted"), &json!(true));
    assert_eq!(sw::doctor::trust::check(), vec![]);
}

#[test]
fn off_the_file_stays_byte_identical_and_doctor_names_the_root() {
    let before = "{\n  \"projects\": {},\n  \"someone\": \"else's\"\n}\n";
    let w = world_with(Opts { repo: true, settings: json!({ "board.trust": false }), ..Default::default() });
    std::fs::write(w.base.join("claude.json"), before).unwrap();
    w.script(json!({ "worker": [{ "status": "working" }] }));
    let id = w.add("Add a greeting");
    delegate(&id);
    assert_eq!(settle(&id).s("stage"), "working");
    assert_eq!(std::fs::read_to_string(w.base.join("claude.json")).unwrap(), before);
    let root = w.root_str();
    let found = sw::doctor::trust::check();
    assert!(found.iter().any(|f| f.problem && f.what.contains(&root) && f.fix == format!("pre-trust {root} by hand")), "{found:?}");
    // Trusted by hand: nothing left to say.
    std::fs::write(w.base.join("claude.json"), json!({ "projects": { root.clone(): { "hasTrustDialogAccepted": true } } }).to_string()).unwrap();
    assert!(sw::doctor::trust::check().iter().all(|f| !f.problem));
}
