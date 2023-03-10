use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};

pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

/// 在 Future 和等待的线程间共享状态
struct SharedState {
    /// 定时（睡眠）是否结束
    completed: bool,

    /// 当睡眠结束后，线程可以用 `waker` 通知 `TimerFuture` 来唤醒任务
    waker: Option<Waker>,
}

impl Future for TimerFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 通过检查共享状态，来确定定时器是否已经完成
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            // 设置 `wkaer`，这样新线程在睡眠结束后可以唤醒当前的任务，接着再次对 `Furture` 进行
            // `poll` 操作
            //
            // 下面的`clone`每次被`poll`时都会发生一次，实际上，应该是只`clone`一次更加合理。
            // 选择每次都`clone`的原因是： `TimerFuture`可以在执行器的不同任务间移动，如果只克隆一次，
            // 那么获取到的`waker`可能已经被篡改并指向了其它任务，最终导致执行器运行了错误的任务
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
