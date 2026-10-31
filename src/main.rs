fn main() {
    println!("Hello, world!");
}

use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Phase {
    Focus,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Status {
    Idle,
    Running,
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
            remaining: Duration::from_secs(25 * 60),
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
            // if start() has not been called
            None => self.remaining,
            // started
            Some(start) => {
                let elapsed = now.saturating_duration_since(start);
                self.remaining.saturating_sub(elapsed)
            }
        }
    }

    fn completed_focus_sessions(&self) -> u32 {
        self.completed_focus_sessions
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
}
