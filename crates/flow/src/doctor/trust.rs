//! Claude's trust marks, for `doctor`, while `board.trust` is off: the board then never marks a repo as trusted in
//! ~/.claude.json, so each repo it knows that has no mark there is named, to be pre-trusted by hand. The file is only
//! read here.
use super::Found;
use crate::json::*;

pub fn check() -> Vec<Found> {
    if crate::agents::installed::board_trust() { return vec![]; }
    let claude: Value = std::fs::read_to_string(crate::agents::homes::claude_file()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(Value::Null);
    let missing: Vec<String> = crate::store::repos::known_repos().values().map(text).filter(|r| !claude.at("projects").at(r).b("hasTrustDialogAccepted")).collect();
    if missing.is_empty() { return vec![Found::ok("Claude trusts every repo the board knows (board.trust is off)")]; }
    missing.into_iter().map(|r| Found::bad(format!("Claude has no trust mark for {r}, and board.trust is off"), format!("pre-trust {r} by hand"))).collect()
}
