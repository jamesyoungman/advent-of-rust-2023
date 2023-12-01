use regex::{Captures, Regex};
use std::str;

fn first_and_last(v: &[u32]) -> Option<(u32, u32)> {
    match v {
        [] => None,
        [d] => Some((*d, *d)),
        [d1, .., d2] => Some((*d1, *d2)),
    }
}

fn first_and_last_p1(s: &str) -> Option<(u32, u32)> {
    let digits: Vec<u32> = s.chars().filter_map(|ch| ch.to_digit(10)).collect();
    first_and_last(digits.as_slice())
}

#[test]
fn test_digit_extraction() {
    assert_eq!(first_and_last_p1(""), None);
    assert_eq!(first_and_last_p1("foo"), None);
    assert_eq!(first_and_last_p1("12"), Some((1, 2)));
    assert_eq!(first_and_last_p1("3"), Some((3, 3)));
    assert_eq!(first_and_last_p1("f6o9o"), Some((6, 9)));
}

#[test]
fn test_part1() {
    let example = concat!("1abc2\n", "pqr3stu8vwx\n", "a1b2c3d4e5f\n", "treb7uchet\n",);
    assert_eq!(part1(example), 142);
}

fn part1(s: &str) -> u32 {
    s.lines()
        .filter_map(|line| match first_and_last_p1(line) {
            Some((a, b)) => Some(10 * a + b),
            _ => None,
        })
        .sum()
}

fn get_part2_digit(cap: &str) -> Option<u32> {
    match cap {
        "0" | "zero" => Some(0),
        "1" | "one" => Some(1),
        "2" | "two" => Some(2),
        "3" | "three" => Some(3),
        "4" | "four" => Some(4),
        "5" | "five" => Some(5),
        "6" | "six" => Some(6),
        "7" | "seven" => Some(7),
        "8" | "eight" => Some(8),
        "9" | "nine" => Some(9),
        _ => None,
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
) -> Option<(u32, u32)> {
    // The wrinkle here is that the first and last digit can overlap.
    let s = line.trim_end();
    let d1: Option<u32> = get_part2_digit(extract_match_str(first_matcher.captures(s)));
    let d2: Option<u32> = get_part2_digit(extract_match_str(last_matcher.captures(s)));
    match (d1, d2) {
        (Some(d1), Some(d2)) => first_and_last(&[d1, d2]),
        _ => None,
    }
}

fn part2(s: &str) -> u32 {
    let (first_matcher, last_matcher) = make_regexes();
    s.lines()
        .filter_map(
            |line| match first_and_last_p2(line, &first_matcher, &last_matcher) {
                Some((a, b)) => Some(10 * a + b),
                _ => None,
            },
        )
        .sum()
}

#[test]
fn test_first_and_last_p2() {
    let (first_matcher, last_matcher) = make_regexes();

    let first_and_last = |s| first_and_last_p2(s, &first_matcher, &last_matcher);
    assert_eq!(first_and_last(""), None);
    assert_eq!(first_and_last("foo"), None);
    assert_eq!(first_and_last("one"), Some((1, 1)));
    assert_eq!(first_and_last("two"), Some((2, 2)));
    assert_eq!(first_and_last("twotwo"), Some((2, 2)));
    assert_eq!(first_and_last("twotwo\n"), Some((2, 2)));
    assert_eq!(first_and_last("12"), Some((1, 2)));

    assert_eq!(first_and_last("two1nine\n"), Some((2, 9)));
    assert_eq!(first_and_last("eightwothree\n"), Some((8, 3)));
    assert_eq!(first_and_last("abcone2threexyz\n"), Some((1, 3)));
    assert_eq!(first_and_last("xtwone3four\n"), Some((2, 4)));
    assert_eq!(first_and_last("4nineeightseven2\n"), Some((4, 2)));
    assert_eq!(first_and_last("zoneight234\n"), Some((1, 4)));
    assert_eq!(first_and_last("7pqrstsixteen\n"), Some((7, 6)));

    // I made this example up
    assert_eq!(first_and_last("twoone\n"), Some((2, 1)));
    // I made this example up; note the numbers overlap.
    assert_eq!(first_and_last("twone\n"), Some((2, 1)));
}

#[test]
fn test_part2() {
    let example = concat!(
        "two1nine\n",
        "eightwothree\n",
        "abcone2threexyz\n",
        "xtwone3four\n",
        "4nineeightseven2\n",
        "zoneight234\n",
        "7pqrstsixteen\n"
    );
    assert_eq!(part2(example), 281);
}

fn main() {
    let input = str::from_utf8(include_bytes!("input.txt")).unwrap();
    println!("part 1: {}", part1(input));
    println!("part 2: {}", part2(input));
    // 54871 is incorrect for part 2.
}
