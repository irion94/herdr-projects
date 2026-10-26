# The Lead

**A Lead is one more agent on a task, whose job is to answer the agent's waits for you.** It reads what the working
agent asks, what the review found and what failed, and says its decision through one board command. It changes nothing
in the repo. What it cannot decide goes to you, as it would without a Lead, with its reason on top.

It is off by default: with `lead.agent none` the board behaves exactly as it did before there was a Lead.

```
herdr-projects set lead.agent claude/claude-opus-5-5 --repo my-app    # a Lead on every task of this repo
herdr-projects delegate 12 --lead.agent claude --lead.tries 3          # or for one task
```

## When it starts and closes

- It starts when the task's agent starts, in a tab of its own, `Lead`, next to `Agent` ([tabs](tabs.md)). Its agent is
  named `<base>-l` after the task's agent (`<base>`). One chat for the whole task, so it remembers what it decided.
- Its first prompt holds the task and its description, the plan if there is one, the task's settings in plain words
  (`review.rounds`, `merge.by`, `plan.approve`, `check.command`, `lead.tries`, `lead.timeout`, `lead.quiet`), the
  board's fixed rules for a Lead, the wake format, and then your `lead.instructions` (they add up across the places that
  set them, as the reviewer's do).
- It runs in a folder of its own, `lead` in the task's folder on the board, and reads the worktree by its path.
- It closes when the task is done, archived, or stopped by you, but not when it stops at a limit. It starts again on the
  next `delegate` or board restart, with a new chat; its counters stay on the task. A wake the old chat had not decided
  is woken again in the new one.
- No Lead for a chat-only task, or for a task done several times at once ([attempts](attempts.md)); the task says so
  in its fallbacks. A Lead that cannot start leaves the task as it was (its waits go to you) and the task says why.
- The Lead is never the planner. The planner keeps its own tab.

## Claude only, and what it may not do

`lead.agent` takes only programs the board can fence off. For now that is `claude`; any other program is refused with
the reason.

The real fence is the board's guard on the Lead's pane. From there it runs only `herdr-projects lead <its own id> ...`
and the read commands `list`, `ls`, `show`, `details`, `models`. Everything else is refused before it runs: `approve`,
`tell`, `retry`, `fix`, `review`, `finish`, `--anyway`, `stop`, `archive`, `set`, `delegate`, `add`, the autopilot
commands, and any other task. The same goes for the commands the board runs before that guard looks at what is asked:
`run`, `keeper`, `keeper-work`, `approve-current`, `remote`, `voice`, `install`, `update`, `board`, `open`,
`open-coordinator`, `serve`, `doctor` and `docs`, whichever path the Lead runs the program by. The board starts its own
jobs and the keeper without a pane, so they never meet this guard. `lead` itself runs only from the task's own Lead tab.

The Lead's program also starts with a deny list, a fence against mistakes: it stops the obvious forms before they reach
the board or git, not every way of writing them. For claude it is `--disallowedTools` on:

```
Edit
Write
NotebookEdit
Bash(git commit*)
Bash(git push*)
Bash(gh pr *)
Bash(gh api*)
Bash(gh issue *)
Bash(git * commit*)
Bash(git * push*)
Bash(herdr *)
Bash(*herdr-projects run*)
Bash(*herdr-projects install*)
Bash(*herdr-projects update*)
Bash(*herdr-projects approve-current*)
Bash(*herdr-projects remote*)
Bash(*herdr-projects keeper*)
```

`git * commit*` and `git * push*` catch `git -C <worktree> commit|push`. `herdr *` keeps the Lead off herdr's own
program, which could answer the working agent's permission dialog. The last six are the board's early commands, by any
path. `lead.access` is `full` by default, inside this fence. `ask` is refused: it would stop the Lead at its first command
with nobody to answer, and the board's wakes would then be typed into that permission dialog. `auto` is allowed.

## What wakes it

The waits below, and nothing else, each once. A wake is a prompt that starts with `Lead woke:`, with the wait's class,
the board's message, the round counters, the tries left for that class and the verbs that fit.

| Wait | The Lead may |
|---|---|
| **question**: the agent asks you something | `lead <id> answer "<text>"`: an answer within the description and plan |
| **plan**: a plan is ready | `lead <id> plan-ok` when it does what the task says and nothing more; `lead <id> answer "<points>"` sends it back |
| **review**: the review failed and its rounds are used up (the task stands at approval with a failed verdict) | `lead <id> round "<findings to fix, findings to skip and why>"`: one more fix round; never the merge |
| **check**: the check or QA failed after its rounds | `lead <id> round "<note>"` |
| **finish**: the push or the pull request failed (stage failed) | `lead <id> round "<note>"`: the failed step runs again; never the merge |
| **quiet**: `lead.quiet` is reached | `lead <id> answer "<text>"`: wrap up, or stop a verification and report |
| any of these but quiet | `lead <id> escalate "<one line why>"` |

A verb only fits its own class: `plan-ok` on a question is refused and says which verbs fit. A verb on a wait that
has already moved on (you answered, or another wait came) is refused with `Nothing waits for the Lead on <id>.`, so
one wait never gets two answers.

`escalate` on a quiet wake is refused too: nobody waits for an agent that works, so there is nothing to hand you. The
refusal says what to do instead: `A quiet wake has no wait to hand over: answer the agent to stop and ask with report
--state blocked; that question can be escalated.`

### What never wakes it

These go to you at once, as without a Lead: a permission or login dialog, a reviewer that needs something
(`reviewneeds`), a pick between attempts, a stop at a limit, a task that waits after a blocked `after`, a task that could
not start, and a task at `approval` that has no failed review (the merge yes is yours, or the Super Lead's by
`merge.by`). A chat-only task has no Lead, so it wakes none.

### Its answers

A Lead's words reach the working agent as the board's own message, never as your `tell`, `retry`, `fix` or reply to a
plan: those reset the round counters (`checkFails`, `reviewRounds`, `prRounds`, `syncRepairs`), and the Lead must not.
A Lead `round` is exactly one more pass for its class: the board raises that class's cap by one for this task, and you
see it as `+1` in the wake's `Rounds:` line. Your own word (`tell`, a retry, a fix) ends that extra pass: the counters
start again and the cap is back to `review.rounds`.

The Lead's own turn ends are counted and nothing else: they never start the finish line on the agent's unfinished work.
A turn that ends with no verb does not escalate by itself; only `lead.timeout` does.

## What you see

- While a wake is open the task's state word is `lead decides`, in the group working, not `needs you`.
- `herdr-projects list --json` has `"lead": "deciding"` on that task, so a tool that lists tasks (crew-queue) can show it.
  Once the wait has gone to you through the Lead (it escalated, or the board did for it), it has `"lead": "escalated"`
  for as long as that wait stands. A quiet wake has no wait of yours, so it never shows as escalated.
- Your notification for that wait is held: while a wake is open, for at most `lead.timeout`; and for a wait the Lead may
  decide that has not reached it yet, for at most two minutes. Nothing is hidden for good: past those, you are told as
  you would be without a Lead.
- A failed review the Lead has handled shows at approval as `ready to approve`, as it does without a Lead, with the
  Lead's line first in the message.

## When it hands a wait to you

The Lead is told to escalate when:

1. the answer would change the task's scope (more or less than the description and plan say);
2. it needs a secret, an account, money, or access the task does not have;
3. the action is destructive or outside the task's worktree and repo;
4. the same failure came back after its own try;
5. it is a merge yes, a handoff, or any `approval` without a failed review (that is yours, or the Super Lead's, by `merge.by`);
6. it does not know.

You then see the wait as you always do, with the Lead's line first in the task's message (`Lead: <why>`), and a
notice, once. An escalated wait never wakes the Lead again. A later wait of that class can, while tries are left.

The board hands a wait over by itself in these cases, each with its line first:

| Setting | Default | The board hands the wait over when | Line |
|---|---|---|---|
| `lead.tries` | `2` | the Lead has acted this many times on one class of wait in the task's whole life (counted per task and class, never reset by a stage change). `0`: every wait goes to you | `Lead: used its N tries on <class> (lead.tries)` |
| `lead.timeout` | `15m` | a wake is open and the Lead has not acted for this long. It takes no `none` | `Lead: did not decide in 15m (lead.timeout)` |
| `lead.quiet` | `none` | not a handover: it wakes the Lead when the working agent has worked this long (`40m`) without a turn end or report, once per quiet stretch. It counts from the later of the turn's start and the last turn end the board handled; the only report an agent can give on its own task, `--state blocked`, ends the stretch. `none`: only waits wake it | |

And two more:

- A review, check or finish wait that comes right after the Lead's own `round` for that class goes to you at once:
  `Lead: its extra round did not fix the <class>`. A step of the finish line that fails again in the same turn is such
  a wait too: a check or finish wait is told apart by its failure as well as by the turn. Only the next wait of that
  class counts: once it has gone to you, once the task is at approval past that class (the round fixed it), or once you
  have said your word (`tell`, a retry, a fix), a later wait of that class wakes the Lead again, while tries are left.
- A Lead whose agent is not in its pane when the wake is due (it ended, or herdr gave its pane to another program) cannot
  decide: `Lead: is not running`.

A quiet wake that runs out of tries or time only puts its line on the task: there is no wait of yours to hand over.

## Not in this release

The Lead's tokens are not counted in `limit.tokens`. The roles table in [agents](agents.md) has no Lead column.

More: the settings [`lead.*`](settings.md), the roles [roles and commands](roles-and-commands.md), its tab [tabs](tabs.md).
