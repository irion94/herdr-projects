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
#[test]
fn a_lead_pane_herdr_cannot_read_still_hands_the_wait_to_the_human_after_lead_timeout() { // fix round 1
    let w = lead_world(json!({})); let id = at_work(&w, 1);
    ask(&id, "Q?");
    w.patch(|s| s["agentGetFails"] = json!(true));
    pass(&w, &id);
    let t = task(&id);
    assert!(t.at("lead").at("wake").is_object() && !t.at("lead").at("wake").b("sent") && t.at("notice").is_null(), "{}", t.at("lead"));
    sw::clock::advance(16 * 60_000); pass(&w, &id);
    let t = task(&id);
    assert_eq!((t.s("stage"), t.at("notice").s("event")), ("asks", "needs you"));
    assert!(t.s("msg").starts_with("Lead: did not decide in 15m (lead.timeout)"), "{}", t.s("msg"));
    assert!(wakes(&w, &id).is_empty());
}
#[test]
fn a_wait_of_a_task_whose_herdr_session_is_gone_reaches_the_human_after_the_grace() { // fix round 1
    let w = lead_world(json!({})); let id = at_work(&w, 1);
    ask(&id, "Q?");
    sw::store::tasks::patch(&id, |t| { t.obj_mut("link").put("socket", "/nowhere/herdr.sock"); }).unwrap();
    pass(&w, &id); // the housekeeping of a live session: no Lead pass for this task, its notice still read
    assert!(task(&id).at("notice").is_null() && wakes(&w, &id).is_empty());
    sw::clock::advance(sw::lead::wake::GRACE_MS); pass(&w, &id);
    assert_eq!(task(&id).at("notice").s("event"), "needs you");
}
