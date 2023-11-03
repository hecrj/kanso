pub mod widget;
pub mod writing;

use crate::widget::fade;
use crate::writing::Writing;

use iced::event::{self, Event};
use iced::font::{self, Font};
use iced::keyboard;
use iced::theme::{self, Theme};
use iced::widget::{column, container, horizontal_space, row, text};
use iced::window;
use iced::{executor, Length};
use iced::{Application, Command, Element, Settings, Subscription};

use std::env;
use std::path::PathBuf;

fn main() -> iced::Result {
    let Some(filepath) = env::args().skip(1).next() else {
        println!("error: no filepath specified");
        println!("usage: kanso <filepath>");

        std::process::exit(1);
    };

    Kanso::run(Settings {
        default_font: Font::MONOSPACE,
        window: window::Settings {
            min_size: Some((800, 800)),
            ..window::Settings::default()
        },
        ..Settings::with_flags(Flags {
            filepath: PathBuf::from(filepath),
        })
    })
}

enum Kanso {
    Loading,
    Editing { writing: Writing },
    Errored { error: writing::Error },
}

struct Flags {
    filepath: PathBuf,
}

#[derive(Debug, Clone)]
enum Message {
    Loaded(Result<Writing, writing::Error>),
    Write(char),
    Amend,
    Save(writing::Version),
    Saved(Result<(), writing::Error>),
}

impl Application for Kanso {
    type Theme = Theme;
    type Message = Message;
    type Executor = executor::Default;
    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Kanso::Loading,
            Command::perform(Writing::load(flags.filepath), Message::Loaded),
        )
    }

    fn title(&self) -> String {
        String::from("Kanso")
    }

    fn update(&mut self, message: Message) -> Command<Self::Message> {
        match message {
            Message::Loaded(Ok(writing)) => {
                *self = Self::Editing { writing };

                Command::none()
            }
            Message::Loaded(Err(error)) => {
                *self = Self::Errored { error };

                Command::none()
            }
            Message::Write(character) => {
                if let Self::Editing { writing } = self {
                    writing.write(character);

                    Command::perform(wait_a_bit(), {
                        let version = writing.version();
                        move |_| Message::Save(version)
                    })
                } else {
                    Command::none()
                }
            }
            Message::Amend => {
                if let Self::Editing { writing } = self {
                    writing.amend();

                    Command::perform(wait_a_bit(), {
                        let version = writing.version();
                        move |_| Message::Save(version)
                    })
                } else {
                    Command::none()
                }
            }
            Message::Save(version) => match self {
                Self::Editing { writing } if writing.is_dirty() && writing.version() == version => {
                    Command::perform(writing.save(), Message::Saved)
                }
                _ => Command::none(),
            },
            Message::Saved(Ok(())) => Command::none(),
            Message::Saved(Err(error)) => {
                *self = Self::Errored { error };

                Command::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, status| {
            if status == event::Status::Captured {
                return None;
            }

            match event {
                Event::Keyboard(keyboard::Event::CharacterReceived(character)) => {
                    (!character.is_control()).then_some(Message::Write(character))
                }
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key_code: keyboard::KeyCode::Enter,
                    ..
                }) => Some(Message::Write('\n')),
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key_code: keyboard::KeyCode::Backspace,
                    ..
                }) => Some(Message::Amend),
                _ => None,
            }
        })
    }

    fn view(&self) -> Element<'_, Message> {
        match self {
            Self::Loading => centered("Loading..."),
            Self::Editing { writing } => {
                let writer = fade(
                    container(
                        text(format!("{}_", {
                            let content = writing.content();
                            &content[content.len().saturating_sub(1_000)..]
                        }))
                        .font(Font {
                            family: font::Family::Serif,
                            ..Font::DEFAULT
                        })
                        .size(40),
                    )
                    .width(700)
                    .padding(20),
                );

                let status_bar = {
                    let word_count_difference = {
                        let difference = writing.word_count_difference();

                        text(format!(
                            "{}{}",
                            if difference > 0 { "+" } else { "" },
                            difference
                        ))
                        .style({
                            let palette = Theme::Dark.extended_palette();

                            if difference == 0 {
                                theme::Text::Default
                            } else {
                                theme::Text::Color(if difference > 0 {
                                    palette.success.strong.color
                                } else {
                                    palette.danger.base.color
                                })
                            }
                        })
                    };

                    row![
                        text(format!(
                            "{}{}",
                            writing.filepath().to_str().unwrap_or(""),
                            if writing.is_dirty() { "*" } else { "" }
                        )),
                        horizontal_space(Length::Fill),
                        text(format!("{}", writing.word_count())),
                        word_count_difference,
                    ]
                    .spacing(10)
                    .padding(20)
                };

                container(column![writer, status_bar])
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
            Self::Errored { error } => centered(text(error)),
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

fn centered<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
}

async fn wait_a_bit() {
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
}
