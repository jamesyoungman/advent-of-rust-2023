use std::collections::HashSet;
use std::num::ParseIntError;
use std::str;

use lib::error::Fail;

#[derive(Debug)]
struct Card {
    have: HashSet<i32>,
    winners: HashSet<i32>,
}

impl Card {
    fn score(&self) -> u32 {
        self.have
            .iter()
            .filter(|have| self.winners.contains(have))
            .fold(0, |acc, _| if acc == 0 { 1 } else { acc * 2 })
    }
}

#[test]
fn test_score_0() {
    let c0 = Card {
        have: vec![1].into_iter().collect(),
        winners: vec![2].into_iter().collect(),
    };
    assert_eq!(c0.score(), 0);
}

#[test]
fn test_score_1() {
    let c1 = Card {
        have: vec![1].into_iter().collect(),
        winners: vec![1].into_iter().collect(),
    };
    assert_eq!(c1.score(), 1);
}

#[test]
fn test_score_2() {
    let c2 = Card {
        have: vec![6, 7, 9].into_iter().collect(),
        winners: vec![6, 7, 10].into_iter().collect(),
    };
    assert_eq!(c2.score(), 2);
}

#[test]
fn test_score_3() {
    let c3 = Card {
        have: vec![6, 7, 9].into_iter().collect(),
        winners: vec![6, 7, 9].into_iter().collect(),
    };
    assert_eq!(c3.score(), 4);
}

fn parse_number_list(s: &str) -> Result<HashSet<i32>, Fail> {
    s.split_whitespace()
        .map(|numstr| {
            numstr
                .parse()
                .map_err(|e: ParseIntError| Fail(format!("{numstr} is invalid: {e}")))
        })
        .collect()
}

impl TryFrom<&str> for Card {
    type Error = Fail;

    fn try_from(s: &str) -> Result<Card, Self::Error> {
        match s.split_once(": ") {
            Some((_prefix, tail)) => match tail.split_once(" | ") {
                Some((have, winners)) => Ok(Card {
                    have: parse_number_list(have)?,
                    winners: parse_number_list(winners)?,
                }),
                None => Err(Fail(format!("expected but did not find '|' in {tail}"))),
            },
            None => Err(Fail(format!("expected card id prefix: {s}"))),
        }
    }
}

fn parse_input(s: &str) -> Result<Vec<Card>, Fail> {
    s.split_terminator('\n')
        .map(Card::try_from)
        .collect::<Result<Vec<Card>, Fail>>()
}

fn part1(cards: &[Card]) -> u32 {
    cards.iter().map(|card| card.score()).sum()
}

#[test]
fn test_part1() {
    let example = concat!(
        "Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53\n",
        "Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19\n",
        "Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1\n",
        "Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83\n",
        "Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36\n",
        "Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11\n",
    );
    let cards = parse_input(example).expect("example should be valid");
    assert_eq!(part1(&cards), 13);
}

fn get_input() -> Vec<Card> {
    let input = str::from_utf8(include_bytes!("input.txt")).unwrap();
    parse_input(input).expect("input should be valid")
}

fn main() {
    println!("day 04 part 1: {}", part1(&get_input()));
}
