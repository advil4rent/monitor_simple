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
    line_handles: Vec<LineHandle>
}
pub struct PeckKeys {
    //pub right: Arc<Mutex<LineHandle>>,  //13
    //pub center: Arc<Mutex<LineHandle>>, //14
    //pub left: Arc<Mutex<LineHandle>>,   //15
    //pub all_keys: Arc<Mutex<MultiLineHandle>>
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

        Ok(PeckLEDs{
            line_handles
        })
    }
}
impl PeckKeys {
    const INTERRUPT_CHIP: &'static str = "/dev/gpiochip2";
    const INTERRUPT_LINE: u32 = 24;
    const PECK_KEY_LINES: [u32; 3] = [13,14,15];

    pub fn new(chip: &mut Chip) -> Result<Self, Error> {

        Ok(PeckKeys{

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