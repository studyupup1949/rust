use std::path::{Path, PathBuf};

use crate::{
    data::{Answers, Puzzle},
    Day, Part, Year,
};

// TODO: Support encryption and decryption.
#[derive(Debug)]
pub struct PuzzleCache {
    cache_dir: PathBuf,
    _encryption_token: Option<String>,
}

impl PuzzleCache {
    const INPUT_FILE_NAME: &'static str = "input.txt";
    const PART_ONE_ANSWERS_FILE_NAME: &'static str = "part-1-answers.txt";
    const PART_TWO_ANSWERS_FILE_NAME: &'static str = "part-2-answers.txt";

    pub fn new<P: Into<PathBuf>, S: Into<String>>(
        cache_dir: P,
        encryption_token: Option<S>,
    ) -> Self {
        // TODO: Validate cache_dir is a directory, and is writable.
        PuzzleCache {
            cache_dir: cache_dir.into(),
            _encryption_token: encryption_token.map(|x| x.into()),
        }
    }

    pub fn load_input(&self, day: Day, year: Year) -> std::io::Result<String> {
        std::fs::read_to_string(Self::input_file_path(&self.cache_dir, day, year))
    }

    pub fn load_answers(&self, part: Part, day: Day, year: Year) -> std::io::Result<Answers> {
        // TODO: Convert unwraps into Errors.
        Ok(Answers::deserialize_from_str(&std::fs::read_to_string(
            Self::answers_file_path(&self.cache_dir, part, day, year),
        )?))
    }

    pub fn load_puzzle(&self, day: Day, year: Year) -> Puzzle {
        // TODO: Convert unwraps into Errors.
        // TODO: Create default input or answers if the files don't exist.
        Puzzle {
            day,
            year,
            input: self.load_input(day, year).unwrap(),
            part_one_answers: self.load_answers(Part::One, day, year).unwrap(),
            part_two_answers: self.load_answers(Part::Two, day, year).unwrap(),
        }
    }

    pub fn save(&self, puzzle: Puzzle) {
        // TODO: Convert unwraps into Errors.
        // Create the puzzle directory in the cache if it doesn't already exist.
        let puzzle_dir = Self::dir_for_puzzle(&self.cache_dir, puzzle.day, puzzle.year);
        std::fs::create_dir_all(puzzle_dir).unwrap();

        self.save_input(&puzzle.input, puzzle.day, puzzle.year)
            .unwrap();

        self.save_answers(&puzzle.part_one_answers, Part::One, puzzle.day, puzzle.year)
            .unwrap();

        self.save_answers(&puzzle.part_two_answers, Part::Two, puzzle.day, puzzle.year)
            .unwrap();
    }

    pub fn save_input(&self, input: &str, day: Day, year: Year) -> std::io::Result<()> {
        let input_path = Self::input_file_path(&self.cache_dir, day, year);
        let mut puzzle_dir = input_path.clone();
        puzzle_dir.pop();

        std::fs::create_dir_all(puzzle_dir).unwrap();

        std::fs::write(input_path, input)
    }

    pub fn save_answers(
        &self,
        answers: &Answers,
        part: Part,
        day: Day,
        year: Year,
    ) -> std::io::Result<()> {
        let answers_path = Self::answers_file_path(&self.cache_dir, part, day, year);
        let mut puzzle_dir = answers_path.clone();
        puzzle_dir.pop();

        std::fs::create_dir_all(puzzle_dir).unwrap();

        std::fs::write(answers_path, answers.serialize_to_string())
    }

    pub fn dir_for_puzzle(cache_dir: &Path, day: Day, year: Year) -> PathBuf {
        cache_dir.join(format!("y{}", year)).join(day.to_string())
    }

    pub fn input_file_path(cache_dir: &Path, day: Day, year: Year) -> PathBuf {
        Self::dir_for_puzzle(cache_dir, day, year).join(Self::INPUT_FILE_NAME)
    }

    pub fn answers_file_path(cache_dir: &Path, part: Part, day: Day, year: Year) -> PathBuf {
        Self::dir_for_puzzle(cache_dir, day, year).join(match part {
            Part::One => Self::PART_ONE_ANSWERS_FILE_NAME,
            Part::Two => Self::PART_TWO_ANSWERS_FILE_NAME,
        })
    }
}
