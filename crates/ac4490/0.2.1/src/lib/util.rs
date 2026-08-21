pub fn hex_to_rssi(hex: u8) -> i8 {

    // Handle out-of-range values
    if hex > 0xC0 {
        return -92
    } else if hex < 0x0B {
        return -44
    }

    let lookup_table: [(u8, i8); 40] = [
        (0xC0, -92), (0xBC, -91), (0xBB, -90), (0xB9, -89), (0xB8, -88),
        (0xAE, -87), (0xA9, -86), (0xA2, -85), (0x92, -84), (0x8D, -83),
        (0x86, -82), (0x82, -81), (0x7D, -80), (0x79, -79), (0x75, -78),
        (0x72, -77), (0x6F, -76), (0x6B, -75), (0x68, -74), (0x66, -73),
        (0x63, -72), (0x5F, -71), (0x5B, -70), (0x58, -69), (0x54, -68),
        (0x4F, -67), (0x4B, -66), (0x47, -65), (0x43, -64), (0x3D, -63),
        (0x2A, -62), (0x25, -60), (0x1A, -58), (0x16, -56), (0x13, -54),
        (0x11, -52), (0x0E, -50), (0x0D, -48), (0x0C, -46), (0x0B, -44),
    ];

    // Handle exact matches
    #[allow(clippy::explicit_iter_loop)]
    if let Some((_, rssi)) = lookup_table.iter().find(|&&(h, _)| h == hex) {
        return *rssi;
    }

    // Find the nearest match
    let mut min_diff = i16::MAX;
    let mut rssi = 0;
    #[allow(clippy::explicit_iter_loop)]
    for &(h, r) in lookup_table.iter() {
        let diff = (i16::from(h) - i16::from(hex)).abs();
        if diff < min_diff {
            min_diff = diff;
            rssi = r;
        }
    }

    rssi

}