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
