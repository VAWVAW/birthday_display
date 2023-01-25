use crate::person::Person;

use std::error::Error;
use std::path::PathBuf;

// add parsing for custom date format
// https://serde.rs/custom-date-format.html
pub mod custom_date_format {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer};

    const FORMAT: &str = "%d.%m.%Y";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

pub fn get_persons(path: &PathBuf, quiet: bool) -> Result<Vec<Person>, Box<dyn Error>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)?;

    Ok(reader
        .deserialize()
        .filter_map(|result| {
            if let Err(error) = result {
                if !quiet {
                    eprintln!("error reading line: {:?}", error)
                };
                None
            } else {
                result.ok()
            }
        })
        .collect())
}
