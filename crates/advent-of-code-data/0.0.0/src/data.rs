use std::{
    io::{BufRead, BufReader, BufWriter, Read, Write},
    str::FromStr,
};

use crate::{Answer, Day, Part, Year};

pub const CORRECT_ANSWER_CHAR: char = '=';
pub const WRONG_ANSWER_CHAR: char = 'X';
pub const LOW_ANSWER_CHAR: char = '[';
pub const HIGH_ANSWER_CHAR: char = ']';

/// Stores puzzle input and answer data.
#[derive(Debug, PartialEq)]
pub struct Puzzle {
    pub day: Day,
    pub year: Year,
    pub input: String,
    pub part_one_answers: Answers,
    pub part_two_answers: Answers,
}

impl Puzzle {
    pub fn answers(&self, part: Part) -> &Answers {
        match part {
            Part::One => &self.part_one_answers,
            Part::Two => &self.part_two_answers,
        }
    }

    pub fn answers_mut(&mut self, part: Part) -> &mut Answers {
        match part {
            Part::One => &mut self.part_one_answers,
            Part::Two => &mut self.part_two_answers,
        }
    }
}

/// Represents the various outcomes of checking an answer against an answers
/// database.
#[derive(Clone, Debug, PartialEq)]
pub enum CheckResult {
    /// The answer is correct.
    Correct,
    /// The answer is incorrect.
    Wrong,
    /// The answer is too low and incorrect.
    TooLow,
    /// The answer is too high and incorrect.
    TooHigh,
}

/// Stores correct and incorrect answers for a puzzle, along with hints such as
/// "too large" and "too small".
#[derive(Debug, PartialEq)]
pub struct Answers {
    correct_answer: Option<Answer>,
    wrong_answers: Vec<Answer>,
    low_bounds: Option<i64>,
    high_bounds: Option<i64>,
}

impl Answers {
    /// Initialize new `Answers` object.
    pub fn new() -> Self {
        Answers {
            correct_answer: None,
            wrong_answers: Vec::new(),
            low_bounds: None,
            high_bounds: None,
        }
    }

    pub fn correct_answer_ref(&self) -> &Option<Answer> {
        &self.correct_answer
    }

    pub fn wrong_answers_ref(&self) -> &Vec<Answer> {
        &self.wrong_answers
    }

    pub fn low_bounds_ref(&self) -> &Option<i64> {
        &self.low_bounds
    }

    pub fn high_bounds_ref(&self) -> &Option<i64> {
        &self.high_bounds
    }

    /// Checks if this answer is correct or incorrect according to the information
    /// stored in this `Answers` database.
    ///
    /// `None` is returned when the supplied answer does not match any known
    /// incorrect values, and the database does not have a known correct value.
    /// When a `None` value is returned, the caller should submit the answer as
    /// a solution to the puzzle using the Advent of Code client. The caller
    /// should then update this object with the response depending on if the
    /// client say it was correct or incorrect.
    pub fn check(&self, answer: &Answer) -> Option<CheckResult> {
        // Check the answer against the optional low and high value boundaries.
        match (answer.to_i64(), &self.low_bounds, &self.high_bounds) {
            (Some(answer), Some(low), _) if answer <= *low => {
                return Some(CheckResult::TooLow);
            }
            (Some(answer), _, Some(high)) if answer >= *high => return Some(CheckResult::TooHigh),
            _ => {}
        };

        // Check if the answer is matches any known incorrect answers.
        for wrong_answer in &self.wrong_answers {
            if wrong_answer == answer {
                return Some(CheckResult::Wrong);
            }
        }

        // Now see if the answer matches the correct answer or return `None` if
        // the correct answer is not known.
        match &self.correct_answer {
            Some(correct) if correct == answer => Some(CheckResult::Correct),
            Some(_) => Some(CheckResult::Wrong),
            None => None,
        }
    }

    /// Adds an answer to the list of known wrong answers.
    pub fn add_wrong_answer(&mut self, answer: Answer) {
        // TODO: Verify that wrong answer is not the correct answer
        // TODO: Error if the wrong answer has a newline.
        if self.wrong_answers.iter().all(|x| x != &answer) {
            self.wrong_answers.push(answer);
        } else {
            tracing::warn!(
                "skipped adding duplicate wrong answer to answers cache: `{}`",
                answer
            );
        }
    }

    /// Sets this answer as the known correct answer.
    pub fn set_correct_answer(&mut self, answer: Answer) {
        // TODO: Verify that correct answer is not a wrong answer, hi or low.
        // TODO: Error if the right answer has a newline.
        self.correct_answer = Some(answer);
    }

    /// Sets a low boundary value in the cache.
    ///
    /// If the cache has an existing low boundary then the highest value will be
    /// used as the new high boundary.
    ///
    /// Any numeric answer passed to `Answers::check` will be returned as
    /// `CheckResult::TooLow` if it equals or is smaller than the low boundary.
    pub fn set_low_bounds(&mut self, answer: Answer) -> i64 {
        // TODO: Verify that low bounds is not a correct answer.
        // TODO: Verify that low bounds is not larger or equal to high bounds.
        // TODO: Remove panic and return Error if the answer is not an integer.
        let answer = answer.to_i64().expect("low bounds answer must be numeric");

        match &self.low_bounds {
            Some(low) if answer > *low => {
                self.low_bounds = Some(answer);
                answer
            }
            Some(low) => *low,
            None => {
                self.low_bounds = Some(answer);
                answer
            }
        }
    }

    /// Sets a high boundary value in the cache.
    ///
    /// If the cache has an existing high boundary then the lowest value will be
    /// used as the new high boundary.
    ///
    /// Any numeric answer passed to `Answers::check` will be returned as
    /// `CheckResult::TooHigh` if it equals or is larger than the high boundary.
    pub fn set_high_bounds(&mut self, answer: Answer) -> i64 {
        // TODO: Verify that high bounds is not a correct answer.
        // TODO: Verify that high bounds is not smaller or equal to low bounds.
        // TODO: Remove panic and return Error if the answer is not an integer.
        let answer = answer.to_i64().expect("high bounds answer must be numeric");

        match &self.high_bounds {
            Some(high) if answer < *high => {
                self.high_bounds = Some(answer);
                answer
            }
            Some(high) => *high,
            None => {
                self.high_bounds = Some(answer);
                answer
            }
        }
    }

    pub fn serialize_to_string(&self) -> String {
        // TODO: Convert unwraps into Errors.
        let mut buf = BufWriter::new(Vec::new());
        self.serialize(&mut buf);

        String::from_utf8(buf.into_inner().unwrap()).unwrap()
    }

    pub fn serialize<W: Write>(&self, writer: &mut BufWriter<W>) {
        // TODO: Verify that both correct and wrong answers do not have newlines.
        // TODO: Convert unwraps to Errors.

        // Sort wrong answers alphabetically to ensure stability with diffs
        // for version control.
        let mut wrong_answers: Vec<String> =
            self.wrong_answers.iter().map(|x| x.to_string()).collect();
        wrong_answers.sort();

        // Serialize all the answers to buffered writer.
        fn write_field<S: ToString, W: Write>(
            field: &Option<S>,
            prefix: char,
            writer: &mut BufWriter<W>,
        ) {
            if let Some(f) = field {
                let s = f.to_string();
                assert!(!s.contains('\n'));

                writeln!(writer, "{} {}", prefix, &s).unwrap();
            }
        }

        write_field(&self.correct_answer, CORRECT_ANSWER_CHAR, writer);
        write_field(&self.low_bounds, LOW_ANSWER_CHAR, writer);
        write_field(&self.high_bounds, HIGH_ANSWER_CHAR, writer);

        for wrong_answer in wrong_answers {
            write_field(&Some(wrong_answer), WRONG_ANSWER_CHAR, writer);
        }
    }

    pub fn deserialize_from_str(text: &str) -> Self {
        // TODO: Convert unwraps to Errors.
        let mut buf = BufReader::new(text.as_bytes());
        Self::deserialize(&mut buf)
    }

    pub fn deserialize<R: Read>(reader: &mut BufReader<R>) -> Self {
        // TODO: Convert unwraps to Errors.
        let mut answers = Answers::new();

        // Each line in the input string is an entry in the answers database.
        // The first character indicates the type of answer, and the characters
        // following the space hold the answer value.
        for line in reader.lines() {
            let line = line.unwrap();
            let (ty, value) = line.split_once(' ').unwrap();

            match ty.chars().next().unwrap() {
                CORRECT_ANSWER_CHAR => {
                    answers.set_correct_answer(Answer::from_str(value).unwrap());
                }
                WRONG_ANSWER_CHAR => {
                    answers.add_wrong_answer(Answer::from_str(value).unwrap());
                }
                LOW_ANSWER_CHAR => {
                    let low = value.parse::<i64>().expect("low bounds value must be int");
                    answers.set_low_bounds(Answer::Int(low));
                }
                HIGH_ANSWER_CHAR => {
                    let high = value.parse::<i64>().expect("high bounds value must be int");
                    answers.set_high_bounds(Answer::Int(high));
                }
                _ => {
                    panic!("unknown answer entry type when deserializing");
                }
            }
        }

        answers
    }
}

impl Default for Answers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_wrong_answers() {
        let mut answers = Answers::new();

        answers.add_wrong_answer(Answer::from_str("hello world").unwrap());
        answers.add_wrong_answer(Answer::from_str("foobar").unwrap());
        answers.add_wrong_answer(Answer::Int(42));

        assert_eq!(
            answers.wrong_answers_ref(),
            &vec![
                Answer::String("hello world".to_string()),
                Answer::String("foobar".to_string()),
                Answer::Int(42)
            ]
        )
    }

    #[test]
    fn correct_answer_when_checking() {
        let mut answers = Answers::new();

        answers.set_correct_answer(Answer::from_str("hello").unwrap());
        answers.add_wrong_answer(Answer::from_str("abc").unwrap());
        answers.add_wrong_answer(Answer::from_str("stop").unwrap());

        assert_eq!(
            answers.check(&Answer::from_str("hello").unwrap()),
            Some(CheckResult::Correct)
        );
    }

    #[test]
    fn wrong_answer_when_checking() {
        let mut answers = Answers::new();

        answers.set_correct_answer(Answer::from_str("hello").unwrap());
        answers.add_wrong_answer(Answer::from_str("abc").unwrap());
        answers.add_wrong_answer(Answer::from_str("stop").unwrap());

        assert_eq!(
            answers.check(&Answer::from_str("abc").unwrap()),
            Some(CheckResult::Wrong)
        );
        assert_eq!(
            answers.check(&Answer::from_str("stop").unwrap()),
            Some(CheckResult::Wrong)
        );
    }

    #[test]
    fn set_lower_high_boundary_replaces_prev() {
        let mut answers = Answers::new();
        assert_eq!(answers.high_bounds_ref(), &None);

        assert_eq!(answers.set_high_bounds(Answer::Int(30)), 30);
        assert_eq!(answers.high_bounds_ref(), &Some(30));

        assert_eq!(answers.set_high_bounds(Answer::Int(31)), 30);
        assert_eq!(answers.high_bounds_ref(), &Some(30));

        assert_eq!(answers.set_high_bounds(Answer::Int(12)), 12);
        assert_eq!(answers.high_bounds_ref(), &Some(12));
    }

    #[test]
    fn set_higher_low_boundary_replaces_prev() {
        let mut answers = Answers::new();
        assert_eq!(answers.low_bounds_ref(), &None);

        assert_eq!(answers.set_low_bounds(Answer::Int(4)), 4);
        assert_eq!(answers.low_bounds_ref(), &Some(4));

        assert_eq!(answers.set_low_bounds(Answer::Int(-2)), 4);
        assert_eq!(answers.low_bounds_ref(), &Some(4));

        assert_eq!(answers.set_low_bounds(Answer::Int(187)), 187);
        assert_eq!(answers.low_bounds_ref(), &Some(187));
    }

    #[test]
    fn check_answer_uses_low_bounds_if_set() {
        let mut answers = Answers::new();
        assert!(answers.check(&Answer::Int(100)).is_none());

        answers.set_low_bounds(Answer::Int(90));

        assert_eq!(answers.check(&Answer::Int(85)), Some(CheckResult::TooLow));
        assert_eq!(answers.check(&Answer::Int(90)), Some(CheckResult::TooLow));
        assert!(answers.check(&Answer::Int(100)).is_none());

        answers.add_wrong_answer(Answer::Int(90));
        assert_eq!(answers.check(&Answer::Int(90)), Some(CheckResult::TooLow));
    }

    #[test]
    fn check_answer_uses_high_bounds_if_set() {
        let mut answers = Answers::new();
        assert!(answers.check(&Answer::Int(100)).is_none());

        answers.set_high_bounds(Answer::Int(90));

        assert_eq!(answers.check(&Answer::Int(100)), Some(CheckResult::TooHigh));
        assert_eq!(answers.check(&Answer::Int(90)), Some(CheckResult::TooHigh));
        assert!(answers.check(&Answer::Int(85)).is_none());

        answers.add_wrong_answer(Answer::Int(90));
        assert_eq!(answers.check(&Answer::Int(90)), Some(CheckResult::TooHigh));
    }

    #[test]
    fn check_answer_checks_high_and_low_bounds_if_set() {
        let mut answers = Answers::new();

        answers.set_low_bounds(Answer::Int(96));
        answers.set_high_bounds(Answer::Int(103));

        assert_eq!(answers.check(&Answer::Int(107)), Some(CheckResult::TooHigh));
        assert_eq!(answers.check(&Answer::Int(103)), Some(CheckResult::TooHigh));
        assert_eq!(answers.check(&Answer::Int(100)), None);
        assert_eq!(answers.check(&Answer::Int(98)), None);
        assert_eq!(answers.check(&Answer::Int(96)), Some(CheckResult::TooLow));
        assert_eq!(answers.check(&Answer::Int(-5)), Some(CheckResult::TooLow));
    }

    #[test]
    fn check_answer_bounds_checked_if_int_or_int_str() {
        let mut answers = Answers::new();

        answers.set_low_bounds(Answer::Int(-50));
        answers.set_high_bounds(Answer::Int(25));
        answers.add_wrong_answer(Answer::Int(-9));
        answers.add_wrong_answer(Answer::Int(1));
        answers.add_wrong_answer(Answer::from_str("xyz").unwrap());

        assert_eq!(
            answers.check(&Answer::from_str("55").unwrap()),
            Some(CheckResult::TooHigh)
        );
        assert_eq!(answers.check(&Answer::Int(55)), Some(CheckResult::TooHigh));

        assert_eq!(answers.check(&Answer::Int(10)), None);
        assert_eq!(answers.check(&Answer::from_str("10").unwrap()), None);

        assert_eq!(
            answers.check(&Answer::from_str("-74").unwrap()),
            Some(CheckResult::TooLow)
        );
        assert_eq!(answers.check(&Answer::Int(-74)), Some(CheckResult::TooLow));
    }

    #[test]
    fn wrong_answers_if_in_bounds() {
        let mut answers = Answers::new();

        answers.set_low_bounds(Answer::Int(-50));
        answers.set_high_bounds(Answer::Int(25));
        answers.add_wrong_answer(Answer::Int(-9));
        answers.add_wrong_answer(Answer::Int(1));
        answers.add_wrong_answer(Answer::Int(100));
        answers.add_wrong_answer(Answer::Int(-100));
        answers.add_wrong_answer(Answer::from_str("xyz").unwrap());

        assert_eq!(
            answers.check(&Answer::from_str("-9").unwrap()),
            Some(CheckResult::Wrong)
        );
        assert_eq!(answers.check(&Answer::Int(-9)), Some(CheckResult::Wrong));
        assert_eq!(
            answers.check(&Answer::from_str("1").unwrap()),
            Some(CheckResult::Wrong)
        );
        assert_eq!(answers.check(&Answer::Int(1)), Some(CheckResult::Wrong));
        assert_eq!(
            answers.check(&Answer::from_str("xyz").unwrap()),
            Some(CheckResult::Wrong)
        );
        assert_eq!(
            answers.check(&Answer::from_str("100").unwrap()),
            Some(CheckResult::TooHigh)
        );
        assert_eq!(answers.check(&Answer::Int(100)), Some(CheckResult::TooHigh));
        assert_eq!(
            answers.check(&Answer::from_str("-100").unwrap()),
            Some(CheckResult::TooLow)
        );
        assert_eq!(answers.check(&Answer::Int(-100)), Some(CheckResult::TooLow));
    }

    #[test]
    fn answers_are_wrong_when_there_is_correct_answer_that_does_not_match() {
        let mut answers = Answers::new();

        answers.set_correct_answer(Answer::from_str("yes").unwrap());

        assert_eq!(
            answers.check(&Answer::from_str("yes").unwrap()),
            Some(CheckResult::Correct)
        );
        assert_eq!(
            answers.check(&Answer::from_str("no").unwrap()),
            Some(CheckResult::Wrong)
        );
        assert_eq!(
            answers.check(&Answer::from_str("maybe").unwrap()),
            Some(CheckResult::Wrong)
        );
    }

    #[test]
    fn serialize_answers_to_text() {
        let text = Answers {
            correct_answer: Some(Answer::Int(12)),
            wrong_answers: vec![
                Answer::Int(-9),
                Answer::Int(1),
                Answer::Int(100),
                Answer::from_str("xyz").unwrap(),
            ],
            low_bounds: Some(-50),
            high_bounds: Some(25),
        }
        .serialize_to_string();

        assert_eq!(text, "= 12\n[ -50\n] 25\nX -9\nX 1\nX 100\nX xyz\n");
    }

    #[test]
    fn serialize_answers_to_text_with_no_correct() {
        let text = Answers {
            correct_answer: None,
            wrong_answers: vec![
                Answer::Int(-9),
                Answer::Int(1),
                Answer::Int(100),
                Answer::from_str("xyz").unwrap(),
            ],
            low_bounds: Some(-50),
            high_bounds: Some(25),
        }
        .serialize_to_string();

        assert_eq!(text, "[ -50\n] 25\nX -9\nX 1\nX 100\nX xyz\n");
    }

    #[test]
    fn serialize_answers_to_text_with_missing_correct_and_high() {
        let text = Answers {
            correct_answer: None,
            wrong_answers: vec![
                Answer::Int(-9),
                Answer::Int(1),
                Answer::Int(100),
                Answer::from_str("xyz").unwrap(),
            ],
            low_bounds: Some(-50),
            high_bounds: None,
        }
        .serialize_to_string();

        assert_eq!(text, "[ -50\nX -9\nX 1\nX 100\nX xyz\n");
    }

    #[test]
    fn deserialize_answers_from_text() {
        let answers = Answers::deserialize_from_str("= 12\n[ -50\n] 25\nX -9\nX 1\nX 100\nX xyz\n");
        assert_eq!(
            answers,
            Answers {
                correct_answer: Some(Answer::Int(12)),
                wrong_answers: vec![
                    Answer::Int(-9),
                    Answer::Int(1),
                    Answer::Int(100),
                    Answer::from_str("xyz").unwrap(),
                ],
                low_bounds: Some(-50),
                high_bounds: Some(25),
            }
        );
    }

    #[test]
    fn deserialize_answers_to_text_with_no_correct() {
        let answers = Answers::deserialize_from_str("[ -50\n] 25\nX -9\nX 1\nX 100\nX xyz\n");
        assert_eq!(
            answers,
            Answers {
                correct_answer: None,
                wrong_answers: vec![
                    Answer::Int(-9),
                    Answer::Int(1),
                    Answer::Int(100),
                    Answer::from_str("xyz").unwrap(),
                ],
                low_bounds: Some(-50),
                high_bounds: Some(25),
            }
        );
    }

    #[test]
    fn deserialize_answers_to_text_with_missing_correct_and_high() {
        let answers = Answers::deserialize_from_str("[ -50\nX -9\nX 1\nX 100\nX xyz\n");
        assert_eq!(
            answers,
            Answers {
                correct_answer: None,
                wrong_answers: vec![
                    Answer::Int(-9),
                    Answer::Int(1),
                    Answer::Int(100),
                    Answer::from_str("xyz").unwrap(),
                ],
                low_bounds: Some(-50),
                high_bounds: None,
            }
        );
    }

    #[test]
    fn deserialize_answers_with_spaces() {
        let answers = Answers::deserialize_from_str("= hello world\nX foobar\nX one two three\n");
        assert_eq!(
            answers,
            Answers {
                correct_answer: Some(Answer::from_str("hello world").unwrap()),
                wrong_answers: vec![
                    Answer::from_str("foobar").unwrap(),
                    Answer::from_str("one two three").unwrap(),
                ],
                low_bounds: None,
                high_bounds: None,
            }
        );
    }
}
