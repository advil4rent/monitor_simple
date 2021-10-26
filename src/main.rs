use peckboard_test::{PeckBoard};
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let peck_board = PeckBoard::new("/dev/gpiochip4").await
        .expect("Couldn't initialize PeckBoard chip");
    peck_board.monitor().await.unwrap();
    loop {
        println!("Main thread doing nothing");
        thread::sleep(Duration::from_secs(100));
    }

}