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
        *s.get_unchecked_mut(1) = ((num >> 16) & 0xFF) as u8;
        *s.get_unchecked_mut(2) = ((num >> 8) & 0xFF) as u8;
        *s.get_unchecked_mut(3) = (num & 0xFF) as u8;
    }
}

pub fn encode_fixed32_ret(num: u32) -> [u8; 4] {
    num.to_be_bytes()
}

pub fn decode_fixed64(s: &[u8]) -> u64 {
    debug_assert_eq!(s.len(), 8);
    unsafe {
        ((*s.get_unchecked(0) as u64) << 56)
            + ((*s.get_unchecked(1) as u64) << 48)
            + ((*s.get_unchecked(2) as u64) << 40)
            + ((*s.get_unchecked(3) as u64) << 32)
            + ((*s.get_unchecked(4) as u64) << 24)
            + ((*s.get_unchecked(5) as u64) << 16)
            + ((*s.get_unchecked(6) as u64) << 8)
            + (*s.get_unchecked(7) as u64)
    }
}

pub fn encode_fixed64(s: &mut [u8], num: u64) {
    debug_assert_eq!(s.len(), 8);
    unsafe {
        *s.get_unchecked_mut(0) = (num >> 56) as u8;
        *s.get_unchecked_mut(1) = ((num >> 48) & 0xFF) as u8;
        *s.get_unchecked_mut(2) = ((num >> 40) & 0xFF) as u8;
        *s.get_unchecked_mut(3) = ((num >> 32) & 0xFF) as u8;
        *s.get_unchecked_mut(4) = ((num >> 24) & 0xFF) as u8;
        *s.get_unchecked_mut(5) = ((num >> 16) & 0xFF) as u8;
        *s.get_unchecked_mut(6) = ((num >> 8) & 0xFF) as u8;
        *s.get_unchecked_mut(7) = (num & 0xFF) as u8;
    }
}

pub fn encode_fixed64_ret(num: u64) -> [u8; 8] {
    num.to_be_bytes()
}

#[cfg(test)]
mod test {
    use rand::{thread_rng, Rng};
    use crate::encode::{encode_fixed32, decode_fixed32, encode_fixed32_ret,
                        encode_fixed64, decode_fixed64, encode_fixed64_ret};

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
            let buffer = encode_fixed32_ret( number);
            assert_eq!(decode_fixed32(&buffer), number)
        }
    }

    #[test]
    fn test_encode_decode_64() {
        for _ in 1..1024 {
            let number = thread_rng().gen_range(0, 0x7FFFFFFFFFFFFFFFu64);
            let mut buffer = [0u8; 8];
            encode_fixed64(&mut buffer, number);
            assert_eq!(decode_fixed64(&buffer), number)
        }
    }

    #[test]
    fn test_encode_decode_64_ret() {
        for _ in 1..1024 {
            let number = thread_rng().gen_range(0, 0x7FFFFFFFFFFFFFFFu64);
            let buffer = encode_fixed64_ret( number);
            assert_eq!(decode_fixed64(&buffer), number)
        }
    }
}
