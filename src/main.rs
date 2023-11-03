pub mod widget;

use crate::widget::fade;

use iced::event::{self, Event};
use iced::font::{self, Font};
use iced::keyboard;
use iced::widget::{column, container, row, text};
use iced::window;
use iced::{executor, Length};
use iced::{Application, Command, Element, Settings, Subscription, Theme};

use std::env;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

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
    Editing {
        filepath: PathBuf,
        content: String,
        is_dirty: bool,
    },
    Errored {
        error: Error,
    },
}

struct Flags {
    filepath: PathBuf,
}

#[derive(Debug, Clone)]
enum Message {
    Loaded(Result<(PathBuf, Arc<String>), Error>),
    Write(char),
    Amend,
    Save(usize),
    Saved(Result<(), Error>),
}

impl Application for Kanso {
    type Theme = Theme;
    type Message = Message;
    type Executor = executor::Default;
    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Kanso::Loading,
            Command::perform(load(flags.filepath), Message::Loaded),
        )
    }

    fn title(&self) -> String {
        String::from("Kanso")
    }

    fn update(&mut self, message: Message) -> Command<Self::Message> {
        match message {
            Message::Loaded(Ok((filepath, content))) => {
                *self = Self::Editing {
                    filepath,
                    content: (*content).clone(),
                    is_dirty: false,
                };

                Command::none()
            }
            Message::Loaded(Err(error)) => {
                *self = Self::Errored { error };

                Command::none()
            }
            Message::Write(character) => {
                if let Self::Editing {
                    content, is_dirty, ..
                } = self
                {
                    content.push(character);
                    *is_dirty = true;

                    Command::perform(wait_a_bit(), {
                        let version = content.len();
                        move |_| Message::Save(version)
                    })
                } else {
                    Command::none()
                }
            }
            Message::Amend => {
                if let Self::Editing {
                    content, is_dirty, ..
                } = self
                {
                    content.pop();
                    *is_dirty = true;

                    Command::perform(wait_a_bit(), {
                        let version = content.len();
                        move |_| Message::Save(version)
                    })
                } else {
                    Command::none()
                }
            }
            Message::Save(version) => match self {
                Self::Editing {
                    filepath,
                    content,
                    is_dirty,
                } if *is_dirty && content.len() == version => {
                    Command::perform(save(filepath.clone(), content.clone()), Message::Saved)
                }
                _ => Command::none(),
            },
            Message::Saved(Ok(())) => {
                if let Self::Editing { is_dirty, .. } = self {
                    *is_dirty = false;
                }

                Command::none()
            }
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
            Self::Editing {
                filepath,
                content,
                is_dirty,
            } => {
                let writer = fade(
                    container(
                        text(format!(
                            "{}_",
                            &content[content.len().saturating_sub(1_000)..]
                        ))
                        .font(Font {
                            family: font::Family::Serif,
                            ..Font::DEFAULT
                        })
                        .size(40),
                    )
                    .width(700)
                    .padding(20),
                );

                let status_bar = row![text(format!(
                    "{}{}",
                    filepath.as_path().to_str().unwrap_or(""),
                    if *is_dirty { "*" } else { "" }
                ))]
                .padding(20);

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

async fn load(filepath: impl AsRef<Path>) -> Result<(PathBuf, Arc<String>), Error> {
    let path = filepath.as_ref().to_path_buf();

    let exists = tokio::fs::try_exists(filepath).await?;

    let content = if exists {
        tokio::fs::read_to_string(&path).await?
    } else {
        String::new()
    };

    Ok((path, Arc::new(content)))
}

async fn save(filepath: impl AsRef<Path>, content: String) -> Result<(), Error> {
    tokio::fs::write(filepath, content).await?;

    Ok(())
}

async fn wait_a_bit() {
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
}

#[derive(Debug, Clone, Error)]
enum Error {
    #[error("IO operation failed: {0}")]
    IOFailed(io::ErrorKind),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::IOFailed(error.kind())
    }
}
