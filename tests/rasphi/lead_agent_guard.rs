//! The Lead's guard (lead.agent): from the Lead's pane only its own task's `lead` verb and the read commands go through;
//! everything else is refused, and the `lead` verb is refused from any pane but the Lead's.
use super::lead_common::*;
use crate::common::pilot::{in_pane, ok, refused};
use serde_json::json;
use sw::json::*;

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
#[test]
fn the_early_commands_are_refused_from_the_leads_pane_and_nothing_runs() { // final review 1: the real fence
    let w = lead_world(json!({})); let id = at_work(&w, 1);
    let main = |a: &[&str]| sw::cli::main(a.iter().map(|x| x.to_string()).collect());
    let before = task(&id);
    in_pane(&w, &lead_pane(&id));
    for a in [vec!["run", "finish", id.as_str()], vec!["run", "pipeline", id.as_str(), "--force"], vec!["keeper-work"], vec!["approve-current"], vec!["remote", "do", "e30="], vec!["install"], vec!["update"]] {
        assert_eq!(main(&a), 1, "{a:?}");
    }
    let me = sw::cli::caller(); assert_eq!(me.as_ref().map(|c| c.1), Some("lead"));
    for c in sw::lead::guard::LEAD_EARLY_REFUSED {
        let e = sw::lead::guard::lead_early(c, me.as_ref().map(|(t, r)| (t, *r))).unwrap_err().msg;
        assert!(e.starts_with("Refused from the Lead of"), "{c}: {e}");
    }
    for c in sw::lead::guard::LEAD_EARLY_OK { sw::lead::guard::lead_early(c, me.as_ref().map(|(t, r)| (t, *r))).unwrap(); }
    in_pane(&w, &task(&id).at("link").t("pane")); // the working agent's own pane: the early commands are not the Lead's guard's
    let worker = sw::cli::caller(); assert_eq!(worker.as_ref().map(|c| c.1), Some("agent"));
    sw::lead::guard::lead_early("run", worker.as_ref().map(|(t, r)| (t, *r))).unwrap();
    in_pane(&w, "");
    sw::lead::guard::lead_early("run", sw::cli::caller().as_ref().map(|(t, r)| (t, *r))).unwrap();
    let t = task(&id);
    assert_eq!((t.s("stage"), t.at("runner").clone(), t.at("events").clone()), (before.s("stage"), before.at("runner").clone(), before.at("events").clone()), "nothing ran");
}
#[test]
fn every_early_command_is_either_refused_or_allowed_for_the_lead() {
    let src = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/cli/mod.rs")).unwrap();
    let early = &src[src.find("entries herdr itself calls").unwrap()..src.find("_ => {}").unwrap()];
    let re = regex::Regex::new(r#"(?m)^\s+(?:"[^"]+"\s*\|\s*)*"([^"]+)"(?:\s*\|\s*"([^"]+)")*\s*(?:if|=>)"#).unwrap();
    let names: Vec<String> = re.captures_iter(early).flat_map(|c| { let all = c.get(0).unwrap().as_str(); regex::Regex::new(r#""([^"]+)""#).unwrap().captures_iter(all).map(|x| x[1].to_string()).collect::<Vec<_>>() }).collect();
    assert!(names.len() >= 20, "{names:?}");
    for n in &names {
        assert!(sw::lead::guard::LEAD_EARLY_REFUSED.contains(&n.as_str()) || sw::lead::guard::LEAD_EARLY_OK.contains(&n.as_str()), "{n} is matched early and the Lead's guard does not say whether a Lead may run it");
    }
}
