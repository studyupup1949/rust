use pretty_assertions::assert_eq;

use abc_parser::abc;
use abc_parser::datatypes::builder::NoteBuilder;
use abc_parser::datatypes::*;

#[test]
fn nothing_up_my_sleve() {
    let data = "M:4/4
O:Irish
R:Reel

X:1
T:Untitled Reel
C:Trad.
K:D
eg|a2ab ageg|agbg agef|g2g2 fgag|f2d2 d2:|
ed|cecA B2ed|cAcA E2ed|cecA B2ed|c2A2 A2:|
AB|cdec BcdB|ABAF GFE2|cdec BcdB|c2A2 A2:|";
    let tb = abc::tune_book(data).unwrap();
    assert_eq!(
        tb,
        TuneBook::new(
            vec![],
            Some(FileHeader::new(vec![
                HeaderLine::Field(InfoField::new('M', "4/4".to_string()), None),
                HeaderLine::Field(InfoField::new('O', "Irish".to_string()), None),
                HeaderLine::Field(InfoField::new('R', "Reel".to_string()), None),
            ])),
            vec![Tune::new(
                TuneHeader::new(vec![
                    HeaderLine::Field(InfoField::new('X', "1".to_string()), None),
                    HeaderLine::Field(InfoField::new('T', "Untitled Reel".to_string()), None),
                    HeaderLine::Field(InfoField::new('C', "Trad.".to_string()), None),
                    HeaderLine::Field(InfoField::new('K', "D".to_string()), None),
                ]),
                Some(TuneBody::new(vec![
                    TuneLine::Music(MusicLine::new(vec![
                        NoteBuilder::new(Note::E).octave(2).build(),
                        NoteBuilder::new(Note::G).octave(2).build(),
                        MusicSymbol::Bar("|".to_string(), None),
                        NoteBuilder::new(Note::A).octave(2).length(2.0).build(),
                        NoteBuilder::new(Note::A).octave(2).build(),
                        NoteBuilder::new(Note::B).octave(2).build(),
                        MusicSymbol::Space(" ".to_string()),
                        NoteBuilder::new(Note::A).octave(2).build(),
                        NoteBuilder::new(Note::G).octave(2).build(),
                        NoteBuilder::new(Note::E).octave(2).build(),
                        NoteBuilder::new(Note::G).octave(2).build(),
                        MusicSymbol::Bar("|".to_string(), None),
                        NoteBuilder::new(Note::A).octave(2).build(),
                        NoteBuilder::new(Note::G).octave(2).build(),
                        NoteBuilder::new(Note::B).octave(2).build(),
                        NoteBuilder::new(Note::G).octave(2).build(),
                        MusicSymbol::Space(" ".to_string()),
                        NoteBuilder::new(Note::A).octave(2).build(),
                        NoteBuilder::new(Note::G).octave(2).build(),
                        NoteBuilder::new(Note::E).octave(2).build(),
                        NoteBuilder::new(Note::F).octave(2).build(),
                        MusicSymbol::Bar("|".to_string(), None),
                        NoteBuilder::new(Note::G).octave(2).length(2.0).build(),
                        NoteBuilder::new(Note::G).octave(2).length(2.0).build(),
                        MusicSymbol::Space(" ".to_string()),
                        NoteBuilder::new(Note::F).octave(2).build(),
                        NoteBuilder::new(Note::G).octave(2).build(),
                        NoteBuilder::new(Note::A).octave(2).build(),
                        NoteBuilder::new(Note::G).octave(2).build(),
                        MusicSymbol::Bar("|".to_string(), None),
                        NoteBuilder::new(Note::F).octave(2).length(2.0).build(),
                        NoteBuilder::new(Note::D).octave(2).length(2.0).build(),
                        MusicSymbol::Space(" ".to_string()),
                        NoteBuilder::new(Note::D).octave(2).length(2.0).build(),
                        MusicSymbol::Bar(":|".to_string(), None),
                    ])),
                    TuneLine::Music(MusicLine::new(vec![
                        NoteBuilder::new(Note::E).octave(2).build(),
                        NoteBuilder::new(Note::D).octave(2).build(),
                        MusicSymbol::Bar("|".to_string(), None),
                        NoteBuilder::new(Note::C).octave(2).build(),
                        NoteBuilder::new(Note::E).octave(2).build(),
                        NoteBuilder::new(Note::C).octave(2).build(),
                        NoteBuilder::new(Note::A).build(),
                        MusicSymbol::Space(" ".to_string()),
                        NoteBuilder::new(Note::B).length(2.0).build(),
                        NoteBuilder::new(Note::E).octave(2).build(),
                        NoteBuilder::new(Note::D).octave(2).build(),
                        MusicSymbol::Bar("|".to_string(), None),
                        NoteBuilder::new(Note::C).octave(2).build(),
                        NoteBuilder::new(Note::A).build(),
                        NoteBuilder::new(Note::C).octave(2).build(),
                        NoteBuilder::new(Note::A).build(),
                        MusicSymbol::Space(" ".to_string()),
                        NoteBuilder::new(Note::E).length(2.0).build(),
                        NoteBuilder::new(Note::E).octave(2).build(),
                        NoteBuilder::new(Note::D).octave(2).build(),
                        MusicSymbol::Bar("|".to_string(), None),
                        NoteBuilder::new(Note::C).octave(2).build(),
                        NoteBuilder::new(Note::E).octave(2).build(),
                        NoteBuilder::new(Note::C).octave(2).build(),
                        NoteBuilder::new(Note::A).build(),
                        MusicSymbol::Space(" ".to_string()),
                        NoteBuilder::new(Note::B).length(2.0).build(),
                        NoteBuilder::new(Note::E).octave(2).build(),
                        NoteBuilder::new(Note::D).octave(2).build(),
                        MusicSymbol::Bar("|".to_string(), None),
                        NoteBuilder::new(Note::C).octave(2).length(2.0).build(),
                        NoteBuilder::new(Note::A).length(2.0).build(),
                        MusicSymbol::Space(" ".to_string()),
                        NoteBuilder::new(Note::A).length(2.0).build(),
                        MusicSymbol::Bar(":|".to_string(), None),
                    ])),
                    TuneLine::Music(MusicLine::new(vec![
                        NoteBuilder::new(Note::A).build(),
                        NoteBuilder::new(Note::B).build(),
                        MusicSymbol::Bar("|".to_string(), None),
                        NoteBuilder::new(Note::C).octave(2).build(),
                        NoteBuilder::new(Note::D).octave(2).build(),
                        NoteBuilder::new(Note::E).octave(2).build(),
                        NoteBuilder::new(Note::C).octave(2).build(),
                        MusicSymbol::Space(" ".to_string()),
                        NoteBuilder::new(Note::B).build(),
                        NoteBuilder::new(Note::C).octave(2).build(),
                        NoteBuilder::new(Note::D).octave(2).build(),
                        NoteBuilder::new(Note::B).build(),
                        MusicSymbol::Bar("|".to_string(), None),
                        NoteBuilder::new(Note::A).build(),
                        NoteBuilder::new(Note::B).build(),
                        NoteBuilder::new(Note::A).build(),
                        NoteBuilder::new(Note::F).build(),
                        MusicSymbol::Space(" ".to_string()),
                        NoteBuilder::new(Note::G).build(),
                        NoteBuilder::new(Note::F).build(),
                        NoteBuilder::new(Note::E).length(2.0).build(),
                        MusicSymbol::Bar("|".to_string(), None),
                        NoteBuilder::new(Note::C).octave(2).build(),
                        NoteBuilder::new(Note::D).octave(2).build(),
                        NoteBuilder::new(Note::E).octave(2).build(),
                        NoteBuilder::new(Note::C).octave(2).build(),
                        MusicSymbol::Space(" ".to_string()),
                        NoteBuilder::new(Note::B).build(),
                        NoteBuilder::new(Note::C).octave(2).build(),
                        NoteBuilder::new(Note::D).octave(2).build(),
                        NoteBuilder::new(Note::B).build(),
                        MusicSymbol::Bar("|".to_string(), None),
                        NoteBuilder::new(Note::C).octave(2).length(2.0).build(),
                        NoteBuilder::new(Note::A).length(2.0).build(),
                        MusicSymbol::Space(" ".to_string()),
                        NoteBuilder::new(Note::A).length(2.0).build(),
                        MusicSymbol::Bar(":|".to_string(), None),
                    ]))
                ]))
            )]
        )
    )
}

#[test]
fn parse_extra_spaces() {
    // From https://thesession.org/tunes/8237.
    let data = "X: 1
T: The Origin Of The World
R: mazurka
M: 3/4
L: 1/8
K: Gmin
|: de dc AB | G2-GGAB | ce ee dc | d2-dd dc |
de dc AB | G2-GG AB | EG BE GB | A2 AA BA |
GE CE GE | F2 FD B,D | GE B,E GE | F2 F2 GA |
B2 Bc-cd | d2-dc Bc | cc cB GF | G2 G4 :|
|: G2 GD GA | B2 BA Bd | c2 cd-dc | F4 F2 |
G2 GD GA | B2 BA Bd | c2 cd-dc | FG Bc dc |
ge cG EC | fdB FDB, | eB GE B,G, | A,C FA cA |
B2 Bc-cd | d2 dc Bc | cc cB GF | G2 G4 :|";
    let tune = abc::tune(data).unwrap();

    assert!(tune.header.lines.contains(&HeaderLine::Field(
        InfoField::new('R', "mazurka".to_string()),
        None
    )));
    assert!(tune.header.lines.contains(&HeaderLine::Field(
        InfoField::new('T', "The Origin Of The World".to_string()),
        None
    )));
}
