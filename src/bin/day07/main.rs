use std::str;

use lib::error::Fail;

#[derive(Debug, Clone, Copy)]
pub enum Label {
    Number(char),
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl TryFrom<char> for Label {
    type Error = Fail;

    fn try_from(ch: char) -> Result<Label, Self::Error> {
        match ch {
            '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => Ok(Label::Number(ch)),
            'T' => Ok(Label::Ten),
            'J' => Ok(Label::Jack),
            'Q' => Ok(Label::Queen),
            'K' => Ok(Label::King),
            'A' => Ok(Label::Ace),
            other => Err(Fail(format!("card {other} is not valid"))),
        }
    }
}

impl Label {}

trait Card: From<Label> + Copy + Clone + PartialEq + Eq + PartialOrd + Ord + std::fmt::Debug {}

mod part1 {
    use std::cmp::Ordering;

    use lib::error::Fail;

    use std::collections::HashMap;

    use super::parse_input;
    use super::Card;
    use super::HandType;
    use super::Label;
    #[cfg(test)]
    use super::{get_example, parse_hand, parse_line, Hand};

    #[derive(Debug, Clone, Copy)]
    pub struct Part1Card {
        pub value: Label,
    }

    impl From<Label> for Part1Card {
        fn from(v: Label) -> Part1Card {
            Part1Card { value: v }
        }
    }

    impl Ord for Part1Card {
        fn cmp(&self, other: &Part1Card) -> Ordering {
            fn part1_label_rank(v: Label) -> u8 {
                match v {
                    Label::Number(ch) => ch
                        .to_digit(10)
                        .expect("number card labels must be valid digits")
                        as u8,
                    Label::Ten => 10,
                    Label::Jack => 11,
                    Label::Queen => 12,
                    Label::King => 13,
                    Label::Ace => 14,
                }
            }

            part1_label_rank(self.value).cmp(&part1_label_rank(other.value))
        }
    }

    impl PartialEq for Part1Card {
        fn eq(&self, other: &Part1Card) -> bool {
            self.cmp(other) == Ordering::Equal
        }
    }

    impl Eq for Part1Card {}

    impl PartialOrd for Part1Card {
        fn partial_cmp(&self, other: &Part1Card) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    #[test]
    fn test_card_ordering() {
        fn card(value: Label) -> Part1Card {
            Part1Card { value: value }
        }
        use Label::*;
        assert!(card(Number('3')) > card(Number('2')));
        assert!(card(Number('4')) > card(Number('3')));
        assert!(card(Number('5')) > card(Number('4')));
        assert!(card(Number('6')) > card(Number('5')));
        assert!(card(Number('7')) > card(Number('6')));
        assert!(card(Number('8')) > card(Number('7')));
        assert!(card(Number('9')) > card(Number('8')));
        assert!((card(Ten)) > card(Number('9')));
        assert!(card(Jack) > card(Ten));
        assert!(card(Queen) > card(Jack));
        assert!(card(King) > card(Queen));
        assert!(card(Ace) > card(King));
    }

    impl Card for Part1Card {}

    pub fn get_hand_type(s: &str) -> Result<HandType, Fail> {
        if s.len() != 5 {
            return Err(Fail(format!(
                "valid hands have 5 cards, this hand has {}: {s}",
                s.len()
            )));
        }
        let counts: HashMap<char, usize> = s.chars().fold(HashMap::new(), |mut acc, card| {
            acc.entry(card)
                .and_modify(|counter| *counter += 1)
                .or_insert(1);
            acc
        });
        match counts.values().max() {
            None => Err(Fail(format!(
                "Hands must contain 5 cards, this one contains 0: [{s}]"
            ))),
            Some(5) => Ok(HandType::FiveOfAKind),
            Some(4) => Ok(HandType::FourOfAKind),
            Some(3) => {
                if counts.len() == 2 {
                    Ok(HandType::FullHouse)
                } else if counts.len() == 3 {
                    Ok(HandType::ThreeOfAKind)
                } else {
                    Err(Fail(format!("did not understand hand type of {s}")))
                }
            }
            Some(2) => {
                // Distinguish "Two pair" from "One pair".
                if counts.len() == 3 {
                    Ok(HandType::TwoPair)
                } else if counts.len() == 4 {
                    Ok(HandType::OnePair)
                } else {
                    Err(Fail(format!("did not understand hand type of {s}")))
                }
            }
            Some(1) => Ok(HandType::HighCard),
            Some(n) => Err(Fail(format!("unexpected max count of same label {n}: {s}"))),
        }
    }

    #[test]
    fn test_get_hand_type_valid() {
        use HandType::*;
        assert_eq!(get_hand_type("AAAAA"), Ok(FiveOfAKind));
        assert_eq!(get_hand_type("AA8AA"), Ok(FourOfAKind));
        assert_eq!(get_hand_type("23332"), Ok(FullHouse));
        assert_eq!(get_hand_type("TTT98"), Ok(ThreeOfAKind));
        assert_eq!(get_hand_type("23432"), Ok(TwoPair));
        assert_eq!(get_hand_type("A23A4"), Ok(OnePair));
        assert_eq!(get_hand_type("23456"), Ok(HighCard));
    }

    #[test]
    fn test_get_hand_type_invalid_count() {
        assert!(get_hand_type("").is_err());
        assert!(get_hand_type("2").is_err());
        assert!(get_hand_type("22").is_err());
        assert!(get_hand_type("333").is_err());
        assert!(get_hand_type("4444").is_err());
        assert!(get_hand_type("666666").is_err());
    }

    #[test]
    fn test_get_hand_type_valid_label() {
        assert!(get_hand_type("AAAAA").is_ok());
        assert!(get_hand_type("22222").is_ok());
        assert!(get_hand_type("33333").is_ok());
        assert!(get_hand_type("44444").is_ok());
        assert!(get_hand_type("55555").is_ok());
        assert!(get_hand_type("66666").is_ok());
        assert!(get_hand_type("77777").is_ok());
        assert!(get_hand_type("88888").is_ok());
        assert!(get_hand_type("99999").is_ok());
        assert!(get_hand_type("TTTTT").is_ok());
        assert!(get_hand_type("JJJJJ").is_ok());
        assert!(get_hand_type("QQQQQ").is_ok());
        assert!(get_hand_type("KKKKK").is_ok());
    }

    #[test]
    fn test_hand_comparison() {
        fn parse(s: &str) -> Result<Hand<Part1Card>, Fail> {
            parse_hand::<Part1Card>(s, get_hand_type)
        }
        assert!(parse("32T3K").unwrap() < parse("KTJJT").unwrap());
        assert!(parse("KTJJT").unwrap() < parse("KK677").unwrap());
        assert!(parse("KK677").unwrap() < parse("T55J5").unwrap());
        assert!(parse("T55J5").unwrap() < parse("QQQJA").unwrap());
    }

    pub fn solve(s: &str) -> u32 {
        let mut hands = parse_input::<Part1Card>(s, get_hand_type).expect("input should be valid");
        hands.sort();
        hands
            .iter()
            .enumerate()
            .map(|(i, (_hand, bid))| (1 + i as u32) * bid)
            .sum()
    }

    #[test]
    fn test_solve() {
        assert_eq!(solve(&get_example()), 6440);
    }

    #[test]
    fn test_parse_line() {
        use Label::*;
        fn card(value: Label) -> Part1Card {
            Part1Card { value: value }
        }
        assert_eq!(
            parse_line::<Part1Card>("KTJJT 220", get_hand_type),
            Ok((
                Hand {
                    hand_type: HandType::TwoPair,
                    cards: [card(King), card(Ten), card(Jack), card(Jack), card(Ten),],
                },
                220
            ))
        );
    }
}

mod part2 {
    use std::cmp::Ordering;
    use std::collections::HashMap;

    use lib::error::Fail;

    use super::parse_input;
    use super::Card;
    use super::HandType;
    use super::Label;
    #[cfg(test)]
    use super::{get_example, parse_hand, Hand};

    #[derive(Debug, Clone, Copy)]
    pub struct Part2Card {
        pub value: Label,
    }

    impl From<Label> for Part2Card {
        fn from(v: Label) -> Part2Card {
            Part2Card { value: v }
        }
    }

    impl Ord for Part2Card {
        fn cmp(&self, other: &Part2Card) -> Ordering {
            fn part2_label_rank(v: Label) -> u8 {
                match v {
                    Label::Jack => 0,
                    Label::Number(ch) => ch
                        .to_digit(10)
                        .expect("number card labels must be valid digits")
                        as u8,
                    Label::Ten => 10,
                    // Jack has lowest rank
                    Label::Queen => 12,
                    Label::King => 13,
                    Label::Ace => 14,
                }
            }
            part2_label_rank(self.value).cmp(&part2_label_rank(other.value))
        }
    }

    impl PartialEq for Part2Card {
        fn eq(&self, other: &Part2Card) -> bool {
            self.cmp(other) == Ordering::Equal
        }
    }

    impl Eq for Part2Card {}

    impl PartialOrd for Part2Card {
        fn partial_cmp(&self, other: &Part2Card) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Card for Part2Card {}

    pub fn get_hand_type(s: &str) -> Result<HandType, Fail> {
        if s.len() != 5 {
            return Err(Fail(format!(
                "valid hands have 5 cards, this hand has {}: {s}",
                s.len()
            )));
        }
        let non_jack_counts: HashMap<char, usize> =
            s.chars()
                .filter(|ch| *ch != 'J')
                .fold(HashMap::new(), |mut acc, card| {
                    acc.entry(card)
                        .and_modify(|counter| *counter += 1)
                        .or_insert(1);
                    acc
                });
        let jack_count = s.chars().filter(|ch| *ch == 'J').count();

        if let Some(largest_non_jack_count) = non_jack_counts.values().max() {
            match dbg!(largest_non_jack_count) + dbg!(jack_count) {
                5 => Ok(HandType::FiveOfAKind),
                4 => Ok(HandType::FourOfAKind),
                3 => {
                    // GGGXX (Full house) or GGGXY (Three of a kind)
                    // or GGJXY (Three of a kind) or GJJXY (three of a
                    // kind) (where X, Y, G are non-jack cards).
                    //
                    // Can't be JJJXY or JJJXX as these are the 4 and
                    // 5 cases above.
                    match non_jack_counts.len() {
                        2 => Ok(HandType::FullHouse),    // GGGXX
                        3 => Ok(HandType::ThreeOfAKind), // GGGXY or GGJXY or GJJXY.
                        _ => {
                            panic!("failed to identify hand type of {s}");
                        }
                    }
                }
                2 => {
                    // GGXYZ (one pair) or GJXYZ (one pair) or GGXXY (two pair).
                    match non_jack_counts.len() {
                        4 => Ok(HandType::OnePair),
                        3 => Ok(HandType::TwoPair),
                        _ => {
                            panic!("failed to identify hand type of {s}");
                        }
                    }
                }
                1 => Ok(HandType::HighCard), // GWXYZ
                other => {
                    panic!("failed to identify hand type of {s} with total count {other}");
                }
            }
        } else {
            Ok(HandType::FiveOfAKind) // only jacks
        }
    }

    #[test]
    fn test_hand_type() {
        fn parse(s: &str) -> Hand<Part2Card> {
            parse_hand::<Part2Card>(s, get_hand_type).expect("hand should be valid")
        }
        assert_eq!(parse("32T3K").get_type(), HandType::OnePair);
        assert_eq!(parse("KK677").get_type(), HandType::TwoPair);

        assert_eq!(parse("T55J5").get_type(), HandType::FourOfAKind);
        assert_eq!(parse("KTJJT").get_type(), HandType::FourOfAKind);
        assert_eq!(parse("QQQJA").get_type(), HandType::FourOfAKind);
    }

    #[test]
    fn test_hand_comparison() {
        fn parse(s: &str) -> Hand<Part2Card> {
            parse_hand::<Part2Card>(s, get_hand_type).expect("hand should be valid")
        }
        assert!(parse("32T3K") < parse("KK677"));

        assert!(parse("KK677") < parse("T55J5"));

        assert!(parse("T55J5") < parse("QQQJA"));
        assert!(parse("QQQJA") < parse("KTJJT"));

        assert!(parse("JKKK2") < parse("QQQQ2"));

        assert!(parse("JJJJJ") < parse("22222"));
    }

    pub fn solve(s: &str) -> u32 {
        let mut hands = parse_input::<Part2Card>(s, get_hand_type).expect("input should be valid");
        hands.sort();
        hands
            .iter()
            .enumerate()
            .map(|(i, (_hand, bid))| (1 + i as u32) * bid)
            .sum()
    }

    #[test]
    fn test_solve() {
        assert_eq!(solve(&get_example()), 5905);
    }
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Copy, Clone)]
pub enum HandType {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

#[test]
fn test_hand_type_ordering() {
    use HandType::*;
    assert!(OnePair > HighCard);
    assert!(TwoPair > OnePair);
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
struct Hand<C: Card> {
    hand_type: HandType,
    cards: [C; 5],
}

impl<C: Card> Hand<C> {
    #[cfg(test)]
    fn get_type(&self) -> HandType {
        self.hand_type
    }
}

fn parse_hand<C: Card>(
    s: &str,
    hand_type_selector: fn(&str) -> Result<HandType, Fail>,
) -> Result<Hand<C>, Fail> {
    let cards: Vec<Label> = s
        .chars()
        .take(6)
        .map(Label::try_from)
        .collect::<Result<Vec<Label>, Fail>>()?;
    let cards: Vec<C> = cards.iter().map(|value| C::from(*value)).collect();
    match cards.as_slice() {
        [c1, c2, c3, c4, c5] => Ok(Hand {
            hand_type: hand_type_selector(s)?,
            cards: [*c1, *c2, *c3, *c4, *c5],
        }),
        _ => Err(Fail(format!("expected 5 cards, got {}: {s}", s.len()))),
    }
}

#[test]
fn test_hand_try_from_str() {
    use part1::Part1Card;
    use Label::*;
    fn card(value: Label) -> part1::Part1Card {
        Part1Card { value: value }
    }
    fn parse(s: &str) -> Result<Hand<Part1Card>, Fail> {
        parse_hand::<Part1Card>(s, part1::get_hand_type)
    }
    assert_eq!(
        parse("AAAAA"),
        Ok(Hand {
            hand_type: HandType::FiveOfAKind,
            cards: [card(Ace), card(Ace), card(Ace), card(Ace), card(Ace),]
        })
    );
    assert!(parse("11111").is_err());
    assert!(parse("qqqqq").is_err());
}

fn parse_line<C: Card>(
    s: &str,
    hand_type_selector: fn(&str) -> Result<HandType, Fail>,
) -> Result<(Hand<C>, u32), Fail> {
    match s.split_once(' ') {
        Some((hand, bid)) => match (
            parse_hand::<C>(hand, hand_type_selector)?,
            bid.parse::<u32>(),
        ) {
            (hand, Ok(bid)) => Ok((hand, bid)),
            (_, Err(e)) => Err(Fail(format!("{bid} is not a valid bid: {e}"))),
        },
        None => Err(Fail(format!("expected to find a space in {s}"))),
    }
}

#[cfg(test)]
fn get_example() -> &'static str {
    concat!(
        "32T3K 765\n",
        "T55J5 684\n",
        "KK677 28\n",
        "KTJJT 220\n",
        "QQQJA 483\n",
    )
}

fn parse_input<C: Card>(
    s: &str,
    hand_type_selector: fn(&str) -> Result<HandType, Fail>,
) -> Result<Vec<(Hand<C>, u32)>, Fail> {
    s.split_terminator('\n')
        .map(|line| parse_line(line, hand_type_selector))
        .collect::<Result<Vec<(Hand<C>, u32)>, Fail>>()
}

#[test]
fn test_parse_input_part1() {
    run_test_parse_input_part::<part1::Part1Card>(part1::get_hand_type)
}

#[test]
fn test_parse_input_part2() {
    run_test_parse_input_part::<part2::Part2Card>(part2::get_hand_type)
}

#[cfg(test)]
fn run_test_parse_input_part<C: Card>(hand_type_selector: fn(&str) -> Result<HandType, Fail>) {
    use HandType::*;
    use Label::*;
    let input: Vec<(Hand<C>, u32)> =
        parse_input(get_example(), hand_type_selector).expect("example should be valid");
    assert_eq!(input.len(), 5);

    fn card<C: Card>(value: Label) -> C {
        C::from(value)
    }

    let expected_first_hand: Hand<C> = Hand::<C> {
        hand_type: OnePair,
        cards: [
            card(Number('3')),
            card(Number('2')),
            card(Ten),
            card(Number('3')),
            card(King),
        ],
    };

    assert_eq!((expected_first_hand, 765), input[0]);
}

/// Reads the puzzle input.
fn get_input() -> String {
    let input = str::from_utf8(include_bytes!("input.txt")).unwrap();
    input.to_string()
}

fn main() {
    println!("day 07 part 1: {}", part1::solve(&get_input()));
    println!("day 07 part 2: {}", part2::solve(&get_input()));
}
