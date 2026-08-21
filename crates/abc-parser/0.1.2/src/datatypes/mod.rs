//! Structures representing parsed ABC music.

pub mod writer;

#[derive(Debug, PartialEq)]
pub struct TuneBook {
    header: Option<FileHeader>,
    tunes: Vec<Tune>
}
impl TuneBook {
    pub fn new(header: Option<FileHeader>, tunes: Vec<Tune>) -> TuneBook {
        TuneBook { header, tunes }
    }
}

#[derive(Debug, PartialEq)]
pub struct FileHeader {
    info: Vec<InfoField>
}
impl FileHeader {
    pub fn new(info: Vec<InfoField>) -> FileHeader {
        FileHeader { info }
    }
}

#[derive(Debug, PartialEq)]
pub struct Tune {
    header: TuneHeader,
    body: Option<TuneBody>
}
impl Tune {
    pub fn new(header: TuneHeader, body: Option<TuneBody>) -> Tune {
        Tune { header, body }
    }
}

#[derive(Debug, PartialEq)]
pub struct TuneHeader {
    info: Vec<InfoField>
}
impl TuneHeader {
    pub fn new(info: Vec<InfoField>) -> TuneHeader {
        TuneHeader { info }
    }
}

#[derive(Debug, PartialEq)]
pub struct InfoField(char, String);
impl InfoField {
    pub fn new(c: char, s: String) -> InfoField {
        InfoField(c, s)
    }
}

#[derive(Debug, PartialEq)]
pub struct TuneBody {
    music: Vec<MusicLine>
}
impl TuneBody {
    pub fn new(music: Vec<MusicLine>) -> TuneBody {
        TuneBody { music }
    }
}

#[derive(Debug, PartialEq)]
pub struct MusicLine {
    symbols: Vec<MusicSymbol>
}
impl MusicLine {
    pub fn new(symbols: Vec<MusicSymbol>) -> MusicLine {
        MusicLine { symbols }
    }
}

#[derive(Debug, PartialEq)]
pub enum MusicSymbol {
    Note {
        decoration: Option<Decoration>,
        accidental: Option<Accidental>,
        note: char,
        octave: i8,
        length: f32
    },
    Bar(String),
    Rest(Rest),
    Ending(u32),
    VisualBreak()
}

impl MusicSymbol {
    pub fn new_note(
        decoration: Option<Decoration>,
        accidental: Option<Accidental>,
        note: char,
        octave: i8,
        length: f32) -> MusicSymbol {
        MusicSymbol::Note { decoration, accidental, note, octave, length }
    }

    pub fn note(note: char) -> MusicSymbol {
        MusicSymbol::Note {
            decoration: None,
            accidental: None,
            note,
            octave: 1,
            length: 1.0
        }
    }

    pub fn note_from_length(note: char, length: f32) -> MusicSymbol {
        MusicSymbol::Note {
            decoration: None,
            accidental: None,
            note,
            octave: 1,
            length
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Decoration {
    Staccato(),
    Roll(),
    Fermata(),
    Accent(),
    LowerMordent(),
    Coda(),
    UpperMordent(),
    Segno(),
    Trill(),
    UpBow(),
    DownBow(),
    Unresolved(String)
}

#[derive(Debug, PartialEq)]
pub enum Accidental {
    Natural(),
    Sharp(),
    Flat(),
    DoubleSharp(),
    DoubleFlat()
}

#[derive(Debug, PartialEq)]
pub enum Rest {
    Note(u32),
    Measure(u32),
    NoteHidden(u32),
    MeasureHidden(u32)
}
