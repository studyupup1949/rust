use crate::datatypes::*;

pub struct NoteBuilder {
    note: MusicSymbol,
}

impl NoteBuilder {
    pub fn build(self) -> MusicSymbol {
        self.note
    }

    pub fn new(note: Note) -> NoteBuilder {
        NoteBuilder {
            note: MusicSymbol::Note {
                accidental: None,
                note,
                octave: 1,
                length: None,
                tie: None,
            },
        }
    }

    pub fn accidental(mut self, accidental: Accidental) -> Self {
        match self.note {
            MusicSymbol::Note {
                accidental: _,
                note,
                octave,
                length,
                tie,
            } => {
                self.note = MusicSymbol::Note {
                    accidental: Some(accidental),
                    note,
                    octave,
                    length,
                    tie,
                }
            }
            _ => unreachable!(),
        }

        self
    }

    pub fn octave(mut self, octave: i8) -> Self {
        match self.note {
            MusicSymbol::Note {
                accidental,
                note,
                octave: _,
                length,
                tie,
            } => {
                self.note = MusicSymbol::Note {
                    accidental,
                    note,
                    octave,
                    length,
                    tie,
                }
            }
            _ => unreachable!(),
        }

        self
    }

    pub fn length(mut self, length: f32) -> Self {
        match self.note {
            MusicSymbol::Note {
                accidental,
                note,
                octave,
                length: _,
                tie,
            } => {
                self.note = MusicSymbol::Note {
                    accidental,
                    note,
                    octave,
                    length: Some(Length::new(length)),
                    tie,
                }
            }
            _ => unreachable!(),
        }

        self
    }

    pub fn tie(mut self, tie: Tie) -> Self {
        match self.note {
            MusicSymbol::Note {
                accidental,
                note,
                octave,
                length,
                tie: _,
            } => {
                self.note = MusicSymbol::Note {
                    accidental,
                    note,
                    octave,
                    length,
                    tie: Some(tie),
                }
            }
            _ => unreachable!(),
        }

        self
    }
}
