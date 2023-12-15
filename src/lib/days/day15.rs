use std::ops::{Add, Mul, Rem};

fn update_hash_value<W>(h: W, codepoint: W) -> W
where
    W: From<u8> + From<u16> + Add<Output = W> + Rem<Output = W> + Mul<Output = W>,
{
    let sum: W = h + codepoint;
    let modulus: W = W::from(256_u16);
    (sum * W::from(17_u8)) % modulus
}

fn convert_ascii_char<W: From<u8>>(ch: char) -> W {
    let codepoint = u8::try_from(ch).expect("input should be all-ASCII");
    W::from(codepoint)
}

#[inline]
pub fn hash_generic<W>(s: &str) -> u8 {
    s.chars()
        .map(convert_ascii_char)
        .fold(0, update_hash_value)
        .try_into()
        .expect("there should be no way for the hash accumulator to get out of range")
}

#[test]
fn test_hash_ascii() {
    fn hash(s: &str) -> u8 {
        hash_generic::<u16>(s)
    }
    assert_eq!(hash("HASH"), 52);
    assert_eq!(hash("rn=1"), 30);
    assert_eq!(hash("cm-"), 253);
    assert_eq!(hash("qp=3"), 97);
}

#[test]
#[should_panic]
fn test_hash_nonascii() {
    fn hash(s: &str) -> u8 {
        hash_generic::<u16>(s)
    }
    hash("😊");
}
