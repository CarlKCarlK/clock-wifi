//! A 4-digit 7-segment clock that can be controlled by a button.
//!
//! Runs on a Raspberry Pi Pico RP2040. See the `README.md` for more information.
#![no_std]
#![no_main]
#![feature(never_type)]
#![allow(clippy::future_not_send, reason = "Single-threaded")]
use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use lib::{
    BlinkState, Blinker, BlinkerNotifier, Button, Clock, ClockNotifier, ClockState, Display,
    DisplayNotifier, Result, TimeSync, TimeSyncNotifier,
}; // This crate's own internal library
use panic_probe as _;

#[embassy_executor::main]
pub async fn main(spawner0: Spawner) -> ! {
    // If it returns, something went wrong.
    let err = inner_main(spawner0).await.unwrap_err();
    panic!("{err}");
}

#[expect(clippy::items_after_statements, reason = "Keeps related code together")]
async fn inner_main(spawner: Spawner) -> Result<!> {
    let hardware = lib::Hardware::default();

    // Create TimeSync virtual device (creates WiFi internally) - not used yet
    static TIME_SYNC: TimeSyncNotifier = TimeSync::notifier();
    let _time_sync = TimeSync::new(
        &TIME_SYNC,
        hardware.wifi.pin_23,
        hardware.wifi.pin_25,
        hardware.wifi.pio0,
        hardware.wifi.pin_24,
        hardware.wifi.pin_29,
        hardware.wifi.dma_ch0,
        spawner,
    );

    static CLOCK_NOTIFIER: ClockNotifier = Clock::notifier();
    let mut clock = Clock::new(hardware.cells, hardware.segments, &CLOCK_NOTIFIER, spawner)?;
    let mut button = Button::new(hardware.button);
    info!("Clock and button created");

    // Run the state machine
    let mut state = ClockState::default();
    loop {
        defmt::info!("State: {:?}", state);
        state = state.execute(&mut clock, &mut button).await;
    }
}

#[expect(dead_code, reason = "for article")]
#[expect(clippy::items_after_statements, reason = "Keeps related code together")]
async fn inner_main_display(spawner: Spawner) -> Result<!> {
    let hardware = lib::Hardware::default();


    // Create TimeSync virtual device (creates WiFi internally)
    static TIME_SYNC: TimeSyncNotifier = TimeSync::notifier();
    let _time_sync = TimeSync::new(
        &TIME_SYNC,
        hardware.wifi.pin_23,      // WiFi power enable
        hardware.wifi.pin_25,      // WiFi SPI chip select
        hardware.wifi.pio0,        // WiFi PIO block for SPI
        hardware.wifi.pin_24,      // WiFi SPI MOSI
        hardware.wifi.pin_29,      // WiFi SPI CLK
        hardware.wifi.dma_ch0,     // WiFi DMA channel for SPI
        spawner,
    );


    let mut button = Button::new(hardware.button);

    static DISPLAY_NOTIFIER: DisplayNotifier = Display::notifier();
    let display = Display::new(
        hardware.cells,
        hardware.segments,
        &DISPLAY_NOTIFIER,
        spawner,
    )?;
    loop {
        display.write_text(['1', '2', '3', '4']);
        button.press_duration().await;
        display.write_text(['r', 'u', 's', 't']);
        button.press_duration().await;
    }
}

#[expect(dead_code, reason = "for article")]
#[expect(clippy::items_after_statements, reason = "Keeps related code together")]
async fn inner_main_blinky(spawner: Spawner) -> Result<!> {
    let hardware = lib::Hardware::default();
    let mut button = Button::new(hardware.button);

    static BLINKER_NOTIFIER: BlinkerNotifier = Blinker::notifier();
    let blinker = Blinker::new(
        hardware.cells,
        hardware.segments,
        &BLINKER_NOTIFIER,
        spawner,
    )?;

    loop {
        blinker.write_text(BlinkState::Solid, ['1', '2', '3', '4']);
        button.press_duration().await;
        blinker.write_text(BlinkState::BlinkingAndOn, ['r', 'u', 's', 't']);
        button.press_duration().await;
    }
}

// TODO: Is testing possible?
