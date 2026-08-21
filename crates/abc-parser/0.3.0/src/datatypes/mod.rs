//! Structures representing parsed ABC music.

pub mod writer;

#[derive(Clone, Debug, PartialEq)]
pub struct TuneBook {
    pub header: Option<FileHeader>,
    pub tunes: Vec<Tune>,
}
impl TuneBook {
    pub fn new(header: Option<FileHeader>, tunes: Vec<Tune>) -> TuneBook {
        TuneBook { header, tunes }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileHeader {
    pub info: Vec<InfoField>,
}
impl FileHeader {
    pub fn new(info: Vec<InfoField>) -> FileHeader {
        FileHeader { info }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tune {
    pub header: TuneHeader,
    pub body: Option<TuneBody>,
}
impl Tune {
    pub fn new(header: TuneHeader, body: Option<TuneBody>) -> Tune {
        Tune { header, body }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TuneHeader {
    pub info: Vec<InfoField>,
}
impl TuneHeader {
    pub fn new(info: Vec<InfoField>) -> TuneHeader {
        TuneHeader { info }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InfoField(pub char, pub String);
impl InfoField {
    pub fn new(c: char, s: String) -> InfoField {
        InfoField(c, s)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TuneBody {
    pub music: Vec<MusicLine>,
}
impl TuneBody {
    pub fn new(music: Vec<MusicLine>) -> TuneBody {
        TuneBody { music }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MusicLine {
    pub symbols: Vec<MusicSymbol>,
}
impl MusicLine {
    pub fn new(symbols: Vec<MusicSymbol>) -> MusicLine {
        MusicLine { symbols }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MusicSymbol {
    Note {
        decorations: Vec<Decoration>,
        accidental: Option<Accidental>,
        note: Note,
        octave: i8,
        length: f32,
        tie: Option<Tie>,
    },
    Chord {
        decorations: Vec<Decoration>,
        notes: Vec<MusicSymbol>,
        length: f32,
    },
    GraceNotes {
        acciaccatura: Option<()>,
        notes: Vec<MusicSymbol>,
    },
    /// An `r` value of 0 indicates that the parser does not have enough information and it must
    /// be determined from the time signature.
    Tuplet {
        p: u32,
        q: u32,
        r: u32,
        notes: Vec<MusicSymbol>,
    },
    Bar(String),
    Rest(Rest),
    Ending(u32),
    VisualBreak,
}

/// A tie which may apply between a note and the following note of the same pitch.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Tie {
    /// A normal tie.
    Solid,
    /// A dotted tie.
    Dotted,
}

static TUPLET_Q: [u32; 8] = [3, 2, 3, 0, 2, 0, 3, 0];
impl MusicSymbol {
    pub fn new_note(
        decorations: Vec<Decoration>,
        accidental: Option<Accidental>,
        note: Note,
        octave: i8,
        length: f32,
        tie: Option<Tie>,
    ) -> MusicSymbol {
        MusicSymbol::Note {
            decorations,
            accidental,
            note,
            octave,
            length,
            tie,
        }
    }

    pub fn note(note: Note) -> MusicSymbol {
        MusicSymbol::Note {
            decorations: vec![],
            accidental: None,
            note,
            octave: 1,
            length: 1.0,
            tie: None,
        }
    }

    pub fn note_from_length(note: Note, length: f32) -> MusicSymbol {
        MusicSymbol::Note {
            decorations: vec![],
            accidental: None,
            note,
            octave: 1,
            length,
            tie: None,
        }
    }

    /// # Panics
    /// If the number of notes is not equal to `p`
    pub fn tuplet_with_defaults(
        p: u32,
        q: Option<u32>,
        r: Option<u32>,
        notes: Vec<MusicSymbol>,
    ) -> Result<MusicSymbol, &'static str> {
        assert_eq!(notes.len(), p as usize);

        Self::new_tuplet(
            p,
            q.unwrap_or(TUPLET_Q[(p - 2) as usize]),
            r.unwrap_or(p),
            notes,
        )
    }

    pub fn new_tuplet(
        p: u32,
        q: u32,
        r: u32,
        notes: Vec<MusicSymbol>,
    ) -> Result<MusicSymbol, &'static str> {
        if p < 2 {
            return Err("Tuplet too small");
        }
        if p > 9 {
            return Err("Tuplet too large");
        }
        if r == 0 {
            return Err("R can't be 0");
        }

        Ok(MusicSymbol::Tuplet { p, q, r, notes })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Note {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

impl From<Note> for char {
    fn from(note: Note) -> Self {
        match note {
            Note::C => 'C',
            Note::D => 'D',
            Note::E => 'E',
            Note::F => 'F',
            Note::G => 'G',
            Note::A => 'A',
            Note::B => 'B',
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Decoration {
    Staccato,
    Roll,
    Fermata,
    Accent,
    LowerMordent,
    Coda,
    UpperMordent,
    Segno,
    Trill,
    UpBow,
    DownBow,
    Unresolved(String),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Accidental {
    Natural,
    Sharp,
    Flat,
    DoubleSharp,
    DoubleFlat,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Rest {
    Note(u32),
    Measure(u32),
    NoteHidden(u32),
    MeasureHidden(u32),
}
