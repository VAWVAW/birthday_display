use crate::person::Person;

use std::error::Error;
use std::path::PathBuf;

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

pub fn get_persons(path: &PathBuf, quiet: bool) -> Result<Vec<Person>, Box<dyn Error>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)?;
    let mut persons: Vec<Person> = Vec::new();

    for result in reader.deserialize() {
        if let Ok(person) = result {
            persons.push(person);
        } else {
            if quiet {
                continue
            }
            let error = result.unwrap_err();
            eprintln!("error reading line: {:?}", error);
        }
    }
    Ok(persons)
}
