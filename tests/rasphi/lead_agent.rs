//! lead.agent and the settings of the Lead (lead.*): a work setting each, upstream's behaviour while `lead.agent` is
//! `none`, and only claude can be the Lead (the board has its deny list for no other program).
use super::lead_common::*;
use crate::common::pilot::{cli, in_pane, ok, refused};
use serde_json::json;
use sw::json::*;

#[test]
fn lead_settings_are_work_settings_with_upstream_defaults() {
    use sw::store::settings_list::entry_of;
    for (n, v) in [("lead.agent", json!("none")), ("lead.tries", json!(2)), ("lead.timeout", json!("15m")), ("lead.quiet", json!("none")), ("lead.access", json!("full"))] {
        let e = entry_of(n).unwrap_or_else(|| panic!("{n}"));
        assert_eq!((e.kind.name(), e.builtin.clone(), e.repo), ("work", v, false), "{n}");
    }
    assert!(entry_of("lead.instructions").unwrap().adds);
    assert!(entry_of("lead.agent").unwrap().doc.contains("no Lead"));
    assert!(!sw::store::settings_list::repo_facts().iter().any(|n| n.starts_with("lead.")));
}
#[test]
fn a_lead_agent_other_than_claude_is_refused() {
    let w = world(); in_pane(&w, "");
    let e = refused(&["set", "lead.agent", "codex"]);
    assert!(e.contains("codex cannot be the Lead") && e.contains("These can: claude"), "{e}");
    ok(&["set", "lead.agent", "claude/claude-opus-5-5"]);
}
#[test]
fn the_deny_list_is_claudes_alone() {
    let f = sw::agents::fence::lead_fence("claude").unwrap();
    assert_eq!(f[0], "--disallowedTools");
    assert_eq!(f[1..], sw::agents::fence::CLAUDE_LEAD_DENY.map(String::from));
    assert!(sw::agents::fence::lead_fence("codex").is_none());
}
#[test]
fn lead_none_is_upstream() {
    let w = world();
    let id = to_approval(&w, "Plain");
    assert!(!calls(&w).iter().any(|c| regex::Regex::new(r"^agent start \S+-l ").unwrap().is_match(c)));
    assert!(!w.state().arr("tabs").iter().any(|t| t.s("label") == "Lead"));
    let t = task(&id); assert!(t.at("lead").is_null() && t.at("link").at("lead").is_null() && t.at("leadRounds").is_null());
    let (v, _) = cli(&["list", "--json"]).unwrap();
    assert!(v.as_array().unwrap().iter().all(|x| x.at("lead").is_null()));
}
#[test]
fn the_lead_world_has_a_claude_lead_and_no_pane_yet() {
    let w = lead_world(json!({}));
    let id = at_work(&w, 1);
    assert_eq!(sw::store::settings::lookup::setting_text("lead.agent", None), "claude");
    assert!(wakes(&w, &id).is_empty());
}
