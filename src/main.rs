mod csv;

use crate::csv::{custom_date_format, get_birthdays};
use chrono::{Datelike, NaiveDate, Utc};
use serde::Deserialize;
use std::rc::Rc;

use iced::time::{every, Duration, Instant};
use iced::widget::{column, container, row, text};
use iced::{Alignment, Application, Command, Element, Length, Settings, Subscription};

fn print_help(args: &Vec<String>) {
    println!("Show birthdays from csv file in window\n");
    println!("Usage: {} [FILENAME]\n", args[0])
}

#[derive(Debug, Deserialize)]
pub struct Birthday {
    last_name: String,
    first_name: String,
    #[serde(deserialize_with = "custom_date_format::deserialize")]
    birthday: NaiveDate,
    gender: char,
    image_url: Option<String>,
    #[serde(skip)]
    image_data: Option<()>,
}

impl Birthday {
    pub fn view(&self) -> Element<Message> {
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
pub enum Message {
    UpdateDay(Instant),
}

struct BirthdayDisplay {
    all_birthdays: Vec<Rc<Birthday>>,
    birthdays_today: Vec<Rc<Birthday>>,
}

impl BirthdayDisplay {
    fn update_day(&mut self) {
        let today = NaiveDate::from(Utc::now().date_naive());
        let mut birthdays_today = Vec::new();

        for birthday in &self.all_birthdays {
            if birthday.birthday.day() == today.day() && birthday.birthday.month() == today.month()
            {
                birthdays_today.push(Rc::clone(birthday));
            }
        }

        self.birthdays_today = birthdays_today;
    }
}

impl Application for BirthdayDisplay {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::theme::Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
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

        let mut birthday_display = Self {
            all_birthdays: birthdays,
            birthdays_today: Vec::new(),
        };

        birthday_display.update_day();

        (birthday_display, Command::none())
    }

    fn title(&self) -> String {
        String::from("Birthday Display")
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::UpdateDay(_) => self.update_day(),
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let mut elements = Vec::new();

        for birthday in &self.birthdays_today {
            elements.push(birthday.view());
        }

        container(row(elements).spacing(15))
            .padding(15)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        every(Duration::from_secs(60)).map(Message::UpdateDay)
    }
}

fn main() -> iced::Result {
    BirthdayDisplay::run(Settings::default())
}
