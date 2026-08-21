use super::{Context, State, TryIndex, Value, v};
use crate::FontError;
use nom::number::complete::{be_i32, be_u8};
use tiny_skia::Point;
use tracing::trace;

fn parse<'a, T>(
    input: &mut &'a [u8],
    parser: fn(&'a [u8]) -> crate::IResult<'a, T>,
) -> Result<T, nom::Err<nom_language::error::VerboseError<&'a [u8]>>> {
    let (i, o) = parser(input)?;
    *input = i;
    Ok(o)
}

pub fn charstring<'a, 'b, T, U>(
    mut input: &'a [u8],
    ctx: &'a Context<T, U>,
    s: &'b mut State,
) -> Result<(), FontError>
where
    T: TryIndex + 'a,
    U: TryIndex + 'a,
{
    let mut ps_stack = vec![];
    while input.len() > 0 {
        //trace!("input: {:?}", slice!(input, .. input.len().min(10)));
        trace!("current: {:?}", s.current);
        let b0 = parse(&mut input, be_u8)?;
        match b0 {
            1 => {
                // ⊦ y dy hstem (1) ⊦
                trace!("hstem");
                s.stack.clear();
            }
            3 => {
                // ⊦ x dx vstem (3) ⊦
                trace!("vstem");
                s.stack.clear();
            }
            4 => {
                // ⊦ dy vmoveto (4) ⊦
                trace!("vmoveto");
                require!(s.stack.len() >= 1);
                s.flush();

                let p = s.current + v(0., s.stack[0]);
                s.outline.move_to(p.x, p.y);
                s.stack.clear();
                s.current = p;
            }
            5 => {
                // ⊦ dx dy rlineto (5) ⊦
                trace!("rlineto");
                require!(s.stack.len() >= 2);
                let p = s.current + v(s.stack[0], s.stack[1]);
                s.outline.line_to(p.x, p.y);
                s.stack.clear();
                s.current = p;
            }
            6 => {
                // ⊦ dx hlineto (6) ⊦
                trace!("hlineto");
                require!(s.stack.len() >= 1);
                let p = s.current + v(s.stack[0], 0.);
                s.outline.line_to(p.x, p.y);
                s.stack.clear();
                s.current = p;
            }
            7 => {
                // dy vlineto (7)
                trace!("vlineto");
                require!(s.stack.len() >= 1);
                let p = s.current + v(0., s.stack[0]);
                s.outline.line_to(p.x, p.y);
                s.stack.clear();
                s.current = p;
            }
            8 => {
                // ⊦ dx1 dy1 dx2 dy2 dx3 dy3 rrcurveto (8) ⊦
                trace!("rrcurveto");
                require!(s.stack.len() >= 6);
                let c1 = s.current + v(s.stack[0], s.stack[1]);
                let c2 = c1 + v(s.stack[2], s.stack[3]);
                let p = c2 + v(s.stack[4], s.stack[5]);
                s.outline.cubic_to(c1.x, c1.y, c2.x, c2.y, p.x, p.y);
                s.stack.clear();
                s.current = p;
            }
            9 => {
                // –closepath (9) ⊦
                trace!("closepath");
                s.outline.close();
                s.stack.clear();
            }
            10 => {
                // subr# callsubr (10) –
                let subr_nr = s.pop()?.to_int()?;
                trace!("callsubr {}", subr_nr);
                let subr = ctx.subr(subr_nr)?;
                charstring(subr, ctx, s)?;
            }
            11 => {
                // return
                trace!("return");
                break;
            }
            12 => {
                let b1 = parse(&mut input, be_u8)?;
                match b1 {
                    0 => {
                        // – dotsection (12 0) ⊦
                        trace!("dotsection");
                        s.stack.clear();
                    }
                    1 => {
                        // ⊦ x0 dx0 x1 dx1 x2 dx2 vstem3 (12 1) ⊦
                        trace!("vstem3");
                        s.stack.clear();
                    }
                    2 => {
                        // ⊦ y0 dy0 y1 dy1 y2 dy2 hstem3 (12 2) ⊦
                        trace!("hstem3");
                        s.stack.clear();
                    }
                    6 => {
                        // ⊦ asb adx ady bchar achar seac (12 6) ⊦
                        trace!("seac");
                        s.stack.clear();
                    }
                    7 => {
                        // ⊦ sbx sby wx wy sbw (12 7) ⊦
                        let [sbx, sby, wx, _wy, _sbw] = s.args()?;
                        trace!("sbw");
                        s.char_width = Some(wx.to_float());
                        s.current = v(sbx, sby);
                        s.stack.clear();
                    }
                    12 => {
                        // num1 num2 div (12 12) quotient
                        trace!("div");
                        let num2 = s.pop()?.to_float();
                        let num1 = s.pop()?.to_float();
                        s.push(num1 / num2);
                    }
                    16 => {
                        //  arg1 . . . argn n othersubr# callothersubr (12 16) –
                        let subr_nr = s.pop()?.to_int()?;
                        trace!("callothersubr {}", subr_nr);
                        let n = s.pop()?.to_int()? as usize;

                        match subr_nr {
                            1 => {
                                require_eq!(n, 0);
                                s.flex_sequence = Some(Vec::with_capacity(7));

                                // first moveto: referece point
                                // then first control point of first curve
                                // second control point of first curve
                                // joining point
                                // first control point of second curve
                                // second control point of second curve
                                // endpoint of second curve
                                // (flex height, final x, final y) 0 callsubr
                            }
                            2 => {
                                require_eq!(n, 0);
                            }
                            0 => {
                                // end of flex sequences
                                require_eq!(n, 3);
                                let [_flex_height, x, y] = s.pop_tuple()?;
                                let flex_sequence =
                                    expect!(s.flex_sequence.take(), "no flex sequence");
                                require_eq!(flex_sequence.len(), 7);
                                let _ref = flex_sequence.get(0).unwrap();
                                let c0 = flex_sequence.get(1).unwrap();
                                let c1 = flex_sequence.get(2).unwrap();
                                let p2 = flex_sequence.get(3).unwrap();
                                let c3 = flex_sequence.get(4).unwrap();
                                let c4 = flex_sequence.get(5).unwrap();
                                let p5 = flex_sequence.get(6).unwrap();

                                //require_eq!(p5, v(x, y));
                                s.outline.cubic_to(c0.x, c0.y, c1.x, c1.y, p2.x, p2.y);
                                s.outline.cubic_to(c3.x, c3.y, c4.x, c4.y, p5.x, p5.y);
                                ps_stack.push(y);
                                ps_stack.push(x);
                            }
                            3 => {
                                require_eq!(n, 1);
                                ps_stack.push(s.pop()?);
                                ps_stack.push(Value::Int(3));
                            }
                            _ => {
                                let m = s.stack.len();
                                ps_stack.clear();
                                ps_stack.extend(s.stack.drain(m - n..).rev());
                            }
                        }
                    }
                    17 => {
                        // – pop (12 17) number
                        trace!("pop");
                        let n = ps_stack.pop().expect("PS stack is empty");
                        s.push(n);
                    }
                    33 => {
                        // ⊦ x y setcurrentpoint (12 33) ⊦
                        trace!("setcurrentpoint");
                        let [x, y] = s.args()?;
                        let p = v(x, y);
                        s.current = p;
                        s.stack.clear();
                    }
                    _ => panic!("invalid operator"),
                }
            }
            13 => {
                // ⊦ sbx wx hsbw (13) ⊦
                trace!("hsbw");
                let [sbx, wx] = s.args()?;
                let lsb = sbx.to_float();
                s.lsb = Some(lsb);
                s.current = Point::from_xy(lsb, 0.0);
                s.char_width = Some(wx.to_float());
                s.stack.clear();
            }
            14 => {
                //– endchar (14) ⊦
                trace!("endchar");
                break;
            }
            21 => {
                // ⊦ dx dy rmoveto (21) ⊦
                trace!("rmoveto");
                let [dx, dy] = s.args()?;
                let p = s.current + v(dx, dy);

                // hack to counter the flex sequences hack by adobe
                if let Some(ref mut points) = s.flex_sequence {
                    points.push(p);
                } else {
                    s.flush();
                    s.outline.move_to(p.x, p.y);
                }
                s.current = p;
                s.stack.clear();
            }
            22 => {
                // ⊦ dx hmoveto (22) ⊦
                trace!("hmoveto");
                let [dx] = s.args()?;
                let p = s.current + v(dx, 0.);
                s.flush();
                s.outline.move_to(p.x, p.y);
                s.current = p;
                s.stack.clear();
            }
            30 => {
                // ⊦ dy1 dx2 dy2 dx3 vhcurveto (30) ⊦
                trace!("vhcurveto");
                let [dy1, dx2, dy2, dx3] = s.args()?;
                let c1 = s.current + v(0., dy1);
                let c2 = c1 + v(dx2, dy2);
                let p = c2 + v(dx3, 0.);
                s.outline.cubic_to(c1.x, c1.y, c2.x, c2.y, p.x, p.y);
                s.stack.clear();
                s.current = p;
            }
            31 => {
                // ⊦ dx1 dx2 dy2 dy3 hvcurveto (31) ⊦
                trace!("hvcurveto");
                let [dx1, dx2, dy2, dy3] = s.args()?;
                let c1 = s.current + v(dx1, 0.);
                let c2 = c1 + v(dx2, dy2);
                let p = c2 + v(0., dy3);
                s.outline.cubic_to(c1.x, c1.y, c2.x, c2.y, p.x, p.y);
                s.stack.clear();
                s.current = p;
            }
            v @ 32..=246 => {
                s.push(v as i32 - 139);
            }
            v @ 247..=250 => {
                let w = parse(&mut input, be_u8)?;
                s.push((v as i32 - 247) * 256 + w as i32 + 108);
            }
            v @ 251..=254 => {
                let w = parse(&mut input, be_u8)?;
                s.push(-(v as i32 - 251) * 256 - w as i32 - 108);
            }
            255 => {
                let v = parse(&mut input, be_i32)?;
                s.push(v);
            }
            c => {
                //panic!("unknown code {c}");
                error!("unknown code {c}");
            }
        }

        trace!("stack: {:?}", s.stack);
    }

    Ok(())
}
