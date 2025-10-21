use crate::{
    button::{Button, PressDuration},
    clock::Clock,
    time_sync::{TimeSync, TimeSyncEvent},
    BlinkState, ClockTime, ONE_MINUTE, ONE_SECOND,
};
use defmt::info;
use embassy_futures::select::{select, Either};
use embassy_time::Duration;

/// Represents the different states the clock can operate in.
///
/// The clock has two display modes: `HoursMinutes` (HH:MM) and `MinutesSeconds` (MM:SS).
/// Short press toggles between them. Long press enters UTC offset edit mode.
#[expect(missing_docs, reason = "The variants are self-explanatory.")]
#[derive(Debug, defmt::Format, Clone, Copy, Default)]
pub enum ClockState {
    #[default]
    HoursMinutes,
    MinutesSeconds,
    EditUtcOffset,
}

impl ClockState {
    /// Run the clock in the current state and return the next state.
    ///
    /// # Returns
    ///
    /// The next state of the clock.
    pub async fn execute(
        self,
        clock: &mut Clock<'_>,
        button: &mut Button<'_>,
        time_sync: &TimeSync,
    ) -> Self {
        match self {
            Self::HoursMinutes => self.execute_hours_minutes(clock, button, time_sync).await,
            Self::MinutesSeconds => self.execute_minutes_seconds(clock, button, time_sync).await,
            Self::EditUtcOffset => self.execute_edit_utc_offset(clock, button).await,
        }
    }

    /// Given the current `ClockMode` and `ClockTime`, generates the information the `Clock` abstraction should display.
    ///
    /// # Example
    ///
    /// If the `ClockState` is `HoursMinutes` and the `ClockTime` is 1:23:45, the function will return:
    /// - Characters: `[' ', '1', '2', '3']`
    /// - Blink Mode: `BlinkState::Solid`
    /// - Sleep Duration: `Duration::from_secs(15)`
    pub(crate) fn render(self, clock_time: &ClockTime) -> (BlinkState, [char; 4], Duration) {
        match self {
            Self::HoursMinutes => Self::render_hours_minutes(clock_time),
            Self::MinutesSeconds => Self::render_minutes_seconds(clock_time),
            Self::EditUtcOffset => Self::render_edit_utc_offset(clock_time),
        }
    }

    async fn execute_hours_minutes(
        self,
        clock: &Clock<'_>,
        button: &mut Button<'_>,
        time_sync: &TimeSync,
    ) -> Self {
        clock.set_state(self).await;
        match select(button.press_duration(), time_sync.wait()).await {
            Either::First(PressDuration::Short) => Self::MinutesSeconds,
            Either::First(PressDuration::Long) => Self::EditUtcOffset,
            Either::Second(event) => {
                Self::handle_time_sync_event(clock, event).await;
                self
            }
        }
    }

    async fn execute_minutes_seconds(
        self,
        clock: &Clock<'_>,
        button: &mut Button<'_>,
        time_sync: &TimeSync,
    ) -> Self {
        clock.set_state(self).await;
        match select(button.press_duration(), time_sync.wait()).await {
            Either::First(PressDuration::Short) => Self::HoursMinutes,
            Either::First(PressDuration::Long) => Self::EditUtcOffset,
            Either::Second(event) => {
                Self::handle_time_sync_event(clock, event).await;
                self
            }
        }
    }

    async fn execute_edit_utc_offset(self, clock: &Clock<'_>, button: &mut Button<'_>) -> Self {
        clock.set_state(self).await;
        match button.press_duration().await {
            PressDuration::Short => {
                // Advance UTC offset by 1 hour
                clock.adjust_utc_offset_hours(1).await;
                clock.set_state(self).await;
                self
            }
            PressDuration::Long => Self::HoursMinutes,
        }
    }

    async fn handle_time_sync_event(clock: &Clock<'_>, event: TimeSyncEvent) {
        match event {
            TimeSyncEvent::Success { unix_seconds } => {
                info!("Time sync success: setting clock to {}", unix_seconds.as_i64());
                clock.set_time_from_unix(unix_seconds).await;
            }
            TimeSyncEvent::Failed(msg) => {
                info!("Time sync failed: {}", msg);
            }
        }
    }

    fn render_hours_minutes(clock_time: &ClockTime) -> (BlinkState, [char; 4], Duration) {
        let (hours, minutes, _, sleep_duration) = clock_time.h_m_s_sleep_duration(ONE_MINUTE);
        (
            BlinkState::Solid,
            [
                tens_hours(hours),
                ones_digit(hours),
                tens_digit(minutes),
                ones_digit(minutes),
            ],
            sleep_duration,
        )
    }

    fn render_minutes_seconds(clock_time: &ClockTime) -> (BlinkState, [char; 4], Duration) {
        let (_, minutes, seconds, sleep_duration) = clock_time.h_m_s_sleep_duration(ONE_SECOND);
        (
            BlinkState::Solid,
            [
                tens_digit(minutes),
                ones_digit(minutes),
                tens_digit(seconds),
                ones_digit(seconds),
            ],
            sleep_duration,
        )
    }

    fn render_edit_utc_offset(clock_time: &ClockTime) -> (BlinkState, [char; 4], Duration) {
        // Display the current time in HH:MM format while blinking
        // This shows what the time looks like with the current UTC offset
        let (hours, minutes, _, _) = clock_time.h_m_s_sleep_duration(ONE_MINUTE);
        
        (
            BlinkState::BlinkingAndOn,
            [
                tens_hours(hours),
                ones_digit(hours),
                tens_digit(minutes),
                ones_digit(minutes),
            ],
            Duration::from_millis(500), // Blink at 1Hz
        )
    }
}

#[inline]
#[expect(
    clippy::arithmetic_side_effects,
    clippy::integer_division_remainder_used,
    reason = "Because value < 60, the division is safe."
)]
const fn tens_digit(value: u8) -> char {
    debug_assert!(value < 60, "Value is between 0 and 59 (inclusive)");
    ((value / 10) + b'0') as char
}

#[inline]
const fn tens_hours(value: u8) -> char {
    debug_assert!(
        1 <= value && value <= 12,
        "Value is between 1 and 12 (inclusive)"
    );
    if value >= 10 {
        '1'
    } else {
        ' '
    }
}

#[expect(
    clippy::arithmetic_side_effects,
    clippy::integer_division_remainder_used,
    reason = "Because value < 60, the division is safe."
)]
#[inline]
const fn ones_digit(value: u8) -> char {
    debug_assert!(value < 60, "Value is be between 0 and 59 (inclusive)");
    ((value % 10) + b'0') as char
}
