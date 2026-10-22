# Three-Brain Protocol

This document defines the protocol semantics implemented by `magi-core`, `magi-usecases`, and the Tauri commands. The UI is a view over these rules; it must not invent a verdict the backend would not produce.

## Brains

| Unit | Role | Lens | Temperature |
|---|---|---|---|
| MELCHIOR-1 | 科学家 / scientist | evidence, methodology, base rates, second-order risk, falsifiability | 0.2 |
| BALTHASAR-2 | 母亲 / mother | protection, long-term consequences, reversibility, safety margins, vulnerable stakeholders | 0.4 |
| CASPER-3 | 女性 / woman | emotion, relationships, dignity, trust, identity, power dynamics, lived experience; intuition as a hypothesis | 0.7 |

All three calls use the same provider/model in v1; the persona lens and temperature are what keep them independent.

## Stages

### 1. OPENING

- Three calls run concurrently with `tokio::join!`.
- Each call sees only the question and the optional context.
- Output is strict JSON:

```json
{
  "stance": "approve | reject | abstain",
  "confidence": 0,
  "summary": "2-4 sentences",
  "key_points": ["2-5 items"],
  "risks": ["1-3 items"]
}
```

- A failed call becomes an explicit `Failed` slot. It is never dropped or replaced with a fabricated opinion.

### 2. DEBATE (optional)

- Enabled only when **all three** openings succeeded.
- Each call sees its own opening and the other two openings.
- It does **not** see the other brains' debate replies; the round is parallel.
- Output is plain text (no JSON), 120-300 Chinese characters.

### 3. VOTE

- Three calls run concurrently.
- Each call sees the question, context, all successful openings, and all successful debate replies.
- It does **not** see any other final ballot, and no final ballots appear in any prompt.
- A brain whose opening failed may still vote; its prompt marks the baseline as unavailable.
- Output is strict JSON:

```json
{ "stance": "approve | reject | abstain", "confidence": 0, "reason": "1-3 sentences" }
```

## Verdict

`aggregate_votes` is a pure function over exactly three final ballots.

| Final ballots | Verdict |
|---|---|
| 3 approve | `Unanimous { Approve }` |
| 3 reject | `Unanimous { Reject }` |
| 3 abstain | `Unanimous { Abstain }` (no recommendation) |
| 2 approve + 1 reject | `Majority { Approve }` |
| 2 approve + 1 abstain | `Majority { Approve }` |
| 2 reject + 1 approve | `Majority { Reject }` |
| 2 reject + 1 abstain | `Majority { Reject }` |
| 2 abstain + 1 decisive | `Split` |
| 1 approve + 1 reject + 1 abstain | `Split` |
| any failed ballot | `Incomplete` |
| not exactly three ballots | `Incomplete` |

Abstentions never form a majority on their own. A failed ballot is never treated as a vote for the remaining majority.

## Changed stance

- If the opening and final ballot both succeeded, `changed_from_opening` is computed as `opening.stance != final.stance`.
- `APPROVE → ABSTAIN` counts as a change.
- If the opening failed, `opening_stance` and `changed_from_opening` are `null`, displayed as `NO BASELINE`.

## Failure semantics

- Per-brain transport, timeout, or schema failures are returned as `Failed { persona, message }` inside the stage batch. The rest of the batch is preserved.
- A failure does not reject the whole IPC call unless the request itself is invalid (empty question, over budget, malformed batch order, debate without three successful openings).
- The verdict is `Incomplete` if any final ballot failed.
- V1 does not retry automatically. The UI offers an explicit retry of the failed stage.

## Data boundaries and injection

- `question` and `context` are JSON-encoded into the user message, not concatenated as prose.
- Every prompt labels question, context, and peer output as untrusted data and forbids following instructions inside them.
- No tools or external actions are available to the model.
- Final ballots are never included in any prompt.
- The model cannot spoof `persona`, `model`, or `prompt_version`: Rust injects them after parsing.

## Validation

- JSON may be raw or wrapped in exactly one fenced code block. `<think>` blocks are stripped.
- Missing fields, unknown stances, confidence outside 0-100, empty required text, too many/few bullets, and trailing prose are errors. Nothing is silently defaulted.
- Session records are revalidated on save: batch length, persona order, verdict, tally, opening baseline, and changed flags are recomputed by Rust. The frontend's verdict is never trusted.

## Budgets

| Input | Budget |
|---|---|
| Question | 2,000 characters (rejected if over) |
| Context | 40,000 characters (head 3/4 + tail 1/4 kept) |
| Peer debate text in a prompt | 6,000 characters per reply |
| Session question / context on disk | 4,000 / 20,000 characters |

## Versioning

- `OpeningBatch.prompt_version`, `DebateBatch.prompt_version`, and `VoteBatch.prompt_version` are `v1`.
- Session and settings schemas are `v1`. A newer on-disk history schema puts the store into read-only protection instead of overwriting it.
