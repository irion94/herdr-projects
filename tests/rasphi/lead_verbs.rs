//! The Lead's verbs on the paths the wake tests do not walk (lead.agent): `escalate` hands the wait to you with the
//! Lead's line, `round` on a finish wait re-runs the step and never the merge, and your word ends the Lead's extra pass.
use super::lead_common::*;
use crate::common::pilot::{in_pane, ok};
use serde_json::json;
use sw::json::*;

#[test]
fn escalate_hands_the_wait_to_the_human_with_the_leads_line_and_a_notice() {
    let w = lead_world(json!({})); let id = at_work(&w, 1);
    ask(&id, "Blue or green?"); pass(&w, &id);
    assert!(task(&id).at("notice").is_null());
    in_pane(&w, &lead_pane(&id)); ok(&["lead", &id, "escalate", "I cannot tell which one you want."]); in_pane(&w, "");
    let t = pass(&w, &id);
    assert!(t.s("msg").starts_with("Lead: I cannot tell which one you want · "), "{}", t.s("msg"));
    assert_eq!((t.s("stage"), t.at("notice").s("event")), ("asks", "needs you"));
    assert_eq!(t.at("lead").at("last").s("question"), "escalated");
    assert_eq!(t.at("lead").at("tries").i("question"), 0, "an escalation is no try");
    assert_eq!(wakes(&w, &id).len(), 1, "it never wakes the Lead again");
    let at = t.at("notice").i("at"); pass(&w, &id); assert_eq!(task(&id).at("notice").i("at"), at);
    let lead_of = || { let (v, _) = crate::common::pilot::cli(&["list", "--json"]).unwrap(); v.as_array().unwrap().iter().find(|x| x.s("id") == id).unwrap().t("lead") };
    assert_eq!(lead_of(), "escalated", "final review 8: while the wait stands");
    sw::actions::talk::tell(&id, "Green.", false).unwrap(); settle(&id);
    assert_eq!(lead_of(), "", "the wait moved on");
}
#[test]
fn a_lead_round_on_a_finish_wait_runs_the_step_again_and_never_the_merge() {
    for step in ["push", "pr"] {
        let w = lead_world(json!({})); let id = at_work(&w, 1);
        sw::store::tasks::set(&id, json!({ "stage": "failed", "failure": { "stage": step, "title": "It failed", "detail": "boom", "at": 1 } })).unwrap();
        pass(&w, &id);
        assert!(wakes(&w, &id)[0].starts_with(&format!("Lead woke: finish on {id}.")), "{step}");
        sw::clock::advance(1000);
        in_pane(&w, &lead_pane(&id)); ok(&["lead", &id, "round", "Try the push again."]); in_pane(&w, "");
        let t = task(&id);
        assert_ne!(t.s("stage"), "failed", "{step}: the step runs again");
        assert!(!["approval", "merged", "done"].contains(&t.s("stage")), "{step}: {}", t.s("stage"));
        assert_eq!((t.at("lead").at("last").s("finish"), t.at("lead").at("tries").i("finish")), ("round", 1), "{step}");
        assert_eq!(t.at("lead").s("note"), "Try the push again.", "{step}");
    }
}
#[test]
fn your_word_ends_the_leads_extra_pass() {
    let fail = json!({ "verdict": { "meets": false, "findings": [{ "severity": "major", "subject": "No test", "where": "f.txt:1", "proof": "cat f.txt" }], "text": "No." } });
    let w = lead_world(json!({ "review.rounds": 0 }));
    w.script(json!({ "worker": [step(1), step(2)], "reviewer": [fail.clone(), fail] }));
    let id = w.add("Reviewed"); delegate(&id); pass(&w, &id);
    sw::clock::advance(1000);
    in_pane(&w, &lead_pane(&id)); ok(&["lead", &id, "round", "Add the test."]); in_pane(&w, "");
    assert_eq!(pass(&w, &id).at("leadRounds").i("review"), 1);
    sw::actions::talk::tell(&id, "Leave it, I will look.", false).unwrap();
    let t = task(&id);
    assert!(t.at("leadRounds").is_null(), "{}", t.at("leadRounds"));
    assert_eq!(sw::lead::counts::lead_rounds(&t, "review"), 0, "the review cap is review.rounds again");
}
#[test]
fn a_second_finish_failure_after_the_leads_round_goes_to_the_human_with_its_line() { // final review 3
    let w = lead_world(json!({})); let id = at_work(&w, 1);
    sw::store::tasks::set(&id, json!({ "stage": "failed", "failure": { "stage": "push", "title": "It failed", "detail": "boom", "at": 1 } })).unwrap();
    pass(&w, &id);
    sw::clock::advance(1000);
    in_pane(&w, &lead_pane(&id)); ok(&["lead", &id, "round", "Try the push again."]); in_pane(&w, "");
    assert_eq!(task(&id).s("stage"), "checking");
    sw::runner::drain_jobs(&|_, _, _| {}); // the step it runs again fails again, in the same turn of the agent:
    sw::clock::advance(1000);
    sw::store::tasks::set(&id, json!({ "stage": "failed", "failure": { "stage": "push", "title": "It failed again", "detail": "boom", "at": sw::clock::now() } })).unwrap();
    let t = pass(&w, &id);
    assert!(t.s("msg").starts_with("Lead: its extra round did not fix the finish"), "{}", t.s("msg"));
    assert!(t.at("notice").is_object(), "the human is told: {}", t.at("notice"));
    assert_eq!(wakes(&w, &id).len(), 1, "it never wakes the Lead");
    assert!(t.at("lead").at("roundKey").s("finish").is_empty(), "the round's mark is used up: {}", t.at("lead"));
}
#[test]
fn after_a_round_that_fixed_the_review_a_later_review_wait_wakes_the_lead_again() { // final review 3
    let fail = json!({ "verdict": { "meets": false, "findings": [{ "severity": "major", "subject": "No test", "where": "f.txt:1", "proof": "cat f.txt" }], "text": "No." } });
    let good = json!({ "verdict": { "meets": true, "findings": [], "text": "Good." } });
    let w = lead_world(json!({ "review.rounds": 0, "lead.tries": 3 }));
    w.script(json!({ "worker": [step(1), step(2), step(3)], "reviewer": [fail.clone(), good, fail] }));
    let id = w.add("Reviewed"); delegate(&id); pass(&w, &id);
    sw::clock::advance(1000);
    in_pane(&w, &lead_pane(&id)); ok(&["lead", &id, "round", "Add the test."]); in_pane(&w, "");
    let t = pass(&w, &id);
    assert_eq!(t.s("stage"), "approval"); assert!(!sw::status_words::review_failed(&t), "the round fixed it");
    assert!(t.at("lead").at("roundKey").s("review").is_empty(), "past the review, its round mark goes: {}", t.at("lead"));
    sw::clock::advance(1000);
    sw::actions::talk::tell(&id, "One more thing: rename it.", false).unwrap();
    let t = pass(&w, &id);
    assert_eq!(t.s("stage"), "approval"); assert!(sw::status_words::review_failed(&t));
    assert_eq!(wakes(&w, &id).len(), 2, "{}", t.s("msg"));
    assert!(!t.s("msg").starts_with("Lead:"), "{}", t.s("msg"));
}
#[test]
fn a_finish_round_on_a_task_now_at_approval_is_refused_and_nothing_is_approved() { // final review 5
    let w = lead_world(json!({})); let id = at_work(&w, 1);
    sw::store::tasks::set(&id, json!({ "stage": "approval", "mergeFailed": true, "failure": null })).unwrap();
    let n = calls(&w).len();
    let e = sw::actions::finish_line::retry_failed(&id).unwrap_err().msg;
    assert!(e.contains("no longer waits at its failed step"), "{e}");
    let t = settle(&id);
    assert_eq!((t.s("stage"), t.b("mergeIntent"), t.b("approvedAt")), ("approval", false, false), "{t:?}");
    assert!(calls(&w)[n..].iter().all(|c| !c.contains("merge")), "{:?}", &calls(&w)[n..]);
}
