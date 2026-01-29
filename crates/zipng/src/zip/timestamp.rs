/// MS-DOS date/time encoding for ZIP file headers.
///
/// Returns `(time_u16, date_u16)` packed in little-endian DOS format.
///
/// If `SOURCE_DATE_EPOCH` is set, parses it as a Unix timestamp and converts.
/// Otherwise returns `(0, 0)` which decodes to 1980-01-01 00:00:00 (the DOS
/// epoch), a conventional "no timestamp" value used by reproducible builds.
///
/// The DOS time field has 2-second granularity; odd seconds are rounded up.
/// Timestamps before 1980-01-01 are clamped to the DOS epoch.
pub fn dos_timestamp() -> [u8; 4] {
    let (time, date) = match std::env::var("SOURCE_DATE_EPOCH") {
        Ok(val) => {
            let epoch: i64 = val.parse().unwrap_or(0);
            unix_to_dos(epoch)
        }
        Err(_) => (0u16, 0u16),
    };
    let mut buf = [0u8; 4];
    buf[0..2].copy_from_slice(&time.to_le_bytes());
    buf[2..4].copy_from_slice(&date.to_le_bytes());
    buf
}

fn unix_to_dos(epoch: i64) -> (u16, u16) {
    // DOS epoch is 1980-01-01. Anything before that clamps to zero.
    const DOS_EPOCH: i64 = 315532800; // 1980-01-01T00:00:00Z
    if epoch < DOS_EPOCH {
        return (0, 0);
    }

    // Days from Unix epoch (1970-01-01) — civil calendar conversion.
    let secs = epoch;
    let days = (secs / 86400) as i32;
    let time_of_day = (secs % 86400) as u32;

    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;
    // DOS has 2-second granularity; round up.
    let dos_seconds = seconds.div_ceil(2);
    // If rounding pushed us to 30 (i.e. 60 seconds), carry into minutes.
    let (dos_seconds, carry_min) = if dos_seconds >= 30 {
        (0u32, 1u32)
    } else {
        (dos_seconds, 0u32)
    };
    let minutes = minutes + carry_min;
    let (minutes, carry_hr) = if minutes >= 60 {
        (minutes - 60, 1u32)
    } else {
        (minutes, 0u32)
    };
    let hours = hours + carry_hr;
    // If hours overflow 23 we'd need to carry into the next day, but this
    // only happens at exactly 23:59:59 → 00:00:00 next day. For simplicity
    // just clamp to 23:59:58.
    let (hours, minutes, dos_seconds) = if hours >= 24 {
        (23u32, 59u32, 29u32)
    } else {
        (hours, minutes, dos_seconds)
    };

    let time = ((hours as u16) << 11) | ((minutes as u16) << 5) | (dos_seconds as u16);

    // Civil date from day count (algorithm from Howard Hinnant).
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i32 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    // Clamp year to DOS range (1980–2107).
    let year = y.max(1980).min(2107);
    let date =
        (((year - 1980) as u16) << 9) | ((m as u16) << 5) | (d as u16);

    (time, date)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_gives_dos_epoch() {
        let (time, date) = unix_to_dos(0);
        assert_eq!(time, 0);
        assert_eq!(date, 0);
    }

    #[test]
    fn known_timestamp() {
        // 2024-01-15 12:30:00 UTC = 1705318200
        let (time, date) = unix_to_dos(1705318200);
        let hours = (time >> 11) & 0x1F;
        let minutes = (time >> 5) & 0x3F;
        let seconds_x2 = time & 0x1F;
        assert_eq!(hours, 11);
        assert_eq!(minutes, 30);
        assert_eq!(seconds_x2, 0);

        let year = ((date >> 9) & 0x7F) + 1980;
        let month = (date >> 5) & 0x0F;
        let day = date & 0x1F;
        assert_eq!(year, 2024);
        assert_eq!(month, 1);
        assert_eq!(day, 15);
    }

    #[test]
    fn odd_seconds_round_up() {
        // 2024-01-15 12:30:01 UTC = 1705318201
        let (time, _) = unix_to_dos(1705318201);
        let seconds_x2 = time & 0x1F;
        // 1 second → rounds up to 2, encoded as 1
        assert_eq!(seconds_x2, 1);
    }

    #[test]
    fn pre_1980_clamps() {
        // 1970-06-15 = well before DOS epoch
        let (time, date) = unix_to_dos(14256000);
        assert_eq!(time, 0);
        assert_eq!(date, 0);
    }
}
