use crate::Birthday;
use std::error::Error;
use std::sync::Arc;

// add parsing for custom date format
// https://serde.rs/custom-date-format.html
pub mod custom_date_format {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer};

    const FORMAT: &'static str = "%d.%m.%Y";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

pub fn get_birthdays(path: &str) -> Result<Vec<Arc<Birthday>>, Box<dyn Error>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)?;
    let mut birthdays: Vec<Arc<Birthday>> = Vec::new();

    for result in reader.deserialize() {
        let birthday: Arc<Birthday> = match result {
            Ok(birthday) => Arc::new(birthday),
            Err(error) => {
                eprintln!("error reading line: {:?}", error);
                continue;
            }
        };
        birthdays.push(birthday);
    }
    Ok(birthdays)
}
