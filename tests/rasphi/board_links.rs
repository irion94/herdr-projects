//! board.links: how review links are shared. `auto` is upstream's (the tailnet when tailscale is on PATH, else
//! 127.0.0.1), `tailscale` always the tailnet, `local` always 127.0.0.1; HERDR_PROJECTS_LINKS=local still wins. Read on
//! every call (the runner is long-lived), and a tailscale that hangs is given up after its timeout.
use crate::common::*;
use serde_json::json;
use std::path::Path;
use sw::review_page::{link_mode, tailscale};

fn set(k: &str, v: &str) { unsafe { std::env::set_var(k, v) } }
/// PATH as it was, put back when the test ends (the other tests of this process need git on it).
struct OldPath(Option<String>);
impl Drop for OldPath { fn drop(&mut self) { match &self.0 { Some(p) => set("PATH", p), None => unsafe { std::env::remove_var("PATH") } } } }
/// PATH with only `sh`, and a `tailscale` running `body` when one is given.
fn path_with(dir: &Path, body: Option<&str>) -> OldPath {
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir).unwrap();
    std::os::unix::fs::symlink("/bin/sh", dir.join("sh")).unwrap();
    if let Some(b) = body {
        let f = dir.join("tailscale");
        std::fs::write(&f, format!("#!/bin/sh\n{b}\n")).unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let old = OldPath(std::env::var("PATH").ok());
    set("PATH", &dir.display().to_string());
    old
}
fn run(argv: &[&str]) -> String { let a = sw::cli::flags::parse(&argv.iter().map(|s| s.to_string()).collect::<Vec<_>>()); sw::cli::tasks::command(&a.pos[0], &a).map(|(_, s)| s).unwrap_or_else(|e| panic!("{argv:?}: {}", e.msg)) }

#[test]
fn board_links_is_a_board_choice_built_in_auto() {
    let e = sw::store::settings_list::entry_of("board.links").expect("board.links is a setting");
    assert_eq!((e.kind.name(), e.builtin.clone()), ("board", json!("auto")));
    assert_eq!(e.values, vec!["auto", "tailscale", "local"]);
}

#[test]
fn the_setting_decides_the_link_mode_on_every_call() {
    let w = world();
    unsafe { std::env::remove_var("HERDR_PROJECTS_LINKS") }
    {
        let _p = path_with(&w.base.join("with-ts"), Some("exit 0"));
        assert_eq!(link_mode(), "tailscale", "auto with tailscale installed: upstream's tailnet");
        run(&["set", "board.links", "local"]);
        assert_eq!(link_mode(), "local", "read again, not fixed at the first call");
        run(&["set", "board.links", "tailscale"]);
        set("HERDR_PROJECTS_LINKS", "local");
        assert_eq!(link_mode(), "local", "the environment's local still wins");
        unsafe { std::env::remove_var("HERDR_PROJECTS_LINKS") }
    }
    let _p = path_with(&w.base.join("no-ts"), None);
    assert_eq!(link_mode(), "tailscale", "tailscale: the tailnet even where PATH lacks it");
    run(&["set", "board.links", "auto"]);
    assert_eq!(link_mode(), "local", "auto without tailscale: 127.0.0.1");
}

#[test]
fn a_tailscale_that_hangs_is_given_up() {
    let w = world();
    {
        let _p = path_with(&w.base.join("ok"), Some("echo \"$@\""));
        assert_eq!(tailscale(&["status", "--json"]).as_deref(), Some("status --json\n"));
    }
    let _p = path_with(&w.base.join("hang"), Some("exec /bin/sleep 60"));
    let t = std::time::Instant::now();
    assert_eq!(tailscale(&["serve", "status", "--json"]), None);
    assert!(t.elapsed() >= std::time::Duration::from_secs(9) && t.elapsed() < std::time::Duration::from_secs(20), "given up after its timeout, took {:?}", t.elapsed());
    assert!(sw::webui::link::read().is_none(), "the web UI's link reads through the same timeout");
}
