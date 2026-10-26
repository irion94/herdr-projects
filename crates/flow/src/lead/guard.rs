//! What the Lead may run (fork, `lead.agent`, spec 4.3): from its own pane, only its own task's `lead` verb and the
//! read commands; everything else is refused before it runs, whatever it is (autopilot/allow.rs hands it here).
use crate::err::{Error, Res};
use crate::json::*;
use crate::store::tasks::find;

/// The guard for a command run from the Lead's pane of task `t`. `reads`: the command only reads the board.
pub fn lead_guard(cmd: &str, pos: &[String], t: &Obj, reads: bool) -> Res<()> {
    let id = t.s("id");
    if cmd == "lead" {
        let mine = pos.get(1).filter(|s| !s.is_empty()).map(|i| find(i)).transpose()?.is_some_and(|x| x.s("id") == id);
        return if mine { Ok(()) } else { Err(Error::refused(format!("A Lead may act only on its own task ({id})."))) };
    }
    if reads { return Ok(()); }
    Err(Error::refused(format!("Refused from the Lead of {id}: {cmd}. A Lead may run only `herdr-projects lead {id} answer|plan-ok|round|escalate` and the read commands (list, show, details, models).")))
}
