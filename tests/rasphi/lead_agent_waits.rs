//! The waits that wake the Lead (lead.agent): a wait it may decide wakes it once, the human's notice for it is held while
//! it decides (`lead decides`, group working, `"lead": "deciding"` in `list --json`), the waits it cannot act on go to
//! the human as upstream has them, and a wait the board escalated is told to the human once and never wakes it again.
use super::lead_common::*;
use crate::common::pilot::cli;
use serde_json::json;
use sw::json::*;

#[test]
fn a_question_wakes_the_lead_once_and_holds_the_human_notice() {  // acceptance 2 (wake half)
    let w = lead_world(json!({})); let id = at_work(&w, 2);
    ask(&id, "Blue or green?");
    pass(&w, &id); pass(&w, &id);
    let ws = wakes(&w, &id);
    assert_eq!(ws.len(), 1);
    assert!(ws[0].starts_with(&format!("Lead woke: question on {id}.")) && ws[0].contains("Blue or green?") && ws[0].contains("Tries left for question: 2 of 2"), "{}", ws[0]);
    let t = task(&id); assert!(t.at("notice").is_null(), "{}", t.at("notice"));
    assert_eq!(sw::shown::states::standing(&sw::shown::states::as_row(&row(&id))), "lead decides");
    assert_eq!(sw::shown::states::group_of(&row(&id)), "working");
    let (v, _) = cli(&["list", "--json"]).unwrap();
    assert_eq!(v.as_array().unwrap().iter().find(|x| x.s("id") == id).unwrap().s("lead"), "deciding");
}
#[test]
fn the_human_is_not_notified_before_the_lead_wakes() {           // Review Focus 2
    let w = lead_world(json!({})); let id = at_work(&w, 1);
    ask(&id, "Which file?");
    sw::notify::notify_pass(&[task(&id)]);
    assert!(task(&id).at("notice").is_null());
    assert!(wakes(&w, &id).is_empty());
}
#[test]
fn waits_the_lead_cannot_act_on_never_wake_it() {                // acceptance 7 (wait half), 5 (approval half)
    use sw::lead::wake::class_of;
    let base = json!({ "link": { "lead": { "pane": "p9" } } });
    for patch in [json!({"stage":"asks","kind":"permission"}), json!({"stage":"asks","kind":"auth"}), json!({"stage":"reviewneeds"}), json!({"stage":"pick"}),
                  json!({"stage":"stopped","limitStop":{"role":"agent"}}), json!({"stage":"after","afterBlocked":true}), json!({"stage":"failed","failure":{"stage":"start"}}),
                  json!({"stage":"failed","failure":{"stage":"push","auth":true}}), json!({"stage":"approval"})] {
        let mut t = base.clone(); for (k, v) in patch.as_object().unwrap() { t[k] = v.clone(); }
        assert_eq!(class_of(t.as_object().unwrap()), None, "{patch}");
    }
    let w = lead_world(json!({})); let id = at_work(&w, 1);
    sw::actions::filing::report(&id, sw::actions::filing::Report { state: "blocked".into(), kind: "permission".into(), msg: "OK to delete?".into(), own: "agent", ..Default::default() }).unwrap();
    pass(&w, &id);
    assert!(wakes(&w, &id).is_empty()); assert_eq!(task(&id).at("notice").s("event"), "needs you");
}
#[test]
fn a_failed_review_at_approval_is_a_review_wait() {
    let w = lead_world(json!({ "review.rounds": 0 }));
    w.script(json!({ "worker": [step(1)], "reviewer": [{ "verdict": { "meets": false, "findings": [{ "severity": "major", "subject": "No test", "where": "f.txt:1", "proof": "cat f.txt" }], "text": "No." } }] }));
    let id = w.add("Reviewed"); delegate(&id); pass(&w, &id);
    let t = task(&id); assert_eq!(t.s("stage"), "approval"); assert!(t.at("failure").is_null());
    assert!(wakes(&w, &id)[0].starts_with(&format!("Lead woke: review on {id}.")));
}
#[test]
fn an_escalated_wait_is_never_woken_again_and_the_human_is_told_once() { // Review Focus 5
    let w = lead_world(json!({ "lead.tries": 0 })); let id = at_work(&w, 1);
    ask(&id, "Q?"); pass(&w, &id); pass(&w, &id);
    let t = task(&id);
    assert!(wakes(&w, &id).is_empty());
    assert!(t.s("msg").starts_with("Lead: used its 0 tries on question (lead.tries)"), "{}", t.s("msg"));
    assert_eq!(t.at("notice").s("event"), "needs you");
    let at = t.at("notice").i("at"); pass(&w, &id); assert_eq!(task(&id).at("notice").i("at"), at);
}
