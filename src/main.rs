fn main() {
    println!("Hello, world!");
}

use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Phase {
    Focus,
    ShortBreak,
    LongBreak,
}

impl Phase {
    fn duration(self) -> Duration {
        match self {
            Self::Focus => Duration::from_secs(25 * 60),
            Self::ShortBreak => Duration::from_secs(5 * 60),
            Self::LongBreak => Duration::from_secs(15 * 60),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Status {
    Idle,
    Running,
    Paused,
}

struct Timer {
    phase: Phase,
    status: Status,
    remaining: Duration,
    completed_focus_sessions: u32,
    //
    start_time: Option<Instant>,
}

impl Timer {
    fn new() -> Self {
        Self {
            phase: Phase::Focus,
            status: Status::Idle,
            remaining: Phase::Focus.duration(),
            // Duration is **time duration type** in std lib
            // from_sec() transform seconds to a Duration type
            //
            // pub struct Duration {
            //    secs: u64,
            //    nanos: u32,
            //
            // }
            completed_focus_sessions: 0,
            //
            start_time: None,
        }
    }

    fn start(&mut self, now: std::time::Instant) {
        if self.status == Status::Idle {
            self.status = Status::Running;
            self.start_time = Some(now);
        }
    }

    fn phase(&self) -> Phase {
        self.phase
    }

    fn status(&self) -> Status {
        self.status
    }

    fn remaining(&self, now: std::time::Instant) -> Duration {
        match self.start_time {
            None => self.remaining,
            Some(start) => {
                let elapsed = now.saturating_duration_since(start);
                self.remaining.saturating_sub(elapsed)
            }
        }
    }

    fn completed_focus_sessions(&self) -> u32 {
        self.completed_focus_sessions
    }

    fn pause(&mut self, now: Instant) {
        // A session already at its deadline completes instead of pausing at zero.
        self.update(now);
        if self.status == Status::Running {
            self.remaining = self.remaining(now);
            self.start_time = None;
            self.status = Status::Paused;
        }
    }

    fn resume(&mut self, now: Instant) {
        if self.status == Status::Paused {
            self.start_time = Some(now);
            self.status = Status::Running;
        }
    }

    fn reset(&mut self) {
        self.status = Status::Idle;
        self.start_time = None;
        self.remaining = self.phase.duration();
    }

    // Explicit reset/skip actions discard progress without counting a completion.
    fn skip(&mut self) {
        self.phase = self.next_phase();
        self.reset();
    }

    fn next_phase(&self) -> Phase {
        match self.phase {
            Phase::Focus => {
                if self.completed_focus_sessions > 0
                    && self.completed_focus_sessions.is_multiple_of(4)
                {
                    Phase::LongBreak
                } else {
                    Phase::ShortBreak
                }
            }
            Phase::ShortBreak | Phase::LongBreak => Phase::Focus,
        }
    }

    fn update(&mut self, now: Instant) {
        if self.status != Status::Running || !self.remaining(now).is_zero() {
            return;
        }

        if self.phase == Phase::Focus {
            self.completed_focus_sessions = self.completed_focus_sessions.saturating_add(1);
        }
        self.phase = self.next_phase();
        self.reset();
    }
}

#[cfg(test)]
// 只在编译测试时包含这个 module，普通构建不包含它。
mod tests {
    // 定义一个子 module。现在直接写在同一个文件里，不必创建新文件
    use super::*;
    // use super::*：将父 module 中的名字引入测试作用域；之后定义在外面的 Timer 等类型就可以直接使用。
    // Question:
    // mod 是什么？ 相当于c中另外写了一个不同的 test.c 文件？ user super::* 类似于 #include？

    #[test]
    // 告诉 Rust test harness，这是需要执行的测试函数
    fn new_timer_is_idle_focus_with_full_duration() {
        let timer = Timer::new();
        let now = Instant::now();

        assert_eq!(timer.phase(), Phase::Focus);
        assert_eq!(timer.status(), Status::Idle);
        assert_eq!(timer.remaining(now), Duration::from_secs(25 * 60),);
        assert_eq!(timer.completed_focus_sessions(), 0);
    }

    #[test]
    fn start_changes_idle_timer_to_running() {
        let mut timer = Timer::new();
        let now = std::time::Instant::now();

        timer.start(now);

        assert_eq!(timer.status(), Status::Running);
    }

    #[test]
    fn running_timer_remaining_uses_elapsed_time() {
        let mut timer = Timer::new();
        let start = std::time::Instant::now();

        timer.start(start);

        let later = start + std::time::Duration::from_secs(3);

        assert_eq!(
            timer.remaining(later),
            std::time::Duration::from_secs(25 * 60 - 3),
        );
    }

    #[test]
    fn remaining_is_zero_after_deadline() {
        let mut timer = Timer::new();
        let start = Instant::now();

        timer.start(start);

        let later = start + Duration::from_secs(25 * 60 + 1);

        assert_eq!(timer.remaining(later), Duration::ZERO);
    }

    #[test]
    fn pause_freezes_remaining_time() {
        let mut timer = Timer::new();
        let start = Instant::now();
        let paused_at = start + Duration::from_secs(3);
        let later = start + Duration::from_secs(13);

        timer.start(start);
        timer.pause(paused_at);

        assert_eq!(timer.status(), Status::Paused);
        assert_eq!(timer.remaining(paused_at), Duration::from_secs(25 * 60 - 3),);
        assert_eq!(timer.remaining(later), Duration::from_secs(25 * 60 - 3),);
    }

    #[test]
    fn resume_excludes_paused_time() {
        let mut timer = Timer::new();
        let start = Instant::now();
        let paused_at = start + Duration::from_secs(3);
        let resumed_at = start + Duration::from_secs(13);
        let later = start + Duration::from_secs(15);

        timer.start(start);
        timer.pause(paused_at);
        timer.resume(resumed_at);

        assert_eq!(timer.status(), Status::Running);
        assert_eq!(
            timer.remaining(resumed_at),
            Duration::from_secs(25 * 60 - 3),
        );
        assert_eq!(timer.remaining(later), Duration::from_secs(25 * 60 - 5),);
    }

    #[test]
    fn reset_running_timer_restores_idle_and_full_duration() {
        let mut timer = Timer::new();
        let start = Instant::now();

        timer.start(start);
        timer.reset();

        let later = start + Duration::from_secs(10);

        assert_eq!(timer.status(), Status::Idle);
        assert_eq!(timer.phase(), Phase::Focus);
        assert_eq!(timer.remaining(later), Duration::from_secs(25 * 60),);
        assert_eq!(timer.completed_focus_sessions(), 0);
    }

    #[test]
    fn reset_paused_timer_restores_idle_and_full_duration() {
        let mut timer = Timer::new();
        let start = Instant::now();
        let paused_at = start + Duration::from_secs(3);

        timer.start(start);
        timer.pause(paused_at);

        assert_eq!(timer.remaining(paused_at), Duration::from_secs(25 * 60 - 3),);

        timer.reset();

        let later = start + Duration::from_secs(20);

        assert_eq!(timer.status(), Status::Idle);
        assert_eq!(timer.phase(), Phase::Focus);
        assert_eq!(timer.remaining(later), Duration::from_secs(25 * 60),);
    }

    #[test]
    fn update_before_deadline_keeps_focus_running() {
        let mut timer = Timer::new();
        let start = Instant::now();
        let later = start + Duration::from_secs(3);

        timer.start(start);
        timer.update(later);

        assert_eq!(timer.phase(), Phase::Focus);
        assert_eq!(timer.status(), Status::Running);
        assert_eq!(timer.completed_focus_sessions(), 0);
        assert_eq!(timer.remaining(later), Duration::from_secs(25 * 60 - 3));
    }

    #[test]
    fn repeated_updates_after_deadline_complete_focus_only_once() {
        let mut timer = Timer::new();
        let start = Instant::now();
        let deadline = start + Duration::from_secs(25 * 60);
        let later = deadline + Duration::from_secs(10 * 60);

        timer.start(start);
        timer.update(deadline);
        timer.update(deadline);
        timer.update(later);

        assert_eq!(timer.completed_focus_sessions(), 1);
        assert_eq!(timer.phase(), Phase::ShortBreak);
        assert_eq!(timer.status(), Status::Idle);
        assert_eq!(timer.remaining(later), Duration::from_secs(5 * 60));
    }

    #[test]
    fn completing_short_break_returns_to_idle_focus() {
        let mut timer = Timer::new();
        let start = Instant::now();
        let focus_deadline = start + Duration::from_secs(25 * 60);
        let break_deadline = focus_deadline + Duration::from_secs(5 * 60);

        timer.start(start);
        timer.update(focus_deadline);
        timer.start(focus_deadline);
        timer.update(break_deadline);

        assert_eq!(timer.phase(), Phase::Focus);
        assert_eq!(timer.status(), Status::Idle);
        assert_eq!(timer.completed_focus_sessions(), 1);
        assert_eq!(
            timer.remaining(break_deadline),
            Duration::from_secs(25 * 60)
        );
    }

    #[test]
    fn reset_short_break_preserves_phase_and_completed_count() {
        let mut timer = Timer::new();
        let start = Instant::now();
        let focus_deadline = start + Duration::from_secs(25 * 60);
        let later = focus_deadline + Duration::from_secs(3);

        timer.start(start);
        timer.update(focus_deadline);
        timer.start(focus_deadline);
        timer.reset();

        assert_eq!(timer.phase(), Phase::ShortBreak);
        assert_eq!(timer.status(), Status::Idle);
        assert_eq!(timer.completed_focus_sessions(), 1);
        assert_eq!(timer.remaining(later), Duration::from_secs(5 * 60));
    }

    #[test]
    fn every_fourth_completed_focus_selects_long_break() {
        let mut timer = Timer::new();
        let mut now = Instant::now();

        for count in 1..=8 {
            timer.start(now);
            now += Phase::Focus.duration();
            timer.update(now);

            let expected = if count % 4 == 0 {
                Phase::LongBreak
            } else {
                Phase::ShortBreak
            };
            assert_eq!(timer.completed_focus_sessions(), count);
            assert_eq!(timer.phase(), expected);
            assert_eq!(timer.status(), Status::Idle);
            assert_eq!(timer.remaining(now), expected.duration());

            timer.start(now);
            now += expected.duration();
            timer.update(now);
            assert_eq!(timer.phase(), Phase::Focus);
            assert_eq!(timer.status(), Status::Idle);
            assert_eq!(timer.remaining(now), Phase::Focus.duration());
            assert_eq!(timer.completed_focus_sessions(), count);
        }
    }

    #[test]
    fn reset_restores_each_phase_from_every_status_without_clearing_count() {
        let now = Instant::now();
        for phase in [Phase::Focus, Phase::ShortBreak, Phase::LongBreak] {
            for status in [Status::Idle, Status::Running, Status::Paused] {
                let mut timer = Timer::new();
                timer.phase = phase;
                timer.remaining = phase.duration();
                timer.completed_focus_sessions = 4;
                if status != Status::Idle {
                    timer.start(now);
                }
                if status == Status::Paused {
                    timer.pause(now + Duration::from_secs(3));
                }

                timer.reset();

                assert_eq!(timer.phase(), phase);
                assert_eq!(timer.status(), Status::Idle);
                assert_eq!(timer.start_time, None);
                assert_eq!(
                    timer.remaining(now + Duration::from_secs(20)),
                    phase.duration()
                );
                assert_eq!(timer.completed_focus_sessions(), 4);
            }
        }
    }

    #[test]
    fn skip_uses_completed_count_without_incrementing_it_from_every_status() {
        let now = Instant::now();
        for count in [0, 1, 3, 4, 7, 8] {
            for status in [Status::Idle, Status::Running, Status::Paused] {
                let mut timer = Timer::new();
                timer.completed_focus_sessions = count;
                if status != Status::Idle {
                    timer.start(now);
                }
                if status == Status::Paused {
                    timer.pause(now + Duration::from_secs(3));
                }

                timer.skip();

                let expected = if count > 0 && count % 4 == 0 {
                    Phase::LongBreak
                } else {
                    Phase::ShortBreak
                };
                assert_eq!(timer.phase(), expected);
                assert_eq!(timer.status(), Status::Idle);
                assert_eq!(timer.start_time, None);
                assert_eq!(timer.remaining(now), expected.duration());
                assert_eq!(timer.completed_focus_sessions(), count);

                timer.skip();
                assert_eq!(timer.phase(), Phase::Focus);
                assert_eq!(timer.remaining(now), Phase::Focus.duration());
                assert_eq!(timer.completed_focus_sessions(), count);
            }
        }
    }

    #[test]
    fn invalid_and_repeated_actions_do_not_restart_the_timer() {
        let mut timer = Timer::new();
        let start = Instant::now();
        let later = start + Duration::from_secs(3);

        timer.pause(start);
        timer.resume(start);
        timer.update(later);
        assert_eq!(timer.status(), Status::Idle);
        assert_eq!(timer.remaining(later), Phase::Focus.duration());

        timer.start(start);
        timer.start(later);
        timer.resume(later);
        assert_eq!(timer.remaining(later), Duration::from_secs(1497));

        timer.pause(later);
        timer.pause(later + Duration::from_secs(10));
        timer.start(later + Duration::from_secs(10));
        timer.update(start + Duration::from_secs(10_000));
        assert_eq!(timer.status(), Status::Paused);
        assert_eq!(timer.remaining(later), Duration::from_secs(1497));
        assert_eq!(timer.completed_focus_sessions(), 0);
    }

    #[test]
    fn pause_at_or_after_deadline_completes_exactly_once() {
        for delay in [0, 10] {
            let mut timer = Timer::new();
            let start = Instant::now();
            let later = start + Phase::Focus.duration() + Duration::from_secs(delay);
            timer.start(start);
            timer.pause(later);
            timer.pause(later);
            timer.update(later);

            assert_eq!(timer.phase(), Phase::ShortBreak);
            assert_eq!(timer.status(), Status::Idle);
            assert_eq!(timer.completed_focus_sessions(), 1);
            assert_eq!(timer.remaining(later), Phase::ShortBreak.duration());
        }
    }

    #[test]
    fn querying_before_start_time_does_not_underflow() {
        let mut timer = Timer::new();
        let earlier = Instant::now();
        timer.start(earlier + Duration::from_secs(1));
        assert_eq!(timer.remaining(earlier), Phase::Focus.duration());
    }

    #[test]
    fn completion_counter_saturates_without_overflow() {
        let mut timer = Timer::new();
        let start = Instant::now();
        timer.completed_focus_sessions = u32::MAX;
        timer.start(start);
        timer.update(start + Phase::Focus.duration());
        assert_eq!(timer.completed_focus_sessions(), u32::MAX);
        assert_eq!(timer.status(), Status::Idle);
    }

    #[test]
    fn delayed_update_completes_focus_and_prepares_idle_short_break() {
        let mut timer = Timer::new();
        let start = Instant::now();
        let later = start + Duration::from_secs(25 * 60 + 7);

        timer.start(start);
        timer.update(later);

        assert_eq!(timer.completed_focus_sessions(), 1);
        assert_eq!(timer.phase(), Phase::ShortBreak);
        assert_eq!(timer.status(), Status::Idle);
        assert_eq!(timer.remaining(later), Duration::from_secs(5 * 60),);
    }
}
