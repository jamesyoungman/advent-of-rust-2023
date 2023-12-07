use std::collections::HashMap;
use std::str;

use lib::error::Fail;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Label {
    Number(u8),
    Jack,
    Queen,
    King,
    Ace,
}

impl TryFrom<char> for Label {
    type Error = Fail;

    fn try_from(ch: char) -> Result<Label, Self::Error> {
        match ch {
            '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => Ok(Label::Number(
                ch.to_digit(10).expect("valid decimal digit") as u8,
            )),
            'T' => Ok(Label::Number(10_u8)),
            'J' => Ok(Label::Jack),
            'Q' => Ok(Label::Queen),
            'K' => Ok(Label::King),
            'A' => Ok(Label::Ace),
            other => Err(Fail(format!("card {other} is not valid"))),
        }
    }
}

#[test]
fn test_label_parsing() {
    assert_eq!(Label::try_from('2'), Ok(Label::Number(2)));
    assert_eq!(Label::try_from('3'), Ok(Label::Number(3)));
    assert_eq!(Label::try_from('4'), Ok(Label::Number(4)));
    assert_eq!(Label::try_from('5'), Ok(Label::Number(5)));
    assert_eq!(Label::try_from('6'), Ok(Label::Number(6)));
    assert_eq!(Label::try_from('7'), Ok(Label::Number(7)));
    assert_eq!(Label::try_from('8'), Ok(Label::Number(8)));
    assert_eq!(Label::try_from('9'), Ok(Label::Number(9)));
    assert_eq!(Label::try_from('T'), Ok(Label::Number(10)));

    assert_eq!(Label::try_from('J'), Ok(Label::Jack));
    assert_eq!(Label::try_from('Q'), Ok(Label::Queen));
    assert_eq!(Label::try_from('K'), Ok(Label::King));
    assert_eq!(Label::try_from('A'), Ok(Label::Ace));

    assert!(Label::try_from('0').is_err());
    assert!(Label::try_from('1').is_err());
    assert!(Label::try_from('Z').is_err());
}

impl Label {
    fn part1_label_rank(&self) -> u8 {
        match self {
            Label::Number(value) => *value,
            Label::Jack => 11,
            Label::Queen => 12,
            Label::King => 13,
            Label::Ace => 14,
        }
    }

    fn part2_label_rank(&self) -> u8 {
        match self {
            Label::Jack => 0,
            Label::Number(value) => *value,
            // Jack has lowest rank
            Label::Queen => 12,
            Label::King => 13,
            Label::Ace => 14,
        }
    }
}

fn parse_hand(s: &str) -> Result<[Label; 5], Fail> {
    let v: Vec<Label> = s
        .chars()
        .map(Label::try_from)
        .collect::<Result<Vec<Label>, Fail>>()?;
    match v.as_slice() {
        [l1, l2, l3, l4, l5] => Ok([*l1, *l2, *l3, *l4, *l5]),
        _ => Err(Fail(format!("hand contains {} cards, expected 5", v.len()))),
    }
}

type ParsedLine = ([Label; 5], u32);

fn parse_line(s: &str) -> Result<ParsedLine, Fail> {
    match s.split_once(' ') {
        Some((hand, bid)) => Ok((
            parse_hand(hand)?,
            bid.parse::<u32>()
                .map_err(|e| Fail(format!("{bid} is not a valid bid: {e}")))?,
        )),
        None => Err(Fail(format!("expected to find a space in {s}"))),
    }
}

fn parse_input(s: &str) -> Result<Vec<ParsedLine>, Fail> {
    s.split_terminator('\n')
        .map(parse_line)
        .collect::<Result<Vec<ParsedLine>, Fail>>()
}

#[test]
fn test_parse_line() {
    use Label::*;
    assert_eq!(
        parse_line("KTJJT 220").expect("valid"),
        ([King, Number(10), Jack, Jack, Number(10)], 220)
    );
}

pub fn get_part1_hand_type(labels: &[Label; 5]) -> Result<HandType, Fail> {
    let counts: HashMap<Label, usize> = labels.iter().fold(HashMap::new(), |mut acc, card| {
        acc.entry(*card)
            .and_modify(|counter| *counter += 1)
            .or_insert(1);
        acc
    });
    match counts.values().max() {
        None => Err(Fail(format!(
            "Hands must contain 5 cards, this one contains 0: {labels:?}"
        ))),
        Some(5) => Ok(HandType::FiveOfAKind),
        Some(4) => Ok(HandType::FourOfAKind),
        Some(3) => {
            if counts.len() == 2 {
                Ok(HandType::FullHouse)
            } else if counts.len() == 3 {
                Ok(HandType::ThreeOfAKind)
            } else {
                Err(Fail(format!("did not understand hand type of {labels:?}")))
            }
        }
        Some(2) => {
            // Distinguish "Two pair" from "One pair".
            if counts.len() == 3 {
                Ok(HandType::TwoPair)
            } else if counts.len() == 4 {
                Ok(HandType::OnePair)
            } else {
                Err(Fail(format!("did not understand hand type of {labels:?}")))
            }
        }
        Some(1) => Ok(HandType::HighCard),
        Some(n) => Err(Fail(format!(
            "unexpected max count of same label {n}: {labels:?}"
        ))),
    }
}

#[test]
fn test_part1_hand_type() {
    fn get_type(s: &str) -> HandType {
        let labels = parse_hand(s).expect("test input should be valid");
        get_part1_hand_type(&labels).expect("test input should be valid")
    }
    assert_eq!(get_type("32T3K"), HandType::OnePair);
    assert_eq!(get_type("KK677"), HandType::TwoPair);
    assert_eq!(get_type("T55J5"), HandType::ThreeOfAKind);
    assert_eq!(get_type("KTJJT"), HandType::TwoPair);
    assert_eq!(get_type("QQQJA"), HandType::ThreeOfAKind);
}

pub fn get_part2_hand_type(labels: &[Label; 5]) -> Result<HandType, Fail> {
    let non_jack_counts: HashMap<Label, usize> = labels
        .iter()
        .filter(|label| **label != Label::Jack)
        .fold(HashMap::new(), |mut acc, card| {
            acc.entry(*card)
                .and_modify(|counter| *counter += 1)
                .or_insert(1);
            acc
        });
    let jack_count = labels.iter().filter(|label| **label == Label::Jack).count();

    if let Some(largest_non_jack_count) = non_jack_counts.values().max() {
        match largest_non_jack_count + jack_count {
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
                        panic!("failed to identify hand type of {labels:?}");
                    }
                }
            }
            2 => {
                // GGXYZ (one pair) or GJXYZ (one pair) or GGXXY (two pair).
                match non_jack_counts.len() {
                    4 => Ok(HandType::OnePair),
                    3 => Ok(HandType::TwoPair),
                    _ => {
                        panic!("failed to identify hand type of {labels:?}");
                    }
                }
            }
            1 => Ok(HandType::HighCard), // GWXYZ
            other => {
                panic!("failed to identify hand type of {labels:?} with total count {other}");
            }
        }
    } else {
        Ok(HandType::FiveOfAKind) // only jacks
    }
}

#[test]
fn test_part2_hand_type() {
    fn get_type(s: &str) -> HandType {
        let labels = parse_hand(s).expect("test input should be valid");
        get_part2_hand_type(&labels).expect("test input should be valid")
    }
    assert_eq!(get_type("32T3K"), HandType::OnePair);
    assert_eq!(get_type("KK677"), HandType::TwoPair);
    assert_eq!(get_type("T55J5"), HandType::FourOfAKind);
    assert_eq!(get_type("KTJJT"), HandType::FourOfAKind);
    assert_eq!(get_type("QQQJA"), HandType::FourOfAKind);
}

fn part1_sort_key(labels: &[Label; 5]) -> Result<SortKey, Fail> {
    Ok(SortKey {
        hand_type: get_part1_hand_type(labels)?,
        label_indices: [
            Label::part1_label_rank(&labels[0]),
            Label::part1_label_rank(&labels[1]),
            Label::part1_label_rank(&labels[2]),
            Label::part1_label_rank(&labels[3]),
            Label::part1_label_rank(&labels[4]),
        ],
    })
}

fn part2_sort_key(labels: &[Label; 5]) -> Result<SortKey, Fail> {
    Ok(SortKey {
        hand_type: get_part2_hand_type(labels)?,
        label_indices: [
            Label::part2_label_rank(&labels[0]),
            Label::part2_label_rank(&labels[1]),
            Label::part2_label_rank(&labels[2]),
            Label::part2_label_rank(&labels[3]),
            Label::part2_label_rank(&labels[4]),
        ],
    })
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct SortKey {
    hand_type: HandType,
    label_indices: [u8; 5],
}

type SortKeyFn = fn(&[Label; 5]) -> Result<SortKey, Fail>;

fn rank_hands(hands: &[ParsedLine], make_key: SortKeyFn) -> Result<Vec<(usize, u32)>, Fail> {
    let mut unsorted_hands: Vec<_> = hands
        .iter()
        .map(|(labels, bid)| make_key(labels).map(|labels| (labels, *bid)))
        .collect::<Result<Vec<(SortKey, u32)>, Fail>>()?;
    unsorted_hands.sort();
    Ok(unsorted_hands
        .iter()
        .enumerate()
        .map(|(i, (_labels, bid))| (i + 1, *bid))
        .collect())
}

trait Card: From<Label> + Copy + Clone + PartialEq + Eq + PartialOrd + Ord + std::fmt::Debug {}

#[test]
fn test_part1_card_ordering() {
    use Label::*;
    assert!(Number(3).part1_label_rank() > Number(2).part1_label_rank());
    assert!(Number(4).part1_label_rank() > Number(3).part1_label_rank());
    assert!(Number(5).part1_label_rank() > Number(4).part1_label_rank());
    assert!(Number(6).part1_label_rank() > Number(5).part1_label_rank());
    assert!(Number(7).part1_label_rank() > Number(6).part1_label_rank());
    assert!(Number(8).part1_label_rank() > Number(7).part1_label_rank());
    assert!(Number(9).part1_label_rank() > Number(8).part1_label_rank());
    assert!(Number(10).part1_label_rank() > Number(9).part1_label_rank());
    assert!(Jack.part1_label_rank() > Number(10).part1_label_rank());
    assert!(Queen.part1_label_rank() > Jack.part1_label_rank());
    assert!(King.part1_label_rank() > Queen.part1_label_rank());
    assert!(Ace.part1_label_rank() > King.part1_label_rank());
}

#[test]
fn test_part1_hand_comparison() {
    fn get_sort_key(s: &str) -> SortKey {
        match parse_hand(s) {
            Ok(labels) => match part1_sort_key(&labels) {
                Ok(result) => result,
                Err(e) => {
                    panic!("invalid test input {labels:?}: {e}");
                }
            },
            Err(e) => {
                panic!("invalid test input {s}: {e}");
            }
        }
    }
    assert!(get_sort_key("32T3K") < get_sort_key("KTJJT"));
    assert!(get_sort_key("KTJJT") < get_sort_key("KK677"));
    assert!(get_sort_key("KK677") < get_sort_key("T55J5"));
    assert!(get_sort_key("T55J5") < get_sort_key("QQQJA"));

    assert_eq!(get_sort_key("T55J5"), get_sort_key("T55J5"));
}

#[test]
fn test_part2_hand_comparison() {
    fn get_sort_key(s: &str) -> SortKey {
        match parse_hand(s) {
            Ok(labels) => match part2_sort_key(&labels) {
                Ok(result) => result,
                Err(e) => {
                    panic!("invalid test input: {e}");
                }
            },
            Err(e) => {
                panic!("invalid test input: {e}");
            }
        }
    }
    assert!(get_sort_key("32T3K") < get_sort_key("KK677"));
    assert!(get_sort_key("KK677") < get_sort_key("T55J5"));
    assert!(get_sort_key("T55J5") < get_sort_key("QQQJA"));
    assert!(get_sort_key("QQQJA") < get_sort_key("KTJJT"));
    assert!(get_sort_key("JKKK2") < get_sort_key("QQQQ2"));
    assert!(get_sort_key("JJJJJ") < get_sort_key("22222"));

    assert!(get_sort_key("AAKKJ") < get_sort_key("AAKKK"));
    assert!(get_sort_key("AAKKJ") > get_sort_key("AAKKQ"));
}

#[test]
fn test_get_part1_hand_type_valid() {
    use HandType::*;
    fn get_hand_type(s: &str) -> HandType {
        match parse_hand(s) {
            Ok(labels) => match part1_sort_key(&labels) {
                Ok(SortKey {
                    hand_type,
                    label_indices: _,
                }) => hand_type,
                Err(e) => {
                    panic!("invalid test input {labels:?}: {e}");
                }
            },
            Err(e) => {
                panic!("invalid test input {s}: {e}");
            }
        }
    }

    assert_eq!(get_hand_type("AAAAA"), FiveOfAKind);
    assert_eq!(get_hand_type("AA8AA"), FourOfAKind);
    assert_eq!(get_hand_type("23332"), FullHouse);
    assert_eq!(get_hand_type("TTT98"), ThreeOfAKind);
    assert_eq!(get_hand_type("23432"), TwoPair);
    assert_eq!(get_hand_type("A23A4"), OnePair);
    assert_eq!(get_hand_type("23456"), HighCard);
}

#[test]
fn test_get_part1_hand_type_invalid_count() {
    fn get_hand_type(s: &str) -> Result<HandType, Fail> {
        parse_hand(s).and_then(|labels| get_part1_hand_type(&labels))
    }

    assert!(get_hand_type("").is_err());
    assert!(get_hand_type("2").is_err());
    assert!(get_hand_type("22").is_err());
    assert!(get_hand_type("333").is_err());
    assert!(get_hand_type("4444").is_err());
    assert!(get_hand_type("666666").is_err());
}

pub fn solve(lines: &[ParsedLine], make_key: SortKeyFn) -> Result<u64, Fail> {
    Ok(rank_hands(lines, make_key)?
        .into_iter()
        .map(|(rank, bid)| (rank as u64) * (bid as u64))
        .sum())
}

#[test]
fn test_solve() {
    const INPUT_TEXT: &str = concat!(
        "32T3K 765\n",
        "T55J5 684\n",
        "KK677 28\n",
        "KTJJT 220\n",
        "QQQJA 483\n",
    );
    let input = parse_input(INPUT_TEXT).expect("example input should be valid");
    assert_eq!(solve(&input, part1_sort_key), Ok(6440));
    assert_eq!(solve(&input, part2_sort_key), Ok(5905));
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

/// Reads the puzzle input.
fn get_input() -> String {
    let input = str::from_utf8(include_bytes!("input.txt")).unwrap();
    input.to_string()
}

fn main() {
    let input = parse_input(&get_input()).expect("puzzle input should be valid");
    println!(
        "day 07 part 1: {}",
        solve(&input, part1_sort_key).expect("data should be valid for part 1")
    );
    println!(
        "day 07 part 2: {}",
        solve(&input, part2_sort_key).expect("data should be valid for part 2")
    );
}
