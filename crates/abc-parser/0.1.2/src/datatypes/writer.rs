//! Turns datatype representations of ABC back into text.

use super::*;

pub trait ToABC {
    /// Creates a valid ABC text representation of the object.
    fn to_abc(&self) -> String;
}

impl<T> ToABC for Option<T>
where T: ToABC {
    fn to_abc(&self) -> String {
        match self {
            Some(ref h) => format!("{}\n", h.to_abc()),
            None => String::new()
        }
    }
}

impl ToABC for TuneBook {
    fn to_abc(&self) -> String {
        let tunes: Vec<String> = self.tunes.iter().map(
            |tune| tune.to_abc()
        ).collect();
        format!("{}{}", self.header.to_abc(), tunes.join("\n"))
    }
}

impl ToABC for FileHeader {
    fn to_abc(&self) -> String {
        let s = String::new();
        let s = self.info.iter().fold(s,
            |s, field| format!("{}{}\n", s, field.to_abc())
        );
        s
    }
}

impl ToABC for InfoField {
    fn to_abc(&self) -> String {
        format!("{}:{}", self.0, self.1)
    }
}

impl ToABC for Tune {
    fn to_abc(&self) -> String {
        format!("{}{}", self.header.to_abc(), self.body.to_abc())
    }
}

impl ToABC for TuneHeader {
    fn to_abc(&self) -> String {
        let s = self.info.iter().fold(String::new(),
            |s, field| format!("{}{}\n", s, field.to_abc())
        );
        s
    }
}

impl ToABC for TuneBody {
    fn to_abc(&self) -> String {
        let lines: Vec<String> = self.music.iter().map(
            |line| line.to_abc()
        ).collect();
        lines.join("\n")
    }
}

impl ToABC for MusicLine {
    fn to_abc(&self) -> String {
        let mut s = String::new();
        self.symbols.iter().map(
            |symbol| s.push_str(&symbol.to_abc())
        ).count();
        s
    }
}

impl ToABC for MusicSymbol {
    fn to_abc(&self) -> String {
        use super::MusicSymbol::*;
        match self {
            VisualBreak() => String::from(" "),
            Note {
                decoration: d,
                accidental: a,
                note: n,
                octave: o,
                length: l
            } => format!("{}{}{}{}{}", d.to_abc(), a.to_abc(), n, octave_to_abc(*o), length_to_abc(*l)),
            Bar(bar) => bar.clone(),
            _ => "?".to_string()
        }
    }
}

impl ToABC for Decoration {
    fn to_abc(&self) -> String {
        use super::Decoration::*;
        match self {
            Staccato() => String::from("."),
            Roll() => String::from("~"),
            Fermata() => String::from("H"),
            Accent() => String::from("L"),
            LowerMordent() => String::from("M"),
            Coda() => String::from("O"),
            UpperMordent() => String::from("P"),
            Segno() => String::from("S"),
            Trill() => String::from("T"),
            UpBow() => String::from("u"),
            DownBow() => String::from("v"),
            Unresolved(s) => format!("!{}!", s)
        }
    }
}

impl ToABC for Accidental {
    fn to_abc(&self) -> String {
        use super::Accidental::*;
        String::from(
            match self {
                Natural() => "=",
                Sharp() => "^",
                Flat() => "_",
                DoubleSharp() => "^^",
                DoubleFlat() => "__"
            }
        )
    }
}

fn octave_to_abc(octave: i8) -> String {
    match octave {
        1 => String::new(),
        o if o > 1 => "'".repeat(octave as usize),
        o if o < 1 => ",".repeat((-octave + 1) as usize),
        _ => panic!("All patterns covered! How did we get here?")
    }
}

fn length_to_abc(length: f32) -> String {
    match length {
        l if l == 1f32 => String::new(),
        l if l > 1f32 => (length as usize).to_string(),
        l if l < 1f32 && l > 0f32 => format!("/{}", l.log2() as usize),
        _ => panic!("Note lengths can't be negative!")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn info_field() {
        let f = InfoField('X', "1".to_string());
        let s = f.to_abc();
        assert_eq!(s, "X:1")
    }

    #[test] fn file_header() {
        let h = FileHeader {
            info: vec![
                InfoField('X', "1".to_string()),
                InfoField('T', "Untitled".to_string()),
                InfoField('K', "G".to_string()),
            ]
        };
        let s = h.to_abc();
        assert_eq!(s, "X:1\nT:Untitled\nK:G\n")
    }

    #[test] fn accidental_1() { assert_eq!(Accidental::Natural().to_abc(), "=") }
    #[test] fn accidental_2() { assert_eq!(Accidental::Sharp().to_abc(), "^") }
    #[test] fn accidental_3() { assert_eq!(Accidental::Flat().to_abc(), "_") }
    #[test] fn accidental_4() { assert_eq!(Accidental::DoubleSharp().to_abc(), "^^") }
    #[test] fn accidental_5() { assert_eq!(Accidental::DoubleFlat().to_abc(), "__") }


    #[test] fn decoration_1() { assert_eq!(Decoration::Staccato().to_abc(), ".") }
    #[test] fn decoration_2() { assert_eq!(Decoration::Roll().to_abc(), "~") }
    #[test] fn decoration_3() { assert_eq!(Decoration::Fermata().to_abc(), "H") }
    #[test] fn decoration_4() { assert_eq!(Decoration::Accent().to_abc(), "L") }
    #[test] fn decoration_5() { assert_eq!(Decoration::LowerMordent().to_abc(), "M") }
    #[test] fn decoration_6() { assert_eq!(Decoration::Coda().to_abc(), "O") }
    #[test] fn decoration_7() { assert_eq!(Decoration::UpperMordent().to_abc(), "P") }
    #[test] fn decoration_8() { assert_eq!(Decoration::Segno().to_abc(), "S") }
    #[test] fn decoration_9() { assert_eq!(Decoration::Trill().to_abc(), "T") }
    #[test] fn decoration_10() { assert_eq!(Decoration::UpBow().to_abc(), "u") }
    #[test] fn decoration_11() { assert_eq!(Decoration::DownBow().to_abc(), "v") }
    #[test] fn decoration_12() { assert_eq!(Decoration::Unresolved("asdf".to_string()).to_abc(), "!asdf!") }
}
