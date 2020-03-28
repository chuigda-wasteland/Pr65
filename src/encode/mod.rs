use std::u32;

pub fn decode_fixed32(s: &[u8]) -> u32 {
    debug_assert_eq!(s.len(), 4);
    unsafe {
        ((*s.get_unchecked(0) as u32) << 24)
        + ((*s.get_unchecked(1) as u32) << 16)
        + ((*s.get_unchecked(2) as u32) << 8)
        + (*s.get_unchecked(3) as u32)
    }
}

pub fn encode_fixed32(s: &mut [u8], num: u32) {
    debug_assert_eq!(s.len(), 4);
    unsafe {
        *s.get_unchecked_mut(0) = (num >> 24) as u8;
        *s.get_unchecked_mut(1) = ((num & 0xFF0000) >> 16) as u8;
        *s.get_unchecked_mut(2) = ((num & 0xFF00) >> 8) as u8;
        *s.get_unchecked_mut(3) = (num & 0xFF) as u8;
    }
}

pub fn encode_fixed32_ret(num: u32) -> [u8; 4] {
    num.to_be_bytes()
}

#[cfg(test)]
mod test {
    use rand::{thread_rng, Rng};
    use crate::encode::{encode_fixed32, decode_fixed32, encode_fixed32_ret};

    #[test]
    fn test_encode_decode_32() {
        for _ in 1..1024 {
            let number = thread_rng().gen_range(0, 0x7FFFFFFFu32);
            let mut buffer = [0u8; 4];
            encode_fixed32(&mut buffer, number);
            assert_eq!(decode_fixed32(&buffer), number)
        }
    }

    #[test]
    fn test_encode_decode_32_ret() {
        for _ in 1..1024 {
            let number = thread_rng().gen_range(0, 0x7FFFFFFFu32);
            let mut buffer = encode_fixed32_ret( number);
            assert_eq!(decode_fixed32(&buffer), number)
        }
    }
}
