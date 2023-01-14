mod csv;

use crate::csv::{custom_date_format, get_birthdays};
use chrono::{NaiveDate, Utc};
use iced::widget::{column, text};
use iced::widget::{container, row};
use iced::{Alignment, Element, Length, Padding, Sandbox, Settings};
use serde::Deserialize;

fn print_help(args: &Vec<String>) {
    println!("Show birthdays from csv file in window\n");
    println!("Usage: {} [FILENAME]\n", args[0])
}

#[derive(Debug, Deserialize)]
pub struct Birthday {
    last_name: String,
    first_name: String,
    #[serde(with = "custom_date_format")]
    birthday: NaiveDate,
    gender: char,
    image_url: Option<String>,
    #[serde(skip)]
    image_data: Option<()>,
}

impl Birthday {
    pub fn gen_column(&self) -> Element<Message> {
        let pronoun = match self.gender {
            'm' | 'M' => "Herr ",
            'f' | 'F' | 'w' | 'W' => "Frau ",
            _ => "",
        };
        let banner_str = match Utc::now().date_naive().years_since(self.birthday) {
            Some(age) => format!(
                "{}{} {} wird heute {} Jahre alt.",
                pronoun,
                self.first_name,
                self.last_name,
                age.to_string()
            ),
            None => format!(
                "{}{} {} hat heute Geburtstag.",
                pronoun, self.first_name, self.last_name
            ),
        };
        column![text(banner_str).size(20),]
            .align_items(Alignment::Center)
            .into()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Message {}

struct BirthdayDisplay {
    all_birthdays: Vec<Birthday>,
    birthdays_today: Vec<Birthday>,
}

impl Sandbox for BirthdayDisplay {
    type Message = Message;

    fn new() -> Self {
        // read commandline arguments
        let args: Vec<String> = std::env::args().collect();
        if args.len() != 2 {
            print_help(&args);
            std::process::exit(1);
        }
        if args[1] == "--help" || args[1] == "-h" {
            print_help(&args);
            std::process::exit(0);
        }

        let birthdays = match get_birthdays(&args[1]) {
            Ok(birthdays) => birthdays,
            Err(error) => {
                eprintln!("{}", error);
                std::process::exit(1);
            }
        };

        Self {
            all_birthdays: Vec::new(),
            birthdays_today: birthdays,
        }
    }

    fn title(&self) -> String {
        String::from("Birthday Display")
    }

    fn update(&mut self, _message: Self::Message) {
        todo!()
    }

    fn view(&self) -> Element<Self::Message> {
        let mut elements = Vec::new();

        for birthday in &self.birthdays_today {
            elements.push(birthday.gen_column());
        }

        container(row(elements).spacing(15))
            .padding(15)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

fn main() -> iced::Result {
    BirthdayDisplay::run(Settings::default())
}
