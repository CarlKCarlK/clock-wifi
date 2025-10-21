use embassy_rp::{
    gpio::{self, Level},
    peripherals::{DMA_CH0, PIN_23, PIN_24, PIN_25, PIN_29, PIO0},
    Peri,
};

use crate::{output_array::OutputArray, CELL_COUNT, SEGMENT_COUNT};

/// WiFi hardware peripherals
pub struct WifiHardware {
    pub pin_23: Peri<'static, PIN_23>,  // WiFi power enable
    pub pin_25: Peri<'static, PIN_25>,  // WiFi SPI chip select
    pub pio0: Peri<'static, PIO0>,      // WiFi PIO block for SPI
    pub pin_24: Peri<'static, PIN_24>,  // WiFi SPI MOSI
    pub pin_29: Peri<'static, PIN_29>,  // WiFi SPI CLK
    pub dma_ch0: Peri<'static, DMA_CH0>, // WiFi DMA channel for SPI
}

/// Represents the hardware components of the clock.
pub struct Hardware {
    // TODO replace the 'static's with <'a> lifetimes
    /// The four cell pins that control the digits of the display.
    pub cells: OutputArray<'static, CELL_COUNT>,
    /// The eight segment pins that control the segments of the display.
    pub segments: OutputArray<'static, SEGMENT_COUNT>,
    /// The button that controls the clock.
    pub button: gpio::Input<'static>,
    /// An LED (not currently used).
    pub led: gpio::Output<'static>,
    /// WiFi hardware peripherals
    pub wifi: WifiHardware,
}

impl Default for Hardware {
    fn default() -> Self {
        let peripherals: embassy_rp::Peripherals =
            embassy_rp::init(embassy_rp::config::Config::default());

        let led = gpio::Output::new(peripherals.PIN_0, Level::Low);

        let cells = OutputArray::new([
            gpio::Output::new(peripherals.PIN_1, Level::High),
            gpio::Output::new(peripherals.PIN_2, Level::High),
            gpio::Output::new(peripherals.PIN_3, Level::High),
            gpio::Output::new(peripherals.PIN_4, Level::High),
        ]);

        let segments = OutputArray::new([
            gpio::Output::new(peripherals.PIN_5, Level::Low),
            gpio::Output::new(peripherals.PIN_6, Level::Low),
            gpio::Output::new(peripherals.PIN_7, Level::Low),
            gpio::Output::new(peripherals.PIN_8, Level::Low),
            gpio::Output::new(peripherals.PIN_9, Level::Low),
            gpio::Output::new(peripherals.PIN_10, Level::Low),
            gpio::Output::new(peripherals.PIN_11, Level::Low),
            gpio::Output::new(peripherals.PIN_12, Level::Low),
        ]);

        let button = gpio::Input::new(peripherals.PIN_13, gpio::Pull::Down);

        let wifi = WifiHardware {
            pin_23: peripherals.PIN_23,
            pin_25: peripherals.PIN_25,
            pio0: peripherals.PIO0,
            pin_24: peripherals.PIN_24,
            pin_29: peripherals.PIN_29,
            dma_ch0: peripherals.DMA_CH0,
        };

        Self {
            cells,
            segments,
            button,
            led,
            wifi,
        }
    }
}
