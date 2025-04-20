#![feature(string_remove_matches)]

use std::fs::File;
use std::io::{BufRead, BufReader};

use eframe::egui::{
    CentralPanel, FontId, RichText, ScrollArea, TextEdit, TextStyle, Ui, Vec2, ViewportBuilder,
};

const FIELD_SIZE: Vec2 = Vec2 { x: 50.0, y: 20.0 };

fn input(buffer: &mut String, height: f32) -> TextEdit {
    TextEdit::singleline(buffer).font(FontId::monospace(height))
}

fn char_field(
    ui: &mut Ui,
    word: &mut Word,
    words: &[String],
    idx: usize,
    height: f32,
) -> Option<Vec<String>> {
    let textedit = input(&mut word.chars[idx], height).char_limit(1);
    if ui.add_sized(FIELD_SIZE, textedit).changed() {
        for w in &mut word.wrong_pos {
            w.remove_matches(&word.chars[idx].clone());
        }
        return Some(words.iter().filter_map(|w| word.filter(w)).collect());
    }

    None
}

fn wrong_pos_field(
    ui: &mut Ui,
    word: &mut Word,
    words: &[String],
    idx: usize,
    height: f32,
) -> Option<Vec<String>> {
    let textedit = input(&mut word.wrong_pos[idx], height);
    if ui.add_sized(FIELD_SIZE, textedit).changed() {
        return Some(words.iter().filter_map(|w| word.filter(w)).collect());
    }

    None
}

fn wrong_field(ui: &mut Ui, word: &mut Word, words: &[String], height: f32) -> Option<Vec<String>> {
    let textedit = input(&mut word.wrong, height);
    let field_size = Vec2 {
        x: ui.available_width(),
        y: 20.0,
    };

    if ui.add_sized(field_size, textedit).changed() {
        return Some(words.iter().filter_map(|w| word.filter(w)).collect());
    }

    None
}

#[derive(Default, Debug)]
struct Word {
    chars: [String; 5],

    wrong: String,
    wrong_pos: [String; 5],
}

impl Word {
    fn filter(&self, w: &str) -> Option<String> {
        // TODO: double letters could be handled better

        let w_chars = w.chars().collect::<Vec<_>>();

        if self
            .chars
            .iter()
            .enumerate()
            .filter(|(_, ch)| !ch.is_empty())
            .any(|(idx, ch)| w_chars[idx] != ch.chars().next().unwrap())
        {
            return None;
        }

        if self.wrong.chars().any(|ch| w.contains(ch)) {
            return None;
        }

        if self.wrong_pos.iter().enumerate().any(|(idx, chars)| {
            chars
                .chars()
                .any(|ch| w_chars[idx] == ch || !w.contains(ch))
        }) {
            return None;
        }

        Some(w.to_string())
    }
}

fn sort_possible_by_entropy(possible: &mut [String]) {
    possible.sort_unstable_by_key(|w| {
        let mut w = w.chars().collect::<Vec<_>>();
        w.sort_unstable();
        w.dedup();
        w.len()
    });
    possible.reverse();
}

fn main() -> Result<(), std::io::Error> {
    let mut word = Word::default();
    let mut words: Vec<String> = Vec::new();
    let mut possible: Vec<String> = Vec::new();
    sort_possible_by_entropy(&mut possible);

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_max_inner_size([298.0, 450.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_simple_native("Wordle Helper", options, move |ctx, _frame| {
        CentralPanel::default().show(ctx, |ui| {
            let monospace_height: f32 = ui.text_style_height(&TextStyle::Monospace);

            ui.vertical_centered(|ui| {
                ui.heading("Found Characters");
            });
            ui.horizontal(|ui| {
                for idx in 0..5 {
                    if let Some(filtered) = char_field(ui, &mut word, &words, idx, monospace_height)
                    {
                        possible = filtered;
                        sort_possible_by_entropy(&mut possible);
                    }
                }
            });

            ui.add_space(10.0);
            ui.vertical_centered(|ui| {
                ui.heading("Characters At Wrong Position");
            });
            ui.horizontal(|ui| {
                for idx in 0..5 {
                    if let Some(filtered) =
                        wrong_pos_field(ui, &mut word, &words, idx, monospace_height)
                    {
                        possible = filtered;
                        sort_possible_by_entropy(&mut possible);
                    }
                }
            });

            ui.add_space(10.0);
            ui.vertical_centered(|ui| {
                ui.heading("Wrong Characters");
            });
            if let Some(filtered) = wrong_field(ui, &mut word, &words, monospace_height) {
                possible = filtered;
                sort_possible_by_entropy(&mut possible);
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("Reset").clicked() {
                    word = Word::default();
                    possible = words.clone();
                    sort_possible_by_entropy(&mut possible);
                }

                let open_file = ui.button("Open wordlist file…");
                if open_file.clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        if let Ok(file) = File::open(path) {
                            words.clear();

                            for word in BufReader::new(file).lines().map_while(Result::ok) {
                                words.push(word);
                            }

                            possible = words.clone();
                        }
                    }
                }
            });

            let area_content = |ui: &mut Ui, range: std::ops::Range<usize>| {
                for row in range {
                    ui.label(
                        RichText::new(&possible[row]).font(FontId::monospace(monospace_height)),
                    );
                }
            };

            ui.add_space(10.0);
            ui.label(format!("{} possible words", possible.len()));
            ScrollArea::vertical().auto_shrink(false).show_rows(
                ui,
                monospace_height,
                possible.len(),
                area_content,
            );
        });
    })
    .expect("eframe error");

    Ok(())
}
