pub mod executor;
pub mod future;
pub mod waker;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{executor::new_executor_and_spawner, waker::TimerFuture};

    #[test]
    fn it_works() {
        let (exec, spawner) = new_executor_and_spawner();

        spawner.spawn(async {
            println!("howdy!");
            TimerFuture::new(Duration::new(2, 0)).await;
            println!("done!")
        });

        drop(spawner);

        exec.run();
    }
}
