use pretty_assertions::assert_eq;

use abc_parser::abc;
use abc_parser::datatypes::writer::ToABC;

#[test]
fn to_ast_and_back_again() {
    let data = "\
%
% Some comments at the start of the file
%
   % Comment with preceding whitespace

%
% Some more comments following an empty line
%

M:4/4
O:Irish
R:Reel   % Field with comment

% Comment at start of tune header
%%newpage
X:1
% Comment before title
   % Comment with preceding whitespace
T:Untitled Reel
C:Trad.  % Field with comment
H:History field with some
% comment between
+:continued text on the next line
m:~n2 = (3o/n/m/ n
%%papersize A4
% Comment in tune header
   % Comment with preceding whitespace
K:D
% Single line comment
   % Comment with preceding whitespace
eg|a2-ab ageg|agbg agef|g2g2 fgag|f2d2 d2:| % Inline comment
ed|cecA B2ed|cAcA E2ed|cecA B2ed|c2A2 A2:|
AB|cdec BcdB|ABAF GFE2|cdec BcdB|c2A2 A2:|
% Comments and remarks
|:DEF FED| % this is an end of line comment
% this is a comment line
DEF [r:and this is a remark] FED:|
% Broken Rhythms
a>b c<d abcd
% Rests
zzz2 | xxx2 | Z | Z2 | Z24 | X | X4 | z/
% Barlines
|] AAAA || AAAA [| AAAA |: AAAA :: AAAA :|: AAAA :||: AAAA :|
AAAA .| AAAA [|] AAAA |[| AAAA [|:::
% First and second repeats
faf gfe|[1 dfe dBA:|[2 d2e dcB|]
faf gfe|1 dfe dBA:|2 d2e dcB|]
% Variant endings
P:A
abcd | [1  abcd  :| [2 abcd :| [3 abcd :| [4 abcd |]
[1,3 abcd :| [1-3 abcd :| [1,3,5-7  abcd  :| [2,4,8 abcd :|
% Slurs
(DEFG) | ( D E F G ) | (c (d e f) g a) | (c d (e) f g a) | .(cde)
% Grace Notes
A<{g}A | A{g}<A
{/g}C | {/gagab}C | {g}A | {gagab} A
{a3b/} | {a>b}
% Tuplets
(2ab (3abc (4abcd (5abcde (6abcdef (7abcdefg (8abcdefga (9abcdefgab
(3:2:3 ABC (3:2:2 G4c2 (3:2:2 G4c2 | (3:2:4 G2A2Bc
% Decorations
(3.a.b.c % staccato triplet
vAuBvA % bowing marks (for fiddlers)
[!1!C!3!E!5!G]  !coda! y  !p! !trill! C   !fermata!|
!sintro!(B2 A2) | F6 !eintro!|]
% Symbol lines
     CDEF    | G```AB`c
s: \"^slow\" | !f! ** !fff!
   C2  C2 Ez   A2|
s: \"C\" *  \"Am\" * |
s: *   *  !>!  * |
\"C\" C2 C2 \"Am\" !>! Ez A2|
% Chords
[!1!C!3!E!5!G]
[CEGc] | [d2f2][ce][df]
( \"^I\" !f! [CEG]- > [CEG] \"^IV\" [F=AC]3/2\"^V\"[GBD]/  H[CEG]2 )
% Inline fields
E2E EFE|[K:G]E2E EFG|[M:9/8] A2G F2E D2|]
E2E EFE|\\
K:G
E2E EFG|\\
M:9/8
A2G F2E D2|]
% Annotations and Chord Symbols
\"<(\" \">)\" C
\"Amaj\" AAAA | \"@Important note\"BBBB | \"_Underneath\" CCCC
% Lyrics
C D E F|
w: doh re mi fa
G G G G|
w:
F E F C|
w: fa mi re doh
gf|e2dc B2A2|B2G2 E2D2|.G2.G2 GABc|d4 B2
w: Sa-ys my au-l' wan to your aul' wan, Will~ye come to the Wa-x-ies dar-gle?
w:Sa-ys my au-l' wan to your aul' wan,
+:will~ye come to the Wa-x-ies dar-gle?
% Order of ABC constructs
\"C\"[CEGc]|
|\"Gm7\"[.=G,^c']
\"Gm7\"(v.=G,2~^c'2)
% Spacers
\"Am\" !pp! y
% Reserved characters
@a !pp! #bc/3* [K:C#] de?f \"@this $2was difficult to parse?\" y |**
";
    let tb = abc::tune_book(data).unwrap();
    let formatted = tb.to_abc();

    assert_eq!(data, formatted);
    abc::tune_book(&formatted).unwrap();
}

#[test]
fn to_ast_and_back_again_multiple_voices_1() {
    let data = "\
X:1
T:Zocharti Loch
C:Louis Lewandowski (1821-1894)
M:C
Q:1/4=76
%%score (T1 T2) (B1 B2)
V:T1           clef=treble-8  name=\"Tenore I\"   snm=\"T.I\"
V:T2           clef=treble-8  name=\"Tenore II\"  snm=\"T.II\"
V:B1  middle=d clef=bass      name=\"Basso I\"    snm=\"B.I\"  transpose=-24
V:B2  middle=d clef=bass      name=\"Basso II\"   snm=\"B.II\" transpose=-24
K:Gm
%            End of header, start of tune body:
% 1
[V:T1]  (B2c2 d2g2)  | f6e2      | (d2c2 d2)e2 | d4 c2z2 |
[V:T2]  (G2A2 B2e2)  | d6c2      | (B2A2 B2)c2 | B4 A2z2 |
[V:B1]       z8      | z2f2 g2a2 | b2z2 z2 e2  | f4 f2z2 |
[V:B2]       x8      |     x8    |      x8     |    x8   |
% 5
[V:T1]  (B2c2 d2g2)  | f8        | d3c (d2fe)  | H d6    ||
[V:T2]       z8      |     z8    | B3A (B2c2)  | H A6    ||
[V:B1]  (d2f2 b2e'2) | d'8       | g3g  g4     | H^f6    ||
[V:B2]       x8      | z2B2 c2d2 | e3e (d2c2)  | H d6    ||
";
    let tb = abc::tune_book(data).unwrap();
    let formatted = tb.to_abc();

    assert_eq!(data, formatted);
    abc::tune_book(&formatted).unwrap();
}

#[test]
fn to_ast_and_back_again_multiple_voices_2() {
    let data = "\
X:2
T:Zocharti Loch
%...skipping rest of the header...
K:Gm
%               Start of tune body:
V:T1
 (B2c2 d2g2) | f6e2 | (d2c2 d2)e2 | d4 c2z2 |
 (B2c2 d2g2) | f8 | d3c (d2fe) | H d6 ||
V:T2
 (G2A2 B2e2) | d6c2 | (B2A2 B2)c2 | B4 A2z2 |
 z8 | z8 | B3A (B2c2) | H A6 ||
V:B1
 z8 | z2f2 g2a2 | b2z2 z2 e2 | f4 f2z2 |
 (d2f2 b2e'2) | d'8 | g3g  g4 | H^f6 ||
V:B2
 x8 | x8 | x8 | x8 |
 x8 | z2B2 c2d2 | e3e (d2c2) | H d6 ||
";
    let tb = abc::tune_book(data).unwrap();
    let formatted = tb.to_abc();

    assert_eq!(data, formatted);
    abc::tune_book(&formatted).unwrap();
}
