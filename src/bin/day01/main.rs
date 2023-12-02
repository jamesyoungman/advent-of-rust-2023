use regex::{Captures, Regex};
use std::error::Error;
use std::fmt::Display;
use std::str;

use lib::iterplus::sum_result;

#[derive(Debug, PartialEq, Eq)]
struct Fail(String);

impl Display for Fail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid: {}", self.0)
    }
}

impl Error for Fail {}

fn first_and_last(v: &[u32]) -> Result<(u32, u32), Fail> {
    match v {
        [] => Err(Fail("there are no digits".to_string())),
        [d] => Ok((*d, *d)),
        [d1, .., d2] => Ok((*d1, *d2)),
    }
}

fn first_and_last_p1(s: &str) -> Result<(u32, u32), Fail> {
    let digits: Vec<u32> = s.chars().filter_map(|ch| ch.to_digit(10)).collect();
    first_and_last(digits.as_slice())
}

#[test]
fn test_digit_extraction() {
    assert!(first_and_last_p1("").is_err());
    assert!(first_and_last_p1("foo").is_err());
    assert_eq!(first_and_last_p1("12"), Ok((1, 2)));
    assert_eq!(first_and_last_p1("3"), Ok((3, 3)));
    assert_eq!(first_and_last_p1("f6o9o"), Ok((6, 9)));
}

#[test]
fn test_part1() {
    let example = concat!("1abc2\n", "pqr3stu8vwx\n", "a1b2c3d4e5f\n", "treb7uchet\n",);
    assert_eq!(part1(example), Ok(142));
}

fn part1(s: &str) -> Result<u32, Fail> {
    s.lines()
        .map(|line| first_and_last_p1(line).map(|(left, right)| 10 * left + right))
        .try_fold(0, sum_result)
}

fn get_part2_digit(cap: &str) -> Result<u32, Fail> {
    match cap {
        "0" | "zero" => Ok(0),
        "1" | "one" => Ok(1),
        "2" | "two" => Ok(2),
        "3" | "three" => Ok(3),
        "4" | "four" => Ok(4),
        "5" | "five" => Ok(5),
        "6" | "six" => Ok(6),
        "7" | "seven" => Ok(7),
        "8" | "eight" => Ok(8),
        "9" | "nine" => Ok(9),
        _ => Err(Fail(format!("{cap} is not a digit"))),
    }
}

fn make_regexes() -> (Regex, Regex) {
    (
        // first digit
        Regex::new("^.*?([0123456789]|one|two|three|four|five|six|seven|eight|nine).*$").unwrap(),
        //  last figit
        Regex::new("^.*([0123456789]|one|two|three|four|five|six|seven|eight|nine).*?$").unwrap(),
    )
}

#[test]
fn test_p2_matchers() {
    let (first_matcher, last_matcher) = make_regexes();

    assert_eq!(
        first_matcher
            .captures("1")
            .unwrap()
            .get(1)
            .unwrap()
            .as_str(),
        "1"
    );
    assert_eq!(
        first_matcher
            .captures("21")
            .unwrap()
            .get(1)
            .unwrap()
            .as_str(),
        "2"
    );
    assert_eq!(
        last_matcher
            .captures("21")
            .unwrap()
            .get(1)
            .unwrap()
            .as_str(),
        "1"
    );
}

fn extract_match_str(m: Option<Captures<'_>>) -> &str {
    match m {
        Some(captures) => match captures.get(1) {
            Some(m) => m.as_str(),
            None => "",
        },
        None => "",
    }
}

fn first_and_last_p2(
    line: &str,
    first_matcher: &Regex,
    last_matcher: &Regex,
) -> Result<(u32, u32), Fail> {
    // The wrinkle here is that the first and last digit can overlap.
    let s = line.trim_end();
    let d1: u32 = get_part2_digit(extract_match_str(first_matcher.captures(s)))?;
    let d2: u32 = get_part2_digit(extract_match_str(last_matcher.captures(s)))?;
    first_and_last(&[d1, d2])
}

fn part2(s: &str) -> Result<u32, Fail> {
    let (first_matcher, last_matcher) = make_regexes();
    s.lines()
        .map(|line| {
            first_and_last_p2(line, &first_matcher, &last_matcher).map(|(a, b)| (10 * a + b))
        })
        .try_fold(0, sum_result)
}

#[test]
fn test_first_and_last_p2() {
    let (first_matcher, last_matcher) = make_regexes();

    let first_and_last = |s| first_and_last_p2(s, &first_matcher, &last_matcher);
    assert!(first_and_last("").is_err());
    assert!(first_and_last("foo").is_err());
    assert_eq!(first_and_last("one"), Ok((1, 1)));
    assert_eq!(first_and_last("two"), Ok((2, 2)));
    assert_eq!(first_and_last("twotwo"), Ok((2, 2)));
    assert_eq!(first_and_last("twotwo\n"), Ok((2, 2)));
    assert_eq!(first_and_last("12"), Ok((1, 2)));

    assert_eq!(first_and_last("two1nine\n"), Ok((2, 9)));
    assert_eq!(first_and_last("eightwothree\n"), Ok((8, 3)));
    assert_eq!(first_and_last("abcone2threexyz\n"), Ok((1, 3)));
    assert_eq!(first_and_last("xtwone3four\n"), Ok((2, 4)));
    assert_eq!(first_and_last("4nineeightseven2\n"), Ok((4, 2)));
    assert_eq!(first_and_last("zoneight234\n"), Ok((1, 4)));
    assert_eq!(first_and_last("7pqrstsixteen\n"), Ok((7, 6)));

    // I made this example up
    assert_eq!(first_and_last("twoone\n"), Ok((2, 1)));
    // I made this example up; note the numbers overlap.
    assert_eq!(first_and_last("twone\n"), Ok((2, 1)));
}

#[test]
fn test_part2() {
    assert_eq!(
        part2(concat!(
            "two1nine\n",
            "eightwothree\n",
            "abcone2threexyz\n",
            "xtwone3four\n",
            "4nineeightseven2\n",
            "zoneight234\n",
            "7pqrstsixteen\n"
        )),
        Ok(281)
    );
    assert_eq!(part2("eighttwo\nfotwooneg\n"), Ok(82 + 21));
}

fn main() {
    let input = str::from_utf8(include_bytes!("input.txt")).unwrap();
    println!(
        "part 1: {}",
        part1(input).expect("part 1 should have a solution")
    );
    println!(
        "part 2: {}",
        part2(input).expect("part 2 should have a solution")
    );
}
