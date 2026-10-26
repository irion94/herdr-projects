//! lead.tries: the Lead acts at most this many times on one class of wait in a task's life; the next wait of that
//! class goes to the human with the Lead's line first.
use super::lead_common::*;
use crate::common::pilot::{in_pane, ok};
use serde_json::json;
use sw::json::*;

#[test]
fn the_same_question_a_third_time_goes_to_the_human_with_the_leads_line() { // acceptance 3
    let w = lead_world(json!({})); let id = at_work(&w, 4);
    for _ in 0..2 {
        ask(&id, "Blue or green?"); pass(&w, &id); sw::clock::advance(1000); // time passes: its answer starts a new turn
        in_pane(&w, &lead_pane(&id)); ok(&["lead", &id, "answer", "Blue."]); in_pane(&w, ""); pass(&w, &id);
        assert_eq!(task(&id).s("stage"), "working");
    }
    ask(&id, "Blue or green?"); pass(&w, &id);
    let t = task(&id);
    assert_eq!(wakes(&w, &id).len(), 2);
    assert!(t.s("msg").starts_with("Lead: used its 2 tries on question (lead.tries) · Blue or green?"), "{}", t.s("msg"));
    assert_eq!((t.s("stage"), t.at("notice").s("event")), ("asks", "needs you"));
}
