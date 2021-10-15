use gpio_cdev::{Chip, LineRequestFlags, LineEventHandle, LineHandle, MultiLineHandle, EventRequestFlags, EventType, errors::Error as GpioError, LineEvent};
use tokio::{task::JoinHandle,
            time::Duration,
            //sync::{oneshot, mpsc}
};
use thiserror;
use std::{sync::{Arc, Mutex, }, iter};
//                atomic::{AtomicI8, AtomicUsize, Ordering}}};
use std::os::unix::io::AsRawFd;
use nix::poll::*;
use std::iter::Map;

type PollEventFlags = nix::poll::PollFlags;

pub struct PeckLEDs {
    pub right_blue: LineHandle, //0
    pub center_blue:LineHandle, //1
    pub left_blue: LineHandle, //2
    pub right_red: LineHandle, //3
    pub center_red: LineHandle, //4
    pub left_red: LineHandle, //5
    pub right_green: LineHandle, //6
    pub center_green: LineHandle, //7
    pub left_green: LineHandle, //8
    right_ir: LineHandle, //9
    center_ir: LineHandle, //10
    left_ir: LineHandle //11
}
pub struct PeckKeys {
    pub right: Arc<Mutex<LineHandle>>,  //13
    pub center: Arc<Mutex<LineHandle>>, //14
    pub left: Arc<Mutex<LineHandle>>,   //15
    pub all_keys: Arc<Mutex<MultiLineHandle>>
}
pub struct PeckBoard {
    pub leds: PeckLEDs,
    pub keys: PeckKeys,
}

impl PeckBoard {
    pub async fn new (chip: &str) -> Result<Self, Error> {
        let mut chip = Chip::new(chip).map_err(|e:GpioError|
            Error::ChipError {source: e,
            chip: ChipNumber::Chip4}
        )?;
        let keys = PeckKeys::new(&mut chip)?;
        let leds = PeckLEDs::new(&mut chip)?;
        Ok(PeckBoard{
            leds,
            keys
        })
    }
}
impl PeckLEDs {
    const LINES: [u32;2] = [0,1];
    pub fn new(chip: &mut Chip) -> Result<Self, Error> {
        let line_handles: Vec<LineHandle> = Self::LINES.iter()
            .map(|&offset| {
                chip.get_line(offset).unwrap()
                    .request(LineRequestFlags::OUTPUT, 0, "peckboard")
                    .unwrap()
            }).collect();

        let right_blue = chip
            .get_line(0)
            .map_err(|e: GpioError| Error::LineGetError { source: e, line: 0 })?
            .request(LineRequestFlags::OUTPUT, 0, "peckboard")
            .map_err(|e: GpioError| Error::LineReqError { source: e, line: 0 })?;
        let center_blue = chip
            .get_line(1)
            .map_err(|e: GpioError| Error::LineGetError { source: e, line: 1 })?
            .request(LineRequestFlags::OUTPUT, 0, "peckboard")
            .map_err(|e: GpioError| Error::LineReqError { source: e, line: 1 })?;
        let left_blue = chip
            .get_line(2)
            .map_err(|e: GpioError| Error::LineGetError { source: e, line: 2 })?
            .request(LineRequestFlags::OUTPUT, 0, "peckboard")
            .map_err(|e: GpioError| Error::LineReqError { source: e, line: 2 })?;
        let right_red = chip
            .get_line(3)
            .map_err(|e: GpioError| Error::LineGetError { source: e, line: 3 })?
            .request(LineRequestFlags::OUTPUT, 0, "peckboard")
            .map_err(|e: GpioError| Error::LineReqError { source: e, line: 3 })?;
        let center_red = chip
            .get_line(4)
            .map_err(|e: GpioError| Error::LineGetError { source: e, line: 4 })?
            .request(LineRequestFlags::OUTPUT, 0, "peckboard")
            .map_err(|e: GpioError| Error::LineReqError { source: e, line: 4 })?;
        let left_red = chip
            .get_line(5)
            .map_err(|e: GpioError| Error::LineGetError { source: e, line: 5 })?
            .request(LineRequestFlags::OUTPUT, 0, "peckboard")
            .map_err(|e: GpioError| Error::LineReqError { source: e, line: 5 })?;
        let right_green = chip
            .get_line(6)
            .map_err(|e: GpioError| Error::LineGetError { source: e, line: 6 })?
            .request(LineRequestFlags::OUTPUT, 0, "peckboard")
            .map_err(|e: GpioError| Error::LineReqError { source: e, line: 6 })?;
        let center_green = chip
            .get_line(7)
            .map_err(|e: GpioError| Error::LineGetError { source: e, line: 7 })?
            .request(LineRequestFlags::OUTPUT, 0, "peckboard")
            .map_err(|e: GpioError| Error::LineReqError { source: e, line: 7 })?;
        let left_green = chip
            .get_line(8)
            .map_err(|e: GpioError| Error::LineGetError { source: e, line: 8 })?
            .request(LineRequestFlags::OUTPUT, 0, "peckboard")
            .map_err(|e: GpioError| Error::LineReqError { source: e, line: 8 })?;
        let right_ir = chip
            .get_line(9)
            .map_err(|e: GpioError| Error::LineGetError { source: e, line: 9 })?
            .request(LineRequestFlags::OUTPUT, 1, "peckboard")
            .map_err(|e: GpioError| Error::LineReqError { source: e, line: 9 })?;
        let center_ir = chip
            .get_line(10)
            .map_err(|e: GpioError| Error::LineGetError { source: e, line: 10 })?
            .request(LineRequestFlags::OUTPUT, 1, "peckboard")
            .map_err(|e: GpioError| Error::LineReqError { source: e, line: 10 })?;
        let left_ir = chip
            .get_line(11)
            .map_err(|e: GpioError| Error::LineGetError { source: e, line: 11 })?
            .request(LineRequestFlags::OUTPUT, 0, "peckboard")
            .map_err(|e: GpioError| Error::LineReqError { source: e, line: 11 })?;

        Ok(PeckLEDs{
            right_blue,
            center_blue,
            left_blue,
            right_red,
            center_red,
            left_red,
            right_green,
            center_green,
            left_green,
            right_ir,
            center_ir,
            left_ir
        })
    }
}
impl PeckKeys {
    const INTERRUPT_CHIP: &'static str = "/dev/gpiochip2";
    const INTERRUPT_LINE: u32 = 24;
    const PECK_KEY_LINES: [u32; 3] = [13,14,15];

    pub fn new(chip: &mut Chip) -> Result<Self, Error> {
        //TODO: make sure request flags for key lines are correct
        let right_key = chip
            .get_line(13)
            .map_err(|e: GpioError| Error::LineGetError { source: e, line: 13 })?
            .request(LineRequestFlags::INPUT, 1, "peckboard")
            .map_err(|e: GpioError| Error::LineReqError { source: e, line: 13 })?;
        let right_key = Arc::new(Mutex::new(right_key));
        let center_key = chip
            .get_line(14)
            .map_err(|e: GpioError| Error::LineGetError { source: e, line: 14 })?
            .request(LineRequestFlags::INPUT, 1, "peckboard")
            .map_err(|e: GpioError| Error::LineReqError { source: e, line: 14 })?;
        let center_key = Arc::new(Mutex::new(center_key));
        let left_key = chip
            .get_line(15)
            .map_err(|e: GpioError| Error::LineGetError { source: e, line: 15 })?
            .request(LineRequestFlags::INPUT, 1, "peckboard")
            .map_err(|e: GpioError| Error::LineReqError { source: e, line: 15 })?;
        let left_key = Arc::new(Mutex::new(left_key));
        let all_keys = chip
            .get_lines(&[13,14,15])
            .map_err(|e:GpioError| Error::LinesGetError {source: e, lines: &[13,14,15]})?
            .request(LineRequestFlags::INPUT, &[1,1,1], "stepper")
            .map_err(|e:GpioError| Error::LinesReqError {source: e, lines: &[13,14,15]})?;
        let all_keys = Arc::new(Mutex::new(all_keys));

        Ok(PeckKeys{
            right: right_key,
            center: center_key,
            left: left_key,
            all_keys
        })
    }

    pub async fn monitor(&mut self) -> Result<(), Error> {

        tokio::spawn( async move {
            let mut chip2 = Chip::new(&Self::INTERRUPT_CHIP)
                .map_err(|e:GpioError| Error::ChipError {source: e, chip: ChipNumber::Chip2})
                .unwrap();
            let mut evt_handles: Vec<LineEventHandle> = iter::once(Self::INTERRUPT_LINE)
                .map(|offset| {
                    let line = chip2.get_line(offset)
                        .map_err(|e:GpioError| Error::LineGetError {source: e, line: 24}).unwrap();
                            //TODO: figure out how to convert &&u32 to u8 for map_err above
                    line.events(
                        LineRequestFlags::INPUT,
                        EventRequestFlags::FALLING_EDGE,
                        "peck_interrupt_monitor",
                    ).unwrap()
                }).collect();
            let mut pollfds: Vec<PollFd> = evt_handles.iter()
                .map(|handle| {
                    PollFd::new(handle.as_raw_fd(),
                                PollEventFlags::POLLIN| PollEventFlags::POLLPRI)
                })
                .collect();

            let mut chip4 = Chip::new("/dev/gpiochip4")
                .map_err(|e:GpioError| Error::ChipError {source: e, chip: ChipNumber::Chip4})
                .unwrap();
            let key_handles: Vec<LineHandle> = Self::PECK_KEY_LINES.iter()
                .map(|&offset| {
                    chip4.get_line(offset).map_err(|e:GpioError| Error::LineGetError {source: e, line: 24}).unwrap()
                        .request(LineRequestFlags::INPUT, 0, "peck_key_monitor")
                        .unwrap()
                }).collect();

            loop {
                if poll(&mut pollfds, -1).unwrap() == 0 {
                    println!("Timeout?!?");
                } else {
                    for i in 0..pollfds.len() {
                        if let Some(revents) = pollfds[i].revents() {
                            let h = &mut evt_handles[i];
                            h.get_event(); //get_event removes the latest event to prevent infinite looping
                            if revents.contains(PollEventFlags::POLLIN) {
                                for handle in key_handles.iter() {
                                    let val = handle.get_value().unwrap();
                                    if val == 1 {
                                        println!("Key {:?} pecked", handle.line().offset())
                                    }
                                }
                            } else if revents.contains(PollEventFlags::POLLPRI) {
                                println!("[{}] Got a POLLPRI", h.line().offset());
                            }
                        }
                    }
                }
            }
        });
        Ok(())
    }

}

#[derive(Debug)]
pub enum ChipNumber {
    Chip1,
    Chip2,
    Chip3,
    Chip4,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to get chip {chip:?}")]
    ChipError {
        source: GpioError,
        chip: ChipNumber,
    },
    #[error("Failed to get line")]
    LineGetError {
        source: GpioError,
        line: u8,
    },
    #[error("Failed to request line")]
    LineReqError {
        source: GpioError,
        line: u8,
    },
    #[error("Failed to get lines")]
    LinesGetError {
        source: GpioError,
        lines: &'static [u32],
    },
    #[error("Failed to request lines")]
    LinesReqError {
        source: GpioError,
        lines: &'static [u32],
    },
    #[error("Failed to set lines")]
    LinesSetError {
        source: GpioError,
        lines: &'static [u32],
    },
}