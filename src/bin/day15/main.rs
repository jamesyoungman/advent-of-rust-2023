use std::str;

#[derive(Debug, Clone, Default)]
struct ReindeerHasher {
    acc: u8,
}

impl ReindeerHasher {
    fn add(&mut self, ch: char) {
        match u8::try_from(ch) {
            Ok(codepoint) => {
                let sum: u16 = u16::from(self.acc) + u16::from(codepoint);
                let product = sum * 17;
                self.acc = u8::try_from(product % 256_u16)
                    .expect("there should be no way for this quantity to get out of range");
            }
            Err(_) => {
                panic!("non-ASCII character in input: {ch}");
            }
        }
    }

    fn get(&self) -> u8 {
        self.acc
    }

    fn hash(s: &str) -> u8 {
        s.chars()
            .fold(ReindeerHasher::default(), |mut h, ch| {
                h.add(ch);
                h
            })
            .get()
    }
}

#[test]
fn test_hash() {
    assert_eq!(ReindeerHasher::hash("HASH"), 52);
    assert_eq!(ReindeerHasher::hash("rn=1"), 30);
    assert_eq!(ReindeerHasher::hash("cm-"), 253);
    assert_eq!(ReindeerHasher::hash("qp=3"), 97);
}

fn part1(s: &str) -> u64 {
    s.split(',')
        .map(|s| u64::from(ReindeerHasher::hash(s)))
        .sum()
}

#[test]
fn test_part1() {
    assert_eq!(
        part1("rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7"),
        1320
    );
}

fn get_input() -> &'static str {
    str::from_utf8(include_bytes!("input.txt")).unwrap().trim()
}

fn main() {
    let input = get_input();
    println!("day 15 part 1: {}", part1(input));
}
