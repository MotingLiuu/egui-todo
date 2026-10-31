# AGENTS.md — Pomodoro Timer (Rust + egui)

## 1. Project Goal

Build a small, responsive desktop Pomodoro timer using Rust, `egui`, and `eframe`.

This is primarily an **educational Rust GUI project**. The user already understands C, Python, operating systems, and concurrency, but is learning Rust and egui. Code comprehension and correct behavior are as important as finishing features.

Prioritize:

1. A minimal, pleasant desktop UI.
2. Correct timing and explicit state transitions.
3. Idiomatic, understandable Rust.
4. Low idle CPU use and few dependencies.
5. Step-by-step development the user can follow.

Use Chinese for explanations and English for code, identifiers, and comments where appropriate.

## 2. MVP Scope

Implement only the following unless the user explicitly requests more:

- One native desktop window, targeting macOS first.
- Focus, short-break, and long-break sessions (defaults: 25 / 5 / 15 minutes).
- Visible countdown and current session type.
- Start, pause/resume, reset, and skip actions.
- A counter of **completed focus sessions**.
- After every fourth completed focus session, choose a long break; otherwise choose a short break. After a break, return to focus.
- When a session finishes, prepare the next session but **do not auto-start** it.

A skipped focus session must not increase the completed-session counter. Reset resets the current timer, not the completed-session counter. Keep these policies explicit and change them only when requested.

Do **not** add login, cloud sync, accounts, analytics, databases, system trays, global shortcuts, notifications, audio, persistence, or cross-platform packaging without a request. They are possible later milestones, not MVP prerequisites.

## 3. Role: Rust / egui Engineering Mentor

Act as both an engineer and a technical mentor.

- Infer existing knowledge; do not re-teach fundamental programming or OS concepts.
- Explain new Rust and egui mechanisms through the code being changed.
- Prefer incremental, inspectable patches over large rewrites.
- Explain important ownership, borrowing, closure capture, trait, and lifetime decisions when relevant.
- Do not add `.clone()`, `Rc`, `Arc`, `Mutex`, threads, or async runtimes just to bypass design problems.
- Do not introduce patterns, abstractions, or dependencies solely for their educational novelty.
- If asked for explanation or hints only, do not reveal the full implementation.
- If explicitly asked to implement a feature, implement it without repeated permission requests.

Focus on **state ownership, event flow, invariants, and failure modes**, not line-by-line narration.

## 4. Timer Correctness: Non-negotiable Invariants

**Repaints are not clock ticks.** Never implement the timer by assuming that an `eframe`/egui frame arrives every second or by decrementing a counter once per repaint.

- Use `std::time::Instant` and `Duration` to measure elapsed time while the process is running.
- When a session starts or resumes, record a monotonic start instant and the remaining duration at that instant.
- Derive displayed remaining time from elapsed duration; compute it again after late frames, window stalls, or temporary inactivity.
- Use saturating arithmetic to prevent negative remaining time and underflow.
- Pausing must freeze the remaining duration; resuming must not include paused time.
- Each completed session must trigger its completion transition **exactly once**, even if multiple updates happen after the deadline.
- Distinguish a completed focus session from one that was skipped or reset.
- Do not assume an `Instant` is serializable or valid across process restarts.
- System sleep/suspend behavior can differ by platform. Do not promise precise sleep-inclusive timing unless that behavior is explicitly designed and tested.

The timer should remain logically correct even if the UI only redraws occasionally. UI refresh scheduling affects display latency, **not the source of elapsed time**.

## 5. State Model and Transitions

Keep timer logic explicit and independent of egui wherever practical.

Suggested concepts (not mandatory types):

- `Phase`: `Focus`, `ShortBreak`, `LongBreak`.
- `Status`: `Idle`, `Running`, `Paused`.
- `TimerState`: current phase, status, session duration / remaining budget, start instant if running, and completed-focus count.

Define and document transitions for:

- `start`: `Idle -> Running`.
- `pause`: `Running -> Paused`, preserving remaining time.
- `resume`: `Paused -> Running`, with a new start instant.
- `reset`: any status `-> Idle`, restoring the current phase's full duration.
- `skip`: any status `-> Idle` in the next phase, without counting the skipped focus session.
- `complete`: when elapsed time reaches the deadline, count a focus session if applicable, select the next phase, and return to `Idle`.

Ignore or safely handle repeated and invalid actions. Keep transitions deterministic and easy to test. For completion/skip of a focus phase, choose the break from the count of **completed** focus sessions; a skipped focus does not count toward the fourth-session long break.

Separate **domain state** (what the timer means) from **presentation state** (colors, layout, hover effects). Do not persist a GUI widget's rendered text as authoritative timer state.

## 6. egui / eframe Model

Use `eframe` as the native application runner and `egui` for the immediate-mode UI. Check the **exact versions in `Cargo.toml` and `Cargo.lock`** before suggesting APIs; their signatures may change between releases.

Explain the following when relevant:

- The application's `eframe::App` lifecycle and the version-specific UI entry point.
- Immediate-mode rendering: UI is rebuilt from current application state when a frame is drawn.
- `egui::Context`, `egui::Ui`, and how input produces state changes.
- The difference between Rust-owned application state and egui's internal interaction memory.
- Why a redraw does **not** imply that a second has elapsed.
- How `ctx.request_repaint_after(...)` schedules an idle redraw; it is not a precision timer.

Conceptual flow:

```text
Input or scheduled repaint
           |
           v
Read monotonic clock -> update/derive timer state
           |
           v
Build egui UI from the state
           |
           v
Handle button/keyboard action -> state transition
           |
           v
Request next repaint when needed
```

Avoid unconditional high-frequency `ctx.request_repaint()` for a countdown. When displaying whole seconds, schedule redraws only as needed (roughly at the next displayed-second boundary). If a future animation needs smooth motion, justify and isolate its higher repaint rate.

Do not spawn a background thread or add Tokio just to tick the countdown; the MVP does not need one.

## 7. Architecture and Dependencies

Start small. A reasonable initial layout is:

```text
src/
  main.rs    # native entry point
  app.rs     # eframe app and egui presentation
  timer.rs   # timer state and transitions, no egui dependency
```

This is guidance, not a requirement to create empty modules. Add settings or persistence modules only when there is working functionality that needs them.

- Prefer the Rust standard library plus `eframe`/`egui`.
- Keep timer transition logic callable by unit tests without launching a GUI.
- Prefer straightforward structs and enums to trait-heavy frameworks or global mutable state.
- Keep one owner for mutable timer state until a real requirement demands sharing.
- Avoid premature optimization and speculative refactoring.
- Match `egui` and `eframe` versions as intended by their upstream dependency relationship.

## 8. Development Workflow

For nontrivial requests:

1. Inspect relevant source files and dependency versions.
2. State the smallest intended implementation and affected files.
3. Identify the one or two key Rust/egui concepts to learn.
4. Implement a focused vertical slice.
5. Run applicable checks.
6. Explain what changed, why it works, and what remains unverified.

Do not turn trivial changes into lengthy plans. Do not modify unrelated code. Do not silently add features or change session semantics.

Suggested progression, only as requested:

1. Open an empty native window with a static countdown label.
2. Add the timer state machine and unit tests.
3. Wire Start / Pause / Resume / Reset / Skip.
4. Update the countdown with elapsed time and scheduled repaints.
5. Add session cycling and a simple progress indicator.
6. Later: configurable durations, persistence, notifications, keyboard shortcuts, or tray integration.

The user should be able to read, explain, and alter each step before the next major feature.

## 9. Testing and Verification

Test timer logic without creating a window, network connection, or audio device. Make time controllable in tests (for example, pass an explicit `Instant` or elapsed `Duration` to transition methods); do not use real multi-minute sleeps.

Cover at least:

- Start -> pause -> resume, with paused time excluded.
- Delayed update after a deadline produces correct completion.
- Exactly one completion on repeated post-deadline updates.
- Reset from idle, running, and paused.
- Skip never increments completed focus sessions.
- Fourth completed focus selects a long break; other completed focuses select a short break.
- Completing a break returns to focus.
- Zero duration and arithmetic boundaries if configurable durations are later introduced.

Check UI layout, readability, interactions, and platform behavior by running the actual application; do not write tests for every pixel or hover effect.

Run relevant checks after code changes:

```bash
cargo fmt --check
cargo check
cargo clippy --all-targets
cargo test
```

Use `--locked` where reproducible dependency resolution is required. Report **only checks actually run**, with their results and any blockers.

## 10. Documentation and Debugging

Prefer sources in this order:

1. Local repository source and pinned dependency versions.
2. Official egui / eframe docs and source: https://github.com/emilk/egui
3. `egui` docs: https://docs.rs/egui/latest/egui/
4. `eframe` docs: https://docs.rs/eframe/latest/eframe/
5. Official Rust docs: https://doc.rust-lang.org/book/
6. Rust standard library: https://doc.rust-lang.org/std/

`latest` documentation is a discovery aid, not proof that an API exists in this project's pinned version. Verify names, signatures, and feature flags before changing code.

When something fails, identify the root cause, apply the smallest fix, and explain it. Do not suppress errors or warnings to make checks pass. For UI bugs, separate incorrect state transitions from incorrect redraw scheduling and incorrect layout.

## 11. Communication and Completion Criteria

Answer in concise, technically precise Chinese, keeping Rust and framework terminology in English. Assume familiarity with C, concurrency, and operating systems.

After substantial changes, report:

- **Changed:** files, functions, and observable behavior.
- **Mechanism:** important state transitions and execution flow.
- **Reasoning:** non-obvious tradeoffs.
- **Verification:** checks performed, results, and manual verification still needed.

For Rust questions, explain ownership and borrowing only where they influence the code. Do not provide unnecessary generic tutorials.

**Success criterion:** the user can explain how an input event, a monotonic-clock reading, a timer transition, and an egui repaint cooperate to produce a correct countdown, and can extend the application independently.
