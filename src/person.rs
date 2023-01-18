use crate::csv::custom_date_format;
use crate::Message;

use chrono::{NaiveDate, Utc};
use serde::Deserialize;

use iced::widget::image::Handle;
use iced::widget::{column, container, text, Column, Image};
use iced::{Alignment, Color, Element, Length};

#[derive(Debug, Deserialize)]
pub struct Person {
    last_name: String,
    first_name: String,
    #[serde(deserialize_with = "custom_date_format::deserialize")]
    pub(crate) birthday: NaiveDate,
    gender: char,
    pub(crate) image_url: Option<String>,
    #[serde(skip)]
    pub(crate) image_data: Option<Result<Handle, String>>,
}

impl Person {
    pub fn view(&self, silent: bool) -> Element<Message> {
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
            let element: Element<Message> = match maybe_image {
                Ok(image_data) => {
                    let image: Image = Image::new((*image_data).clone()).into();
                    image.into()
                }
                Err(error) => {
                    let text = if silent { text("") } else { text(error) };
                    text.size(20).style(Color::from_rgb(0.7, 0.0, 0.0)).into()
                }
            };
            column = column.push(container(element).width(Length::Units(300)));
        }

        column.align_items(Alignment::Center).spacing(20).into()
    }
}
