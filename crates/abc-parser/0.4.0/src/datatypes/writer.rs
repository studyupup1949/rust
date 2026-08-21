//! Turns datatype representations of ABC back into text.

use super::*;

pub trait ToABC {
    // Using the &mut writer trick causes a recursion error in the compiler:
    // https://github.com/rust-lang/rust/issues/37748#issuecomment-1368984423
    //
    // We work around it by using a dedicated function
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result;

    fn write_abc(&self, mut writer: impl std::fmt::Write) -> std::fmt::Result {
        self.write_mut_abc(&mut writer)
    }

    /// Creates a valid ABC text representation of the object.
    fn to_abc(&self) -> String {
        let mut s = String::new();
        self.write_abc(&mut s).unwrap();
        s
    }
}

impl<T: ToABC> ToABC for Option<T> {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self {
            Some(h) => h.write_mut_abc(writer),
            None => Ok(()),
        }
    }
}

impl<T: ToABC> ToABC for Vec<T> {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        self.iter().try_for_each(|i| i.write_mut_abc(writer))
    }
}

impl ToABC for Length {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        #[allow(clippy::redundant_guards)]
        match self.0 {
            l if l < 1.0 && l > 0.0 => {
                let inv = 1.0 / l;
                let num_halves = inv.log2();
                if num_halves.fract() == 0.0 {
                    (0..(num_halves as u64)).try_for_each(|_| writer.write_str("/"))
                } else {
                    write!(writer, "/{}", inv as usize)
                }
            }
            l if l >= 0.0 => write!(writer, "{}", fraction::Fraction::from(l)),
            _ => panic!("Note lengths must be greater than 0!"),
        }
    }
}

impl ToABC for TuneBook {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        self.comments.iter().try_for_each(|c| {
            c.write_mut_abc(writer)?;
            writer.write_str("\n")
        })?;

        if let Some(ref header) = self.header {
            header.write_mut_abc(writer)?;
            writer.write_str("\n")?;
        }

        self.tunes.iter().try_for_each(|t| {
            t.write_mut_abc(writer)?;
            writer.write_str("\n")
        })
    }
}

impl ToABC for IgnoredLine {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self {
            IgnoredLine::Comment(comment) => comment.write_mut_abc(writer),
            IgnoredLine::EmptyLine => Ok(()),
        }
    }
}

impl ToABC for FileHeader {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        join_header_lines(&self.lines, writer)
    }
}

impl ToABC for InfoField {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(writer, "{}:{}", self.0, self.1)
    }
}

impl ToABC for Tune {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        self.header.write_mut_abc(writer)?;
        self.body.write_mut_abc(writer)
    }
}

impl ToABC for TuneHeader {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        join_header_lines(&self.lines, writer)
    }
}

impl ToABC for HeaderLine {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self {
            HeaderLine::Field(field, comment) => {
                field.write_mut_abc(writer)?;
                comment.write_mut_abc(writer)
            }
            HeaderLine::Comment(comment) => comment.write_mut_abc(writer),
        }
    }
}

fn join_header_lines(lines: &[HeaderLine], writer: &mut impl std::fmt::Write) -> std::fmt::Result {
    for line in lines {
        line.write_mut_abc(writer)?;
        writer.write_str("\n")?;
    }

    Ok(())
}

impl ToABC for TuneBody {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (i, line) in self.lines.iter().enumerate() {
            line.write_mut_abc(writer)?;
            if i + 1 != self.lines.len() {
                writer.write_str("\n")?;
            }
        }

        Ok(())
    }
}

impl ToABC for TuneLine {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self {
            TuneLine::Comment(comment) => comment.write_mut_abc(writer),
            TuneLine::Music(line) => line.write_mut_abc(writer),
            TuneLine::Symbol(line) => line.write_mut_abc(writer),
            TuneLine::Lyric(line) => line.write_mut_abc(writer),
        }
    }
}

impl ToABC for MusicLine {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        self.symbols
            .iter()
            .try_for_each(|symbol| symbol.write_mut_abc(writer))
    }
}

impl ToABC for SymbolLine {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        writer.write_str("s:")?;
        self.symbols
            .iter()
            .try_for_each(|symbol| symbol.write_mut_abc(writer))
    }
}

impl ToABC for LyricLine {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        writer.write_str("w:")?;
        self.symbols
            .iter()
            .try_for_each(|symbol| symbol.write_mut_abc(writer))
    }
}

impl ToABC for MusicSymbol {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        use super::MusicSymbol::*;
        match self {
            Note {
                accidental,
                note,
                octave,
                length,
                tie,
            } => {
                let (note, octave) = denormalise_octave(*note, *octave);
                accidental.write_mut_abc(writer)?;
                writer.write_char(note)?;
                write_octave(octave, writer)?;
                length.write_mut_abc(writer)?;
                tie.write_mut_abc(writer)
            }
            GraceNotes {
                acciaccatura,
                notes,
            } => {
                write!(writer, "{{{}", acciaccatura.map_or("", |_| "/"))?;
                notes.write_mut_abc(writer)?;
                writer.write_str("}")
            }
            Chord { notes, length, tie } => {
                writer.write_str("[")?;
                notes.write_mut_abc(writer)?;
                writer.write_str("]")?;
                length.write_mut_abc(writer)?;
                tie.write_mut_abc(writer)
            }
            Tuplet { p, q, r } => match (q, r) {
                (None, None) => write!(writer, "({}", p),
                (Some(q), None) => write!(writer, "({}:{}", p, q),
                (None, Some(r)) => write!(writer, "({}::{}", p, r),
                (Some(q), Some(r)) => write!(writer, "({}:{}:{}", p, q, r),
            },
            BrokenRhythm {
                rhythm,
                before,
                after,
            } => {
                before.write_mut_abc(writer)?;
                writer.write_str(rhythm)?;
                after.write_mut_abc(writer)
            }
            Decoration(decoration) => decoration.write_mut_abc(writer),
            Annotation(annotation) => annotation.write_mut_abc(writer),
            Bar(bar, ending) => {
                writer.write_str(bar)?;
                if let Some(ending) = ending {
                    writer.write_str(ending)?;
                }
                Ok(())
            }
            Beam(beam) => writer.write_str(beam),
            Slur(slur) => slur.write_mut_abc(writer),
            Comment(comment) => comment.write_mut_abc(writer),
            Rest(rest) => rest.write_mut_abc(writer),
            Spacer => writer.write_str("y"),
            Ending(n) => write!(writer, "[{}", n),
            InlineField(info_field, bracketed) => {
                if *bracketed {
                    writer.write_str("[")?;
                }
                info_field.write_mut_abc(writer)?;
                if *bracketed {
                    writer.write_str("]")?;
                }
                Ok(())
            }
            Space(s) => writer.write_str(s),
            Reserved(s) => writer.write_str(s),
        }
    }
}

impl ToABC for SymbolLineSymbol {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        use super::SymbolLineSymbol::*;
        match self {
            Annotation(annotation) => annotation.write_mut_abc(writer),
            Decoration(decoration) => decoration.write_mut_abc(writer),
            SymbolAlignment(alignment) => alignment.write_mut_abc(writer),
            Space(s) => writer.write_str(s),
        }
    }
}

impl ToABC for LyricSymbol {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        use super::LyricSymbol::*;
        match self {
            Syllable(syllable) => writer.write_str(syllable),
            SymbolAlignment(alignment) => alignment.write_mut_abc(writer),
            Space(s) => writer.write_str(s),
        }
    }
}

impl ToABC for Annotation {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        writer.write_char('"')?;
        self.placement.write_mut_abc(writer)?;
        write!(writer, "{}\"", self.text)
    }
}

impl ToABC for Placement {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        use super::Placement::*;
        writer.write_str(match self {
            Above => "^",
            Below => "_",
            Left => "<",
            Right => ">",
            Auto => "@",
        })
    }
}

impl ToABC for Tie {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        use super::Tie::*;
        writer.write_str(match self {
            Solid => "-",
            Dotted => ".-",
        })
    }
}

impl ToABC for Slur {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        use super::Slur::*;
        writer.write_str(match self {
            Begin => "(",
            BeginDotted => ".(",
            End => ")",
        })
    }
}

/// Given a note and an octave, returns the uppercase or lowercase form of the note which makes the
/// octave closest to 1, and the octave adjusted accordingly.
fn denormalise_octave(note: Note, octave: i8) -> (char, i8) {
    let note_char: char = note.into();
    if octave > 1 {
        (note_char.to_ascii_lowercase(), octave - 1)
    } else {
        (note_char, octave)
    }
}

impl ToABC for Decoration {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        use super::Decoration::*;
        match self {
            Staccato => writer.write_str("."),
            Roll => writer.write_str("~"),
            Fermata => writer.write_str("H"),
            Accent => writer.write_str("L"),
            LowerMordent => writer.write_str("M"),
            Coda => writer.write_str("O"),
            UpperMordent => writer.write_str("P"),
            Segno => writer.write_str("S"),
            Trill => writer.write_str("T"),
            UpBow => writer.write_str("u"),
            DownBow => writer.write_str("v"),
            Unresolved(s) => write!(writer, "!{}!", s),
        }
    }
}

impl ToABC for Accidental {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        use super::Accidental::*;
        writer.write_str(match self {
            Natural => "=",
            Sharp => "^",
            Flat => "_",
            DoubleSharp => "^^",
            DoubleFlat => "__",
        })
    }
}

impl ToABC for SymbolAlignment {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        use super::SymbolAlignment::*;
        writer.write_str(match self {
            Break => "-",
            Extend => "_",
            Skip => "*",
            Space => "~",
            Hyphen => "\\-",
            Bar => "|",
        })
    }
}

fn write_octave(octave: i8, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
    match octave {
        o if o > 1 => (0..(octave - 1) as usize).try_for_each(|_| write!(writer, "'")),
        o if o < 1 => (0..(-octave + 1) as usize).try_for_each(|_| write!(writer, ",")),
        _ => Ok(()),
    }
}

impl ToABC for Rest {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        use super::Rest::*;
        match self {
            Note(l) => {
                writer.write_str("z")?;
                l.write_mut_abc(writer)
            }
            Measure(l) => {
                writer.write_str("Z")?;
                l.write_mut_abc(writer)
            }
            NoteHidden(l) => {
                writer.write_str("x")?;
                l.write_mut_abc(writer)
            }
            MeasureHidden(l) => {
                writer.write_str("X")?;
                l.write_mut_abc(writer)
            }
        }
    }
}

impl ToABC for Comment {
    fn write_mut_abc(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        use super::Comment::*;
        match self {
            Comment(comment) => write!(writer, "%{comment}"),
            CommentLine(space, comment) => write!(writer, "{space}%{comment}"),
            StylesheetDirective(directive) => write!(writer, "%%{directive}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::datatypes::builder::NoteBuilder;

    #[test]
    fn info_field() {
        let f = InfoField('X', "1".to_string());
        let s = f.to_abc();
        assert_eq!(s, "X:1")
    }

    #[test]
    fn file_header() {
        let h = FileHeader {
            lines: vec![
                HeaderLine::Field(InfoField('X', "1".to_string()), None),
                HeaderLine::Field(InfoField('T', "Untitled".to_string()), None),
                HeaderLine::Field(InfoField('K', "G".to_string()), None),
            ],
        };
        let s = h.to_abc();
        assert_eq!(s, "X:1\nT:Untitled\nK:G\n")
    }

    #[test]
    fn file_header_with_comments() {
        let h = FileHeader {
            lines: vec![
                HeaderLine::Field(InfoField('X', "1".to_string()), None),
                HeaderLine::Field(
                    InfoField('T', "Untitled   ".to_string()),
                    Some(Comment::Comment(" Inline comment".to_string())),
                ),
                HeaderLine::Field(InfoField('K', "G".to_string()), None),
            ],
        };
        let s = h.to_abc();
        assert_eq!(s, "X:1\nT:Untitled   % Inline comment\nK:G\n")
    }

    #[test]
    fn file_header_with_stylesheet_directive() {
        let h = FileHeader {
            lines: vec![
                HeaderLine::Field(InfoField('X', "1".to_string()), None),
                HeaderLine::Field(InfoField('T', "Untitled".to_string()), None),
                HeaderLine::Comment(Comment::StylesheetDirective("papersize A4".to_string())),
                HeaderLine::Field(InfoField('K', "G".to_string()), None),
            ],
        };
        let s = h.to_abc();
        assert_eq!(s, "X:1\nT:Untitled\n%%papersize A4\nK:G\n")
    }

    #[test]
    fn accidental_1() {
        assert_eq!(Accidental::Natural.to_abc(), "=")
    }
    #[test]
    fn accidental_2() {
        assert_eq!(Accidental::Sharp.to_abc(), "^")
    }
    #[test]
    fn accidental_3() {
        assert_eq!(Accidental::Flat.to_abc(), "_")
    }
    #[test]
    fn accidental_4() {
        assert_eq!(Accidental::DoubleSharp.to_abc(), "^^")
    }
    #[test]
    fn accidental_5() {
        assert_eq!(Accidental::DoubleFlat.to_abc(), "__")
    }

    #[test]
    fn note_with_accidental() {
        assert_eq!(
            NoteBuilder::new(Note::C)
                .accidental(Accidental::Sharp)
                .build()
                .to_abc(),
            "^C"
        );
    }

    #[test]
    fn decoration_1() {
        assert_eq!(Decoration::Staccato.to_abc(), ".")
    }
    #[test]
    fn decoration_2() {
        assert_eq!(Decoration::Roll.to_abc(), "~")
    }
    #[test]
    fn decoration_3() {
        assert_eq!(Decoration::Fermata.to_abc(), "H")
    }
    #[test]
    fn decoration_4() {
        assert_eq!(Decoration::Accent.to_abc(), "L")
    }
    #[test]
    fn decoration_5() {
        assert_eq!(Decoration::LowerMordent.to_abc(), "M")
    }
    #[test]
    fn decoration_6() {
        assert_eq!(Decoration::Coda.to_abc(), "O")
    }
    #[test]
    fn decoration_7() {
        assert_eq!(Decoration::UpperMordent.to_abc(), "P")
    }
    #[test]
    fn decoration_8() {
        assert_eq!(Decoration::Segno.to_abc(), "S")
    }
    #[test]
    fn decoration_9() {
        assert_eq!(Decoration::Trill.to_abc(), "T")
    }
    #[test]
    fn decoration_10() {
        assert_eq!(Decoration::UpBow.to_abc(), "u")
    }
    #[test]
    fn decoration_11() {
        assert_eq!(Decoration::DownBow.to_abc(), "v")
    }
    #[test]
    fn decoration_12() {
        assert_eq!(
            Decoration::Unresolved("asdf".to_string()).to_abc(),
            "!asdf!"
        )
    }

    #[test]
    fn rest_1() {
        assert_eq!(Rest::Note(None).to_abc(), "z")
    }
    #[test]
    fn rest_2() {
        assert_eq!(Rest::Note(Some(Length::new(2.0))).to_abc(), "z2")
    }
    #[test]
    fn rest_3() {
        assert_eq!(Rest::Measure(None).to_abc(), "Z")
    }
    #[test]
    fn rest_4() {
        assert_eq!(Rest::Measure(Some(Length::new(2.0))).to_abc(), "Z2")
    }
    #[test]
    fn rest_5() {
        assert_eq!(Rest::NoteHidden(None).to_abc(), "x")
    }
    #[test]
    fn rest_6() {
        assert_eq!(Rest::NoteHidden(Some(Length::new(4.0))).to_abc(), "x4")
    }
    #[test]
    fn rest_7() {
        assert_eq!(Rest::MeasureHidden(None).to_abc(), "X")
    }
    #[test]
    fn rest_8() {
        assert_eq!(Rest::MeasureHidden(Some(Length::new(3.0))).to_abc(), "X3")
    }

    #[test]
    fn spacer() {
        assert_eq!(MusicSymbol::Spacer.to_abc(), "y")
    }

    #[test]
    fn ending_1() {
        assert_eq!(MusicSymbol::Ending("1".to_string()).to_abc(), "[1")
    }
    #[test]
    fn ending_2() {
        assert_eq!(MusicSymbol::Ending("2".to_string()).to_abc(), "[2")
    }
    #[test]
    fn ending_3() {
        assert_eq!(
            MusicSymbol::Ending("1,3,5-7,8-10".to_string()).to_abc(),
            "[1,3,5-7,8-10"
        )
    }

    #[test]
    fn bar_ending() {
        assert_eq!(
            MusicSymbol::Bar("|".to_string(), Some("2".to_string())).to_abc(),
            "|2"
        )
    }

    #[test]
    fn octave_1() {
        assert_eq!(NoteBuilder::new(Note::C).octave(2).build().to_abc(), "c");
    }
    #[test]
    fn octave_2() {
        assert_eq!(NoteBuilder::new(Note::C).octave(3).build().to_abc(), "c'");
    }
    #[test]
    fn octave_3() {
        assert_eq!(NoteBuilder::new(Note::C).octave(4).build().to_abc(), "c''");
    }
    #[test]
    fn octave_4() {
        assert_eq!(NoteBuilder::new(Note::C).octave(0).build().to_abc(), "C,");
    }
    #[test]
    fn octave_5() {
        assert_eq!(NoteBuilder::new(Note::C).octave(-1).build().to_abc(), "C,,");
    }

    #[test]
    fn length_1() {
        assert_eq!(NoteBuilder::new(Note::C).build().to_abc(), "C");
    }
    #[test]
    fn length_2() {
        assert_eq!(NoteBuilder::new(Note::C).length(2.0).build().to_abc(), "C2");
    }
    #[test]
    fn length_3() {
        assert_eq!(NoteBuilder::new(Note::C).length(0.5).build().to_abc(), "C/");
    }
    #[test]
    fn length_4() {
        assert_eq!(NoteBuilder::new(Note::C).length(1.0).build().to_abc(), "C1");
    }
    #[test]
    fn length_5() {
        assert_eq!(
            NoteBuilder::new(Note::C).length(1.5).build().to_abc(),
            "C3/2"
        );
    }
    #[test]
    fn length_6() {
        assert_eq!(
            NoteBuilder::new(Note::C).length(0.125).build().to_abc(),
            "C///"
        );
    }
    #[test]
    fn length_7() {
        assert_eq!(
            NoteBuilder::new(Note::C).length(1.0 / 3.0).build().to_abc(),
            "C/3"
        );
    }
    #[test]
    fn length_8() {
        assert_eq!(
            NoteBuilder::new(Note::C).length(0.2).build().to_abc(),
            "C/5"
        );
    }
    #[ignore]
    #[test]
    fn length_9() {
        assert_eq!(
            NoteBuilder::new(Note::C).length(2.0 / 3.0).build().to_abc(),
            "C2/3"
        );
    }

    #[test]
    fn tie() {
        assert_eq!(
            NoteBuilder::new(Note::C).tie(Tie::Solid).build().to_abc(),
            "C-"
        );
    }

    #[test]
    fn slur_1() {
        assert_eq!(MusicSymbol::Slur(Slur::Begin).to_abc(), "(");
    }
    #[test]
    fn slur_2() {
        assert_eq!(MusicSymbol::Slur(Slur::BeginDotted).to_abc(), ".(");
    }
    #[test]
    fn slur_3() {
        assert_eq!(MusicSymbol::Slur(Slur::End).to_abc(), ")");
    }

    #[test]
    fn beam() {
        assert_eq!(MusicSymbol::Beam("````".to_string()).to_abc(), "````")
    }

    #[test]
    fn grace_notes_1() {
        assert_eq!(
            MusicSymbol::GraceNotes {
                acciaccatura: None,
                notes: vec![
                    NoteBuilder::new(Note::A)
                        .accidental(Accidental::DoubleSharp)
                        .build(),
                    NoteBuilder::new(Note::B).build(),
                    NoteBuilder::new(Note::C)
                        .accidental(Accidental::Flat)
                        .octave(2)
                        .length(2.0)
                        .build(),
                ]
            }
            .to_abc(),
            "{^^AB_c2}"
        );
    }

    #[test]
    fn grace_notes_2() {
        assert_eq!(
            MusicSymbol::GraceNotes {
                acciaccatura: Some(()),
                notes: vec![
                    NoteBuilder::new(Note::A)
                        .accidental(Accidental::DoubleSharp)
                        .build()
                ]
            }
            .to_abc(),
            "{/^^A}"
        );
    }

    #[test]
    fn tuplets_1() {
        assert_eq!(
            MusicSymbol::Tuplet {
                p: 3,
                q: None,
                r: None,
            }
            .to_abc(),
            "(3"
        );
    }

    #[test]
    fn tuplets_2() {
        assert_eq!(
            MusicSymbol::Tuplet {
                p: 3,
                q: Some(2),
                r: None,
            }
            .to_abc(),
            "(3:2"
        );
    }

    #[test]
    fn tuplets_3() {
        assert_eq!(
            MusicSymbol::Tuplet {
                p: 3,
                q: None,
                r: Some(2),
            }
            .to_abc(),
            "(3::2"
        );
    }

    #[test]
    fn broken_rhythm_1() {
        assert_eq!(
            MusicSymbol::BrokenRhythm {
                rhythm: "<".to_string(),
                before: vec![NoteBuilder::new(Note::A).build()],
                after: vec![NoteBuilder::new(Note::B).build()]
            }
            .to_abc(),
            "A<B"
        );
    }

    #[test]
    fn broken_rhythm_2() {
        assert_eq!(
            MusicSymbol::BrokenRhythm {
                rhythm: ">".to_string(),
                before: vec![NoteBuilder::new(Note::A).build()],
                after: vec![NoteBuilder::new(Note::B).build()]
            }
            .to_abc(),
            "A>B"
        );
    }

    #[test]
    fn broken_rhythm_3() {
        assert_eq!(
            MusicSymbol::BrokenRhythm {
                rhythm: "<<".to_string(),
                before: vec![NoteBuilder::new(Note::A).build()],
                after: vec![NoteBuilder::new(Note::B).build()]
            }
            .to_abc(),
            "A<<B"
        );
    }

    #[test]
    fn broken_rhythm_4() {
        assert_eq!(
            MusicSymbol::BrokenRhythm {
                rhythm: ">>>".to_string(),
                before: vec![NoteBuilder::new(Note::A).build()],
                after: vec![NoteBuilder::new(Note::B).build()]
            }
            .to_abc(),
            "A>>>B"
        );
    }

    #[test]
    fn broken_rhythm_5() {
        assert_eq!(
            MusicSymbol::BrokenRhythm {
                rhythm: ">".to_string(),
                before: vec![
                    NoteBuilder::new(Note::A).build(),
                    MusicSymbol::Space(" ".to_string()),
                ],
                after: vec![NoteBuilder::new(Note::B).build()]
            }
            .to_abc(),
            "A >B"
        );
    }

    #[test]
    fn chord_1() {
        assert_eq!(
            MusicSymbol::Chord {
                notes: vec![
                    NoteBuilder::new(Note::C).build(),
                    NoteBuilder::new(Note::E).build(),
                    NoteBuilder::new(Note::G).build(),
                    NoteBuilder::new(Note::C).octave(2).build(),
                ],
                length: None,
                tie: None,
            }
            .to_abc(),
            "[CEGc]"
        );
    }

    #[test]
    fn chord_2() {
        assert_eq!(
            MusicSymbol::Chord {
                notes: vec![
                    NoteBuilder::new(Note::C).build(),
                    NoteBuilder::new(Note::E).length(2.0).build(),
                    NoteBuilder::new(Note::G)
                        .accidental(Accidental::Sharp)
                        .build(),
                    NoteBuilder::new(Note::C).octave(2).build(),
                ],
                length: Some(Length::new(3.0)),
                tie: Some(Tie::Solid),
            }
            .to_abc(),
            "[CE2^Gc]3-"
        );
    }

    #[test]
    fn annotation_1() {
        assert_eq!(
            Annotation::new(Some(Placement::Above), "foobar".to_string()).to_abc(),
            "\"^foobar\""
        );
    }
    #[test]
    fn annotation_2() {
        assert_eq!(
            Annotation::new(Some(Placement::Below), "foobar".to_string()).to_abc(),
            "\"_foobar\""
        );
    }
    #[test]
    fn annotation_3() {
        assert_eq!(
            Annotation::new(Some(Placement::Left), "foobar".to_string()).to_abc(),
            "\"<foobar\""
        );
    }
    #[test]
    fn annotation_4() {
        assert_eq!(
            Annotation::new(Some(Placement::Right), "foobar".to_string()).to_abc(),
            "\">foobar\""
        );
    }
    #[test]
    fn annotation_5() {
        assert_eq!(
            Annotation::new(Some(Placement::Auto), "foobar".to_string()).to_abc(),
            "\"@foobar\""
        );
    }
    #[test]
    fn annotation_6() {
        assert_eq!(
            Annotation::new(None, "foobar".to_string()).to_abc(),
            "\"foobar\""
        );
    }
}
