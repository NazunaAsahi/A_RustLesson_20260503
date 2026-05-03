use iced::event::Status;
use iced::keyboard::{self, Key};
use iced::widget::{column, container, row, text};
use iced::{Color, Element, Event, Length, Subscription, Task, Theme};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fs;

// ─── データ型 ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct WordEntry {
    display: String,  // 表示単語（漢字・かな混じり）
    hiragana: String, // ひらがな表記
    romaji: String,   // ローマ字表記
}

// ─── アプリケーション状態 ───────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum GameState {
    Waiting, // スペースキー待機中
    Playing, // ゲームプレイ中
    End,     // 3単語終了後
}

#[derive(Debug, Clone)]
struct TypingGame {
    state: GameState,
    word_list: Vec<WordEntry>,     // 全単語リスト
    current_words: Vec<WordEntry>, // 今回選ばれた3単語
    current_index: usize,          // 現在の単語インデックス（0〜2）
    input_buffer: String,          // 現在の入力バッファ
    error_flash: bool,             // ミス時の赤フラッシュ用フラグ
}

// ─── メッセージ ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum Message {
    KeyEvent(keyboard::Key, keyboard::Modifiers),
}

// ─── メイン関数 ──────────────────────────────────────────────────────────────

fn main() -> iced::Result {
    // iced 0.14: 第1引数は boot 関数。タイトルは .title() で指定する。
    iced::application(TypingGame::new, TypingGame::update, TypingGame::view)
        .title("タイピングゲーム")
        .subscription(TypingGame::subscription)
        .theme(theme)
        .run()
}

// クロージャではなく fn ポインタで定義するとライフタイムが汎化される
fn theme(_state: &TypingGame) -> Theme {
    Theme::Dark
}



impl TypingGame {
    fn new() -> (Self, Task<Message>) {
        let word_list = load_words("words.txt");
        (
            Self {
                state: GameState::Waiting,
                word_list,
                current_words: vec![],
                current_index: 0,
                input_buffer: String::new(),
                error_flash: false,
            },
            Task::none(),
        )
    }

    // ─── サブスクリプション ─────────────────────────────────────────────────

    fn subscription(&self) -> Subscription<Message> {
        iced::event::listen_with(|event, status, _| {
            // テキスト入力ウィジェットが捕捉済みのイベントは無視
            if status == Status::Captured {
                return None;
            }
            match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key, modifiers, ..
                }) => Some(Message::KeyEvent(key, modifiers)),
                _ => None,
            }
        })
    }

    // ─── 更新ロジック ────────────────────────────────────────────────────────

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::KeyEvent(key, _modifiers) => {
                self.handle_key(key);
            }
        }
        Task::none()
    }

    fn handle_key(&mut self, key: Key) {
        match &self.state {
            // ─ 待機中・終了後：スペースキーでスタート ──────────────────────
            GameState::Waiting | GameState::End => {
                if key == Key::Named(keyboard::key::Named::Space) {
                    self.start_game();
                }
            }
            // ─ プレイ中：文字入力処理 ────────────────────────────────────────
            GameState::Playing => {
                match key {
                    Key::Named(keyboard::key::Named::Backspace) => {
                        self.input_buffer.pop();
                        self.error_flash = false;
                    }
                    Key::Named(keyboard::key::Named::Space) => {
                        // スペースは無視（誤入力防止）
                    }
                    Key::Character(s) => {
                        let ch_lower = s.to_lowercase();
                        let ch_str = ch_lower.as_str();

                        if let Some(word) = self.current_words.get(self.current_index) {
                            let target = word.romaji.clone();
                            let candidate = format!("{}{}", self.input_buffer, ch_str);

                            if target.starts_with(&candidate) {
                                // 正しい入力
                                self.input_buffer = candidate;
                                self.error_flash = false;

                                // 完全一致 → 次の単語へ
                                if self.input_buffer == target {
                                    self.advance();
                                }
                            } else {
                                // 誤入力フラッシュ
                                self.error_flash = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn start_game(&mut self) {
        let mut rng = thread_rng();
        let pool = &self.word_list;
        let n = 3.min(pool.len());
        let mut chosen: Vec<WordEntry> = pool.choose_multiple(&mut rng, n).cloned().collect();

        // 3単語に満たない場合は繰り返しで補填
        while chosen.len() < 3 {
            if let Some(w) = pool.choose(&mut rng) {
                chosen.push(w.clone());
            } else {
                break;
            }
        }
        self.current_words = chosen;
        self.current_index = 0;
        self.input_buffer = String::new();
        self.error_flash = false;
        self.state = GameState::Playing;
    }

    fn advance(&mut self) {
        self.current_index += 1;
        self.input_buffer = String::new();
        self.error_flash = false;
        if self.current_index >= self.current_words.len() {
            self.state = GameState::End;
        }
    }

    // ─── ビュー ──────────────────────────────────────────────────────────────

    fn view(&self) -> Element<Message> {
        let content: Element<Message> = match &self.state {
            GameState::Waiting => self.view_waiting(),
            GameState::Playing => self.view_playing(),
            GameState::End => self.view_end(),
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(
                    0.07, 0.07, 0.12,
                ))),
                ..Default::default()
            })
            .into()
    }

    fn view_waiting(&self) -> Element<Message> {
        column![
            text("Typing Game")
                .size(52)
                .color(Color::from_rgb(0.9, 0.85, 1.0)),
            text(""),
            text("Press [Space] to Start")
                .size(28)
                .color(Color::from_rgb(0.6, 0.8, 1.0)),
        ]
        .spacing(20)
        .align_x(iced::Alignment::Center)
        .into()
    }

    fn view_playing(&self) -> Element<Message> {
        let word = match self.current_words.get(self.current_index) {
            Some(w) => w,
            None => return text("...").into(),
        };

        // 進捗インジケーター
        let progress = format!("{} / {}", self.current_index + 1, self.current_words.len());

        // 入力済み部分と未入力部分を色分け
        let typed_len = self.input_buffer.len();
        let target = &word.romaji;
        let typed_str = &target[..typed_len.min(target.len())];
        let remaining_str = &target[typed_len.min(target.len())..];

        let romaji_row: Element<Message> = row![
            text(typed_str)
                .size(38)
                .color(Color::from_rgb(0.3, 1.0, 0.55)),
            text(remaining_str).size(38).color(if self.error_flash {
                Color::from_rgb(1.0, 0.3, 0.3)
            } else {
                Color::WHITE
            }),
        ]
        .into();

        column![
            text(progress)   // &String → String で値渡し（ローカル変数への参照を返さない）
                .size(20)
                .color(Color::from_rgb(0.5, 0.5, 0.7)),
            text(""),
            text(&word.display)
                .size(64)
                .color(Color::from_rgb(1.0, 0.95, 0.8)),
            text(&word.hiragana)
                .size(32)
                .color(Color::from_rgb(0.75, 0.88, 1.0)),
            text(""),
            romaji_row,
        ]
        .spacing(12)
        .align_x(iced::Alignment::Center)
        .into()
    }

    fn view_end(&self) -> Element<Message> {
        column![
            text("End!")
                .size(80)
                .color(Color::from_rgb(1.0, 0.85, 0.2)),
            text(""),
            text("よくできました！")
                .size(32)
                .color(Color::from_rgb(0.9, 0.9, 1.0)),
            text(""),
            text("Press [Space] to Restart")
                .size(24)
                .color(Color::from_rgb(0.6, 0.8, 1.0)),
        ]
        .spacing(16)
        .align_x(iced::Alignment::Center)
        .into()
    }
}

// ─── ファイル読み込み ─────────────────────────────────────────────────────────

fn load_words(path: &str) -> Vec<WordEntry> {
    let content = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("⚠️  {path} の読み込みに失敗しました: {e}");
        // フォールバック用サンプルデータ
        "日本語|にほんご|nihongo\n東京|とうきょう|toukyou\n富士山|ふじさん|fujisan\n桜|さくら|sakura\n侍|さむらい|samurai".to_string()
    });

    content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            if parts.len() == 3 {
                Some(WordEntry {
                    display: parts[0].trim().to_string(),
                    hiragana: parts[1].trim().to_string(),
                    romaji: parts[2].trim().to_string(),
                })
            } else {
                eprintln!("⚠️  不正な行をスキップ: {line}");
                None
            }
        })
        .collect()
}
