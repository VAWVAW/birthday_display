mod csv;
mod error_wrapper;
mod person;

use crate::csv::get_persons;
use crate::error_wrapper::ErrorDisplayWrapper;
use crate::person::Person;

use std::borrow::Cow;
use std::error::Error;
use std::fmt::Debug;
use std::path::PathBuf;

use bytes::Bytes;
use chrono::{Datelike, Utc};
use clap::{ArgGroup, Parser};
use reqwest::{Client, RequestBuilder};

use iced::time::{every, Duration, Instant};
use iced::widget::image::Handle;
use iced::widget::{container, row};
use iced::{Application, Command, Element, Length, Settings, Subscription};

#[derive(Parser, Default)]
#[command(author, version, about, long_about = None)]
#[command(group(ArgGroup::new("verbosity").args(["quiet", "verbose"])))]
struct Cli {
    /// csv file in format "lastname,firstname,dd.mm.YYYY,gender,[image url]"
    file: PathBuf,

    #[arg(short, long)]
    quiet: bool,
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// hide errors in gui
    #[arg(short, long)]
    silent: bool,
}

/// Types of updates for the BirthdayDisplay application.
#[derive(Debug)]
pub enum Message {
    /// Periodic update with the time that has passed.
    UpdateDay(Instant),
    /// Data with the associated url as second String.
    DataReceived(Result<Handle, String>, String),
}

async fn request_birthday_image(
    request: RequestBuilder,
    orig_url: String,
    verbosity: u8,
) -> (Result<Handle, String>, String) {
    async fn try_get_data(request: RequestBuilder) -> Result<Bytes, reqwest::Error> {
        request.send().await?.error_for_status()?.bytes().await
    }

    let image_data = match try_get_data(request).await {
        Ok(bytes) => {
            let cow: Cow<'_, [u8]> = Cow::from(bytes.to_vec());
            Ok(Handle::from_memory(cow))
        }
        Err(error) => {
            if verbosity > 0 {
                println!("error loading image: {error}");
            }
            Err(String::from("[failed to load image]"))
        }
    };

    (image_data, orig_url)
}

struct BirthdayDisplay {
    persons: Vec<Person>,
    indexes_birthdays_today: Vec<usize>,
    cli: Cli,
}

impl BirthdayDisplay {
    fn update_day(&mut self) {
        let today = Utc::now().date_naive();

        self.indexes_birthdays_today = self
            .persons
            .iter()
            .enumerate()
            .filter(|(_, person)| {
                person.birthday.day() == today.day() && person.birthday.month() == today.month()
            })
            .map(|(i, _)| i)
            .collect();
    }
}

impl Application for BirthdayDisplay {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::theme::Theme;
    type Flags = (Cli, Vec<Person>);

    fn new(flags: (Cli, Vec<Person>)) -> (Self, Command<Message>) {
        let (cli, persons) = flags;

        // prepare loading of images
        let loadable_persons: Vec<&Person> = persons
            .iter()
            .filter(|person| person.image_url.is_some())
            .collect();

        // try to generate reqwest client if needed
        let reqwest_client = match loadable_persons.len() {
            0 => None,
            _ => match Client::builder().build() {
                Ok(client) => Some(client),
                Err(error) => {
                    if cli.verbose > 0 {
                        println!("error while initializing web client: {error}");
                    }
                    None
                }
            },
        };

        // generate Command to load images async
        let command = if let Some(client) = reqwest_client {
            Command::batch(
                loadable_persons
                    .iter()
                    .map(|person| {
                        Command::perform(
                            request_birthday_image(
                                client.get(person.image_url.as_ref().unwrap()),
                                person.image_url.as_ref().unwrap().clone(),
                                cli.verbose,
                            ),
                            |(data, url)| Message::DataReceived(data, url),
                        )
                    })
                    .collect::<Vec<Command<Message>>>(),
            )
        } else {
            Command::none()
        };

        // display initial data
        let mut birthday_display = Self {
            persons,
            cli,
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
                self.persons
                    .iter_mut()
                    .filter(|person| person.image_url == url)
                    .for_each(|person| {
                        person.image_data.replace(image_data.clone());
                    });
            }
        }
        iced::window::maximize(true)
    }

    fn view(&self) -> Element<Self::Message> {
        let elements = self
            .indexes_birthdays_today
            .iter()
            .map(|i| self.persons[*i].view(self.cli.silent))
            .collect();

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

fn main() -> Result<(), ErrorDisplayWrapper> {
    let cli: Cli = Cli::parse();

    let persons = get_persons(&cli.file, cli.quiet)?;

    let settings = Settings {
        flags: (cli, persons),
        window: iced::window::Settings {
            #[cfg(not(debug_assertions))]
            decorations: false,

            ..Default::default()
        },
        ..Default::default()
    };
    BirthdayDisplay::run(settings)
        .map_err(|error| ErrorDisplayWrapper::from(Box::new(error) as Box<dyn Error>))?;
    Ok(())
}
