use tokio::*;
use monitor_sample::{PeckBoard, PeckKeys, PeckLEDs};
use std::thread;

#[tokio::main]
async fn main() {
    let mut peck_board = PeckBoard::new("/dev/gpiochip4").await
        .expect("Couldn't initialize PeckBoard chip");
    println!("starting monitor");
    peck_board.keys.monitor().await.unwrap();
    loop {
        println!("A little tune");
        thread::sleep_ms(10000);
    }
    println!("awaiting monitor")
}