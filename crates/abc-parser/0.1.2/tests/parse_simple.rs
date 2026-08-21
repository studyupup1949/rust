extern crate abc_parser;
use abc_parser::abc;
use abc_parser::datatypes::*;

macro_rules! n {
    ($n:tt) => (MusicSymbol::note($n));
    ($n:tt, $l:tt) => (
        MusicSymbol::note_from_length($n, $l)
    );
}

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
    assert_eq!(tb,
        TuneBook::new(
            Some(FileHeader::new(
                vec![
                    InfoField::new('M', "4/4".to_string()),
                    InfoField::new('O', "Irish".to_string()),
                    InfoField::new('R', "Reel".to_string())
                ]
            )),
            vec![
                Tune::new(
                    TuneHeader::new(
                        vec![
                            InfoField::new('X', "1".to_string()),
                            InfoField::new('T', "Untitled Reel".to_string()),
                            InfoField::new('C', "Trad.".to_string()),
                            InfoField::new('K', "D".to_string())
                        ]
                    ),
                    Some(TuneBody::new(
                        vec![
                            MusicLine::new(
                                vec![
                                    n!('e'), n!('g'),
                                    MusicSymbol::Bar("|".to_string()),
                                    n!('a', 2.0), n!('a'), n!('b'),
                                    MusicSymbol::VisualBreak(),
                                    n!('a'), n!('g'), n!('e'), n!('g'),
                                    MusicSymbol::Bar("|".to_string()),
                                    n!('a'), n!('g'), n!('b'), n!('g'),
                                    MusicSymbol::VisualBreak(),
                                    n!('a'), n!('g'), n!('e'), n!('f'),
                                    MusicSymbol::Bar("|".to_string()),
                                    n!('g', 2.0), n!('g', 2.0),
                                    MusicSymbol::VisualBreak(),
                                    n!('f'), n!('g'), n!('a'), n!('g'),
                                    MusicSymbol::Bar("|".to_string()),
                                    n!('f', 2.0), n!('d', 2.0),
                                    MusicSymbol::VisualBreak(),
                                    n!('d', 2.0),
                                    MusicSymbol::Bar(":|".to_string()),
                                ]
                            ),
                            MusicLine::new(
                                vec![
                                    n!('e'), n!('d'),
                                    MusicSymbol::Bar("|".to_string()),
                                    n!('c'), n!('e'), n!('c'), n!('A'),
                                    MusicSymbol::VisualBreak(),
                                    n!('B', 2.0), n!('e'), n!('d'),
                                    MusicSymbol::Bar("|".to_string()),
                                    n!('c'), n!('A'), n!('c'), n!('A'),
                                    MusicSymbol::VisualBreak(),
                                    n!('E', 2.0), n!('e'), n!('d'),
                                    MusicSymbol::Bar("|".to_string()),
                                    n!('c'), n!('e'), n!('c'), n!('A'),
                                    MusicSymbol::VisualBreak(),
                                    n!('B', 2.0), n!('e'), n!('d'),
                                    MusicSymbol::Bar("|".to_string()),
                                    n!('c', 2.0), n!('A', 2.0),
                                    MusicSymbol::VisualBreak(),
                                    n!('A', 2.0),
                                    MusicSymbol::Bar(":|".to_string()),
                                ]
                            ),
                            MusicLine::new(
                                vec![
                                    n!('A'), n!('B'),
                                    MusicSymbol::Bar("|".to_string()),
                                    n!('c'), n!('d'), n!('e'), n!('c'),
                                    MusicSymbol::VisualBreak(),
                                    n!('B'), n!('c'), n!('d'), n!('B'),
                                    MusicSymbol::Bar("|".to_string()),
                                    n!('A'), n!('B'), n!('A'), n!('F'),
                                    MusicSymbol::VisualBreak(),
                                    n!('G'), n!('F'), n!('E', 2.0),
                                    MusicSymbol::Bar("|".to_string()),
                                    n!('c'), n!('d'), n!('e'), n!('c'),
                                    MusicSymbol::VisualBreak(),
                                    n!('B'), n!('c'), n!('d'), n!('B'),
                                    MusicSymbol::Bar("|".to_string()),
                                    n!('c', 2.0), n!('A', 2.0),
                                    MusicSymbol::VisualBreak(),
                                    n!('A', 2.0),
                                    MusicSymbol::Bar(":|".to_string()),
                                ]
                            )
                        ]
                    ))
                )
            ]
        )
    )
}
