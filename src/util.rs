pub fn write_u16(buf: &mut [u8], pos: usize, val: u16) {
    buf[pos] = (val & 0xFF) as u8;
    buf[pos + 1] = ((val >> 8) & 0xFF) as u8;
}

pub fn read_u16(buf: &[u8], pos: usize) -> u16 {
    let lo = buf[pos];
    let hi = buf[pos + 1];
    concat_u16(hi, lo)
}

pub fn read_u16_array<const S: usize>(src: &[u8], start: usize) -> [u16; S] {
    let mut out = [0; S];

    for i in 0..S {
        let src_i = start + i * 2;
        out[i] = concat_u16(src[src_i + 1], src[src_i]);
    }

    out
}

pub fn concat_u16(hi: u8, lo: u8) -> u16 {
    ((hi as u16) << 8) | (lo as u16)
}

pub fn u8_array_to_u16(src: &[u8]) -> Vec<u16> {
    src.chunks(2)
        .map(|chunk| concat_u16(chunk[1], chunk[0]))
        .collect()
}

pub fn u16_array_to_u8(src: &[u16]) -> Vec<u8> {
    src.iter()
        .map(|v| [(v & 0xFF) as u8, ((v >> 8) & 0xFF) as u8])
        .collect::<Vec<[u8; 2]>>().concat()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concat_u16_test() {
        assert_eq!(concat_u16(0x27, 0x18), 0x2718);
        assert_eq!(concat_u16(0x00, 0x01), 0x0001);
        assert_eq!(concat_u16(0x12, 0x13), 0x1213);
    }

    #[test]
    fn write_u16_test() {
        let mut buf = [0xFF; 6];
        write_u16(&mut buf, 1, 0xC08F);
        write_u16(&mut buf, 3, 0xAE24);

        assert_eq!(buf, [0xFF, 0x8F, 0xC0, 0x24, 0xAE, 0xFF]);
    }

    #[test]
    fn read_u16_array_test() {
        let src = [0x10, 0x24, 0x56, 0xA0, 0x4B, 0x17, 0x56, 0xEA];
        let arr = read_u16_array::<3>(&src, 1);

        assert_eq!(arr, [0x5624, 0x4BA0, 0x5617]);
    }

    #[test]
    fn read_u16_test() {
        let buf = [0x10, 0x62, 0x1A, 0xB4];

        assert_eq!(read_u16(&buf, 0), 0x6210);
        assert_eq!(read_u16(&buf, 2), 0xB41A);
        assert_eq!(read_u16(&buf, 1), 0x1A62);
    }

    #[test]
    fn u8_array_to_u16_test() {
        let src = [0x10, 0x24, 0x56, 0xA0, 0x4B, 0x17, 0x56, 0xEA];
        let out = u8_array_to_u16(&src);

        assert_eq!(out, vec![0x2410, 0xA056, 0x174B, 0xEA56]);
    }

    #[test]
    fn u16_array_to_u8_test() {
        let src = [0x2410, 0xA056, 0x174B];
        let out = u16_array_to_u8(&src);

        assert_eq!(out, vec![0x10, 0x24, 0x56, 0xA0, 0x4B, 0x17]);
    }
}