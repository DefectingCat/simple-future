use std::pin::Pin;

use {
    futures::task::{waker_ref, ArcWake},
    std::{
        future::Future,
        sync::mpsc::{sync_channel, Receiver, SyncSender},
        sync::{Arc, Mutex},
        task::Context,
    },
    // 引入之前实现的定时器模块
};

pub struct Executor {
    pub ready_queue: Receiver<Arc<Task>>,
}

#[derive(Clone)]
pub struct Spawner {
    pub task_sender: SyncSender<Arc<Task>>,
}

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

pub struct Task {
    pub future: Mutex<Option<BoxFuture<'static, ()>>>,

    pub task_sender: SyncSender<Arc<Task>>,
}

pub fn new_executor_and_spawner() -> (Executor, Spawner) {
    const MAX_QUEUED_TAKSS: usize = 10_000;
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TAKSS);
    (Executor { ready_queue }, Spawner { task_sender })
}

impl Spawner {
    pub fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = Box::pin(future);
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });
        self.task_sender.send(task).expect("任务队列已满");
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let cloned = arc_self.clone();
        arc_self.task_sender.send(cloned).expect("任务队列已满");
    }
}

impl Executor {
    pub fn run(&self) {
        while let Ok(task) = self.ready_queue.recv() {
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&*waker);
                if future.as_mut().poll(context).is_pending() {
                    *future_slot = Some(future)
                }
            }
        }
    }
}
