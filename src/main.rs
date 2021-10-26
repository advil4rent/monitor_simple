use tokio::*;
use monitor_sample::{PeckBoard, PeckKeys, PeckLEDs};
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let mut peck_board = PeckBoard::new("/dev/gpiochip4").await
        .expect("Couldn't initialize PeckBoard chip");
    println!("starting monitor");
    peck_board.monitor().await.unwrap();
    loop {
        println!("A little tune");
        thread::sleep(Duration::from_secs(10000));
    }
    println!("awaiting monitor")
}