//! The task's Lead (fork, `lead.agent`): an agent beside the working one that decides the task's waits it may decide
//! and hands the rest to you (docs/lead.md). Each area is a file in lead/: its life (start, turn ends, close) and its
//! first prompt with the board's fixed rules for it.
pub mod brief;
pub mod life;
