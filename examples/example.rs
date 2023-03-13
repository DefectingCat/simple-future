use std::time::Duration;

use simple_future::{executor::new_executor_and_spawner, waker::TimerFuture};
use tokio::time::sleep;

fn main() {
    let (exec, spawner) = new_executor_and_spawner();

    spawner.spawn(async {
        println!("howdy!");
        TimerFuture::new(Duration::new(2, 0)).await;
        println!("done!")
    });

    drop(spawner);

    exec.run();
}

async fn simple() {
    sleep(Duration::new(2, 0)).await;
}

async fn async_main() {
    let test = simple();
}
