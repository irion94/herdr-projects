//! The Lead's guard (lead.agent): from the Lead's pane only its own task's `lead` verb and the read commands go through;
//! everything else is refused, and the `lead` verb is refused from any pane but the Lead's.
use super::lead_common::*;
use crate::common::pilot::{in_pane, ok, refused};
use serde_json::json;

#[test]
fn from_the_leads_pane_only_its_own_lead_verb_and_reads_go_through() { // acceptance 7 (guard half), 5
    let w = lead_world(json!({})); let id = at_work(&w, 1); let other = w.add("Other");
    in_pane(&w, &lead_pane(&id));
    assert!(refused(&["lead", &other, "answer", "x"]).contains("only on its own task"));
    for a in [vec!["stop", &id], vec!["set", "merge.by", "itself", "--task", &id], vec!["archive", &id], vec!["approve", &id], vec!["retry", &id], vec!["fix", &id], vec!["review", &id], vec!["delegate", &other], vec!["add", "More"]] {
        assert!(refused(&a).starts_with("Refused from the Lead of"), "{a:?}");
    }
    ok(&["list"]); ok(&["details", &id]); ok(&["show", &id]);
    in_pane(&w, ""); assert!(refused(&["lead", &id, "answer", "x"]).contains("the Lead's own command"));
}
