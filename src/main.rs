use tokio::*;
use monitor_sample::{PeckBoard, PeckKeys, PeckLEDs};

#[tokio::main]
async fn main() {
    let mut peck_board = PeckBoard::new("/dev/gpiochip4").await
        .expect("Couldn't initialize PeckBoard chip");
    peck_board.keys.monitor().await.unwrap();
}