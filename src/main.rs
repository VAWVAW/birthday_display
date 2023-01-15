mod csv;

use crate::csv::{custom_date_format, get_birthdays};

use std::borrow::Cow;
use std::sync::{Arc, Mutex};

use chrono::{Datelike, NaiveDate, Utc};
use reqwest::{Client, RequestBuilder};
use serde::Deserialize;

use iced::time::{every, Duration, Instant};
use iced::widget::image::Handle;
use iced::widget::{column, container, row, Column, Image, Text, text};
use iced::{Alignment, Application, Color, Command, Element, Length, Settings, Subscription};
use iced::alignment::{Horizontal, Vertical};

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
    image_data: Mutex<Option<Result<Handle, String>>>,
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
        let mut column: Column<Message> = column![text(banner_str).size(20)];

        if let Some(maybe_image) = &*self.image_data.lock().unwrap() {
            match maybe_image {
                Ok(image_data) => {
                    let image: Image = Image::new((*image_data).clone()).into();
                    column = column.push(image);
                }
                Err(error) => {
                    let text: Text = text(error)
                        .size(20)
                        .style(Color::from_rgb(0.7, 0.0, 0.0))
                        .vertical_alignment(Vertical::Center)
                        .horizontal_alignment(Horizontal::Center)
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
    DataReceived,
}

async fn request_birthday_image(request: RequestBuilder, birthday: Arc<Birthday>) -> Message {
    let result = match request.send().await {
        Ok(response) => match response.error_for_status() {
            Ok(response) => response.bytes().await,
            Err(error) => Err(error),
        },
        Err(error) => Err(error),
    };

    let mut image = birthday.image_data.lock().unwrap();


    let image_data = Some(match result {
        Ok(bytes) => {
            let cow: Cow<'_, [u8]> = Cow::from(bytes.to_vec());
            Ok(Handle::from_memory(cow))
        }
        Err(_error) => Err(String::from("[failed to load image]")),
    });

    *image = image_data;
    Message::DataReceived
}

struct BirthdayDisplay {
    all_birthdays: Vec<Arc<Birthday>>,
    birthdays_today: Vec<Arc<Birthday>>,
}

impl BirthdayDisplay {
    fn update_day(&mut self) {
        let today = NaiveDate::from(Utc::now().date_naive());
        let mut birthdays_today = Vec::new();

        for birthday in &self.all_birthdays {
            if birthday.birthday.day() == today.day() && birthday.birthday.month() == today.month()
            {
                birthdays_today.push(Arc::clone(birthday));
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

        // load data
        let birthdays = match get_birthdays(&args[1]) {
            Ok(birthdays) => birthdays,
            Err(error) => {
                eprintln!("{}", error);
                std::process::exit(1);
            }
        };

        // prepare loading of images
        let loadable_elements: Vec<&Arc<Birthday>> = birthdays
            .iter()
            .filter(|birthday| birthday.image_url.is_some())
            .collect();

        // try to generate reqwest client if needed
        let reqwest_client = match loadable_elements.len() {
            0 => None,
            _ => Client::builder().build().ok(),
        };

        // generate Command to load images async
        let mut command = Command::none();

        if let Some(client) = reqwest_client {
            let mut requests = Vec::new();
            for birthday in loadable_elements {
                requests.push(Command::perform(
                    request_birthday_image(
                        client.get(birthday.image_url.as_ref().unwrap()),
                        Arc::clone(birthday),
                    ),
                    |x| x,
                ));
            }
            command = Command::batch(requests);
        }

        // display initial data
        let mut birthday_display = Self {
            all_birthdays: birthdays,
            birthdays_today: Vec::new(),
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
            _ => {}
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let mut elements = Vec::new();

        for birthday in &self.birthdays_today {
            elements.push(birthday.view());
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
