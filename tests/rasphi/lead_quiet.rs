//! lead.quiet: a working agent quiet for this long wakes the Lead, once per quiet stretch; `none` never does.
use super::lead_common::*;
use serde_json::json;
use sw::json::*;

#[test]
fn lead_quiet_wakes_the_lead_once_per_quiet_stretch() {          // acceptance 10
    let w = lead_world(json!({ "lead.quiet": "1m" })); let id = at_work(&w, 3);
    sw::clock::advance(61_000); pass(&w, &id); assert_eq!(wakes(&w, &id).len(), 1);
    assert!(wakes(&w, &id)[0].starts_with(&format!("Lead woke: quiet on {id}.")));
    sw::clock::advance(3 * 60_000); pass(&w, &id); assert_eq!(wakes(&w, &id).len(), 1);
    sw::actions::talk::tell(&id, "Carry on.", false).unwrap(); settle(&id); // a new stretch, from when the words went in
    sw::clock::advance(61_000); pass(&w, &id); assert_eq!(wakes(&w, &id).len(), 2);
}
#[test]
fn lead_quiet_none_never_wakes_a_working_agent() {
    let w = lead_world(json!({})); let id = at_work(&w, 1);
    sw::clock::advance(3 * 3_600_000); pass(&w, &id); assert!(wakes(&w, &id).is_empty());
}
#[test]
fn escalate_on_a_quiet_wake_is_refused_with_what_to_do_instead() { // final review 2
    let w = lead_world(json!({ "lead.quiet": "1m" })); let id = at_work(&w, 3);
    sw::clock::advance(61_000); pass(&w, &id);
    let wake = wakes(&w, &id)[0].clone();
    assert!(wake.contains("You may: `lead") && !wake.contains("or escalate"), "{wake}");
    crate::common::pilot::in_pane(&w, &lead_pane(&id));
    let e = crate::common::pilot::refused(&["lead", &id, "escalate", "It is slow."]);
    crate::common::pilot::in_pane(&w, "");
    assert_eq!(e, sw::lead::verb::QUIET_ESCALATE);
    assert_eq!(e, "A quiet wake has no wait to hand over: answer the agent to stop and ask with report --state blocked; that question can be escalated.");
    let t = task(&id);
    assert!(sw::lead::wake::deciding(&t) && t.at("lead").at("escalated").is_null(), "the wake stays open: {}", t.at("lead"));
    assert!(sw::lead::brief::LEAD_RULES.contains("any of these but quiet"));
}
