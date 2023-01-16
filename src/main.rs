mod csv;

use crate::csv::{custom_date_format, get_persons};

use std::borrow::Cow;

use chrono::{Datelike, NaiveDate, Utc};
use reqwest::{Client, RequestBuilder};
use serde::Deserialize;

use iced::time::{every, Duration, Instant};
use iced::widget::image::Handle;
use iced::widget::{column, container, row, text, Column, Image, Text};
use iced::{Alignment, Application, Color, Command, Element, Length, Settings, Subscription};

fn print_help(args: &Vec<String>) {
    println!("Show birthdays from csv file in window\n");
    println!("Usage: {} [FILENAME]\n", args[0])
}

#[derive(Debug, Deserialize)]
pub struct Person {
    last_name: String,
    first_name: String,
    #[serde(deserialize_with = "custom_date_format::deserialize")]
    birthday: NaiveDate,
    gender: char,
    image_url: Option<String>,
    #[serde(skip)]
    image_data: Option<Result<Handle, String>>,
}

impl Person {
    pub fn view(&self) -> Element<Message> {
        let pronoun = match self.gender {
            'm' | 'M' => "Herr ",
            'f' | 'F' | 'w' | 'W' => "Frau ",
            _ => "",
        };
        let banner_str = match Utc::now().date_naive().years_since(self.birthday) {
            Some(age) => format!(
                "{}{} {} wird heute {} Jahre alt.",
                pronoun, self.first_name, self.last_name, age
            ),
            None => format!(
                "{}{} {} hat heute Geburtstag.",
                pronoun, self.first_name, self.last_name
            ),
        };
        let mut column: Column<Message> = column![text(banner_str).size(20)];

        if let Some(maybe_image) = &self.image_data {
            match maybe_image {
                Ok(image_data) => {
                    let image: Image = Image::new((*image_data).clone()).into();
                    column = column.push(image);
                }
                Err(error) => {
                    let text: Text = text(error)
                        .size(20)
                        .style(Color::from_rgb(0.7, 0.0, 0.0))
                        .into();
                    column = column.push(text);
                }
            };
        }

        column.align_items(Alignment::Center).spacing(20).into()
    }
}

#[derive(Debug)]
pub enum Message {
    UpdateDay(Instant),
    DataReceived(Result<Handle, String>, String),
}

async fn request_birthday_image(
    request: RequestBuilder,
    orig_url: String,
) -> (Result<Handle, String>, String) {
    let result = match request.send().await {
        Ok(response) => match response.error_for_status() {
            Ok(response) => response.bytes().await,
            Err(error) => Err(error),
        },
        Err(error) => Err(error),
    };

    let image_data = match result {
        Ok(bytes) => {
            let cow: Cow<'_, [u8]> = Cow::from(bytes.to_vec());
            Ok(Handle::from_memory(cow))
        }
        Err(_error) => Err(String::from("[failed to load image]")),
    };

    (image_data, orig_url)
}

struct BirthdayDisplay {
    persons: Vec<Person>,
    indexes_birthdays_today: Vec<usize>,
}

impl BirthdayDisplay {
    fn update_day(&mut self) {
        let today = Utc::now().date_naive();
        self.indexes_birthdays_today.clear();

        for (i, person) in self.persons.iter().enumerate() {
            if person.birthday.day() == today.day() && person.birthday.month() == today.month()
            {
                self.indexes_birthdays_today.push(i);
            }
        }
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

        // load data
        let persons = match get_persons(&args[1]) {
            Ok(persons) => persons,
            Err(error) => {
                eprintln!("{}", error);
                std::process::exit(1);
            }
        };

        // prepare loading of images
        let mut loadable_indexes: Vec<usize> = Vec::new();
        for (i, person) in persons.iter().enumerate() {
            if person.image_url.is_some() {
                loadable_indexes.push(i);
            }
        }

        // try to generate reqwest client if needed
        let reqwest_client = match loadable_indexes.len() {
            0 => None,
            _ => Client::builder().build().ok(),
        };

        // generate Command to load images async
        let mut command = Command::none();

        if let Some(client) = reqwest_client {
            let mut request_commands = Vec::new();
            for i in loadable_indexes {
                let person: &Person = persons.get(i).unwrap();
                request_commands.push(Command::perform(
                    request_birthday_image(
                        client.get(person.image_url.as_ref().unwrap()),
                        person.image_url.as_ref().unwrap().clone(),
                    ),
                    |(data, url)| Message::DataReceived(data, url),
                ));
            }
            command = Command::batch(request_commands);
        }

        // display initial data
        let mut birthday_display = Self {
            persons,
            indexes_birthdays_today: Vec::new(),
        };
        birthday_display.update_day();

        (birthday_display, command)
    }

    fn title(&self) -> String {
        String::from("Birthday Display")
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::UpdateDay(_) => self.update_day(),
            Message::DataReceived(image_data, orig_url) => {
                let url = Some(orig_url);
                for person in &mut self.persons {
                    if person.image_url == url {
                        person.image_data.replace(image_data.clone());
                    }
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let mut elements = Vec::new();

        for i in &self.indexes_birthdays_today {
            elements.push(self.persons.get(*i).unwrap().view());
        }

        container(row(elements).spacing(15))
            .padding(20)
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
