use iced::executor;
use iced::keyboard;
use iced::widget::{column, container, text, Center};
use iced::{Application, Command, Element, Event, Length, Settings, Subscription, Theme};
use rand::seq::SliceRandom;
use std::fs;

pub fn main() -> iced::Result {
    TypingGame::run(Settings::default())
}

#[derive(Debug, Clone)]
struct WordData {
    display: String,
    kana: String,
    roman: String,
}

enum GameState {
    Idle,
    Playing {
        words: Vec<WordData>,
        current_index: usize,
        typed_count: usize,
    },
    End,
}

struct TypingGame {
    state: GameState,
    all_words: Vec<WordData>,
}

#[derive(Debug, Clone)]
enum Message {
    EventOccurred(Event),
}

impl Application for TypingGame {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        // テキストファイルからデータをパース
        let content = fs::read_to_string("words.txt").unwrap_or_default();
        let all_words: Vec<WordData> = content
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() == 3 {
                    Some(WordData {
                        display: parts[0].to_string(),
                        kana: parts[1].to_string(),
                        roman: parts[2].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();

        (
            Self {
                state: GameState::Idle,
                all_words,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Rust Typing Game - e-typing Style")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::EventOccurred(Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Space),
                ..
            })) => {
                match self.state {
                    GameState::Idle | GameState::End => {
                        // ランダムに3つの単語を抽出して開始
                        let mut rng = rand::thread_rng();
                        let selected_words = self.all_words.choose_multiple(&mut rng, 3).cloned().collect();
                        self.state = GameState::Playing {
                            words: selected_words,
                            current_index: 0,
                            typed_count: 0,
                        };
                    }
                    _ => {}
                }
            }
            Message::EventOccurred(Event::Keyboard(keyboard::Event::CharacterReceived(c))) => {
                if let GameState::Playing {
                    ref words,
                    ref mut current_index,
                    ref mut typed_count,
                } = self.state
                {
                    let current_word = &words[*current_index];
                    let target_char = current_word.roman.chars().nth(*typed_count);

                    // 入力文字の照合（大文字小文字を区別しない場合は .to_uppercase() 等を検討）
                    if Some(c.to_ascii_uppercase()) == target_char {
                        *typed_count += 1;

                        // 単語の全文字を打ち終えたか判定
                        if *typed_count >= current_word.roman.len() {
                            *typed_count = 0;
                            *current_index += 1;

                            // 3つの単語が終了したか判定
                            if *current_index >= words.len() {
                                self.state = GameState::End;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::event::listen().map(Message::EventOccurred)
    }

    fn view(&self) -> Element<Message> {
        let content = match &self.state {
            GameState::Idle => column![
                text("Typing Game").size(40),
                text("Press [ Space ] to Start").size(20),
            ]
            .spacing(20)
            .align_items(iced::Alignment::Center),

            GameState::Playing {
                words,
                current_index,
                typed_count,
            } => {
                let current_word = &words[*current_index];
                let (typed, remaining) = current_word.roman.split_at(*typed_count);

                column![
                    // 上段：表示単語
                    text(&current_word.display).size(50).style(iced::Color::BLACK),
                    // 中段：ひらがな
                    text(&current_word.kana).size(25).style(iced::Color::from_rgb(0.4, 0.4, 0.4)),
                    // 下段：ローマ字（タイピング済みは色を変える）
                    iced::widget::row![
                        text(typed).style(iced::Color::from_rgb(0.0, 0.6, 0.0)),
                        text(remaining).style(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                    ]
                    .spacing(0)
                    .align_items(iced::Alignment::Center),
                ]
                .spacing(15)
                .align_items(iced::Alignment::Center)
            }

            GameState::End => column![
                text("End").size(60).style(iced::Color::from_rgb(0.8, 0.0, 0.0)),
                text("Press [ Space ] to Restart").size(20),
            ]
            .spacing(20)
            .align_items(iced::Alignment::Center),
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}