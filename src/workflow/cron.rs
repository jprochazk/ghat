#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cron {
    pub minute: CronField,
    pub hour: CronField,
    pub day_of_the_month: CronField,
    pub month: CronField,
    pub day_of_the_week: CronField,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CronField {
    Value(u8),
    Any,
    Multi { values: Vec<u8> },
    Range { start: u8, end: u8 },
    Step { start: u8, by: u8 },
}

impl CronField {
    pub fn values(&self) -> impl Iterator<Item = u8> {
        CronFieldValues {
            field: self,
            index: 0,
        }
    }
}

struct CronFieldValues<'a> {
    field: &'a CronField,
    index: usize,
}

impl<'a> Iterator for CronFieldValues<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let next = match (self.field, self.index) {
            (CronField::Value(v), 0) => Some(*v),
            (CronField::Value(_), _) => None,

            (CronField::Any, _) => None,

            (CronField::Multi { values }, index) => values.get(index).copied(),

            (CronField::Range { start, .. }, 0) => Some(*start),
            (CronField::Range { end, .. }, 1) => Some(*end),
            (CronField::Range { .. }, _) => None,

            (CronField::Step { start, .. }, 0) => Some(*start),
            (CronField::Step { by, .. }, 1) => Some(*by),
            (CronField::Step { .. }, _) => None,
        };

        self.index = self.index.saturating_add(1);

        next
    }
}

impl std::fmt::Display for Cron {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            minute,
            hour,
            day_of_the_month,
            month,
            day_of_the_week,
        } = self;
        write!(
            f,
            "{minute} {hour} {day_of_the_month} {month} {day_of_the_week}"
        )
    }
}

#[derive(Debug, Clone)]
pub struct InvalidCronError;

impl std::error::Error for InvalidCronError {}

impl std::fmt::Display for InvalidCronError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid cron syntax")
    }
}

impl std::str::FromStr for Cron {
    type Err = InvalidCronError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_ascii_whitespace();

        let minute = parts.next().ok_or(InvalidCronError)?.parse::<CronField>()?;
        if minute.values().any(|v| v > 59) {
            return Err(InvalidCronError);
        }

        let hour = parts.next().ok_or(InvalidCronError)?.parse::<CronField>()?;
        if hour.values().any(|v| v > 23) {
            return Err(InvalidCronError);
        }

        let day_of_the_month = parts.next().ok_or(InvalidCronError)?.parse::<CronField>()?;
        if day_of_the_month.values().any(|v| v < 1 || v > 31) {
            return Err(InvalidCronError);
        }

        let month = parts.next().ok_or(InvalidCronError)?.parse::<CronField>()?;
        if month.values().any(|v| v < 1 || v > 12) {
            return Err(InvalidCronError);
        }

        let day_of_the_week = parts.next().ok_or(InvalidCronError)?.parse::<CronField>()?;
        if day_of_the_week.values().any(|v| v > 6) {
            return Err(InvalidCronError);
        }

        Ok(Self {
            minute,
            hour,
            day_of_the_month,
            month,
            day_of_the_week,
        })
    }
}

impl std::fmt::Display for CronField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CronField::Value(v) => write!(f, "{v}"),
            CronField::Any => write!(f, "*"),
            CronField::Multi { values } => {
                if let Some(v) = values.get(0) {
                    write!(f, "{v}")?;
                }
                for v in values.iter().skip(1) {
                    write!(f, ",{v}")?;
                }
                Ok(())
            }
            CronField::Range { start, end } => write!(f, "{start}-{end}"),
            CronField::Step { start, by } => write!(f, "{start}/{by}"),
        }
    }
}

impl std::str::FromStr for CronField {
    type Err = InvalidCronError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_number(bytes: &mut &[u8]) -> Option<u8> {
            let mut i = 0;
            while matches!(bytes.get(i), Some(b'0'..=b'9')) {
                i += 1;
            }
            if i == 0 {
                return None;
            }
            // safety: decimal digits are valid utf-8
            let v = unsafe { std::str::from_utf8_unchecked(&bytes[0..i]) };
            let r = v.parse().ok()?;
            *bytes = &bytes[i..];
            Some(r)
        }

        let mut bytes = s.as_bytes();

        if bytes == b"*" {
            return Ok(CronField::Any);
        }

        // */N
        if bytes.starts_with(b"*/") {
            bytes = &bytes[2..];
            let by = parse_number(&mut bytes).ok_or(InvalidCronError)?;
            if !bytes.is_empty() {
                return Err(InvalidCronError);
            }
            return Ok(CronField::Step { start: 0, by });
        }

        let first = parse_number(&mut bytes).ok_or(InvalidCronError)?;

        if bytes.is_empty() {
            return Ok(CronField::Value(first));
        }

        match bytes[0] {
            b'-' => {
                bytes = &bytes[1..];
                let end = parse_number(&mut bytes).ok_or(InvalidCronError)?;
                if !bytes.is_empty() {
                    return Err(InvalidCronError);
                }
                Ok(CronField::Range { start: first, end })
            }
            b'/' => {
                bytes = &bytes[1..];
                let by = parse_number(&mut bytes).ok_or(InvalidCronError)?;
                if !bytes.is_empty() {
                    return Err(InvalidCronError);
                }
                Ok(CronField::Step { start: first, by })
            }
            b',' => {
                let mut values = vec![first];
                while !bytes.is_empty() {
                    if bytes[0] != b',' {
                        return Err(InvalidCronError);
                    }
                    bytes = &bytes[1..];
                    let v = parse_number(&mut bytes).ok_or(InvalidCronError)?;
                    values.push(v);
                }
                Ok(CronField::Multi { values })
            }
            _ => Err(InvalidCronError),
        }
    }
}

impl serde::Serialize for Cron {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for Cron {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(input: &str) {
        let cron: Cron = input.parse().unwrap();
        assert_eq!(cron.to_string(), input);
    }

    #[test]
    fn parse_all_any() {
        let cron: Cron = "* * * * *".parse().unwrap();
        assert_eq!(cron.minute, CronField::Any);
        assert_eq!(cron.hour, CronField::Any);
        assert_eq!(cron.day_of_the_month, CronField::Any);
        assert_eq!(cron.month, CronField::Any);
        assert_eq!(cron.day_of_the_week, CronField::Any);
    }

    #[test]
    fn parse_values() {
        let cron: Cron = "30 12 15 6 3".parse().unwrap();
        assert_eq!(cron.minute, CronField::Value(30));
        assert_eq!(cron.hour, CronField::Value(12));
        assert_eq!(cron.day_of_the_month, CronField::Value(15));
        assert_eq!(cron.month, CronField::Value(6));
        assert_eq!(cron.day_of_the_week, CronField::Value(3));
    }

    #[test]
    fn parse_range() {
        let cron: Cron = "0 9-17 * * 1-5".parse().unwrap();
        assert_eq!(cron.hour, CronField::Range { start: 9, end: 17 });
        assert_eq!(cron.day_of_the_week, CronField::Range { start: 1, end: 5 });
    }

    #[test]
    fn parse_step() {
        let cron: Cron = "*/15 0/2 * * *".parse().unwrap();
        assert_eq!(cron.minute, CronField::Step { start: 0, by: 15 });
        assert_eq!(cron.hour, CronField::Step { start: 0, by: 2 });
    }

    #[test]
    fn parse_multi() {
        let cron: Cron = "0 0 1,15 * *".parse().unwrap();
        assert_eq!(
            cron.day_of_the_month,
            CronField::Multi {
                values: vec![1, 15]
            }
        );
    }

    #[test]
    fn roundtrip_all_any() {
        roundtrip("* * * * *");
    }

    #[test]
    fn roundtrip_values() {
        roundtrip("30 12 15 6 3");
    }

    #[test]
    fn roundtrip_range() {
        roundtrip("0 9-17 * * 1-5");
    }

    #[test]
    fn roundtrip_step() {
        roundtrip("0/15 0/2 * * *");
    }

    #[test]
    fn roundtrip_multi() {
        roundtrip("0,30 0 1,15 * *");
    }

    #[test]
    fn roundtrip_complex() {
        roundtrip("0 0 1,15 1-6 0");
    }

    #[test]
    fn invalid_too_few_fields() {
        assert!("* * *".parse::<Cron>().is_err());
    }

    #[test]
    fn invalid_minute_out_of_range() {
        assert!("60 * * * *".parse::<Cron>().is_err());
    }

    #[test]
    fn invalid_hour_out_of_range() {
        assert!("0 24 * * *".parse::<Cron>().is_err());
    }

    #[test]
    fn invalid_day_of_month_zero() {
        assert!("0 0 0 * *".parse::<Cron>().is_err());
    }

    #[test]
    fn invalid_day_of_month_out_of_range() {
        assert!("0 0 32 * *".parse::<Cron>().is_err());
    }

    #[test]
    fn invalid_month_zero() {
        assert!("0 0 1 0 *".parse::<Cron>().is_err());
    }

    #[test]
    fn invalid_month_out_of_range() {
        assert!("0 0 1 13 *".parse::<Cron>().is_err());
    }

    #[test]
    fn invalid_day_of_week_out_of_range() {
        assert!("0 0 * * 7".parse::<Cron>().is_err());
    }

    #[test]
    fn invalid_syntax() {
        assert!("abc".parse::<Cron>().is_err());
        assert!("* * * * &".parse::<CronField>().is_err());
        assert!("1-".parse::<CronField>().is_err());
        assert!("/5".parse::<CronField>().is_err());
        assert!("1,".parse::<CronField>().is_err());
    }

    #[test]
    fn serde_roundtrip() {
        let cron: Cron = "30 12 * * 1-5".parse().unwrap();
        let json = serde_json::to_string(&cron).unwrap();
        assert_eq!(json, "\"30 12 * * 1-5\"");
        let parsed: Cron = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, cron);
    }
}
