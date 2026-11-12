# 0467 — `make next-task` after decision 0437: first milestones, the busy-lane pass, the critical lanes (proposed)

- **Status:** proposed (2026-11-12, Taskei P560278403). It needs the project lead's ruling on its three
  questions; until then nothing changes, and the tool and docs/status.md stay as decision 0437 describes them.
- **Context:**
  - Decision 0437 (2026-11-11) retired the phase gate and recorded the lead's direction: the features the UI
    waits on first (M15, M16 and what else the collaborative sheet needs), balanced with engine work, which
    does not stop. Its item 5 said what the task tool does and changed none of it; its review found that the
    tool works against the direction in three places (Taskei P560278403), and docs/status.md has asked since
    ("Needs owner").
  - **The order.** `make next-task` sorts a lane's ready tasks by P0, handoffs, campaign stages, then the
    milestone number, then the priority (`scripts/tasks.py`). So M15 and M16 come after every lower-numbered
    milestone in every lane. On main at 2026-11-12 11:45Z, with 652 tasks ready, counting by priority P0, the
    milestone number, the priority and the id:

    | Lane | Ready | Of M15 or M16 | The first of them is at rank |
    |---|---|---|---|
    | research | 226 | 39 | 187 |
    | engine | 109 | 45 | 65 |
    | harness | 58 | 14 | 45 |
    | api | 14 | 6 | 9 |
    | functions-lookup | 11 | 1 | 11 |

    The other lanes hold no M15 or M16 task that is ready.
  - **The busy-lane pass** (decision 0332; `scripts/milestone_order.py`, `passes_later_work`). A ready task of
    the current milestone is offered in a lane that is busy when every branch in flight there works a task of
    a later milestone in the phase table's order. The current milestone is M6. So a Phase 1 task passes a lane
    busy with M15 or M16 work, and never the reverse.
  - **The critical lanes.** docs/status.md has `**Critical lane:** engine`, the owner ruling of 2026-11-04,
    made "until Phase 1 exits": the routines ask that lane first and then fall back to `functions`. No routine
    asks for harness, research or api, where 59 of the 105 ready M15 and M16 tasks are, and the engine's
    stage-5 chain waits on a harness task (T-5004 on T-5005).
- **Questions for the project lead**, each with the recommendation; "no" to all three leaves everything as it
  is:
  1. **A list of first milestones.** docs/status.md gains one line, `**First milestones:** M16, M15`, which the
     steward keeps as the lead directs, and `make next-task` sorts a task of a listed milestone ahead of the
     others, in the list's order, after P0, handoffs and campaign stages and before the milestone number.
     Recommended: yes. It says "these first" in one line that a person can change, and it changes nothing for a
     lane with no task of those milestones. The other ways: sorting every lane by priority before the milestone
     (priorities were given within a milestone, not across them, so the order would follow no one's intent),
     or leaving the number (the table above).
  2. **The busy-lane pass follows the same list.** A ready task passes a busy lane when it is of a first
     milestone or of the current one, and every branch in flight there is of neither. Recommended: yes. Today
     the pass lets Phase 1 work pass M15 and M16 work, which is the direction reversed; with the list alone and
     the pass unchanged, a lane busy with one M16 branch would still be passed by M6 work only.
  3. **The critical lanes are `engine`, `harness`, `research`, `api`,** in that order, in place of `engine`
     alone. Recommended: yes. They are the lanes that hold the M15 and M16 work; engine stays first, so engine
     work does not stop, and the workers still fall back to `functions` after them.
- **Consequences**, once ruled:
  - Question 1 and 2 are a change to `scripts/tasks.py` and `scripts/milestone_order.py` (a protected path:
    this record is its decision), with tests beside the ones decision 0332 added. The steward and harness
    lanes make it; the design desk does not edit scripts.
  - Question 3 is one line of docs/status.md, which the steward writes, and the "Needs owner" entry goes.
  - Nothing here touches a gate, a task's status or the Phase 1 exit measures.
