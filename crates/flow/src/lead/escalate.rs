//! The Lead hands a wait to the human (fork, `lead.agent`, spec 4.4): the wake is closed, the wait is marked as gone
//! through the Lead (it never wakes it again, and is no longer held), and the Lead's line goes first in the task's
//! message, so the human sees the wait as upstream has it with the reason on top.
use crate::clock::now;
use crate::err::Res;
use crate::json::*;
use crate::store::tasks::{update, Change, Task};

/// The Lead's line: "Lead: <why>", one sentence without its full stop.
pub fn line(why: &str) -> String { format!("Lead: {}", why.trim().trim_end_matches('.')) }
/// Hand the wait `key` of class `class` to the human with `line`. Once per key: a second call changes nothing.
pub fn escalate(id: &str, key: &str, class: &str, line: &str) -> Res<Task> {
    update(id, |t| {
        if t.at("lead").at("escalated").s("key") == key && !super::wake::deciding(t) { return Ok(Change::Keep); }
        let msg = t.t("msg");
        let l = t.obj_mut("lead");
        l.del("wake");
        l.put("lastKey", key);
        l.put("escalated", json!({ "key": key, "class": class, "line": line, "at": now() }));
        l.obj_mut("last").put(class, "escalated");
        t.put("msg", if msg.is_empty() { line.to_string() } else { format!("{line} · {msg}") });
        Ok(Change::Write)
    })
}
