//! Structures representing parsed ABC music.

pub mod builder;
pub mod writer;

#[derive(Clone, Debug, PartialEq)]
pub struct TuneBook {
    pub comments: Vec<IgnoredLine>,
    pub header: Option<FileHeader>,
    pub tunes: Vec<Tune>,
}
impl TuneBook {
    pub fn new(
        comments: Vec<IgnoredLine>,
        header: Option<FileHeader>,
        tunes: Vec<Tune>,
    ) -> TuneBook {
        TuneBook {
            comments,
            header,
            tunes,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileHeader {
    pub lines: Vec<HeaderLine>,
}
impl FileHeader {
    pub fn new(lines: Vec<HeaderLine>) -> FileHeader {
        FileHeader { lines }
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
    pub lines: Vec<HeaderLine>,
}
impl TuneHeader {
    pub fn new(lines: Vec<HeaderLine>) -> TuneHeader {
        TuneHeader { lines }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IgnoredLine {
    Comment(Comment),
    EmptyLine,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HeaderLine {
    Field(InfoField, Option<Comment>),
    Comment(Comment),
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
    pub lines: Vec<TuneLine>,
}
impl TuneBody {
    pub fn new(lines: Vec<TuneLine>) -> TuneBody {
        TuneBody { lines }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TuneLine {
    Comment(Comment),
    Music(MusicLine),
    Symbol(SymbolLine),
    Lyric(LyricLine),
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SymbolLine {
    pub symbols: Vec<SymbolLineSymbol>,
}
impl SymbolLine {
    pub fn new(symbols: Vec<SymbolLineSymbol>) -> SymbolLine {
        SymbolLine { symbols }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LyricLine {
    pub symbols: Vec<LyricSymbol>,
}
impl LyricLine {
    pub fn new(symbols: Vec<LyricSymbol>) -> LyricLine {
        LyricLine { symbols }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MusicSymbol {
    Note {
        accidental: Option<Accidental>,
        note: Note,
        octave: i8,
        length: Option<Length>,
        tie: Option<Tie>,
    },
    Chord {
        notes: Vec<MusicSymbol>,
        length: Option<Length>,
        tie: Option<Tie>,
    },
    GraceNotes {
        acciaccatura: Option<()>,
        notes: Vec<MusicSymbol>,
    },
    /// An `r` value of 0 indicates that the parser does not have enough information and it must
    /// be determined from the time signature.
    Tuplet {
        p: u32,
        q: Option<u32>,
        r: Option<u32>,
    },
    BrokenRhythm {
        rhythm: String,
        before: Vec<MusicSymbol>,
        after: Vec<MusicSymbol>,
    },
    Decoration(Decoration),
    Annotation(Annotation),
    Bar(String, Option<String>),
    Beam(String),
    Slur(Slur),
    Comment(Comment),
    Rest(Rest),
    Spacer,
    Ending(String),
    InlineField(InfoField, bool),
    Space(String),
    Reserved(String),
}

impl MusicSymbol {
    pub fn new_note(
        accidental: Option<Accidental>,
        note: Note,
        octave: i8,
        length: Option<Length>,
        tie: Option<Tie>,
    ) -> MusicSymbol {
        MusicSymbol::Note {
            accidental,
            note,
            octave,
            length,
            tie,
        }
    }

    pub fn new_tuplet(p: u32, q: Option<u32>, r: Option<u32>) -> MusicSymbol {
        assert!(p > 1 && p < 10);
        assert_ne!(r, Some(0));

        MusicSymbol::Tuplet { p, q, r }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SymbolLineSymbol {
    Annotation(Annotation),
    Decoration(Decoration),
    SymbolAlignment(SymbolAlignment),
    Space(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LyricSymbol {
    Syllable(String),
    SymbolAlignment(SymbolAlignment),
    Space(String),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SymbolAlignment {
    /// (hyphen) break between syllables within a word
    Break,
    /// (underscore) previous syllable is to be held for an extra note
    Extend,
    /// (star) one note is skipped (i.e. * is equivalent to a blank syllable)
    Skip,
    /// (tilde) appears as a space; aligns multiple words under one note
    Space,
    /// (backslash hyphen) appears as hyphen; aligns multiple syllables under one note
    Hyphen,
    /// (vertical bar) advances to the next bar
    Bar,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Annotation {
    placement: Option<Placement>,
    text: String,
}

impl Annotation {
    pub fn new(placement: Option<Placement>, text: String) -> Annotation {
        Annotation { placement, text }
    }
}

/// The placement of an annotation.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Placement {
    Above,
    Below,
    Left,
    Right,
    Auto,
}

/// A tie which may apply between a note and the following note of the same pitch.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Tie {
    /// A normal tie.
    Solid,
    /// A dotted tie.
    Dotted,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Slur {
    Begin,
    BeginDotted,
    End,
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Rest {
    Note(Option<Length>),
    Measure(Option<Length>),
    NoteHidden(Option<Length>),
    MeasureHidden(Option<Length>),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Length(f32);

impl Length {
    pub fn new(length: f32) -> Length {
        Length(length)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Comment {
    Comment(String),
    CommentLine(String, String),
    StylesheetDirective(String),
}
