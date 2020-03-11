pub fn encode(value: i64) -> u64 {
    ((value << 1) ^ (value >> 63)) as u64
}

pub fn decode(value: u64) -> i64 {
    if (value & 0x1) == 0x1 {
        (-1 * ((value >> 1) as i64)) - 1
    } else {
        (value >> 1) as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        assert_eq!(std::i64::MAX, decode(encode(std::i64::MAX)));
        assert_eq!(std::i64::MIN, decode(encode(std::i64::MIN)));
        assert_eq!(0, decode(encode(0)));
        assert_eq!(15, decode(encode(15)));
        assert_eq!(-15, decode(encode(-15)));
        assert_eq!(132456789, decode(encode(132456789)));
        assert_eq!(-132456789, decode(encode(-132456789)));
    }

    #[test]
    fn test_encode() {
        assert_eq!(0, encode(0));
        assert_eq!(1, encode(-1));
        assert_eq!(2, encode(1));
        assert_eq!(3, encode(-2));
        assert_eq!(4, encode(2));
    }

    #[test]
    fn test_decode() {
        assert_eq!(0, decode(0));
        assert_eq!(-1, decode(1));
        assert_eq!(1, decode(2));
        assert_eq!(-2, decode(3));
        assert_eq!(2, decode(4));
    }
}
