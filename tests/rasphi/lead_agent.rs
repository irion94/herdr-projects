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
#[test]
fn the_lead_starts_in_its_own_tab_with_the_deny_list() { // acceptance 8
    let w = lead_world(json!({ "worker.agent": "codex" }));
    unsafe { std::env::remove_var("HERDR_PROJECTS_NO_HOOK") } // the Lead's hook file is what is looked at; the next world sets it again
    let id = at_work(&w, 1);
    let t = task(&id);
    let lead = t.at("link").at("lead");
    assert_eq!(lead.s("kind"), "claude");
    assert!(lead.s("agent").ends_with("-l"), "{lead}");
    assert!(w.state().arr("tabs").iter().any(|x| x.s("label") == "Lead" && x.s("tab_id") == lead.s("tab")));
    let start = calls(&w).into_iter().find(|c| c.starts_with(&format!("agent start {} ", lead.s("agent")))).unwrap();
    for d in sw::agents::fence::CLAUDE_LEAD_DENY { assert!(start.contains(d), "{d}: {start}"); }
    assert!(start.contains("--disallowedTools"));
    let p = w.state().arr("panes").iter().find(|p| p.s("pane_id") == lead.s("pane")).cloned().unwrap();
    assert_eq!(p.s("cwd"), sw::store::files::dir_of(&id).join("lead").display().to_string());
    let first = prompts(&w, lead.s("pane")).first().cloned().unwrap();
    assert!(first.starts_with(&format!("Work board task {id}, role: lead.")), "{first}");
    assert!(first.find(sw::lead::brief::LEAD_RULES).unwrap() < first.len());
    assert!(t.at("lead").at("unsent").is_null(), "the first prompt went in: {}", t.at("lead"));
    let hook = sw::store::files::read_json(&sw::store::files::dir_of(&id).join("hook-lead.json")).unwrap();
    assert!(hook.at("permissions").is_null(), "no extra writable dirs for the Lead: {hook}");
}
#[test]
fn the_leads_token_does_not_overwrite_the_agents_and_its_turn_ends_are_recorded() { // acceptance 8, trap 4.5
    let w = lead_world(json!({ "worker.agent": "codex" }));
    let id = at_work(&w, 1);
    let tokens = sw::store::files::read_json(&sw::store::files::dir_of(&id).join("tokens.json")).unwrap();
    let (a, l) = (tokens.t("agent"), tokens.t("lead"));
    assert!(!a.is_empty() && !l.is_empty() && a != l);
    assert!(sw::store::marks::token_ok(&id, "agent", &a, 0) && sw::store::marks::token_ok(&id, "lead", &l, 0) && !sw::store::marks::token_ok(&id, "agent", &l, 0));
    sw::turn::turn_ended(&id, "lead", &l, 0, &[json!({ "session_id": "s" }).to_string()]);
    assert_eq!(task(&id).at("lead").i("turns"), 1);
}
#[test]
fn a_lead_turn_end_starts_no_finish_runner_and_adds_no_event() { // Review Focus 4
    let w = lead_world(json!({})); let id = at_work(&w, 1);
    let before = task(&id).i("eventSeq");
    let l = sw::store::files::read_json(&sw::store::files::dir_of(&id).join("tokens.json")).unwrap().t("lead");
    sw::turn::turn_ended(&id, "lead", &l, 0, &["{}".into()]);
    let t = task(&id);
    assert_eq!((t.i("eventSeq"), t.s("stage")), (before, "working"));
    assert!(sw::runner::JOBS.with(|j| j.borrow().is_empty()));
}
#[test]
fn a_reused_lead_pane_id_with_another_terminal_is_not_the_lead() { // Review Focus 1
    let w = lead_world(json!({})); let id = at_work(&w, 1);
    let mut t = task(&id);
    assert_eq!(sw::flow::pane_owner::whose(vec![t.clone()], &w.socket, &lead_pane(&id)).map(|x| x.1), Some("lead"));
    t.obj_mut("link").obj_mut("lead").put("terminal", "term-other");
    assert!(sw::flow::pane_owner::whose(vec![t], &w.socket, &lead_pane(&id)).is_none());
}
#[test]
fn a_restart_with_the_leads_pane_id_taken_by_another_agent_starts_a_fresh_lead() { // Review Focus 1, on a restart
    let w = lead_world(json!({})); let id = at_work(&w, 2); // its second step: still at work once started again
    let (old, worker) = (task(&id).at("link").at("lead").clone(), task(&id).at("link").t("pane"));
    // herdr gave the Lead's pane id to another pane (another terminal, another agent in it), and the worker exited.
    w.patch(|s| for p in s["panes"].as_array_mut().unwrap() {
        if p.s("pane_id") == old.s("pane") { p["terminal_id"] = json!("term-other"); p["agent"]["name"] = json!("someone-else"); }
        if p.s("pane_id") == worker { p["agent"] = serde_json::Value::Null; }
    });
    sw::actions::talk::tell(&id, "Go on.", false).unwrap();
    settle(&id);
    let l = task(&id).at("link").at("lead").clone();
    assert!(l.b("pane") && l.s("pane") != old.s("pane") && l.s("terminal") != "term-other", "{l}");
    assert_eq!(calls(&w).iter().filter(|c| regex::Regex::new(r"^agent start \S+-l ").unwrap().is_match(c)).count(), 2);
}
#[test]
fn a_lead_that_cannot_start_leaves_the_task_as_today_and_says_so() {
    let w = lead_world(json!({ "worker.agent": "codex" }));
    w.patch(|s| s["missing"] = json!(["claude"]));
    let id = at_work(&w, 1);
    let t = task(&id);
    assert_eq!(t.s("stage"), "working");
    assert!(t.at("link").at("lead").is_null() && t.at("fallbacks").s("lead").starts_with("no Lead:"), "{}", t.at("fallbacks"));
    assert!(!w.state().arr("tabs").iter().any(|x| x.s("label") == "Lead"), "its tab is not left behind");
}
#[test]
fn the_lead_closes_with_its_task() {
    // Archived: a task waiting for approval (one still at work cannot be archived).
    let w = lead_world(json!({}));
    let id = to_approval(&w, "Closes");
    let tab = task(&id).at("link").at("lead").t("tab");
    assert!(!tab.is_empty());
    sw::actions::archive::archive(&id).unwrap();
    pass(&w, &id);
    assert!(task(&id).at("link").at("lead").is_null());
    assert!(!w.state().arr("tabs").iter().any(|x| x.s("tab_id") == tab));
}
#[test]
fn the_lead_closes_when_its_task_is_stopped() {
    let w = lead_world(json!({}));
    let id = at_work(&w, 1);
    let tab = task(&id).at("link").at("lead").t("tab");
    sw::actions::talk::stop(&id).unwrap();
    pass(&w, &id);
    assert!(task(&id).at("link").at("lead").is_null());
    assert!(!w.state().arr("tabs").iter().any(|x| x.s("tab_id") == tab));
}
