use gpio_cdev::{Chip, AsyncLineEventHandle,
                LineRequestFlags, LineEventHandle,
                LineHandle, MultiLineHandle,
                EventRequestFlags, EventType,
                errors::Error as GpioError, LineEvent};
use futures::stream::StreamExt;
use tokio::{task::JoinHandle,
            time::Duration,
            //sync::{oneshot, mpsc}
};
use thiserror;
use std::{sync::{Arc, Mutex, }, iter};
//                atomic::{AtomicI8, AtomicUsize, Ordering}}};
use std::iter::Map;

pub struct PeckLEDs {
    pub light_handles: Vec<LineHandle>,
    pub ir_handles: Vec<LineHandle>,
}
pub struct PeckKeys {
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
    const LINES: [u32;9] = [0,1,2,3,4,5,6,7,8];
    const IR: [u32; 3] = [9,10,11];
    pub fn new(chip: &mut Chip) -> Result<Self, Error> {
        let light_handles: Vec<LineHandle> = Self::LINES.iter()
            .map(|&offset| {
                chip.get_line(offset).unwrap()
                    .request(LineRequestFlags::OUTPUT, 0, "peckboard")
                    .unwrap()
            }).collect();
        let ir_handles: Vec<LineHandle> = Self::IR.iter()
            .map(|&offset| {
                chip.get_line(offset).unwrap()
                    .request(LineRequestFlags::OUTPUT, 1, "peckboard_ir")
                    .unwrap()
            }).collect();
        Ok(PeckLEDs{
            light_handles,
            ir_handles,
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
            let interrupt_line = chip2.get_line(Self::INTERRUPT_LINE)
                .map_err(|e:GpioError| Error::LineGetError {source:e, line: Self::INTERRUPT_LINE}).unwrap();
            let mut events = AsyncLineEventHandle::new(interrupt_line.events(
                LineRequestFlags::INPUT,
                EventRequestFlags::FALLING_EDGE,
                "async peckboard interrupt",
            ).unwrap()).unwrap();

            let mut chip4 = Chip::new("/dev/gpiochip4")
                .map_err(|e:GpioError| Error::ChipError {source: e, chip: ChipNumber::Chip4})
                .unwrap();
            let key_handles: MultiLineHandle = chip4.get_lines(&Self::PECK_KEY_LINES).unwrap()
                .request(LineRequestFlags::INPUT, &[0,0,0], "peck_keys").unwrap();

            loop {
                match events.next().await {
                    Some(event) => { println!("Values are: {:?}", key_handles.get_values().unwrap()) },
                    None => break,
                };
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
        line: u32,
    },
    #[error("Failed to request line")]
    LineReqError {
        source: GpioError,
        line: u32,
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