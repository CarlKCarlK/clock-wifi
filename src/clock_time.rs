use core::ops::AddAssign;

use defmt::info;
use embassy_time::{Duration, Instant};

use crate::TICKS_IN_ONE_DAY;

/// The system time along with an offset to represent time
/// to display on the clock.
pub struct ClockTime {
    offset: Duration,
    /// UTC offset in minutes
    utc_offset_minutes: i32,
}

impl Default for ClockTime {
    /// By default, `ClockTime` starts at 12:00:00.
    fn default() -> Self {
        info!("Now: {:?}", Instant::now());
        // Start at 12:00:00 (12 hours * 3600 seconds/hour * 1000 milliseconds/second)
        let utc_offset_minutes = option_env!("UTC_OFFSET_MINUTES")
            .and_then(|val| val.parse::<i32>().ok())
            .unwrap_or(0);
        Self {
            offset: Duration::from_millis(12 * 3600 * 1000),
            utc_offset_minutes,
        }
    }
}

impl ClockTime {
    /// Sets the time from a Unix timestamp with UTC offset applied.
    ///
    /// Uses the current UTC offset stored in the struct.
    #[expect(
        clippy::integer_division_remainder_used,
        clippy::arithmetic_side_effects,
        reason = "The modulo operations prevent overflow."
    )]
    pub fn set_from_unix(&mut self, unix_seconds: crate::UnixSeconds) {
        // Convert to local time
        let local_seconds = unix_seconds.as_i64() + i64::from(self.utc_offset_minutes) * 60;
        
        // Get seconds since local midnight
        let seconds_since_midnight = (local_seconds % 86400) as u64;
        let millis_since_midnight = seconds_since_midnight * 1000;
        
        // Calculate offset needed to make now() return the target time
        let current_instant_ticks = Instant::now().as_ticks() % TICKS_IN_ONE_DAY;
        let target_ticks = Duration::from_millis(millis_since_midnight).as_ticks() % TICKS_IN_ONE_DAY;
        
        // offset = target - instant (mod 1 day)
        let offset_ticks = if target_ticks >= current_instant_ticks {
            target_ticks - current_instant_ticks
        } else {
            TICKS_IN_ONE_DAY + target_ticks - current_instant_ticks
        };
        
        self.offset = Duration::from_ticks(offset_ticks % TICKS_IN_ONE_DAY);
        info!(
            "Set time from Unix: {} -> offset: {:?}",
            unix_seconds.as_i64(),
            self.offset.as_millis()
        );
    }

    /// Returns the current time with the offset applied wrapped around to be less than one day.
    #[expect(
        clippy::arithmetic_side_effects,
        clippy::integer_division_remainder_used,
        reason = "Because of %'s will never overflow."
    )]
    #[inline]
    #[must_use]
    pub fn now(&self) -> Duration {
        let ticks = Instant::now().as_ticks() % TICKS_IN_ONE_DAY
            + self.offset.as_ticks() % TICKS_IN_ONE_DAY;
        Duration::from_ticks(ticks % TICKS_IN_ONE_DAY)
    }

    /// Returns the current hours, minutes, seconds, and wait duration until the next unit of time.
    ///
    /// For example, if `unit` is `ONE_MINUTE`, this function will tell how long to wait
    /// until the top of the next minute. This is used to put the microcontroller to sleep
    /// until the next time the display needs to be updated.
    ///
    /// The function is in-line so that the compiler can optimize return values that
    /// are not used.
    #[expect(
        clippy::cast_possible_truncation,
        clippy::integer_division_remainder_used,
        clippy::arithmetic_side_effects,
        reason = "The modulo operations prevent overflow."
    )]
    #[must_use]
    #[inline]
    pub fn h_m_s_sleep_duration(&self, unit: Duration) -> (u8, u8, u8, Duration) {
        let now = self.now();
        let sleep_duration = Self::till_next(now, unit);
        let elapsed_seconds = now.as_secs();
        let hours = ((elapsed_seconds / 3600) + 11) % 12 + 1; // 1-12 instead of 0-11
        let minutes = (elapsed_seconds % 3600) / 60;
        let seconds = elapsed_seconds % 60;
        (hours as u8, minutes as u8, seconds as u8, sleep_duration)
    }

    #[inline]
    #[must_use]
    #[expect(
        clippy::integer_division_remainder_used,
        clippy::arithmetic_side_effects,
        reason = "The modulo operations prevent overflow."
    )]
    /// Returns the duration until the next unit of time.
    ///
    /// For example, if `a` is 1:02:03 and `unit` is `ONE_HOUR`, this function will return
    /// the duration until 2:00:00 which is 57 minutes and 57 seconds.
    pub const fn till_next(time: Duration, unit: Duration) -> Duration {
        let unit_ticks = unit.as_ticks();
        Duration::from_ticks(unit_ticks - time.as_ticks() % unit_ticks)
    }

    /// Returns the current UTC offset in hours (rounded to nearest hour).
    #[expect(
        clippy::integer_division_remainder_used,
        reason = "Division is intentional for converting minutes to hours."
    )]
    #[must_use]
    pub fn utc_offset_hours(&self) -> i32 {
        // Round to nearest hour
        if self.utc_offset_minutes >= 0 {
            (self.utc_offset_minutes + 30) / 60
        } else {
            (self.utc_offset_minutes - 30) / 60
        }
    }

    /// Adjusts the UTC offset by the given number of hours.
    /// The offset wraps around from +14 to -12 (27 total values: -12 to +14).
    #[expect(
        clippy::arithmetic_side_effects,
        clippy::integer_division_remainder_used,
        reason = "Wrapping arithmetic is intentional."
    )]
    pub fn adjust_utc_offset_hours(&mut self, hours: i32) {
        let current_offset_hours = self.utc_offset_hours();
        let new_offset_hours = current_offset_hours + hours;
        
        // Wrap around: -12 to +14 (27 values)
        // Map to 0-26 range, wrap, then map back to -12 to +14
        let wrapped = ((new_offset_hours + 12) % 27 + 27) % 27 - 12;
        
        // Calculate the change in hours
        let delta_hours = wrapped - current_offset_hours;
        
        // Adjust the display offset to reflect the timezone change
        // When UTC offset increases by 1 hour, display should show 1 hour later
        if delta_hours >= 0 {
            self.offset += Duration::from_secs((delta_hours * 3600) as u64);
        } else {
            self.offset -= Duration::from_secs(((-delta_hours) * 3600) as u64);
        }
        
        self.utc_offset_minutes = wrapped * 60;
        info!(
            "Adjusted UTC offset from {} to {} hours (delta: {} hours)",
            current_offset_hours, wrapped, delta_hours
        );
    }
}

impl AddAssign<Duration> for ClockTime {
    #[expect(
        clippy::integer_division_remainder_used,
        clippy::arithmetic_side_effects,
        reason = "The modulo operations prevent overflow."
    )]
    /// Adds the given duration to offset, wrapping around to be less than one day.
    fn add_assign(&mut self, duration: Duration) {
        let ticks =
            self.offset.as_ticks() % TICKS_IN_ONE_DAY + duration.as_ticks() % TICKS_IN_ONE_DAY;
        self.offset = Duration::from_ticks(ticks % TICKS_IN_ONE_DAY);
        info!(
            "Now: {:?}, Offset: {:?}",
            Instant::now().as_millis(),
            self.offset.as_millis()
        );
    }
}
