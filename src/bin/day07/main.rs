use std::collections::HashMap;
use std::str;

use lib::error::Fail;

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
enum Card {
    Number(char),
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl TryFrom<char> for Card {
    type Error = Fail;

    fn try_from(ch: char) -> Result<Card, Self::Error> {
        match ch {
            '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => Ok(Card::Number(ch)),
            'T' => Ok(Card::Ten),
            'J' => Ok(Card::Jack),
            'Q' => Ok(Card::Queen),
            'K' => Ok(Card::King),
            'A' => Ok(Card::Ace),
            other => Err(Fail(format!("card {other} is not valid"))),
        }
    }
}

#[test]
fn test_card_ordering() {
    use Card::*;
    assert!(Number('3') > Number('2'));
    assert!(Number('4') > Number('3'));
    assert!(Number('5') > Number('4'));
    assert!(Number('6') > Number('5'));
    assert!(Number('7') > Number('6'));
    assert!(Number('8') > Number('7'));
    assert!(Number('9') > Number('8'));
    assert!((Ten) > Number('9'));
    assert!(Jack > Ten);
    assert!(Queen > Jack);
    assert!(King > Queen);
    assert!(Ace > King);
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
enum HandType {
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
struct Hand {
    hand_type: HandType,
    cards: [Card; 5],
}

fn get_hand_type(s: &str) -> Result<HandType, Fail> {
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

impl TryFrom<&str> for Hand {
    type Error = Fail;

    fn try_from(s: &str) -> Result<Hand, Self::Error> {
        let cards: Vec<Card> = s
            .chars()
            .take(6)
            .map(Card::try_from)
            .collect::<Result<Vec<Card>, Fail>>()?;
        match cards.as_slice() {
            [c1, c2, c3, c4, c5] => Ok(Hand {
                hand_type: get_hand_type(s)?,
                cards: [*c1, *c2, *c3, *c4, *c5],
            }),
            _ => Err(Fail(format!("expected 5 cards, got {}: {s}", s.len()))),
        }
    }
}

#[test]
fn test_hand_try_from_str() {
    assert_eq!(
        Hand::try_from("AAAAA"),
        Ok(Hand {
            hand_type: HandType::FiveOfAKind,
            cards: [Card::Ace, Card::Ace, Card::Ace, Card::Ace, Card::Ace]
        })
    );
    assert!(Hand::try_from("11111").is_err());
    assert!(Hand::try_from("qqqqq").is_err());
}

#[test]
fn test_hand_comparison() {
    assert!(Hand::try_from("32T3K").unwrap() < Hand::try_from("KTJJT").unwrap());
    assert!(Hand::try_from("KTJJT").unwrap() < Hand::try_from("KK677").unwrap());
    assert!(Hand::try_from("KK677").unwrap() < Hand::try_from("T55J5").unwrap());
    assert!(Hand::try_from("T55J5").unwrap() < Hand::try_from("QQQJA").unwrap());
}

fn parse_line(s: &str) -> Result<(Hand, u32), Fail> {
    match s.split_once(' ') {
        Some((hand, bid)) => match (Hand::try_from(hand)?, bid.parse::<u32>()) {
            (hand, Ok(bid)) => Ok((hand, bid)),
            (_, Err(e)) => Err(Fail(format!("{bid} is not a valid bid: {e}"))),
        },
        None => Err(Fail(format!("expected to find a space in {s}"))),
    }
}

#[test]
fn test_parse_line() {
    assert_eq!(
        parse_line("KTJJT 220"),
        Ok((
            Hand {
                hand_type: HandType::TwoPair,
                cards: [Card::King, Card::Ten, Card::Jack, Card::Jack, Card::Ten,],
            },
            220
        ))
    );
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

fn parse_input(s: &str) -> Result<Vec<(Hand, u32)>, Fail> {
    s.split_terminator('\n')
        .map(parse_line)
        .collect::<Result<Vec<(Hand, u32)>, Fail>>()
}

#[test]
fn test_parse_input() {
    use Card::*;
    use HandType::*;
    let input = parse_input(get_example()).expect("example should be valid");
    assert_eq!(input.len(), 5);
    const EXPECTED_FIRST_HAND: Hand = Hand {
        hand_type: OnePair,
        cards: [Number('3'), Number('2'), Ten, Number('3'), King],
    };

    assert_eq!((EXPECTED_FIRST_HAND, 765), input[0]);
}

/// Reads the puzzle input.
fn get_input() -> String {
    let input = str::from_utf8(include_bytes!("input.txt")).unwrap();
    input.to_string()
}

fn part1(s: &str) -> u32 {
    let mut hands = parse_input(s).expect("input should be valid");
    hands.sort();
    hands
        .iter()
        .enumerate()
        .map(|(i, (_hand, bid))| (1 + i as u32) * bid)
        .sum()
}

#[test]
fn test_part1() {
    assert_eq!(part1(&get_example()), 6440);
}

fn main() {
    println!("day 07 part 1: {}", part1(&get_input()));
}
