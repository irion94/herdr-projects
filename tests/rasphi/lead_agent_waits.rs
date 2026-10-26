//! The waits that wake the Lead (lead.agent): a wait it may decide wakes it once, the human's notice for it is held while
//! it decides (`lead decides`, group working, `"lead": "deciding"` in `list --json`), the waits it cannot act on go to
//! the human as upstream has them, and a wait the board escalated is told to the human once and never wakes it again.
use super::lead_common::*;
use crate::common::pilot::{cli, in_pane, ok, refused};
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
#[test]
fn the_leads_answer_reaches_the_worker_and_keeps_the_round_counters() { // acceptance 2
    let w = lead_world(json!({})); let id = at_work(&w, 2);
    sw::store::tasks::set(&id, json!({ "checkFails": 1, "reviewRounds": 2, "prRounds": 1, "syncRepairs": 1 })).unwrap();
    ask(&id, "Blue or green?"); pass(&w, &id);
    in_pane(&w, &lead_pane(&id)); ok(&["lead", &id, "answer", "Blue, as the description says."]); in_pane(&w, "");
    let t = pass(&w, &id);
    assert!(prompts(&w, t.at("link").s("pane")).last().unwrap().contains("Blue, as the description says."));
    assert_eq!((t.i("checkFails"), t.i("reviewRounds"), t.i("prRounds"), t.i("syncRepairs")), (1, 2, 1, 1));
    assert!(t.at("notice").is_null()); assert_eq!(t.at("lead").at("tries").i("question"), 1);
}
#[test]
fn plan_ok_from_the_lead_starts_the_build() {                     // acceptance 5
    let w = lead_world(json!({ "plan.approve": "you", "worker.agent": "codex", "check.command": "true" }));
    unsafe { std::env::set_var("HERDR_PROJECTS_SANDBOX", "bwrap") }; w.patch(|s| s["pids"] = json!(true));
    w.script(json!({ "worker": [{ "plan": "# Plan\n\n1. Add f.txt.\n", "note": "Written." }, step(1)] }));
    let id = w.add("Planned"); delegate(&id); pass(&w, &id);
    assert!(wakes(&w, &id)[0].starts_with(&format!("Lead woke: plan on {id}.")));
    in_pane(&w, &lead_pane(&id));
    for a in [vec!["approve", &id], vec!["approve", &id, "--anyway"], vec!["tell", &id, "go"], vec!["finish", &id], vec!["lead", &id, "plan-ok", "--anyway"]] { refused(&a); }
    sw::clock::advance(1000); ok(&["lead", &id, "plan-ok"]); in_pane(&w, ""); // time passes: the build's turn is a new one
    let t = pass(&w, &id);
    assert_eq!((t.at("plan").s("approvedBy"), t.s("stage")), ("lead", "approval"));
    assert_eq!(wakes(&w, &id).len(), 1, "a passed review at approval wakes nobody");
    let mut b = crate::common::board::open(&w.root, 90); b.s.sel = id.clone(); b.load(); b.press(&["alt+t"]); // final review 4
    let whole = b.text();
    assert!(regex::Regex::new(r"Stands\s+approved by the Lead").unwrap().is_match(&whole), "{whole}");
}
#[test]
fn a_lead_round_sends_the_findings_back_once_and_a_second_failure_goes_to_the_human() { // acceptance 6
    let fail = json!({ "verdict": { "meets": false, "findings": [{ "severity": "major", "subject": "No test", "where": "f.txt:1", "proof": "cat f.txt" }], "text": "No." } });
    let w = lead_world(json!({ "review.rounds": 0 }));
    let mut again = fail.clone(); again["verdict"]["findings"][0]["id"] = json!("F1"); // still open in the next round
    w.script(json!({ "worker": [step(1), step(2)], "reviewer": [fail, again] }));
    let id = w.add("Reviewed"); delegate(&id); pass(&w, &id);
    let n = prompts(&w, task(&id).at("link").s("pane")).len();
    sw::clock::advance(1000); // time passes: the Lead's round starts a new turn
    in_pane(&w, &lead_pane(&id)); ok(&["lead", &id, "round", "Fix F1, add the test."]); in_pane(&w, "");
    let t = pass(&w, &id);
    let ps = prompts(&w, t.at("link").s("pane"));
    assert_eq!(ps.len(), n + 1); assert!(ps[n].contains("No test") && ps[n].contains("Fix F1, add the test."));
    assert_eq!((t.at("leadRounds").i("review"), t.i("reviewRounds"), t.s("stage")), (1, 1, "approval"));
    assert_eq!(wakes(&w, &id).len(), 1);
    assert!(t.s("msg").starts_with("Lead: its extra round did not fix the review"));
    assert_eq!(t.at("notice").s("event"), "ready to approve", "told as upstream tells a failed review at approval (spec 4.4)");
    assert!(sw::actions::finish_line::approve(&id, None, false).is_err(), "the merge still waits for the human");
}
#[test]
fn a_lead_verb_after_the_wait_moved_on_is_refused() {             // Review Focus 3
    let w = lead_world(json!({})); let id = at_work(&w, 2);
    ask(&id, "Q?"); pass(&w, &id);
    sw::actions::talk::tell(&id, "My own answer.", false).unwrap();
    let n = prompts(&w, task(&id).at("link").s("pane")).len();
    in_pane(&w, &lead_pane(&id)); let e = refused(&["lead", &id, "answer", "late"]); in_pane(&w, "");
    assert!(e.contains("Nothing waits for the Lead"), "{e}");
    assert_eq!(prompts(&w, task(&id).at("link").s("pane")).len(), n);
}
