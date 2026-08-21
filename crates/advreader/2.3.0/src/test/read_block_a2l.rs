#[cfg(test)]
mod tests {
    use std::iter::Enumerate;

    use crate::{
        block::{A2LBlock, Block},
        AdvReturnValue, FnBlockReturnType,
    };

    const LINENR: usize = 1;

    fn start<'a>(
        item: &[u8],
        buf: &'a [u8],
        skip_comments: bool,
    ) -> (
        Box<A2LBlock>,
        Enumerate<std::slice::Iter<'a, u8>>,
        usize,
        isize,
        usize,
        Option<Vec<AdvReturnValue>>,
    ) {
        let mut reader = <A2LBlock as Block>::new(b'\n', skip_comments);
        let mut itr = buf.iter().enumerate();
        let mut i = 0;
        let mut c = 0;
        loop {
            (i, c) = match itr.next() {
                Some((i, c)) => (i, *c),
                None => break,
            };
            if i >= item.len() {
                break;
            }
        }
        let mut line_num: usize = 1;
        let (i, c, result) = reader.read_block(item, buf, &mut itr, i, c, &mut line_num);
        (reader, itr, i, c, line_num, result)
    }

    fn next(
        reader: &mut Box<A2LBlock>,
        buf: &[u8],
        itr: &mut Enumerate<std::slice::Iter<'_, u8>>,
        mut i: usize,
        mut c: u8,
        line_num: &mut usize,
    ) -> (usize, isize, Option<Vec<AdvReturnValue>>) {
        // Search for next token start
        if c == b'\n' {
            *line_num += 1;
        }
        let mut start = 0;
        if reader.block_mode {
            (i, c) = match itr.next() {
                Some((i, c)) => (i, *c),
                None => (i, c),
            };
        } else {
            loop {
                (i, c) = match itr.next() {
                    Some((i, c)) => (i, *c),
                    None => break,
                };
                if c == b'\n' {
                    *line_num += 1;
                }
                if c.is_ascii_alphanumeric() || c == b'_' || c == b'.' || c == b'/' {
                    break;
                }
            }
            if i >= buf.len() {
                return (i, c as isize, None);
            }
            start = i;
            // Search for token end
            loop {
                (i, c) = match itr.next() {
                    Some((i, c)) => (i, *c),
                    None => {
                        i += 1;
                        break;
                    }
                };
                if !c.is_ascii_alphanumeric() && c != b'_' && c != b'.' && c != b'/' {
                    break;
                }
            }
            if c == b'\n' {
                *line_num += 1;
            }
        }
        let item = &buf[start..i];
        if *line_num >= LINENR {
            println!("item={:?}", std::str::from_utf8(&item));
        }
        let (i, c, result) = reader.read_block(item, buf, itr, i, c, line_num);
        (i, c, result)
    }

    fn check(
        reader: &Box<A2LBlock>,
        buf: &[u8],
        line_num: usize,
        result: FnBlockReturnType,
        // Check status and results
        begin: bool,      // /begin detected
        block_mode: bool, // We are within a block
        escape: bool,
        slash: bool,
        asterisk: bool,
        comment: bool,
        line_comment: bool,
        expected_line_num: usize,
        level: i32,                     // /begin - /end level
        end_pos: usize,                 // Start position of /end keyword
        end_before: usize,              // Reverse word count after last /end beginning from 2
        expected_item: Option<Vec<u8>>, // Block type A2ML or IF_DATA
        key: Option<Vec<u8>>,           // Block name in case of IF_DATA
        block: Vec<Vec<u8>>,            // Block contents
        expected_result: FnBlockReturnType,
    ) {
        assert_eq!(
            reader.begin,
            begin,
            "Wrong begin with {:?}",
            std::str::from_utf8(&buf)
        );
        assert_eq!(
            reader.block_mode,
            block_mode,
            "Wrong block_mode with {:?}",
            std::str::from_utf8(&buf)
        );
        assert_eq!(
            reader.level,
            level,
            "Wrong level with {:?}",
            std::str::from_utf8(&buf)
        );
        assert_eq!(
            reader.end_pos,
            end_pos,
            "Wrong end_pos with {:?}",
            std::str::from_utf8(&buf)
        );
        assert_eq!(
            reader.end_before,
            end_before,
            "Wrong end_before with {:?}",
            std::str::from_utf8(&buf)
        );
        assert_eq!(
            reader.item,
            expected_item,
            "Wrong item with {:?}",
            std::str::from_utf8(&buf)
        );
        assert_eq!(
            reader.key,
            key,
            "Wrong key with {:?}",
            std::str::from_utf8(&buf)
        );
        assert_eq!(
            reader.block,
            block,
            "Wrong block with {:?}",
            std::str::from_utf8(&buf)
        );
        assert_eq!(
            reader.escape,
            escape,
            "Wrong escape with {:?}",
            std::str::from_utf8(&buf)
        );
        assert_eq!(
            reader.slash,
            slash,
            "Wrong slash with {:?}",
            std::str::from_utf8(&buf)
        );
        assert_eq!(
            reader.asterisk,
            asterisk,
            "Wrong asterisk with {:?}",
            std::str::from_utf8(&buf)
        );
        assert_eq!(
            reader.comment,
            comment,
            "Wrong comment with {:?}",
            std::str::from_utf8(&buf)
        );
        assert_eq!(
            reader.line_comment,
            line_comment,
            "Wrong line_comment with {:?}",
            std::str::from_utf8(&buf)
        );
        assert_eq!(
            line_num,
            expected_line_num,
            "Wrong line_num with {:?}",
            std::str::from_utf8(&buf)
        );
        assert_eq!(
            result,
            expected_result,
            "Wrong result with {:?}",
            std::str::from_utf8(&buf)
        );
    }

    #[test]
    fn read_block_a2l_no_block() {
        let buf = b"Test";
        let (reader, _, i, c, line_num, result) = start(b"Test", buf, false);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            None,
            None,
            Vec::new(),
            (3, 116, None),
        );

        let buf = b"Test ";
        let (reader, _, i, c, line_num, result) = start(b"Test", buf, false);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            None,
            None,
            Vec::new(),
            (4, 32, None),
        );

        let buf = b"/begin ";
        let (reader, _, i, c, line_num, result) = start(b"/begin", buf, false);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            None,
            None,
            Vec::new(),
            (6, 32, None),
        );

        let buf = b"/begin /end";
        let (mut reader, mut itr, i, c, mut line_num, result) = start(b"/begin", buf, false);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            None,
            None,
            Vec::new(),
            (6, 32, None),
        );
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            None,
            None,
            Vec::new(),
            (11, -5, None),
        );

        let buf = b"/begin AnyBlock /end ";
        let (mut reader, mut itr, i, c, mut line_num, result) = start(b"/begin", buf, false);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            None,
            None,
            Vec::new(),
            (6, 32, None),
        );
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            None,
            None,
            Vec::new(),
            (15, 32, None),
        );
    }

    #[test]
    fn read_block_a2l_valid_a2ml_block_with_contents() {
        let buf = b"/begin A2ML Contents /end A2ML";
        let (mut reader, mut itr, i, c, mut line_num, result) = start(b"/begin", buf, false);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            None,
            None,
            Vec::new(),
            (6, 32, None),
        );
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            0,
            21,
            2,
            None,
            None,
            Vec::new(),
            (
                30,
                76,
                Some(vec![
                    AdvReturnValue::Bytes(b"A2ML".to_vec()),
                    AdvReturnValue::Block(b" Contents ".to_vec()),
                ]),
            ),
        );
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            0,
            21,
            2,
            None,
            None,
            Vec::new(),
            (30, 76, None),
        );
    }

    #[test]
    fn read_block_a2l_valid_a2ml_block_edge1() {
        let buf = b"/begin A2ML ";
        let (mut reader, mut itr, i, c, mut line_num, result) = start(b"/begin", buf, false);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            None,
            None,
            Vec::new(),
            (6, 32, None),
        );
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            true,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            Some(b"A2ML".to_vec()),
            None,
            vec![b" ".to_vec()],
            (11, -13, None),
        );
        let buf = b"Contents /end A2ML ";
        let mut itr = buf.iter().enumerate();
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            0,
            9,
            1,
            None,
            None,
            Vec::new(),
            (
                18,
                32,
                Some(vec![
                    AdvReturnValue::Bytes(b"A2ML".to_vec()),
                    AdvReturnValue::Block(b" Contents ".to_vec()),
                ]),
            ),
        );
    }

    #[test]
    fn read_block_a2l_valid_a2ml_block_edge2() {
        let buf = b"/begin A2ML";
        let (mut reader, mut itr, i, c, mut line_num, result) = start(b"/begin", buf, false);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            None,
            None,
            Vec::new(),
            (6, 32, None),
        );
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            true,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            Some(b"A2ML".to_vec()),
            None,
            Vec::new(),
            (11, -12, None),
        );
        let buf = b" Contents /end A2ML ";
        let mut itr = buf.iter().enumerate();
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            0,
            10,
            1,
            None,
            None,
            Vec::new(),
            (
                19,
                32,
                Some(vec![
                    AdvReturnValue::Bytes(b"A2ML".to_vec()),
                    AdvReturnValue::Block(b" Contents ".to_vec()),
                ]),
            ),
        );
    }

    #[test]
    fn read_block_a2l_valid_a2ml_block_edge3() {
        let buf = b"/begin A2M";
        let (mut reader, mut itr, i, c, mut line_num, result) = start(b"/begin", buf, false);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            None,
            None,
            Vec::new(),
            (6, 32, None),
        );
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            None,
            None,
            Vec::new(),
            (10, -4, None),
        );
        let buf = b"A2ML Contents /end A2ML ";
        let mut itr = buf.iter().enumerate();
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            0,
            14,
            1,
            None,
            None,
            Vec::new(),
            (
                23,
                32,
                Some(vec![
                    AdvReturnValue::Bytes(b"A2ML".to_vec()),
                    AdvReturnValue::Block(b" Contents ".to_vec()),
                ]),
            ),
        );
    }

    #[test]
    fn read_block_a2l_valid_a2ml_block_edge4() {
        let buf = b"/begin A";
        let (mut reader, mut itr, i, c, mut line_num, result) = start(b"/begin", buf, false);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            None,
            None,
            Vec::new(),
            (6, 32, None),
        );
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            None,
            None,
            Vec::new(),
            (8, -2, None),
        );
        let buf = b"A2ML Contents /end A2ML ";
        let mut itr = buf.iter().enumerate();
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            0,
            14,
            1,
            None,
            None,
            Vec::new(),
            (
                23,
                32,
                Some(vec![
                    AdvReturnValue::Bytes(b"A2ML".to_vec()),
                    AdvReturnValue::Block(b" Contents ".to_vec()),
                ]),
            ),
        );
    }

    #[test]
    fn read_block_a2l_valid_a2ml_block_edge5() {
        let buf = b"/begin ";
        let (mut reader, mut itr, i, c, mut line_num, result) = start(b"/begin", buf, false);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            None,
            None,
            Vec::new(),
            (6, 32, None),
        );
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            None,
            None,
            Vec::new(),
            (7, -2, None),
        );
        let buf = b" A2ML Contents /end A2ML ";
        let mut itr = buf.iter().enumerate();
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            0,
            15,
            1,
            None,
            None,
            Vec::new(),
            (
                24,
                32,
                Some(vec![
                    AdvReturnValue::Bytes(b"A2ML".to_vec()),
                    AdvReturnValue::Block(b" Contents ".to_vec()),
                ]),
            ),
        );
    }

    #[test]
    fn read_block_a2l_valid_a2ml_block_edge6() {
        let buf = b"/begin";
        let (mut reader, mut itr, i, c, mut line_num, result) = start(b"/begin", buf, false);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            None,
            None,
            Vec::new(),
            (5, 110, None),
        );
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            None,
            None,
            Vec::new(),
            (6, -2, None),
        );
        let buf = b" A2ML Contents /end A2ML ";
        let mut itr = buf.iter().enumerate();
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            0,
            15,
            1,
            None,
            None,
            Vec::new(),
            (
                24,
                32,
                Some(vec![
                    AdvReturnValue::Bytes(b"A2ML".to_vec()),
                    AdvReturnValue::Block(b" Contents ".to_vec()),
                ]),
            ),
        );
    }

    #[test]
    fn read_block_a2l_valid_a2ml_block_edge7() {
        let buf = b"/begin A2ML C";
        let (mut reader, mut itr, i, c, mut line_num, result) = start(b"/begin", buf, false);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            None,
            None,
            Vec::new(),
            (6, 32, None),
        );
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            true,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            Some(b"A2ML".to_vec()),
            None,
            vec![b" ".to_vec()],
            (12, -13, None),
        );
        let buf = b"Contents /end A2ML ";
        let mut itr = buf.iter().enumerate();
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            0,
            9,
            1,
            None,
            None,
            Vec::new(),
            (
                18,
                32,
                Some(vec![
                    AdvReturnValue::Bytes(b"A2ML".to_vec()),
                    AdvReturnValue::Block(b" Contents ".to_vec()),
                ]),
            ),
        );
    }

    #[test]
    fn read_block_a2l_valid_a2ml_block_edge8() {
        let buf = b"/begin A2ML Contents ";
        let (mut reader, mut itr, i, c, mut line_num, result) = start(b"/begin", buf, false);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            None,
            None,
            Vec::new(),
            (6, 32, None),
        );
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            true,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            Some(b"A2ML".to_vec()),
            None,
            vec![b" Contents ".to_vec()],
            (20, -22, None),
        );
        let buf = b"/end A2ML ";
        let mut itr = buf.iter().enumerate();
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            0,
            0,
            1,
            None,
            None,
            Vec::new(),
            (
                9,
                32,
                Some(vec![
                    AdvReturnValue::Bytes(b"A2ML".to_vec()),
                    AdvReturnValue::Block(b" Contents ".to_vec()),
                ]),
            ),
        );
    }

    #[test]
    fn read_block_a2l_valid_if_data_block_with_contents() {
        let buf = b"/begin IF_DATA FOO Contents /end IF_DATA";
        let (mut reader, mut itr, i, c, mut line_num, result) = start(b"/begin", buf, false);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            None,
            None,
            Vec::new(),
            (6, 32, None),
        );
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            0,
            28,
            2,
            None,
            None,
            Vec::new(),
            (
                40,
                65,
                Some(vec![
                    AdvReturnValue::Bytes(b"IF_DATA".to_vec()),
                    AdvReturnValue::Bytes(b"FOO".to_vec()),
                    AdvReturnValue::Block(b" Contents ".to_vec()),
                ]),
            ),
        );
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            0,
            28,
            2,
            None,
            None,
            Vec::new(),
            (40, 65, None),
        );
    }

    #[test]
    fn read_block_a2l_valid_if_data_block_edge1() {
        let buf = b"/begin IF_DATA FOO Contents ";
        let (mut reader, mut itr, i, c, mut line_num, result) = start(b"/begin", buf, false);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            None,
            None,
            Vec::new(),
            (6, 32, None),
        );
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            true,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            Some(b"IF_DATA".to_vec()),
            Some(b"FOO".to_vec()),
            vec![b" Contents ".to_vec()],
            (27, -29, None),
        );
        let buf = b"/end IF_DATA ";
        let mut itr = buf.iter().enumerate();
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            0,
            0,
            1,
            None,
            None,
            Vec::new(),
            (
                12,
                32,
                Some(vec![
                    AdvReturnValue::Bytes(b"IF_DATA".to_vec()),
                    AdvReturnValue::Bytes(b"FOO".to_vec()),
                    AdvReturnValue::Block(b" Contents ".to_vec()),
                ]),
            ),
        );
    }

    #[test]
    fn read_block_a2l_valid_if_data_block_edge2() {
        let buf = b"/begin IF_DATA FOO";
        let (mut reader, mut itr, i, c, mut line_num, result) = start(b"/begin", buf, false);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            None,
            None,
            Vec::new(),
            (6, 32, None),
        );
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            true,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            Some(b"IF_DATA".to_vec()),
            None,
            Vec::new(),
            (17, -16, None),
        );
        let buf = b"FOO Contents /end IF_DATA ";
        let mut itr = buf.iter().enumerate();
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            0,
            13,
            1,
            None,
            None,
            Vec::new(),
            (
                25,
                32,
                Some(vec![
                    AdvReturnValue::Bytes(b"IF_DATA".to_vec()),
                    AdvReturnValue::Bytes(b"FOO".to_vec()),
                    AdvReturnValue::Block(b" Contents ".to_vec()),
                ]),
            ),
        );
    }

    #[test]
    fn read_block_a2l_valid_if_data_block_edge3() {
        let buf = b"/begin IF_DATA FOO ";
        let (mut reader, mut itr, i, c, mut line_num, result) = start(b"/begin", buf, false);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            None,
            None,
            Vec::new(),
            (6, 32, None),
        );
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            true,
            false,
            false,
            false,
            false,
            false,
            1,
            1,
            0,
            0,
            Some(b"IF_DATA".to_vec()),
            Some(b"FOO".to_vec()),
            vec![b" ".to_vec()],
            (18, -20, None),
        );
        let buf = b"Contents /end IF_DATA ";
        let mut itr = buf.iter().enumerate();
        let (i, c, result) = next(&mut reader, buf, &mut itr, i, c as u8, &mut line_num);
        check(
            &reader,
            buf,
            line_num,
            (i, c, result),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            1,
            0,
            9,
            1,
            None,
            None,
            Vec::new(),
            (
                21,
                32,
                Some(vec![
                    AdvReturnValue::Bytes(b"IF_DATA".to_vec()),
                    AdvReturnValue::Bytes(b"FOO".to_vec()),
                    AdvReturnValue::Block(b" Contents ".to_vec()),
                ]),
            ),
        );
    }
}
