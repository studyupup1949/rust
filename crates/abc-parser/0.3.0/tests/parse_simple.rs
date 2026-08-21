extern crate abc_parser;
use abc_parser::abc;
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
            Some(FileHeader::new(vec![
                InfoField::new('M', "4/4".to_string()),
                InfoField::new('O', "Irish".to_string()),
                InfoField::new('R', "Reel".to_string())
            ])),
            vec![Tune::new(
                TuneHeader::new(vec![
                    InfoField::new('X', "1".to_string()),
                    InfoField::new('T', "Untitled Reel".to_string()),
                    InfoField::new('C', "Trad.".to_string()),
                    InfoField::new('K', "D".to_string())
                ]),
                Some(TuneBody::new(vec![
                    MusicLine::new(vec![
                        MusicSymbol::new_note(vec![], None, Note::E, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::G, 2, 1.0, None),
                        MusicSymbol::Bar("|".to_string()),
                        MusicSymbol::new_note(vec![], None, Note::A, 2, 2.0, None),
                        MusicSymbol::new_note(vec![], None, Note::A, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::B, 2, 1.0, None),
                        MusicSymbol::VisualBreak,
                        MusicSymbol::new_note(vec![], None, Note::A, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::G, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::E, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::G, 2, 1.0, None),
                        MusicSymbol::Bar("|".to_string()),
                        MusicSymbol::new_note(vec![], None, Note::A, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::G, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::B, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::G, 2, 1.0, None),
                        MusicSymbol::VisualBreak,
                        MusicSymbol::new_note(vec![], None, Note::A, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::G, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::E, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::F, 2, 1.0, None),
                        MusicSymbol::Bar("|".to_string()),
                        MusicSymbol::new_note(vec![], None, Note::G, 2, 2.0, None),
                        MusicSymbol::new_note(vec![], None, Note::G, 2, 2.0, None),
                        MusicSymbol::VisualBreak,
                        MusicSymbol::new_note(vec![], None, Note::F, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::G, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::A, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::G, 2, 1.0, None),
                        MusicSymbol::Bar("|".to_string()),
                        MusicSymbol::new_note(vec![], None, Note::F, 2, 2.0, None),
                        MusicSymbol::new_note(vec![], None, Note::D, 2, 2.0, None),
                        MusicSymbol::VisualBreak,
                        MusicSymbol::new_note(vec![], None, Note::D, 2, 2.0, None),
                        MusicSymbol::Bar(":|".to_string()),
                    ]),
                    MusicLine::new(vec![
                        MusicSymbol::new_note(vec![], None, Note::E, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::D, 2, 1.0, None),
                        MusicSymbol::Bar("|".to_string()),
                        MusicSymbol::new_note(vec![], None, Note::C, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::E, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::C, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::A, 1, 1.0, None),
                        MusicSymbol::VisualBreak,
                        MusicSymbol::new_note(vec![], None, Note::B, 1, 2.0, None),
                        MusicSymbol::new_note(vec![], None, Note::E, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::D, 2, 1.0, None),
                        MusicSymbol::Bar("|".to_string()),
                        MusicSymbol::new_note(vec![], None, Note::C, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::A, 1, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::C, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::A, 1, 1.0, None),
                        MusicSymbol::VisualBreak,
                        MusicSymbol::new_note(vec![], None, Note::E, 1, 2.0, None),
                        MusicSymbol::new_note(vec![], None, Note::E, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::D, 2, 1.0, None),
                        MusicSymbol::Bar("|".to_string()),
                        MusicSymbol::new_note(vec![], None, Note::C, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::E, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::C, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::A, 1, 1.0, None),
                        MusicSymbol::VisualBreak,
                        MusicSymbol::new_note(vec![], None, Note::B, 1, 2.0, None),
                        MusicSymbol::new_note(vec![], None, Note::E, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::D, 2, 1.0, None),
                        MusicSymbol::Bar("|".to_string()),
                        MusicSymbol::new_note(vec![], None, Note::C, 2, 2.0, None),
                        MusicSymbol::new_note(vec![], None, Note::A, 1, 2.0, None),
                        MusicSymbol::VisualBreak,
                        MusicSymbol::new_note(vec![], None, Note::A, 1, 2.0, None),
                        MusicSymbol::Bar(":|".to_string()),
                    ]),
                    MusicLine::new(vec![
                        MusicSymbol::new_note(vec![], None, Note::A, 1, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::B, 1, 1.0, None),
                        MusicSymbol::Bar("|".to_string()),
                        MusicSymbol::new_note(vec![], None, Note::C, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::D, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::E, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::C, 2, 1.0, None),
                        MusicSymbol::VisualBreak,
                        MusicSymbol::new_note(vec![], None, Note::B, 1, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::C, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::D, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::B, 1, 1.0, None),
                        MusicSymbol::Bar("|".to_string()),
                        MusicSymbol::new_note(vec![], None, Note::A, 1, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::B, 1, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::A, 1, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::F, 1, 1.0, None),
                        MusicSymbol::VisualBreak,
                        MusicSymbol::new_note(vec![], None, Note::G, 1, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::F, 1, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::E, 1, 2.0, None),
                        MusicSymbol::Bar("|".to_string()),
                        MusicSymbol::new_note(vec![], None, Note::C, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::D, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::E, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::C, 2, 1.0, None),
                        MusicSymbol::VisualBreak,
                        MusicSymbol::new_note(vec![], None, Note::B, 1, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::C, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::D, 2, 1.0, None),
                        MusicSymbol::new_note(vec![], None, Note::B, 1, 1.0, None),
                        MusicSymbol::Bar("|".to_string()),
                        MusicSymbol::new_note(vec![], None, Note::C, 2, 2.0, None),
                        MusicSymbol::new_note(vec![], None, Note::A, 1, 2.0, None),
                        MusicSymbol::VisualBreak,
                        MusicSymbol::new_note(vec![], None, Note::A, 1, 2.0, None),
                        MusicSymbol::Bar(":|".to_string()),
                    ])
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

    assert!(tune
        .header
        .info
        .contains(&InfoField::new('R', "mazurka".to_string())));
    assert!(tune
        .header
        .info
        .contains(&InfoField::new('T', "The Origin Of The World".to_string())));
}
