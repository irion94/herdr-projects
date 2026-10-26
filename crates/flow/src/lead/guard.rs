//! What the Lead may run (fork, `lead.agent`, spec 4.3): from its own pane, only its own task's `lead` verb and the
//! read commands; everything else is refused before it runs, whatever it is (autopilot/allow.rs hands it here). The
//! commands src/cli/mod.rs runs before that guard are refused here too (lead_early): this is the real fence; the deny
//! list its program starts with (agents/fence.rs) only catches mistakes.
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
/// The commands src/cli/mod.rs matches before the guard of the task commands that a Lead may not run: the pipeline's
/// jobs (`run`), the keeper, the merge's yes from herdr (`approve-current`), another machine's actions (`remote`),
/// voice calls, install and update, the board and its doors, the doctor and the docs. Each of them is started by the
/// board without a pane (runner.rs, housekeeping.rs: `HERDR_PANE_ID` is dropped), so only a hand in a pane meets this.
pub const LEAD_EARLY_REFUSED: [&str; 14] = ["run", "keeper", "keeper-work", "approve-current", "remote", "voice", "install", "update", "board", "open", "open-coordinator", "serve", "doctor", "docs"];
/// The early commands a Lead may run: its own hook and herdr's, and the help.
pub const LEAD_EARLY_OK: [&str; 7] = ["turn-ended", "plan-written", "hook", "help", "--skill", "skill", "version"];
/// An early command from `me` (cli/mod.rs: caller): refused when it is the Lead's and not one it may run.
pub fn lead_early(cmd: &str, me: Option<(&Obj, &str)>) -> Res<()> {
    match me { Some((t, "lead")) if !LEAD_EARLY_OK.contains(&cmd) => lead_guard(cmd, &[], t, false), _ => Ok(()) }
}
