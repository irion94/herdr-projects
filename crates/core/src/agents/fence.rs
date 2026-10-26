//! What the Lead may not do: the deny list its program starts with (docs/lead.md). The Lead reads, runs checks and
//! says its decision through the board's own verb; it never edits, commits, pushes or acts on GitHub. The board has
//! this list for claude (`--disallowedTools`) and for no other program, so no other program can be the Lead.

/// The eight tool patterns, in the spec's order.
pub const CLAUDE_LEAD_DENY: [&str; 8] = ["Edit", "Write", "NotebookEdit", "Bash(git commit*)", "Bash(git push*)", "Bash(gh pr *)", "Bash(gh api*)", "Bash(gh issue *)"];

/// The arguments that deny them to this program, or None when the board cannot. The flag takes several values, so
/// each pattern is an argument of its own, and the caller puts the flag before the hook's `--settings`.
pub fn lead_fence(kind: &str) -> Option<Vec<String>> {
    if kind != "claude" { return None; }
    let mut v = vec!["--disallowedTools".to_string()];
    v.extend(CLAUDE_LEAD_DENY.iter().map(|s| s.to_string()));
    Some(v)
}
