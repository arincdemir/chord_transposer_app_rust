#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::ChordTransposerApp;

use std::fmt;

const NOTES_SHARPS: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];
const NOTES_FLATS: [&str; 12] = [
    "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
];
const MODIFIERS: [&str; 24] = [
    "m", "7", "maj7", "m7", "sus2", "sus4", "dim", "aug", "5", "add9", "9", "6", "11", "13",
    "7sus4", "dim7", "m6", "m9", "maj9", "m11", "m13", "maj13", "add11", "7b9",
];

// line is determined to be a chord line if
// chords - DETERMINATION_SLOPE * non_chords + DETERMINATION_BIAS > 0
const DETERMINATION_SLOPE: f64 = 0.8;
const DETERMINATION_BIAS: f64 = 0.7;

#[derive(Debug)]
enum Text {
    Chord(Chord),
    NonChord(String),
    Space(String),
}

impl Text {
    fn to_string(&self) -> String {
        match self {
            Self::Chord(chord) => chord.to_string(),
            Self::NonChord(s) => s.clone(),
            Self::Space(s) => s.clone(),
        }
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Text::Chord(chord) => write!(f, "{}", chord.to_string()),
            Text::NonChord(text) => write!(f, "{}", text),
            Text::Space(text) => write!(f, "{}", text),
        }
    }
}

impl From<&str> for Text {
    fn from(value: &str) -> Self {
        if value.chars().next().unwrap().is_whitespace() {
            return Self::Space(value.to_string());
        }
        let mut chord_base = "";
        for base in NOTES_FLATS {
            if value.starts_with(base) && base.len() > chord_base.len() {
                chord_base = base;
            }
        }
        for base in NOTES_SHARPS {
            if value.starts_with(base) && base.len() > chord_base.len() {
                chord_base = base;
            }
        }

        if chord_base == "" {
            return Self::NonChord(value.to_string());
        }

        let supposed_modifier = &value[chord_base.len()..];
        if supposed_modifier.is_empty() {
            return Self::Chord(Chord::new(chord_base, None));
        }
        let is_valid_modifier = MODIFIERS
            .iter()
            .any(|&modifier| modifier == supposed_modifier);

        if is_valid_modifier {
            Self::Chord(Chord::new(chord_base, Some(supposed_modifier)))
        } else {
            Self::NonChord(value.to_string())
        }
    }
}

#[derive(Debug)]
struct Chord {
    base: String,
    modifier: Option<String>,
}

impl Chord {
    fn new(base: &str, modifier: Option<&str>) -> Chord {
        Chord {
            base: base.to_string(),
            modifier: match modifier {
                Some(m) => Some(String::from(m)),
                None => None,
            },
        }
    }

    fn transpose(&mut self, half_steps: i32) {
        let base_index = if let Some(index) = NOTES_FLATS
            .iter()
            .position(|&note| note == self.base.as_str())
        {
            index
        } else {
            NOTES_SHARPS
                .iter()
                .position(|&note| note == self.base.as_str())
                .unwrap()
        };

        let new_index = (base_index as i32 + half_steps).rem_euclid(12) as usize;
        self.base = NOTES_SHARPS[new_index].to_string();
    }

    fn to_string(&self) -> String {
        match &self.modifier {
            Some(m) => format!("{}{}", self.base, m),
            None => self.base.clone(),
        }
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Debug)]
struct Line {
    texts: Vec<Text>,
    chord_number: usize,
    non_chord_number: usize,
}

impl Line {
    fn transpose(&mut self, half_steps: i32) {
        if !self.is_chord_line() {
            return;
        }
        for text in &mut self.texts {
            if let Text::Chord(chord) = text {
                chord.transpose(half_steps);
            }
        }
    }

    fn is_chord_line(&self) -> bool {
        if self.non_chord_number == 0 && self.chord_number == 0 {
            return false;
        } else if self.non_chord_number == 0 {
            return true;
        } else if self.non_chord_number == 0 {
            return true;
        } else {
            return (self.chord_number as f64)
                - (self.non_chord_number as f64 * DETERMINATION_SLOPE)
                + DETERMINATION_BIAS
                > 0.0;
        }
    }
}

impl From<&str> for Line {
    fn from(value: &str) -> Self {
        let mut texts = Vec::new();
        let mut chord_number = 0;
        let mut non_chord_number = 0;
        let mut left = 0;
        let chars: Vec<char> = value.chars().collect();

        for (right, ch) in value.char_indices() {
            if chars[left].is_whitespace() != ch.is_whitespace() {
                let new_text = Text::from(&value[left..right]);
                match new_text {
                    Text::Chord(_) => chord_number += 1,
                    Text::NonChord(_) => non_chord_number += 1,
                    Text::Space(_) => (),
                }
                texts.push(new_text);
                left = right;
            }
        }

        if !value.is_empty() {
            let last_text = Text::from(&value[left..]);
            match last_text {
                Text::Chord(_) => chord_number += 1,
                Text::NonChord(_) => non_chord_number += 1,
                Text::Space(_) => (),
            }
            texts.push(last_text);
        }

        Self {
            texts: texts,
            chord_number: chord_number,
            non_chord_number: non_chord_number,
        }
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.texts
                .iter()
                .map(|text| text.to_string())
                .collect::<String>()
        )
    }
}

pub fn transpose_text(text: &String, half_steps: i32) -> String {
    let mut lines: Vec<Line> = text.lines().map(|line| (Line::from(line))).collect();
    let transposed_text = lines
        .iter_mut()
        .map(|line| {
            line.transpose(half_steps);
            line.to_string()
        })
        .collect::<Vec<String>>()
        .join("\n");
    transposed_text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chord_transpose() {
        let mut chord = Chord {
            base: "C".to_string(),
            modifier: Some("maj7".to_string()),
        };
        chord.transpose(3);
        assert_eq!(chord.to_string(), "D#maj7");
    }

    #[test]
    fn test_negative_chord_transpose() {
        let mut chord = Chord {
            base: "D#".to_string(),
            modifier: Some("m".to_string()),
        };
        chord.transpose(-2);
        assert_eq!(chord.to_string(), "C#m");
    }

    #[test]
    fn test_text_from_non_chord() {
        let text = Text::from("Hello");
        assert!(matches!(text, Text::NonChord(_)));
        assert_eq!(text.to_string(), "Hello");
    }

    #[test]
    fn test_text_from_basic_chord() {
        let text = Text::from("C#");
        assert!(matches!(text, Text::Chord(_)));
        assert_eq!(text.to_string(), "C#");
    }

    #[test]
    fn test_text_from_chord_with_modifier() {
        let text = Text::from("Am");
        assert!(matches!(text, Text::Chord(_)));
        assert_eq!(text.to_string(), "Am");
    }

    #[test]
    fn test_text_from_complex_chord() {
        let text = Text::from("Cmaj7");
        assert!(matches!(text, Text::Chord(_)));
        assert_eq!(text.to_string(), "Cmaj7");
    }

    #[test]
    fn text_text_from_nonvalid_modifier() {
        let text = Text::from("Cmaj78");
        assert!(matches!(text, Text::NonChord(_)));
        assert_eq!(text.to_string(), "Cmaj78");
    }

    #[test]
    #[should_panic]
    fn test_text_from_empty_string() {
        let _ = Text::from("");
    }

    #[test]
    fn test_text_from_whitespace() {
        let text = Text::from("   ");
        assert!(matches!(text, Text::Space(_)));
        assert_eq!(text.to_string(), "   ");
    }

    #[test]
    fn test_line_chord_count() {
        let line_text = "Am I Wanna Smile Cmaj7 D#";
        let line = Line::from(line_text);
        assert_eq!(line.chord_number, 3);
        assert_eq!(line.non_chord_number, 3);
    }

    #[test]
    fn test_line_is_chord_line() {
        let line_text = "Am I Wanna Smile Cmaj7 D#";
        let line = Line::from(line_text);
        assert!(line.is_chord_line());
    }
}
