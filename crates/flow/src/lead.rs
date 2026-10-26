//! The task's Lead (fork, `lead.agent`): an agent beside the working one that decides the task's waits it may decide
//! and hands the rest to you (docs/lead.md). Each area is a file in lead/: its life (start, turn ends, close), its
//! first prompt and the words of a wake, what wakes it, its pass in the housekeeping, its escalation to you, and what
//! it has used of its tries and rounds.
pub mod brief;
pub mod counts;
pub mod escalate;
pub mod life;
pub mod pass;
pub mod wake;
