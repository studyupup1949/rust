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

pub use crate::grammar::abc;

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::datatypes::builder::NoteBuilder;
    use crate::datatypes::*;

    fn make_tune_header(x: &str, t: &str, k: &str) -> TuneHeader {
        TuneHeader::new(vec![
            HeaderLine::Field(InfoField::new('X', x.to_string()), None),
            HeaderLine::Field(InfoField::new('T', t.to_string()), None),
            HeaderLine::Field(InfoField::new('K', k.to_string()), None),
        ])
    }

    #[test]
    fn empty_tune_book() {
        let t = abc::tune_book("").unwrap();
        assert_eq!(t, TuneBook::new(vec![], None, vec![]));
    }

    #[test]
    fn tune_book() {
        let t = abc::tune_book("X:1\nT:Some Title\nK:G\n").unwrap();
        assert_eq!(
            t,
            TuneBook::new(
                vec![],
                None,
                vec![Tune::new(make_tune_header("1", "Some Title", "G"), None)]
            )
        );
    }

    #[test]
    fn tune_book_comments() {
        let t = abc::tune_book("\n\n% A comment\n\n% Some empty lines\nX:1\nT:Some Title\nK:G\n")
            .unwrap();
        assert_eq!(
            t,
            TuneBook::new(
                vec![
                    IgnoredLine::EmptyLine,
                    IgnoredLine::EmptyLine,
                    IgnoredLine::Comment(Comment::CommentLine(
                        "".to_string(),
                        " A comment".to_string()
                    )),
                    IgnoredLine::EmptyLine,
                    IgnoredLine::Comment(Comment::CommentLine(
                        "".to_string(),
                        " Some empty lines".to_string()
                    )),
                ],
                None,
                vec![Tune::new(make_tune_header("1", "Some Title", "G"), None)]
            )
        );
    }

    #[test]
    fn tune_book_comments_2() {
        let t = abc::tune_book("%%%%%%%\n% C1\n% C2\n\n% C3\nX:1\nT:Some Title\nK:G\n").unwrap();
        assert_eq!(
            t,
            TuneBook::new(
                vec![
                    IgnoredLine::Comment(Comment::StylesheetDirective("%%%%%".to_string())),
                    IgnoredLine::Comment(Comment::CommentLine("".to_string(), " C1".to_string())),
                    IgnoredLine::Comment(Comment::CommentLine("".to_string(), " C2".to_string())),
                    IgnoredLine::EmptyLine,
                    IgnoredLine::Comment(Comment::CommentLine("".to_string(), " C3".to_string())),
                ],
                None,
                vec![Tune::new(make_tune_header("1", "Some Title", "G"), None)]
            )
        );
    }

    #[test]
    fn file_header() {
        let t = abc::tune_book("O:Some origin info\n\nX:1\nT:Some Title\nK:G\n").unwrap();
        assert_eq!(
            t,
            TuneBook::new(
                vec![],
                Some(FileHeader::new(vec![HeaderLine::Field(
                    InfoField::new('O', "Some origin info".to_string()),
                    None
                )]),),
                vec![Tune::new(make_tune_header("1", "Some Title", "G"), None)]
            )
        );
    }

    #[test]
    fn file_header_comments() {
        let t = abc::tune_book(
            "\
% Comment 1
O:Some origin info
% Comment 2

X:1
T:Some Title
K:G
",
        )
        .unwrap();
        assert_eq!(
            t,
            TuneBook::new(
                vec![IgnoredLine::Comment(Comment::CommentLine(
                    "".to_string(),
                    " Comment 1".to_string()
                ))],
                Some(FileHeader::new(vec![
                    HeaderLine::Field(InfoField::new('O', "Some origin info".to_string()), None),
                    HeaderLine::Comment(Comment::CommentLine(
                        "".to_string(),
                        " Comment 2".to_string()
                    )),
                ])),
                vec![Tune::new(make_tune_header("1", "Some Title", "G"), None)]
            )
        );
    }

    #[test]
    fn file_header_comment_empty() {
        assert_eq!(
            abc::file_header("%\n\n").unwrap(),
            FileHeader::new(vec![HeaderLine::Comment(Comment::CommentLine(
                "".to_string(),
                "".to_string()
            ))])
        );
    }

    #[test]
    fn file_header_comment_mixed() {
        assert_eq!(
            abc::file_header("% Some text 123-45[\teu] \n\n").unwrap(),
            FileHeader::new(vec![HeaderLine::Comment(Comment::CommentLine(
                "".to_string(),
                " Some text 123-45[\teu] ".to_string()
            ))])
        );
    }

    #[test]
    fn file_header_comment_preceding_whitespace() {
        assert_eq!(
            abc::file_header("    % Some text\n\n").unwrap(),
            FileHeader::new(vec![HeaderLine::Comment(Comment::CommentLine(
                "    ".to_string(),
                " Some text".to_string()
            ))])
        );
    }

    #[test]
    fn file_header_trailing_comments() {
        let file_header = abc::file_header(
            "\
O:Origin % Comment
B:Book%Another comment

",
        )
        .unwrap();
        assert_eq!(
            file_header,
            FileHeader::new(vec![
                HeaderLine::Field(
                    InfoField::new('O', "Origin ".to_string()),
                    Some(Comment::Comment(" Comment".to_string()))
                ),
                HeaderLine::Field(
                    InfoField::new('B', "Book".to_string()),
                    Some(Comment::Comment("Another comment".to_string()))
                ),
            ])
        );
    }

    #[test]
    fn file_header_stylesheet_directive_empty() {
        let text = "%%\n\n";
        assert_eq!(
            abc::file_header(text).unwrap(),
            FileHeader::new(vec![HeaderLine::Comment(Comment::StylesheetDirective(
                "".to_string()
            ))])
        );
    }

    #[test]
    fn file_header_stylesheet_directive() {
        let text = "%%pagewidth\t21cm\n\n";
        assert_eq!(
            abc::file_header(text).unwrap(),
            FileHeader::new(vec![HeaderLine::Comment(Comment::StylesheetDirective(
                "pagewidth\t21cm".to_string()
            ))])
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
                HeaderLine::Field(InfoField::new('X', "1".to_string()), None),
                HeaderLine::Field(InfoField::new('T', "Some Title".to_string()), None),
                HeaderLine::Field(InfoField::new('O', "England".to_string()), None),
                HeaderLine::Field(InfoField::new('K', "G".to_string()), None),
            ])
        );
    }

    #[test]
    fn tune_header_comments() {
        let t = abc::tune_header(
            "\
% Comment 1
X:1
% Comment 2
T:Some Title
% Comment 3
O:England
% Comment 4
K:G
",
        )
        .unwrap();
        assert_eq!(
            t,
            TuneHeader::new(vec![
                HeaderLine::Comment(Comment::CommentLine(
                    "".to_string(),
                    " Comment 1".to_string()
                )),
                HeaderLine::Field(InfoField::new('X', "1".to_string()), None),
                HeaderLine::Comment(Comment::CommentLine(
                    "".to_string(),
                    " Comment 2".to_string()
                )),
                HeaderLine::Field(InfoField::new('T', "Some Title".to_string()), None),
                HeaderLine::Comment(Comment::CommentLine(
                    "".to_string(),
                    " Comment 3".to_string()
                )),
                HeaderLine::Field(InfoField::new('O', "England".to_string()), None),
                HeaderLine::Comment(Comment::CommentLine(
                    "".to_string(),
                    " Comment 4".to_string()
                )),
                HeaderLine::Field(InfoField::new('K', "G".to_string()), None),
            ])
        );
    }

    #[test]
    fn tune_header_trailing_comments() {
        let t = abc::tune_header(
            "\
X:1%Comment 1
T:Some Title%Comment 2
O:England%Comment 3
K:G%Comment 4
",
        )
        .unwrap();
        assert_eq!(
            t,
            TuneHeader::new(vec![
                HeaderLine::Field(
                    InfoField::new('X', "1".to_string()),
                    Some(Comment::Comment("Comment 1".to_string()))
                ),
                HeaderLine::Field(
                    InfoField::new('T', "Some Title".to_string()),
                    Some(Comment::Comment("Comment 2".to_string()))
                ),
                HeaderLine::Field(
                    InfoField::new('O', "England".to_string()),
                    Some(Comment::Comment("Comment 3".to_string()))
                ),
                HeaderLine::Field(
                    InfoField::new('K', "G".to_string()),
                    Some(Comment::Comment("Comment 4".to_string()))
                ),
            ])
        );
    }

    #[test]
    fn tune_header_comment_preceding_whitespace() {
        let t = abc::tune_header(
            "\
X:1
T:Some Title
O:England
    % Some comment
K:G
",
        )
        .unwrap();
        assert_eq!(
            t,
            TuneHeader::new(vec![
                HeaderLine::Field(InfoField::new('X', "1".to_string()), None),
                HeaderLine::Field(InfoField::new('T', "Some Title".to_string()), None),
                HeaderLine::Field(InfoField::new('O', "England".to_string()), None),
                HeaderLine::Comment(Comment::CommentLine(
                    "    ".to_string(),
                    " Some comment".to_string()
                )),
                HeaderLine::Field(InfoField::new('K', "G".to_string()), None),
            ])
        );
    }

    #[test]
    fn body_with_music() {
        let b = abc::tune_body("A").unwrap();
        assert_eq!(
            b,
            TuneBody::new(vec![TuneLine::Music(MusicLine::new(vec![
                NoteBuilder::new(Note::A).build()
            ]))])
        )
    }

    #[test]
    fn body_with_music_2() {
        let b = abc::tune_body("A\n").unwrap();
        assert_eq!(
            b,
            TuneBody::new(vec![TuneLine::Music(MusicLine::new(vec![
                NoteBuilder::new(Note::A).build()
            ]))])
        )
    }

    #[test]
    fn tune_line_comment_line_1() {
        let m = abc::tune_line("% This is a comment\n").unwrap();
        assert_eq!(
            m,
            TuneLine::Comment(Comment::CommentLine(
                "".to_string(),
                " This is a comment".to_string()
            ))
        )
    }

    #[test]
    fn tune_line_comment_line_2() {
        let m = abc::tune_line("% This is a comment").unwrap();
        assert_eq!(
            m,
            TuneLine::Comment(Comment::CommentLine(
                "".to_string(),
                " This is a comment".to_string()
            ))
        )
    }

    #[test]
    fn tune_line_comment_line_3() {
        let m = abc::tune_line("    % This is a comment").unwrap();
        assert_eq!(
            m,
            TuneLine::Comment(Comment::CommentLine(
                "    ".to_string(),
                " This is a comment".to_string()
            ))
        )
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
    fn music_line_comment_err() {
        abc::music_line("% This is a comment\n").unwrap_err();
    }

    #[test]
    fn music_line_comment_line() {
        // The parser would never produce this when invoked through a higher
        // rule like `tune_line`. This would produce a TuneLine::Comment instead.
        let m = abc::music_line("    % This is a comment").unwrap();
        assert_eq!(
            m,
            MusicLine::new(vec![
                MusicSymbol::Space("    ".to_string()),
                MusicSymbol::Comment(Comment::Comment(" This is a comment".to_string()))
            ])
        )
    }

    #[test]
    fn music_line() {
        let m = abc::music_line("A\n").unwrap();
        assert_eq!(m, MusicLine::new(vec![NoteBuilder::new(Note::A).build()]))
    }

    #[test]
    fn music_line_2() {
        let m = abc::music_line("A").unwrap();
        assert_eq!(m, MusicLine::new(vec![NoteBuilder::new(Note::A).build()]))
    }

    #[test]
    fn music_line_multiple_notes() {
        let m = abc::music_line("AB cd").unwrap();
        assert_eq!(
            m,
            MusicLine::new(vec![
                NoteBuilder::new(Note::A).build(),
                NoteBuilder::new(Note::B).build(),
                MusicSymbol::Space(" ".to_string()),
                NoteBuilder::new(Note::C).octave(2).build(),
                NoteBuilder::new(Note::D).octave(2).build(),
            ])
        )
    }

    #[test]
    fn music_line_trailing_comment() {
        let m = abc::music_line("AB cd% Inline comment\n").unwrap();
        assert_eq!(
            m,
            MusicLine::new(vec![
                NoteBuilder::new(Note::A).build(),
                NoteBuilder::new(Note::B).build(),
                MusicSymbol::Space(" ".to_string()),
                NoteBuilder::new(Note::C).octave(2).build(),
                NoteBuilder::new(Note::D).octave(2).build(),
                MusicSymbol::Comment(Comment::Comment(" Inline comment".to_string())),
            ])
        )
    }

    #[test]
    fn music_line_chord() {
        let m =
            abc::music_line("( \"^I\" !f! [CEG]- > [CEG] \"^IV\" [F=AC]3/2\"^V\"[GBD]/  H[CEG]2 )")
                .unwrap();
        assert_eq!(
            m,
            MusicLine::new(vec![
                MusicSymbol::Slur(Slur::Begin),
                MusicSymbol::Space(" ".to_string()),
                MusicSymbol::Annotation(Annotation::new(Some(Placement::Above), "I".to_string())),
                MusicSymbol::Space(" ".to_string()),
                MusicSymbol::Decoration(Decoration::Unresolved("f".to_string())),
                MusicSymbol::Space(" ".to_string()),
                MusicSymbol::BrokenRhythm {
                    rhythm: ">".to_string(),
                    before: vec![
                        MusicSymbol::Chord {
                            notes: vec![
                                NoteBuilder::new(Note::C).build(),
                                NoteBuilder::new(Note::E).build(),
                                NoteBuilder::new(Note::G).build(),
                            ],
                            length: None,
                            tie: Some(Tie::Solid),
                        },
                        MusicSymbol::Space(" ".to_string())
                    ],
                    after: vec![
                        MusicSymbol::Space(" ".to_string()),
                        MusicSymbol::Chord {
                            notes: vec![
                                NoteBuilder::new(Note::C).build(),
                                NoteBuilder::new(Note::E).build(),
                                NoteBuilder::new(Note::G).build(),
                            ],
                            length: None,
                            tie: None,
                        }
                    ]
                },
                MusicSymbol::Space(" ".to_string()),
                MusicSymbol::Annotation(Annotation::new(Some(Placement::Above), "IV".to_string())),
                MusicSymbol::Space(" ".to_string()),
                MusicSymbol::Chord {
                    notes: vec![
                        NoteBuilder::new(Note::F).build(),
                        NoteBuilder::new(Note::A)
                            .accidental(Accidental::Natural)
                            .build(),
                        NoteBuilder::new(Note::C).build(),
                    ],
                    length: Some(Length::new(3.0 / 2.0)),
                    tie: None,
                },
                MusicSymbol::Annotation(Annotation::new(Some(Placement::Above), "V".to_string())),
                MusicSymbol::Chord {
                    notes: vec![
                        NoteBuilder::new(Note::G).build(),
                        NoteBuilder::new(Note::B).build(),
                        NoteBuilder::new(Note::D).build(),
                    ],
                    length: Some(Length::new(0.5)),
                    tie: None,
                },
                MusicSymbol::Space("  ".to_string()),
                MusicSymbol::Decoration(Decoration::Fermata),
                MusicSymbol::Chord {
                    notes: vec![
                        NoteBuilder::new(Note::C).build(),
                        NoteBuilder::new(Note::E).build(),
                        NoteBuilder::new(Note::G).build(),
                    ],
                    length: Some(Length::new(2.0)),
                    tie: None,
                },
                MusicSymbol::Space(" ".to_string()),
                MusicSymbol::Slur(Slur::End),
            ])
        );
    }

    #[test]
    fn music_line_beams_1() {
        let m = abc::music_line("A2``B``C").unwrap();
        assert_eq!(
            m,
            MusicLine::new(vec![
                NoteBuilder::new(Note::A).length(2.0).build(),
                MusicSymbol::Beam("``".to_string()),
                NoteBuilder::new(Note::B).build(),
                MusicSymbol::Beam("``".to_string()),
                NoteBuilder::new(Note::C).build(),
            ])
        )
    }

    #[test]
    fn music_line_beams_2() {
        let m = abc::music_line("```A2``").unwrap();
        assert_eq!(
            m,
            MusicLine::new(vec![
                MusicSymbol::Beam("```".to_string()),
                NoteBuilder::new(Note::A).length(2.0).build(),
                MusicSymbol::Beam("``".to_string()),
            ])
        )
    }

    #[test]
    fn symbol_line_err_1() {
        abc::symbol_line("s: A\n").unwrap_err();
    }
    #[test]
    fn symbol_line_err_2() {
        abc::symbol_line("s: z\n").unwrap_err();
    }
    #[test]
    fn symbol_line_err_3() {
        abc::symbol_line("s: [AB]\n").unwrap_err();
    }
    #[test]
    fn symbol_line_err_4() {
        abc::symbol_line("s: {AB}\n").unwrap_err();
    }
    #[test]
    fn symbol_line_err_5() {
        abc::symbol_line("s: (3abc\n").unwrap_err();
    }

    #[test]
    fn symbol_line_1() {
        let m = abc::symbol_line("s: |\n").unwrap();
        assert_eq!(
            m,
            SymbolLine::new(vec![
                SymbolLineSymbol::Space(" ".to_string()),
                SymbolLineSymbol::SymbolAlignment(SymbolAlignment::Bar),
            ])
        )
    }

    #[test]
    fn symbol_line_2() {
        let m = abc::symbol_line("s: \"C\" *  \"Am\" * |\n").unwrap();
        assert_eq!(
            m,
            SymbolLine::new(vec![
                SymbolLineSymbol::Space(" ".to_string()),
                SymbolLineSymbol::Annotation(Annotation::new(None, "C".to_string())),
                SymbolLineSymbol::Space(" ".to_string()),
                SymbolLineSymbol::SymbolAlignment(SymbolAlignment::Skip),
                SymbolLineSymbol::Space("  ".to_string()),
                SymbolLineSymbol::Annotation(Annotation::new(None, "Am".to_string())),
                SymbolLineSymbol::Space(" ".to_string()),
                SymbolLineSymbol::SymbolAlignment(SymbolAlignment::Skip),
                SymbolLineSymbol::Space(" ".to_string()),
                SymbolLineSymbol::SymbolAlignment(SymbolAlignment::Bar),
            ])
        )
    }

    #[test]
    fn lyric_line_1() {
        let m = abc::lyric_line("w: doh\n").unwrap();
        assert_eq!(
            m,
            LyricLine::new(vec![
                LyricSymbol::Space(" ".to_string()),
                LyricSymbol::Syllable("doh".to_string()),
            ])
        )
    }

    #[test]
    fn lyric_line_2() {
        let m = abc::lyric_line("w: doh re mi fa").unwrap();
        assert_eq!(
            m,
            LyricLine::new(vec![
                LyricSymbol::Space(" ".to_string()),
                LyricSymbol::Syllable("doh".to_string()),
                LyricSymbol::Space(" ".to_string()),
                LyricSymbol::Syllable("re".to_string()),
                LyricSymbol::Space(" ".to_string()),
                LyricSymbol::Syllable("mi".to_string()),
                LyricSymbol::Space(" ".to_string()),
                LyricSymbol::Syllable("fa".to_string()),
            ])
        )
    }

    #[test]
    fn lyric_line_3() {
        let m = abc::lyric_line("w: syll-a-ble").unwrap();
        assert_eq!(
            m,
            LyricLine::new(vec![
                LyricSymbol::Space(" ".to_string()),
                LyricSymbol::Syllable("syll".to_string()),
                LyricSymbol::SymbolAlignment(SymbolAlignment::Break),
                LyricSymbol::Syllable("a".to_string()),
                LyricSymbol::SymbolAlignment(SymbolAlignment::Break),
                LyricSymbol::Syllable("ble".to_string()),
            ])
        )
    }

    #[test]
    fn lyric_line_4() {
        let m = abc::lyric_line("w: Sa-ys my au-l' aul' wan, Will~ye come dar-gle?").unwrap();
        assert_eq!(
            m,
            LyricLine::new(vec![
                LyricSymbol::Space(" ".to_string()),
                LyricSymbol::Syllable("Sa".to_string()),
                LyricSymbol::SymbolAlignment(SymbolAlignment::Break),
                LyricSymbol::Syllable("ys".to_string()),
                LyricSymbol::Space(" ".to_string()),
                LyricSymbol::Syllable("my".to_string()),
                LyricSymbol::Space(" ".to_string()),
                LyricSymbol::Syllable("au".to_string()),
                LyricSymbol::SymbolAlignment(SymbolAlignment::Break),
                LyricSymbol::Syllable("l'".to_string()),
                LyricSymbol::Space(" ".to_string()),
                LyricSymbol::Syllable("aul'".to_string()),
                LyricSymbol::Space(" ".to_string()),
                LyricSymbol::Syllable("wan,".to_string()),
                LyricSymbol::Space(" ".to_string()),
                LyricSymbol::Syllable("Will".to_string()),
                LyricSymbol::SymbolAlignment(SymbolAlignment::Space),
                LyricSymbol::Syllable("ye".to_string()),
                LyricSymbol::Space(" ".to_string()),
                LyricSymbol::Syllable("come".to_string()),
                LyricSymbol::Space(" ".to_string()),
                LyricSymbol::Syllable("dar".to_string()),
                LyricSymbol::SymbolAlignment(SymbolAlignment::Break),
                LyricSymbol::Syllable("gle?".to_string()),
            ])
        )
    }

    #[test]
    fn note() {
        let s = abc::music_symbol("A").unwrap();
        assert_eq!(s, NoteBuilder::new(Note::A).build())
    }

    macro_rules! assert_note_len {
        ($input:tt, $expected:expr) => {
            assert_eq!(
                abc::music_symbol($input).unwrap(),
                NoteBuilder::new(Note::A).length($expected).build()
            )
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
        abc::music_symbol("I").unwrap_err();
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
        abc::music_symbol("j").unwrap_err();
    }
    #[test]
    fn note_invalid_5() {
        abc::music_symbol("m").unwrap_err();
    }
    #[test]
    fn note_invalid_6() {
        abc::music_symbol("U").unwrap_err();
    }

    macro_rules! assert_annotation {
        ($input:tt, $placement:expr, $expected: tt) => {
            assert_eq!(
                abc::music_symbol($input).unwrap(),
                MusicSymbol::Annotation(Annotation::new($placement, $expected.to_string()))
            )
        };
    }

    #[test]
    fn annotation_1() {
        assert_annotation!("\"^foo\"", Some(Placement::Above), "foo")
    }
    #[test]
    fn annotation_2() {
        assert_annotation!("\"_foo\"", Some(Placement::Below), "foo")
    }
    #[test]
    fn annotation_3() {
        assert_annotation!("\"<foo\"", Some(Placement::Left), "foo")
    }
    #[test]
    fn annotation_4() {
        assert_annotation!("\">foo\"", Some(Placement::Right), "foo")
    }
    #[test]
    fn annotation_5() {
        assert_annotation!("\"@foo\"", Some(Placement::Auto), "foo")
    }
    #[test]
    fn annotation_6() {
        assert_annotation!("\"foo\"", None, "foo")
    }

    macro_rules! assert_bar {
        ($bar:tt) => {
            assert_eq!(
                abc::music_symbol($bar).unwrap(),
                MusicSymbol::Bar($bar.to_string(), None)
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
    #[test]
    fn bar_9() {
        assert_bar!(".|")
    }
    #[test]
    fn bar_10() {
        assert_bar!("[|:::")
    }

    #[test]
    fn beam_1() {
        assert_eq!(
            abc::music_symbol("```").unwrap(),
            MusicSymbol::Beam("```".to_string())
        )
    }

    #[test]
    fn slur_1() {
        assert_eq!(
            abc::music_symbol("(").unwrap(),
            MusicSymbol::Slur(Slur::Begin)
        )
    }
    #[test]
    fn slur_2() {
        assert_eq!(
            abc::music_symbol(".(").unwrap(),
            MusicSymbol::Slur(Slur::BeginDotted)
        )
    }
    #[test]
    fn slur_3() {
        assert_eq!(
            abc::music_symbol(")").unwrap(),
            MusicSymbol::Slur(Slur::End)
        )
    }

    macro_rules! assert_dec {
        ($input:tt, $expected:expr) => {
            assert_eq!(
                abc::music_symbol($input).unwrap(),
                MusicSymbol::Decoration($expected)
            )
        };
    }

    #[test]
    fn decoration_1() {
        assert_dec!(".", Decoration::Staccato)
    }
    #[test]
    fn decoration_2() {
        assert_dec!("~", Decoration::Roll)
    }
    #[test]
    fn decoration_3() {
        assert_dec!("H", Decoration::Fermata)
    }
    #[test]
    fn decoration_4() {
        assert_dec!("L", Decoration::Accent)
    }
    #[test]
    fn decoration_5() {
        assert_dec!("M", Decoration::LowerMordent)
    }
    #[test]
    fn decoration_6() {
        assert_dec!("O", Decoration::Coda)
    }
    #[test]
    fn decoration_7() {
        assert_dec!("P", Decoration::UpperMordent)
    }
    #[test]
    fn decoration_8() {
        assert_dec!("S", Decoration::Segno)
    }
    #[test]
    fn decoration_9() {
        assert_dec!("T", Decoration::Trill)
    }
    #[test]
    fn decoration_10() {
        assert_dec!("u", Decoration::UpBow)
    }
    #[test]
    fn decoration_11() {
        assert_dec!("v", Decoration::DownBow)
    }
    #[test]
    fn decoration_12() {
        assert_dec!("!somedec!", Decoration::Unresolved("somedec".to_string()))
    }

    #[test]
    fn decoration_invalid() {
        abc::music_symbol("!A").unwrap_err();
    }

    macro_rules! assert_acc {
        ($input:tt, $expected:expr) => {
            assert_eq!(
                abc::music_symbol($input).unwrap(),
                NoteBuilder::new(Note::A).accidental($expected).build()
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
                NoteBuilder::new(Note::A).octave($expected).build()
            )
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
                NoteBuilder::new(Note::A).tie(Tie::Solid).build(),
                NoteBuilder::new(Note::A).build(),
            ])
        );
    }

    #[test]
    fn tie_across_bars() {
        assert_eq!(
            abc::music_line("abc-|cba").unwrap(),
            MusicLine::new(vec![
                NoteBuilder::new(Note::A).octave(2).build(),
                NoteBuilder::new(Note::B).octave(2).build(),
                NoteBuilder::new(Note::C).octave(2).tie(Tie::Solid).build(),
                MusicSymbol::Bar("|".to_string(), None),
                NoteBuilder::new(Note::C).octave(2).build(),
                NoteBuilder::new(Note::B).octave(2).build(),
                NoteBuilder::new(Note::A).octave(2).build(),
            ])
        );
    }

    #[test]
    fn tie_with_length() {
        assert_eq!(
            abc::music_line("c4-c4").unwrap(),
            MusicLine::new(vec![
                NoteBuilder::new(Note::C)
                    .octave(2)
                    .length(4.0)
                    .tie(Tie::Solid)
                    .build(),
                NoteBuilder::new(Note::C).octave(2).length(4.0).build(),
            ])
        );
    }

    #[test]
    fn tie_dotted() {
        assert_eq!(
            abc::music_line("C.-C").unwrap(),
            MusicLine::new(vec![
                NoteBuilder::new(Note::C).tie(Tie::Dotted).build(),
                NoteBuilder::new(Note::C).build(),
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
        assert_rst!("z", Rest::Note(None))
    }
    #[test]
    fn rest_2() {
        assert_rst!("x", Rest::NoteHidden(None))
    }
    #[test]
    fn rest_3() {
        assert_rst!("Z", Rest::Measure(None))
    }
    #[test]
    fn rest_4() {
        assert_rst!("X", Rest::MeasureHidden(None))
    }
    #[test]
    fn rest_5() {
        assert_rst!("z3", Rest::Note(Some(Length::new(3.0))))
    }
    #[test]
    fn rest_6() {
        assert_rst!("Z10", Rest::Measure(Some(Length::new(10.0))))
    }
    #[test]
    fn rest_7() {
        assert_rst!("x7", Rest::NoteHidden(Some(Length::new(7.0))))
    }
    #[test]
    fn rest_8() {
        assert_rst!("X900", Rest::MeasureHidden(Some(Length::new(900.0))))
    }

    #[test]
    fn spacer() {
        assert_eq!(abc::music_symbol("y").unwrap(), MusicSymbol::Spacer)
    }

    #[test]
    fn endings_error_1() {
        abc::ending("[abc").unwrap_err();
    }
    #[test]
    fn endings_error_2() {
        abc::ending("[1-").unwrap_err();
    }
    #[test]
    fn endings_error_3() {
        abc::ending("[1,").unwrap_err();
    }
    #[test]
    fn endings_error_4() {
        abc::ending("[1-3,").unwrap_err();
    }

    #[test]
    fn endings_general() {
        let m = abc::music_line("f|[1 d:|[2 d2 B|]").unwrap();
        assert_eq!(
            m,
            MusicLine::new(vec![
                NoteBuilder::new(Note::F).octave(2).build(),
                MusicSymbol::Bar("|".to_string(), None),
                MusicSymbol::Ending("1".to_string()),
                MusicSymbol::Space(" ".to_string()),
                NoteBuilder::new(Note::D).octave(2).build(),
                MusicSymbol::Bar(":|".to_string(), None),
                MusicSymbol::Ending("2".to_string()),
                MusicSymbol::Space(" ".to_string()),
                NoteBuilder::new(Note::D).octave(2).length(2.0).build(),
                MusicSymbol::Space(" ".to_string()),
                NoteBuilder::new(Note::B).build(),
                MusicSymbol::Bar("|]".to_string(), None)
            ])
        );
    }

    #[test]
    fn endings_1() {
        assert_eq!(
            abc::ending("[1").unwrap(),
            MusicSymbol::Ending("1".to_string())
        )
    }
    #[test]
    fn endings_2() {
        assert_eq!(
            abc::ending("[2").unwrap(),
            MusicSymbol::Ending("2".to_string())
        )
    }
    #[test]
    fn endings_3() {
        assert_eq!(
            abc::ending("[87654").unwrap(),
            MusicSymbol::Ending("87654".to_string())
        )
    }
    #[test]
    fn endings_4() {
        assert_eq!(
            abc::ending("[1,3").unwrap(),
            MusicSymbol::Ending("1,3".to_string())
        )
    }
    #[test]
    fn endings_5() {
        assert_eq!(
            abc::ending("[1-3").unwrap(),
            MusicSymbol::Ending("1-3".to_string())
        )
    }
    #[test]
    fn endings_6() {
        assert_eq!(
            abc::ending("[1,3,5-7").unwrap(),
            MusicSymbol::Ending("1,3,5-7".to_string())
        )
    }
    #[test]
    fn endings_7() {
        assert_eq!(
            abc::ending("[1,3,5-7,8-10").unwrap(),
            MusicSymbol::Ending("1,3,5-7,8-10".to_string())
        )
    }

    #[test]
    fn ending_after_bar() {
        let m = abc::music_line("|[1").unwrap();
        assert_eq!(
            m,
            MusicLine::new(vec![
                MusicSymbol::Bar("|".to_string(), None),
                MusicSymbol::Ending("1".to_string())
            ])
        )
    }

    #[test]
    fn ending_combined_with_bar() {
        let m = abc::music_line("|1").unwrap();
        assert_eq!(
            m,
            MusicLine::new(vec![MusicSymbol::Bar(
                "|".to_string(),
                Some("1".to_string())
            )])
        )
    }

    #[test]
    fn chord_1() {
        let c = abc::music_symbol("[CEG]").unwrap();
        assert_eq!(
            c,
            MusicSymbol::Chord {
                notes: vec![
                    NoteBuilder::new(Note::C).build(),
                    NoteBuilder::new(Note::E).build(),
                    NoteBuilder::new(Note::G).build(),
                ],
                length: None,
                tie: None,
            }
        );
    }

    #[test]
    fn chord_2() {
        let c = abc::music_symbol("[C2E2G2]3").unwrap();
        assert_eq!(
            c,
            MusicSymbol::Chord {
                notes: vec![
                    NoteBuilder::new(Note::C).length(2.0).build(),
                    NoteBuilder::new(Note::E).length(2.0).build(),
                    NoteBuilder::new(Note::G).length(2.0).build(),
                ],
                length: Some(Length::new(3.0)),
                tie: None,
            }
        );
    }

    #[test]
    fn chord_3() {
        let c = abc::music_symbol("[!1!C!3!E!5!G]").unwrap();
        assert_eq!(
            c,
            MusicSymbol::Chord {
                notes: vec![
                    MusicSymbol::Decoration(Decoration::Unresolved("1".to_string())),
                    NoteBuilder::new(Note::C).build(),
                    MusicSymbol::Decoration(Decoration::Unresolved("3".to_string())),
                    NoteBuilder::new(Note::E).build(),
                    MusicSymbol::Decoration(Decoration::Unresolved("5".to_string())),
                    NoteBuilder::new(Note::G).build(),
                ],
                length: None,
                tie: None,
            }
        );
    }

    #[test]
    fn grace_notes_empty() {
        abc::music_symbol("{}").unwrap_err();
    }
    #[test]
    fn grace_notes_err_1() {
        abc::music_symbol("{vA}").unwrap_err();
    }
    #[test]
    fn grace_notes_err_2() {
        abc::music_symbol("{HA}").unwrap_err();
    }
    #[test]
    fn grace_notes_err_3() {
        abc::music_symbol("{!fermata!A}").unwrap_err();
    }

    #[test]
    fn grace_notes_1() {
        let g = abc::music_symbol("{GdGe}").unwrap();
        assert_eq!(
            g,
            MusicSymbol::GraceNotes {
                acciaccatura: None,
                notes: vec![
                    NoteBuilder::new(Note::G).build(),
                    NoteBuilder::new(Note::D).octave(2).build(),
                    NoteBuilder::new(Note::G).build(),
                    NoteBuilder::new(Note::E).octave(2).build()
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
                notes: vec![NoteBuilder::new(Note::G).octave(2).build()]
            }
        )
    }
    #[test]
    fn grace_notes_3() {
        let g = abc::music_symbol("{A<B}").unwrap();
        assert_eq!(
            g,
            MusicSymbol::GraceNotes {
                acciaccatura: None,
                notes: vec![MusicSymbol::BrokenRhythm {
                    rhythm: "<".to_string(),
                    before: vec![NoteBuilder::new(Note::A).build()],
                    after: vec![NoteBuilder::new(Note::B).build()],
                }]
            }
        )
    }
    #[test]
    fn grace_notes_4() {
        let g = abc::music_symbol("{A B}").unwrap();
        assert_eq!(
            g,
            MusicSymbol::GraceNotes {
                acciaccatura: None,
                notes: vec![
                    NoteBuilder::new(Note::A).build(),
                    MusicSymbol::Space(" ".to_string()),
                    NoteBuilder::new(Note::B).build(),
                ]
            }
        )
    }

    macro_rules! assert_tup {
        ($input:tt, $p:expr, $q:expr, $r:expr) => {
            assert_eq!(
                abc::music_symbol($input).unwrap(),
                MusicSymbol::Tuplet {
                    p: $p,
                    q: $q,
                    r: $r,
                }
            )
        };
    }

    #[test]
    fn tuplet_1() {
        assert_tup!("(2", 2, None, None)
    }
    #[test]
    fn tuplet_2() {
        assert_tup!("(3", 3, None, None)
    }
    #[test]
    fn tuplet_3() {
        assert_tup!("(3::", 3, None, None)
    }
    #[test]
    fn tuplet_5() {
        assert_tup!("(3:2", 3, Some(2), None)
    }
    #[test]
    fn tuplet_6() {
        assert_tup!("(3:2:3", 3, Some(2), Some(3))
    }
    #[test]
    fn tuplet_7() {
        assert_tup!("(9", 9, None, None)
    }
    #[test]
    fn tuplet_8() {
        assert_eq!(
            abc::music_symbol("(3:2:2").unwrap(),
            MusicSymbol::Tuplet {
                p: 3,
                q: Some(2),
                r: Some(2),
            }
        )
    }

    #[test]
    fn broken_rhythm_1() {
        assert_eq!(
            abc::music_symbol("A<B").unwrap(),
            MusicSymbol::BrokenRhythm {
                rhythm: "<".to_string(),
                before: vec![NoteBuilder::new(Note::A).build()],
                after: vec![NoteBuilder::new(Note::B).build()],
            }
        )
    }

    #[test]
    fn broken_rhythm_2() {
        assert_eq!(
            abc::music_symbol("A>>>B").unwrap(),
            MusicSymbol::BrokenRhythm {
                rhythm: ">>>".to_string(),
                before: vec![NoteBuilder::new(Note::A).build()],
                after: vec![NoteBuilder::new(Note::B).build()],
            }
        )
    }

    #[test]
    fn broken_rhythm_3() {
        assert_eq!(
            abc::music_symbol("z<B").unwrap(),
            MusicSymbol::BrokenRhythm {
                rhythm: "<".to_string(),
                before: vec![MusicSymbol::Rest(Rest::Note(None)),],
                after: vec![NoteBuilder::new(Note::B).build()],
            }
        )
    }

    #[test]
    fn broken_rhythm_4() {
        assert_eq!(
            abc::music_symbol("A < B").unwrap(),
            MusicSymbol::BrokenRhythm {
                rhythm: "<".to_string(),
                before: vec![
                    NoteBuilder::new(Note::A).build(),
                    MusicSymbol::Space(" ".to_string())
                ],
                after: vec![
                    MusicSymbol::Space(" ".to_string()),
                    NoteBuilder::new(Note::B).build()
                ],
            }
        )
    }

    #[test]
    fn broken_rhythm_5() {
        assert_eq!(
            abc::music_symbol("[AB] < B").unwrap(),
            MusicSymbol::BrokenRhythm {
                rhythm: "<".to_string(),
                before: vec![
                    MusicSymbol::Chord {
                        notes: vec![
                            NoteBuilder::new(Note::A).build(),
                            NoteBuilder::new(Note::B).build()
                        ],
                        length: None,
                        tie: None,
                    },
                    MusicSymbol::Space(" ".to_string()),
                ],
                after: vec![
                    MusicSymbol::Space(" ".to_string()),
                    NoteBuilder::new(Note::B).build()
                ],
            }
        )
    }

    #[ignore]
    #[test]
    fn broken_rhythm_6() {
        assert_eq!(
            abc::music_symbol("A{g}<A").unwrap(),
            MusicSymbol::BrokenRhythm {
                rhythm: "<".to_string(),
                before: vec![MusicSymbol::GraceNotes {
                    acciaccatura: None,
                    notes: vec![NoteBuilder::new(Note::G).octave(2).build()]
                }],
                after: vec![NoteBuilder::new(Note::B).build()],
            }
        )
    }

    #[test]
    fn broken_rhythm_recursive() {
        assert_eq!(
            abc::music_symbol("A<B<C").unwrap(),
            MusicSymbol::BrokenRhythm {
                rhythm: "<".to_string(),
                before: vec![NoteBuilder::new(Note::A).build()],
                after: vec![MusicSymbol::BrokenRhythm {
                    rhythm: "<".to_string(),
                    before: vec![NoteBuilder::new(Note::B).build()],
                    after: vec![NoteBuilder::new(Note::C).build()],
                }],
            }
        )
    }

    #[test]
    fn inline_field() {
        assert_eq!(
            abc::music_symbol("[M:9/8]").unwrap(),
            MusicSymbol::InlineField(InfoField('M', "9/8".to_string()), true),
        )
    }

    #[test]
    fn inline_field_line() {
        assert_eq!(
            abc::tune_line("M:9/8\n").unwrap(),
            TuneLine::Music(MusicLine::new(vec![MusicSymbol::InlineField(
                InfoField('M', "9/8".to_string()),
                false
            )])),
        )
    }

    #[test]
    fn inline_field_err_1() {
        abc::music_symbol("[M:9/8\n").unwrap_err();
    }
    #[test]
    fn inline_field_err_2() {
        abc::music_symbol("[C:Trad.]").unwrap_err();
    }

    macro_rules! assert_reserved {
        ($input:expr) => {
            assert_eq!(
                abc::music_symbol($input).unwrap(),
                MusicSymbol::Reserved($input.to_string())
            )
        };
    }

    #[test]
    fn reserved_1() {
        assert_reserved!("#");
    }
    #[test]
    fn reserved_2() {
        assert_reserved!("*");
    }
    #[test]
    fn reserved_3() {
        assert_reserved!(";");
    }
    #[test]
    fn reserved_4() {
        assert_reserved!("?");
    }
    #[test]
    fn reserved_5() {
        assert_reserved!("@");
    }
}
