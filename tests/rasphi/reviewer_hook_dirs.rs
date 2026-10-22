//! The reviewer's launch settings file (hook-reviewer.json) also names the task's review and plan folders as
//! additional directories, so a reviewer under `access: auto` writes its verdict and evidence there without a
//! permission dialog. The working agent's file stays as upstream writes it.
use serde_json::{json, Value};
use std::path::Path;
use sw::json::*;

fn settings_of(role: &str, dir: &Path) -> Value {
    std::fs::create_dir_all(dir).unwrap();
    let h = sw::agents::adapter::Hook { shim: "/bin/board".into(), id: "proj/1".into(), role: role.into(), token: "t".into(), round: 1, dir: dir.to_path_buf(), plan: false };
    let a = sw::agents::adapter::hook_args("claude", &h).unwrap().unwrap();
    let file = &a[a.iter().position(|x| x == "--settings").unwrap() + 1];
    assert!(file.ends_with(&format!("hook-{role}.json")), "{file}");
    sw::store::files::read_json(Path::new(file)).unwrap()
}

#[test]
fn the_reviewer_may_use_the_task_review_and_plan_folders() {
    let dir = std::env::temp_dir().join(format!("sw-reviewer-dirs-{}", std::process::id()));
    let rv = settings_of("reviewer", &dir);
    let d = |p: &str| dir.join(p).display().to_string();
    assert_eq!(rv.at("permissions").at("additionalDirectories"), &json!([d("review"), d("artifacts/plan")]));
    assert!(rv.at("hooks").at("Stop").is_array(), "the hook is still there: {rv}");
    let agent = settings_of("agent", &dir);
    assert!(agent.at("permissions").is_null(), "the working agent's file is unchanged: {agent}");
    assert_eq!(agent.as_object().unwrap().keys().collect::<Vec<_>>(), vec!["hooks"]);
    let _ = std::fs::remove_dir_all(&dir);
}
