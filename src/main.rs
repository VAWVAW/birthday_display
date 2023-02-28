mod csv;
mod error_wrapper;
mod person;

use crate::csv::get_persons;
use crate::error_wrapper::ErrorDisplayWrapper;
use crate::person::Person;

use std::borrow::Cow;
use std::collections::HashMap;
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
#[non_exhaustive]
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
    persons_by_birthday: HashMap<(u32, u32), Vec<Person>>,
    cli: Cli,
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

        let mut persons_by_birthday: HashMap<(u32, u32), Vec<Person>> = HashMap::new();
        for person in persons {
            let key = (person.birthday.day(), person.birthday.month());

            persons_by_birthday.entry(key).or_default();

            persons_by_birthday.get_mut(&key).unwrap().push(person);
        }

        (
            Self {
                persons_by_birthday,
                cli,
            },
            command,
        )
    }

    fn title(&self) -> String {
        String::from("Birthday Display")
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        if let Message::DataReceived(image_data, orig_url) = message {
            let url = Some(orig_url);
            self.persons_by_birthday
                .iter_mut()
                .flat_map(|(_, persons)| persons.iter_mut())
                .filter(|person| person.image_url == url)
                .for_each(|person| {
                    person.image_data.replace(image_data.clone());
                });
        }
        iced::window::maximize(true)
    }

    fn view(&self) -> Element<Self::Message> {
        let today = Utc::now().date_naive();
        let key = (today.day(), today.month());

        let maybe_persons_today = self.persons_by_birthday.get(&key);

        let elements: Vec<Element<Message>> = match maybe_persons_today {
            Some(persons_today) => persons_today
                .iter()
                .map(|person| person.view(self.cli.silent))
                .collect(),
            None => Vec::new(),
        };

        container(row(elements).spacing(15))
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        every(Duration::from_secs(5)).map(Message::UpdateDay)
    }
}

fn main() -> Result<(), ErrorDisplayWrapper> {
    let cli: Cli = Cli::parse();

    let persons = get_persons(&cli.file, cli.quiet)?;

    BirthdayDisplay::run(Settings::with_flags((cli, persons)))
        .map_err(|error| ErrorDisplayWrapper::from(Box::new(error) as Box<dyn Error>))?;
    Ok(())
}
