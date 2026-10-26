//! lead.timeout: a Lead that does not decide a wake in time hands the wait to the human, with the Lead's line first.
use super::lead_common::*;
use serde_json::json;
use sw::json::*;

#[test]
fn a_silent_lead_hands_the_wait_to_the_human_after_lead_timeout() { // acceptance 4
    let w = lead_world(json!({})); let id = at_work(&w, 1);
    ask(&id, "Q?"); pass(&w, &id);
    sw::clock::advance(14 * 60_000); pass(&w, &id); assert!(task(&id).at("lead").at("wake").is_object());
    sw::clock::advance(2 * 60_000); pass(&w, &id);
    let t = task(&id);
    assert!(t.at("lead").at("wake").is_null() && t.s("msg").starts_with("Lead: did not decide in 15m (lead.timeout)"), "{}", t.s("msg"));
    assert_eq!((t.s("stage"), t.at("notice").s("event")), ("asks", "needs you"));
    assert_eq!(wakes(&w, &id).len(), 1);
}
