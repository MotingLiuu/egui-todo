# Note

## timer.remaining()

Original:

```rust
fn remaining(&self) -> Duration {
    self.remaining
}
```

Task:

```rust
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
```

1. `remaining()` 接受一个 `Instant`，之后返回一个剩余时间的 `Duration`
2. `timer.remaining` 接受一个 `later` 就可以计算剩余的时间，因此必须用某种方式保存 `Instant` 到 `timer` 中。
