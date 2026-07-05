//! Minimal 5-field cron (`minute hour dom month dow`) — no external deps.
//!
//! Supports `*`, `N`, `*/N`, `N-M`, `N,M`, presets (`@hourly`, `@daily`, …).

use crate::core::{Result, ResumaError};

/// Parsed cron schedule (UTC).
#[derive(Debug, Clone)]
pub struct CronSchedule {
    minute: Field,
    hour: Field,
    day_of_month: Field,
    month: Field,
    day_of_week: Field,
}

#[derive(Debug, Clone)]
enum Field {
    Any,
    Values(Vec<u32>),
}

impl Field {
    fn matches(&self, value: u32) -> bool {
        match self {
            Self::Any => true,
            Self::Values(v) => v.contains(&value),
        }
    }
}

/// Parse a cron expression or preset alias.
pub fn parse(expr: &str) -> Result<CronSchedule> {
    let expr = expr.trim();
    let expr = match expr {
        "@every_minute" | "@minutely" => "* * * * *",
        "@hourly" => "0 * * * *",
        "@daily" => "0 0 * * *",
        "@weekly" => "0 0 * * 0",
        "@monthly" => "0 0 1 * *",
        other => other,
    };
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if parts.len() != 5 {
        return Err(ResumaError::validation(
            "cron must have 5 fields: minute hour dom month dow",
        ));
    }
    Ok(CronSchedule {
        minute: parse_field(parts[0], 0, 59)?,
        hour: parse_field(parts[1], 0, 23)?,
        day_of_month: parse_field(parts[2], 1, 31)?,
        month: parse_field(parts[3], 1, 12)?,
        day_of_week: parse_field(parts[4], 0, 6)?,
    })
}

fn parse_field(raw: &str, min: u32, max: u32) -> Result<Field> {
    if raw == "*" {
        return Ok(Field::Any);
    }
    let mut values = Vec::new();
    for part in raw.split(',') {
        if let Some(step) = part.strip_prefix("*/") {
            let n: u32 = step
                .parse()
                .map_err(|_| ResumaError::validation("invalid cron step"))?;
            if n == 0 {
                return Err(ResumaError::validation("cron step must be > 0"));
            }
            let mut v = min;
            while v <= max {
                values.push(v);
                v += n;
            }
        } else if let Some((a, b)) = part.split_once('-') {
            let lo: u32 = a
                .parse()
                .map_err(|_| ResumaError::validation("invalid cron range"))?;
            let hi: u32 = b
                .parse()
                .map_err(|_| ResumaError::validation("invalid cron range"))?;
            if lo > hi || lo < min || hi > max {
                return Err(ResumaError::validation("cron range out of bounds"));
            }
            for v in lo..=hi {
                values.push(v);
            }
        } else {
            let n: u32 = part
                .parse()
                .map_err(|_| ResumaError::validation("invalid cron value"))?;
            if n < min || n > max {
                return Err(ResumaError::validation("cron value out of bounds"));
            }
            values.push(n);
        }
    }
    values.sort_unstable();
    values.dedup();
    Ok(Field::Values(values))
}

impl CronSchedule {
    /// True when `ts_ms` (UTC) falls on a matching minute boundary.
    pub fn matches_ms(&self, ts_ms: u64) -> bool {
        let secs = ts_ms / 1000;
        let (min, hour, dom, mon, dow) = utc_parts(secs);
        if !self.minute.matches(min) || !self.hour.matches(hour) || !self.month.matches(mon) {
            return false;
        }
        // Standard cron: dom and dow are OR when both are restricted.
        let dom_any = matches!(self.day_of_month, Field::Any);
        let dow_any = matches!(self.day_of_week, Field::Any);
        if dom_any && dow_any {
            return true;
        }
        if dom_any {
            return self.day_of_week.matches(dow);
        }
        if dow_any {
            return self.day_of_month.matches(dom);
        }
        self.day_of_month.matches(dom) || self.day_of_week.matches(dow)
    }

    /// Next matching UTC timestamp (ms) strictly after `after_ms`.
    pub fn next_after_ms(&self, after_ms: u64) -> u64 {
        let start = (after_ms / 60_000) * 60_000 + 60_000;
        let mut t = start;
        let limit = start + 366 * 24 * 60 * 60_000;
        while t < limit {
            if self.matches_ms(t) {
                return t;
            }
            t += 60_000;
        }
        start + 60_000
    }
}

/// UTC calendar parts from unix seconds: (minute, hour, dom, month, dow).
/// `dow`: 0 = Sunday.
fn utc_parts(secs: u64) -> (u32, u32, u32, u32, u32) {
    let days = secs / 86_400;
    let time = secs % 86_400;
    let minute = ((time % 3600) / 60) as u32;
    let hour = (time / 3600) as u32;

    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let dom = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 {
        (mp + 3) as u32
    } else {
        (mp - 9) as u32
    };
    let year = y + if month <= 2 { 1 } else { 0 };

    // 1970-01-01 was a Thursday (dow 4, with 0 = Sunday).
    let dow = ((days + 4) % 7) as u32;

    let _ = year;
    (minute, hour, dom, month, dow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hourly_matches_top_of_hour() {
        let c = parse("@hourly").unwrap();
        let ts = utc_to_ms(2025, 6, 15, 14, 0);
        assert!(c.matches_ms(ts));
        assert!(!c.matches_ms(ts + 60_000));
    }

    #[test]
    fn day_of_week_is_correct() {
        // 2025-03-01 is a Saturday (dow = 6, 0 = Sunday).
        let (_, _, _, _, dow) = utc_parts(utc_to_ms(2025, 3, 1, 0, 0) / 1000);
        assert_eq!(dow, 6);
        // 1970-01-01 was a Thursday.
        let (_, _, _, _, dow) = utc_parts(0);
        assert_eq!(dow, 4);
        // @weekly fires on Sunday: 2025-03-02, not 2025-03-01.
        let c = parse("@weekly").unwrap();
        assert!(c.matches_ms(utc_to_ms(2025, 3, 2, 0, 0)));
        assert!(!c.matches_ms(utc_to_ms(2025, 3, 1, 0, 0)));
    }

    #[test]
    fn every_five_minutes() {
        let c = parse("*/5 * * * *").unwrap();
        assert!(c.matches_ms(utc_to_ms(2025, 1, 1, 0, 0)));
        assert!(c.matches_ms(utc_to_ms(2025, 1, 1, 0, 5)));
        assert!(!c.matches_ms(utc_to_ms(2025, 1, 1, 0, 3)));
    }

    fn utc_to_ms(year: u32, month: u32, day: u32, hour: u32, minute: u32) -> u64 {
        let mut y = year as i64;
        let mut m = month as i64;
        if m <= 2 {
            y -= 1;
            m += 12;
        }
        let era = y / 400;
        let yoe = y - era * 400;
        let doy = (153 * (m - 3) + 2) / 5 + day as i64 - 1 + 365 * yoe + yoe / 4 - yoe / 100;
        let days = era * 146_097 + doy - 719_468;
        let secs = days * 86_400 + hour as i64 * 3600 + minute as i64 * 60;
        secs as u64 * 1000
    }
}
