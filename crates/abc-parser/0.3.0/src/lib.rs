//! # ABC Parser (WIP)
//!
//! An ABC music notation parser for Rust based on the
//! [2.1 Standard](http://abcnotation.com/wiki/abc:standard:v2.1). This crate is currently a work
//! in progress and not fully compliant. For full documentation on which parts of the standard
//! are implemented, take a look at the tests.
//!
//! For parsing examples see [abc](abc/index.html) module.

pub mod datatypes;
mod grammar;

pub use grammar::abc;

#[cfg(test)]
mod tests {
    use super::*;
    use datatypes::*;

    fn make_tune_header(x: &str, t: &str, k: &str) -> TuneHeader {
        TuneHeader::new(vec![
            InfoField::new('X', x.to_string()),
            InfoField::new('T', t.to_string()),
            InfoField::new('K', k.to_string()),
        ])
    }

    #[test]
    fn empty_tunebook() {
        let t = abc::tune_book("").unwrap();
        assert_eq!(t, TuneBook::new(None, vec![]));
    }

    #[test]
    fn tune_book() {
        let t = abc::tune_book("X:1\nT:Some Title\nK:G\n").unwrap();
        assert_eq!(
            t,
            TuneBook::new(
                None,
                vec![Tune::new(make_tune_header("1", "Some Title", "G"), None)]
            )
        );
    }

    #[test]
    fn invalid_tune() {
        abc::tune("a").unwrap_err();
    }

    #[test]
    fn empty_tune_no_body() {
        let t = abc::tune("X:1\nT:Some Title\nK:G\n").unwrap();
        assert_eq!(t, Tune::new(make_tune_header("1", "Some Title", "G"), None));
    }

    #[test]
    fn empty_tune_empty_body() {
        let t = abc::tune("X:1\nT:Some Title\nK:G\n\n").unwrap();
        assert_eq!(
            t,
            Tune::new(
                make_tune_header("1", "Some Title", "G"),
                Some(TuneBody::new(vec![]))
            )
        );
    }

    #[test]
    fn tune_header() {
        let t = abc::tune_header("X:1\nT:Some Title\nK:G\n").unwrap();
        assert_eq!(t, make_tune_header("1", "Some Title", "G"));
    }

    #[test]
    fn tune_header_unicode() {
        let t = abc::tune_header("X:1\nT:S©ome ©Title©\nK:G\n").unwrap();
        assert_eq!(t, make_tune_header("1", "S©ome ©Title©", "G"));
    }

    #[test]
    fn tune_header_extra_fields() {
        let t = abc::tune_header("X:1\nT:Some Title\nO:England\nK:G\n").unwrap();
        assert_eq!(
            t,
            TuneHeader::new(vec![
                InfoField::new('X', "1".to_string()),
                InfoField::new('T', "Some Title".to_string()),
                InfoField::new('O', "England".to_string()),
                InfoField::new('K', "G".to_string())
            ])
        );
    }

    #[test]
    fn file_header() {
        let t = abc::tune_book("O:Some origin info\n\nX:1\nT:Some Title\nK:G\n").unwrap();
        assert_eq!(
            t,
            TuneBook::new(
                Some(FileHeader::new(vec![InfoField::new(
                    'O',
                    "Some origin info".to_string()
                )])),
                vec![Tune::new(make_tune_header("1", "Some Title", "G"), None)]
            )
        );
    }

    #[test]
    fn music_line_empty_1() {
        abc::music_line("").unwrap_err();
    }
    #[test]
    fn music_line_empty_2() {
        abc::music_line("\n").unwrap_err();
    }

    #[test]
    fn body_with_music() {
        let b = abc::tune_body("A").unwrap();
        assert_eq!(
            b,
            TuneBody::new(vec![MusicLine::new(vec![MusicSymbol::note(Note::A)])])
        )
    }

    #[test]
    fn body_with_music_2() {
        let b = abc::tune_body("A\n").unwrap();
        assert_eq!(
            b,
            TuneBody::new(vec![MusicLine::new(vec![MusicSymbol::note(Note::A)])])
        )
    }

    #[test]
    fn music_line() {
        let m = abc::music_line("A\n").unwrap();
        assert_eq!(m, MusicLine::new(vec![MusicSymbol::note(Note::A)]))
    }

    #[test]
    fn music_line_2() {
        let m = abc::music_line("A").unwrap();
        assert_eq!(m, MusicLine::new(vec![MusicSymbol::note(Note::A)]))
    }

    #[test]
    fn music_line_multiple_notes() {
        let m = abc::music_line("AB cd").unwrap();
        assert_eq!(
            m,
            MusicLine::new(vec![
                MusicSymbol::note(Note::A),
                MusicSymbol::note(Note::B),
                MusicSymbol::VisualBreak,
                MusicSymbol::new_note(vec![], None, Note::C, 2, 1.0, None),
                MusicSymbol::new_note(vec![], None, Note::D, 2, 1.0, None)
            ])
        )
    }

    #[test]
    fn music_line_chord() {
        let m = abc::music_line("!f! [F=AC]3/2[GBD] H[CEG]2").unwrap();
        assert_eq!(
            m,
            MusicLine::new(vec![
                MusicSymbol::Chord {
                    decorations: vec![Decoration::Unresolved("f".to_string())],
                    notes: vec![
                        MusicSymbol::note(Note::F),
                        MusicSymbol::new_note(
                            vec![],
                            Some(Accidental::Natural),
                            Note::A,
                            1,
                            1.0,
                            None
                        ),
                        MusicSymbol::note(Note::C)
                    ],
                    length: 3.0 / 2.0
                },
                MusicSymbol::Chord {
                    decorations: vec![],
                    notes: vec![
                        MusicSymbol::note(Note::G),
                        MusicSymbol::note(Note::B),
                        MusicSymbol::note(Note::D)
                    ],
                    length: 1f32
                },
                MusicSymbol::VisualBreak,
                MusicSymbol::Chord {
                    decorations: vec![Decoration::Fermata],
                    notes: vec![
                        MusicSymbol::note(Note::C),
                        MusicSymbol::note(Note::E),
                        MusicSymbol::note(Note::G)
                    ],
                    length: 2.0
                }
            ])
        );
    }

    #[test]
    fn music_line_backticks() {
        let m = abc::music_line("A2``B``C").unwrap();
        assert_eq!(
            m,
            MusicLine::new(vec![
                MusicSymbol::note_from_length(Note::A, 2.0),
                MusicSymbol::note(Note::B),
                MusicSymbol::note(Note::C)
            ])
        )
    }

    #[test]
    fn note() {
        let s = abc::music_symbol("A").unwrap();
        assert_eq!(s, MusicSymbol::note(Note::A))
    }

    macro_rules! assert_note_len {
        ($input:tt, $expected:expr) => {
            assert_eq!(
                abc::music_symbol($input).unwrap(),
                MusicSymbol::note_from_length(Note::A, $expected)
            );
        };
    }

    #[test]
    fn note_length_1() {
        assert_note_len!("A3", 3f32);
    }
    #[test]
    fn note_length_2() {
        assert_note_len!("A9001", 9001f32)
    }
    #[test]
    fn note_length_3() {
        assert_note_len!("A/2", 0.5)
    }
    #[test]
    fn note_length_4() {
        assert_note_len!("A/", 0.5)
    }
    #[test]
    fn note_length_5() {
        assert_note_len!("A/4", 0.25)
    }
    #[test]
    fn note_length_6() {
        assert_note_len!("A//", 0.25)
    }
    #[test]
    fn note_length_7() {
        assert_note_len!("A3/2", 3.0 / 2.0)
    }
    #[test]
    fn note_length_8() {
        assert_note_len!("A6821/962", 6821.0 / 962.0)
    }

    #[test]
    fn note_length_invalid_1() {
        abc::music_symbol("A0").unwrap_err();
    }
    #[test]
    fn note_length_invalid_2() {
        abc::music_symbol("A-1").unwrap_err();
    }

    #[test]
    fn note_invalid_1() {
        abc::music_symbol("H").unwrap_err();
    }
    #[test]
    fn note_invalid_2() {
        abc::music_symbol("h").unwrap_err();
    }
    #[test]
    fn note_invalid_3() {
        abc::music_symbol("Y").unwrap_err();
    }
    #[test]
    fn note_invalid_4() {
        abc::music_symbol("y").unwrap_err();
    }
    #[test]
    fn note_invalid_5() {
        abc::music_symbol("m").unwrap_err();
    }
    #[test]
    fn note_invalid_6() {
        abc::music_symbol("T").unwrap_err();
    }

    macro_rules! assert_bar {
        ($bar:tt) => {
            assert_eq!(
                abc::music_symbol($bar).unwrap(),
                MusicSymbol::Bar($bar.to_string())
            )
        };
    }

    #[test]
    fn bar_1() {
        assert_bar!("|")
    }
    #[test]
    fn bar_2() {
        assert_bar!("|]")
    }
    #[test]
    fn bar_3() {
        assert_bar!("||")
    }
    #[test]
    fn bar_4() {
        assert_bar!("[|")
    }
    #[test]
    fn bar_5() {
        assert_bar!("|:")
    }
    #[test]
    fn bar_6() {
        assert_bar!(":|")
    }
    #[test]
    fn bar_7() {
        assert_bar!("::")
    }
    #[test]
    fn bar_8() {
        assert_bar!("[|]")
    }

    macro_rules! assert_dec {
        ($input:tt, $expected:expr) => {
            assert_eq!(
                abc::music_symbol($input).unwrap(),
                MusicSymbol::new_note($expected, None, Note::A, 1, 1.0, None)
            );
        };
    }

    #[test]
    fn decoration_1() {
        assert_dec!(".A", vec![Decoration::Staccato])
    }
    #[test]
    fn decoration_2() {
        assert_dec!("~A", vec![Decoration::Roll])
    }
    #[test]
    fn decoration_3() {
        assert_dec!("HA", vec![Decoration::Fermata])
    }
    #[test]
    fn decoration_4() {
        assert_dec!("LA", vec![Decoration::Accent])
    }
    #[test]
    fn decoration_5() {
        assert_dec!("MA", vec![Decoration::LowerMordent])
    }
    #[test]
    fn decoration_6() {
        assert_dec!("OA", vec![Decoration::Coda])
    }
    #[test]
    fn decoration_7() {
        assert_dec!("PA", vec![Decoration::UpperMordent])
    }
    #[test]
    fn decoration_8() {
        assert_dec!("SA", vec![Decoration::Segno])
    }
    #[test]
    fn decoration_9() {
        assert_dec!("TA", vec![Decoration::Trill])
    }
    #[test]
    fn decoration_10() {
        assert_dec!("uA", vec![Decoration::UpBow])
    }
    #[test]
    fn decoration_11() {
        assert_dec!("vA", vec![Decoration::DownBow])
    }
    #[test]
    fn decoration_12() {
        assert_dec!(
            "!somedec!A",
            vec![Decoration::Unresolved("somedec".to_string())]
        )
    }

    #[test]
    fn multiple_decorations() {
        assert_dec!("v.A", vec![Decoration::DownBow, Decoration::Staccato]);
    }

    #[test]
    fn decoration_invalid() {
        abc::music_symbol("!A").unwrap_err();
    }

    macro_rules! assert_acc {
        ($input:tt, $expected:expr) => {
            assert_eq!(
                abc::music_symbol($input).unwrap(),
                MusicSymbol::new_note(vec![], Some($expected), Note::A, 1, 1.0, None)
            )
        };
    }

    #[test]
    fn accidental_1() {
        assert_acc!("^A", Accidental::Sharp)
    }
    #[test]
    fn accidental_2() {
        assert_acc!("_A", Accidental::Flat)
    }
    #[test]
    fn accidental_3() {
        assert_acc!("=A", Accidental::Natural)
    }
    #[test]
    fn accidental_4() {
        assert_acc!("^^A", Accidental::DoubleSharp)
    }
    #[test]
    fn accidental_5() {
        assert_acc!("__A", Accidental::DoubleFlat)
    }

    #[test]
    fn accidental_invalid_1() {
        abc::music_symbol("^^^A").unwrap_err();
    }
    #[test]
    fn accidental_invalid_2() {
        abc::music_symbol("___A").unwrap_err();
    }
    #[test]
    fn accidental_invalid_3() {
        abc::music_symbol("_^_A").unwrap_err();
    }
    #[test]
    fn accidental_invalid_4() {
        abc::music_symbol("^_A").unwrap_err();
    }

    macro_rules! assert_oct {
        ($input:tt, $expected:expr) => {
            assert_eq!(
                abc::music_symbol($input).unwrap(),
                MusicSymbol::new_note(vec![], None, Note::A, $expected, 1.0, None)
            );
        };
    }

    #[test]
    fn octave_1() {
        assert_oct!("A,", 0)
    }
    #[test]
    fn octave_2() {
        assert_oct!("A'", 2)
    }
    #[test]
    fn octave_3() {
        assert_oct!("A,,,", -2)
    }
    #[test]
    fn octave_4() {
        assert_oct!("A''''", 5)
    }
    #[test]
    fn octave_5() {
        assert_oct!("A,'", 1)
    }
    #[test]
    fn octave_6() {
        assert_oct!("A,'',,','", 1)
    }

    #[test]
    fn tie() {
        assert_eq!(
            abc::music_line("A-A").unwrap(),
            MusicLine::new(vec![
                MusicSymbol::new_note(vec![], None, Note::A, 1, 1.0, Some(Tie::Solid)),
                MusicSymbol::new_note(vec![], None, Note::A, 1, 1.0, None)
            ])
        );
    }

    #[test]
    fn tie_across_bars() {
        assert_eq!(
            abc::music_line("abc-|cba").unwrap(),
            MusicLine::new(vec![
                MusicSymbol::new_note(vec![], None, Note::A, 2, 1.0, None),
                MusicSymbol::new_note(vec![], None, Note::B, 2, 1.0, None),
                MusicSymbol::new_note(vec![], None, Note::C, 2, 1.0, Some(Tie::Solid)),
                MusicSymbol::Bar("|".to_string()),
                MusicSymbol::new_note(vec![], None, Note::C, 2, 1.0, None),
                MusicSymbol::new_note(vec![], None, Note::B, 2, 1.0, None),
                MusicSymbol::new_note(vec![], None, Note::A, 2, 1.0, None),
            ])
        );
    }

    #[test]
    fn tie_with_length() {
        assert_eq!(
            abc::music_line("c4-c4").unwrap(),
            MusicLine::new(vec![
                MusicSymbol::new_note(vec![], None, Note::C, 2, 4.0, Some(Tie::Solid)),
                MusicSymbol::new_note(vec![], None, Note::C, 2, 4.0, None)
            ])
        );
    }

    #[test]
    fn tie_dotted() {
        assert_eq!(
            abc::music_line("C.-C").unwrap(),
            MusicLine::new(vec![
                MusicSymbol::new_note(vec![], None, Note::C, 1, 1.0, Some(Tie::Dotted)),
                MusicSymbol::new_note(vec![], None, Note::C, 1, 1.0, None)
            ])
        );
    }

    #[test]
    fn invalid_tie() {
        assert!(abc::music_line("c4 -c4").is_err());
        assert!(abc::music_line("abc|-cba").is_err());
    }

    macro_rules! assert_rst {
        ($input:tt, $expected:expr) => {
            assert_eq!(
                abc::music_symbol($input).unwrap(),
                MusicSymbol::Rest($expected)
            )
        };
    }

    #[test]
    fn rest_1() {
        assert_rst!("z", Rest::Note(1))
    }
    #[test]
    fn rest_2() {
        assert_rst!("x", Rest::NoteHidden(1))
    }
    #[test]
    fn rest_3() {
        assert_rst!("Z", Rest::Measure(1))
    }
    #[test]
    fn rest_4() {
        assert_rst!("X", Rest::MeasureHidden(1))
    }
    #[test]
    fn rest_5() {
        assert_rst!("z3", Rest::Note(3))
    }
    #[test]
    fn rest_6() {
        assert_rst!("Z10", Rest::Measure(10))
    }
    #[test]
    fn rest_7() {
        assert_rst!("x7", Rest::NoteHidden(7))
    }
    #[test]
    fn rest_8() {
        assert_rst!("X900", Rest::MeasureHidden(900))
    }

    #[ignore]
    #[test]
    fn endings_general() {
        let m = abc::music_line("f|[1 d:|[2 d2 B|]").unwrap();
        assert_eq!(
            m,
            MusicLine::new(vec![
                MusicSymbol::new_note(vec![], None, Note::F, 2, 1.0, None),
                MusicSymbol::Bar("|".to_string()),
                MusicSymbol::Ending(1),
                MusicSymbol::new_note(vec![], None, Note::D, 2, 1.0, None),
                MusicSymbol::Bar(":|".to_string()),
                MusicSymbol::Ending(2),
                MusicSymbol::new_note(vec![], None, Note::D, 2, 2.0, None),
                MusicSymbol::VisualBreak,
                MusicSymbol::note(Note::B),
                MusicSymbol::Bar("|]".to_string())
            ])
        );
    }

    #[test]
    fn endings_1() {
        assert_eq!(abc::ending("[1").unwrap(), MusicSymbol::Ending(1))
    }
    #[test]
    fn endings_2() {
        assert_eq!(abc::ending("[2").unwrap(), MusicSymbol::Ending(2))
    }
    #[test]
    fn endings_3() {
        assert_eq!(abc::ending("[87654").unwrap(), MusicSymbol::Ending(87654))
    }

    #[ignore]
    #[test]
    fn ending_after_bar() {
        let m = abc::music_line("|[1").unwrap();
        assert_eq!(
            m,
            MusicLine::new(vec![
                MusicSymbol::Bar("|".to_string()),
                MusicSymbol::Ending(1)
            ])
        )
    }

    #[test]
    fn chord_1() {
        let c = abc::music_symbol("[CEG]").unwrap();
        assert_eq!(
            c,
            MusicSymbol::Chord {
                decorations: vec![],
                notes: vec![
                    MusicSymbol::note(Note::C),
                    MusicSymbol::note(Note::E),
                    MusicSymbol::note(Note::G)
                ],
                length: 1.0
            }
        );
    }

    #[test]
    fn chord_2() {
        let c = abc::music_symbol("[C2E2G2]3").unwrap();
        assert_eq!(
            c,
            MusicSymbol::Chord {
                decorations: vec![],
                notes: vec![
                    MusicSymbol::note_from_length(Note::C, 2.0),
                    MusicSymbol::note_from_length(Note::E, 2.0),
                    MusicSymbol::note_from_length(Note::G, 2.0)
                ],
                length: 3.0
            }
        );
    }

    #[test]
    fn grace_notes_1() {
        let g = abc::music_symbol("{GdGe}").unwrap();
        assert_eq!(
            g,
            MusicSymbol::GraceNotes {
                acciaccatura: None,
                notes: vec![
                    MusicSymbol::note(Note::G),
                    MusicSymbol::new_note(vec![], None, Note::D, 2, 1.0, None),
                    MusicSymbol::note(Note::G),
                    MusicSymbol::new_note(vec![], None, Note::E, 2, 1.0, None)
                ]
            }
        )
    }

    #[test]
    fn grace_notes_2() {
        let g = abc::music_symbol("{/g}").unwrap();
        assert_eq!(
            g,
            MusicSymbol::GraceNotes {
                acciaccatura: Some(()),
                notes: vec![MusicSymbol::new_note(vec![], None, Note::G, 2, 1.0, None)]
            }
        )
    }

    macro_rules! assert_tup {
        ($input:tt, $p:expr, $q:expr, $r:expr, $($expected:expr), *) => {
            assert_eq!(
                abc::music_symbol($input).unwrap(),
                MusicSymbol::Tuplet {
                    p: $p,
                    q: $q,
                    r: $r,
                    notes: vec![
                        $(
                            MusicSymbol::note($expected),
                        )*
                    ]
                }
            )
        };
    }

    #[test]
    fn tuplet_1() {
        assert_tup!("(2AB", 2, 3, 2, Note::A, Note::B)
    }
    #[test]
    fn tuplet_2() {
        assert_tup!("(2 AB", 2, 3, 2, Note::A, Note::B)
    }
    #[test]
    fn tuplet_3() {
        assert_tup!("(3ABC", 3, 2, 3, Note::A, Note::B, Note::C)
    }
    #[test]
    fn tuplet_4() {
        assert_tup!("(3::ABC", 3, 2, 3, Note::A, Note::B, Note::C)
    }
    #[test]
    fn tuplet_5() {
        assert_tup!("(3:2ABC", 3, 2, 3, Note::A, Note::B, Note::C)
    }
    #[test]
    fn tuplet_6() {
        assert_tup!("(3:2:3ABC", 3, 2, 3, Note::A, Note::B, Note::C)
    }
    #[test]
    fn tuplet_7() {
        assert_tup!(
            "(9ABCDEFGAB",
            9,
            0,
            9,
            Note::A,
            Note::B,
            Note::C,
            Note::D,
            Note::E,
            Note::F,
            Note::G,
            Note::A,
            Note::B
        )
    }
}
