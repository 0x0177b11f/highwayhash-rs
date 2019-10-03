mod highwayhash;
pub use self::highwayhash::{highway_hash128, highway_hash256, highway_hash64, HighwayHashCat};

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_KEY1: [u64; 4] = [
        0x0706050403020100u64,
        0x0F0E0D0C0B0A0908u64,
        0x1716151413121110u64,
        0x1F1E1D1C1B1A1918u64,
    ];

    const TEST_KEY1234: [u64; 4] = [1u64, 2u64, 3u64, 4u64];

    const EXPECTED64: [u64; 65] = [
        0x907A56DE22C26E53u64,
        0x7EAB43AAC7CDDD78u64,
        0xB8D0569AB0B53D62u64,
        0x5C6BEFAB8A463D80u64,
        0xF205A46893007EDAu64,
        0x2B8A1668E4A94541u64,
        0xBD4CCC325BEFCA6Fu64,
        0x4D02AE1738F59482u64,
        0xE1205108E55F3171u64,
        0x32D2644EC77A1584u64,
        0xF6E10ACDB103A90Bu64,
        0xC3BBF4615B415C15u64,
        0x243CC2040063FA9Cu64,
        0xA89A58CE65E641FFu64,
        0x24B031A348455A23u64,
        0x40793F86A449F33Bu64,
        0xCFAB3489F97EB832u64,
        0x19FE67D2C8C5C0E2u64,
        0x04DD90A69C565CC2u64,
        0x75D9518E2371C504u64,
        0x38AD9B1141D3DD16u64,
        0x0264432CCD8A70E0u64,
        0xA9DB5A6288683390u64,
        0xD7B05492003F028Cu64,
        0x205F615AEA59E51Eu64,
        0xEEE0C89621052884u64,
        0x1BFC1A93A7284F4Fu64,
        0x512175B5B70DA91Du64,
        0xF71F8976A0A2C639u64,
        0xAE093FEF1F84E3E7u64,
        0x22CA92B01161860Fu64,
        0x9FC7007CCF035A68u64,
        0xA0C964D9ECD580FCu64,
        0x2C90F73CA03181FCu64,
        0x185CF84E5691EB9Eu64,
        0x4FC1F5EF2752AA9Bu64,
        0xF5B7391A5E0A33EBu64,
        0xB9B84B83B4E96C9Cu64,
        0x5E42FE712A5CD9B4u64,
        0xA150F2F90C3F97DCu64,
        0x7FA522D75E2D637Du64,
        0x181AD0CC0DFFD32Bu64,
        0x3889ED981E854028u64,
        0xFB4297E8C586EE2Du64,
        0x6D064A45BB28059Cu64,
        0x90563609B3EC860Cu64,
        0x7AA4FCE94097C666u64,
        0x1326BAC06B911E08u64,
        0xB926168D2B154F34u64,
        0x9919848945B1948Du64,
        0xA2A98FC534825EBEu64,
        0xE9809095213EF0B6u64,
        0x582E5483707BC0E9u64,
        0x086E9414A88A6AF5u64,
        0xEE86B98D20F6743Du64,
        0xF89B7FF609B1C0A7u64,
        0x4C7D9CC19E22C3E8u64,
        0x9A97005024562A6Fu64,
        0x5DD41CF423E6EBEFu64,
        0xDF13609C0468E227u64,
        0x6E0DA4F64188155Au64,
        0xB755BA4B50D7D4A1u64,
        0x887A3484647479BDu64,
        0xAB8EEBE9BF2139A0u64,
        0x75542C5D4CD2A6FFu64,
    ];

    fn test_hash64(expected: u64, data: &Vec<u8>, key: [u64; 4]) {
        let hash = highway_hash64(data, key);
        assert_eq!(expected, hash);
    }

    #[test]
    fn test() {
        let mut data: [u8; 65] = [0; 65];

        for i in 0..64 {
            data[i] = i as u8;
            let v: Vec<u8> = data[0..i].to_vec();
            test_hash64(EXPECTED64[i], &v, TEST_KEY1);
        }
    }

    #[test]
    fn test_key1234() {
        let mut data: [u8; 33] = [0; 33];
        for i in 0..33 {
            data[i] = (128 + i) as u8;
        }
        test_hash64(0x53c516cce478cad7u64, &data.to_vec(), TEST_KEY1234);
        let v = vec![255u8];
        test_hash64(0x7858f24d2d79b2b2u64, &v, TEST_KEY1234);
    }

    #[test]
    fn test_cat() {
        let mut data: [u8; 65] = [0; 65];

        for i in 0..64 {
            data[i] = i as u8;
            let v: Vec<u8> = data[0..i].to_vec();
            let mut cat = HighwayHashCat::new(TEST_KEY1);
            cat.append(&v);
            let hash = cat.finish64();
            println!(
                "num: {}, hash require: {}, value : {}",
                i, EXPECTED64[i], hash
            );
            assert_eq!(EXPECTED64[i], hash);
        }
    }

    #[test]
    fn test_cat_key1234() {
        let mut cat = HighwayHashCat::new(TEST_KEY1234);
        let mut data: [u8; 33] = [0; 33];
        for i in 0..33 {
            data[i] = (128 + i) as u8;
        }

        cat.append(&data[0..32]);
        cat.append(&data[32..33]);

        let hash = cat.finish64();

        assert_eq!(0x53c516cce478cad7u64, hash);
    }
}
