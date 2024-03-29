use std::collections::HashSet;
use std::num::ParseIntError;
use std::str;

use lib::error::Fail;

/// Represents a single card.
#[derive(Debug)]
struct Card {
    have: HashSet<i32>,
    winners: HashSet<i32>,
}

impl Card {
    /// Counts how many wins a card has.
    fn count_wins(&self) -> usize {
        self.have
            .iter()
            .filter(|have| self.winners.contains(have))
            .count()
    }

    /// Computes the score we use for part 1 (only).
    fn part1_score(&self) -> u32 {
        (1..=self.count_wins()).fold(0, |acc, _| if acc == 0 { 1 } else { acc * 2 })
    }
}

#[test]
fn test_part1_score_0() {
    let c0 = Card {
        have: vec![1].into_iter().collect(),
        winners: vec![2].into_iter().collect(),
    };
    assert_eq!(c0.count_wins(), 0);
    assert_eq!(c0.part1_score(), 0);
}

#[test]
fn test_part1_score_1() {
    let c1 = Card {
        have: vec![1].into_iter().collect(),
        winners: vec![1].into_iter().collect(),
    };
    assert_eq!(c1.count_wins(), 1);
    assert_eq!(c1.part1_score(), 1);
}

#[test]
fn test_part1_score_2() {
    let c2 = Card {
        have: vec![6, 7, 9].into_iter().collect(),
        winners: vec![6, 7, 10].into_iter().collect(),
    };
    assert_eq!(c2.count_wins(), 2);
    assert_eq!(c2.part1_score(), 2);
}

#[test]
fn test_part1_score_3() {
    let c3 = Card {
        have: vec![6, 7, 9].into_iter().collect(),
        winners: vec![6, 7, 9].into_iter().collect(),
    };
    assert_eq!(c3.count_wins(), 3);
    assert_eq!(c3.part1_score(), 4);
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

/// Parses a card from an input string.
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

/// Parse a sequence of cards from an input string.
fn parse_input(s: &str) -> Result<Vec<Card>, Fail> {
    s.split_terminator('\n')
        .map(Card::try_from)
        .collect::<Result<Vec<Card>, Fail>>()
}

#[cfg(test)]
fn get_example() -> Vec<Card> {
    parse_input(concat!(
        "Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53\n",
        "Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19\n",
        "Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1\n",
        "Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83\n",
        "Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36\n",
        "Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11\n",
    ))
    .expect("example should be valid")
}

/// Computes the answer to part 1.
fn part1(cards: &[Card]) -> u32 {
    cards.iter().map(|card| card.part1_score()).sum()
}

#[test]
fn test_part1() {
    assert_eq!(part1(&get_example()), 13);
}

/// Determines the updated counts of cards we hold following a win.
///
/// Arguments
/// * `holding` - number of each card we have.  Cards appear in the same
///               order they appear in the input (IDs are ignored).
/// * `card_num` - the index of the card that won
/// * `wins` - the number of wins on card `card_num`.
fn won(mut holding: Vec<usize>, (card_num, wins): (usize, usize)) -> Vec<usize> {
    // Card `card_num` is a winner, with `wins` wins on it.  But we
    // hold `holding[card_num]` of that card.  For each one we hold,
    // we win a copy of each of the next `wins` cards.

    // Each winning card will win us an extra copy of one of more
    // following cards.
    let number_held_of_winning_card = holding[card_num];

    // We use `card_num+1` here because the first card of which we win
    // a copy is the one which immediately follows the winning card.
    for count in holding.iter_mut().skip(card_num + 1).take(wins) {
        *count += number_held_of_winning_card;
    }
    holding
}

/// Computes the final number of each card that we will hold after
/// taking into account all the wins.
fn part2_holding(cards: &[Card]) -> Vec<usize> {
    let initial_holding: Vec<usize> = {
        let mut v = Vec::with_capacity(cards.len());
        v.resize(cards.len(), 1); // Initially we have 1 of each card.
        v
    };
    cards
        .iter()
        .map(|card| card.count_wins())
        .enumerate()
        .fold(initial_holding, won)
}

/// Computes the answer to part 2.
fn part2(cards: &[Card]) -> usize {
    part2_holding(cards).iter().sum()
}

#[test]
fn test_part2() {
    assert_eq!(part2(&get_example()), 30);
}

/// Reads the puzzle input.
fn get_input() -> Vec<Card> {
    let input = str::from_utf8(include_bytes!("input.txt")).unwrap();
    parse_input(input).expect("input should be valid")
}

fn main() {
    let input = get_input();
    println!("day 04 part 1: {}", part1(&input));
    println!("day 04 part 2: {}", part2(&input));
}
