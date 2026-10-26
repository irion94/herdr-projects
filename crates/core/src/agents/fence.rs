//! What the Lead may not do: the deny list its program starts with (docs/lead.md). The Lead reads, runs checks and
//! says its decision through the board's own verb; it never edits, commits, pushes or acts on GitHub. The board has
//! this list for claude (`--disallowedTools`) and for no other program, so no other program can be the Lead.

/// The twelve tool patterns, in the spec's order, then the board's own commands that skip its guard (src/cli/mod.rs
/// matches them early): a Lead never runs a pipeline of another task, installs or swaps the program, or approves.
pub const CLAUDE_LEAD_DENY: [&str; 12] = [
    "Edit", "Write", "NotebookEdit", "Bash(git commit*)", "Bash(git push*)", "Bash(gh pr *)", "Bash(gh api*)", "Bash(gh issue *)",
    "Bash(herdr-projects run*)", "Bash(herdr-projects install*)", "Bash(herdr-projects update*)", "Bash(herdr-projects approve-current*)", // (fork)
];

/// The arguments that deny them to this program, or None when the board cannot. The flag takes several values, so
/// each pattern is an argument of its own, and the caller puts the flag before the hook's `--settings`.
pub fn lead_fence(kind: &str) -> Option<Vec<String>> {
    if kind != "claude" { return None; }
    let mut v = vec!["--disallowedTools".to_string()];
    v.extend(CLAUDE_LEAD_DENY.iter().map(|s| s.to_string()));
    Some(v)
}
