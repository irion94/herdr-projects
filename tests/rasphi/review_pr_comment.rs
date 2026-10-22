//! review.prComment: whether a review's verdict is also posted on the task's pull request as a comment
//! (`**Review 1 · Pass …**`). Built in on, as upstream; off, the review still runs, gates and shows on the board.
use crate::common::*;
use serde_json::json;
use sw::json::*;

fn reviewed(settings: serde_json::Value) -> World {
    let w = world_with(Opts { repo: true, origin: true, settings, ..Default::default() });
    w.script(json!({ "worker": [{ "files": { "f.txt": "1\n" }, "commit": "v1", "note": "Did v1." }], "reviewer": [{ "verdict": { "meets": true, "text": "Looked at it.", "findings": [] } }] }));
    let id = w.add("With a pull request");
    delegate(&id);
    let t = settle(&id);
    assert_eq!(t.s("stage"), "approval", "{}", t.at("failure"));
    w
}

#[test]
fn review_pr_comment_is_a_work_switch_built_in_on() {
    let e = sw::store::settings_list::entry_of("review.prComment").expect("review.prComment is a setting");
    assert_eq!((e.kind.name(), e.builtin.clone()), ("work", json!(true)));
}

#[test]
fn built_in_the_verdict_is_posted_on_the_pull_request() {
    let w = reviewed(json!({}));
    let log = gh_log(&w);
    assert!(log.contains("pr comment 7 --body **Review 1 ·"), "{log}");
}

#[test]
fn off_no_comment_is_posted_and_the_pull_request_is_still_opened() {
    let w = reviewed(json!({ "review.prComment": false }));
    let log = gh_log(&w);
    assert!(log.contains("pr create"), "{log}");
    assert!(!log.contains("pr comment"), "{log}");
}
