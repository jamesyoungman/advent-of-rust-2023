use std::str;

fn win(charge_time: i32, race_time: i32, record: i32) -> bool {
    charge_time > record / (race_time - charge_time)
}

fn approx_win_region_width(race_time: i32, record: i32) -> f64 {
    let r = f64::from(record);
    let t = f64::from(race_time);
    (t * t - 4.0 * r).sqrt()
}

fn charge_time_for_max_distance(race_time: i32) -> f64 {
    f64::from(race_time) / 2.0
}

fn exact_win_region(race_time: i32, record: i32) -> (i32, i32) {
    let halfwidth = approx_win_region_width(race_time, record) / 2.0;
    let lower = ((charge_time_for_max_distance(race_time) - halfwidth).floor()) as i32;
    let upper = (charge_time_for_max_distance(race_time) + halfwidth).ceil() as i32;
    let is_win = |x: &i32| win(*x, race_time, record);
    let lower = (lower..)
        .find(is_win)
        .expect("should be able to find lower bound");
    let upper = (0..upper)
        .rev()
        .find(is_win)
        .expect("should be able to find upper bound");
    (lower, upper)
}

#[test]
fn test_exact_win_region() {
    assert_eq!(exact_win_region(7, 9), (2, 5));
    assert_eq!(exact_win_region(15, 40), (4, 11));
    assert_eq!(exact_win_region(30, 200), (11, 19));
}

fn count_ways_to_win(race_time: i32, record: i32) -> i32 {
    let (lower, upper) = exact_win_region(race_time, record);
    1 + upper - lower
}

#[test]
fn test_count_ways_to_win() {
    assert_eq!(count_ways_to_win(7, 9), 4);
    assert_eq!(count_ways_to_win(15, 40), 8);
    assert_eq!(count_ways_to_win(30, 200), 9);
}

fn parse_numbers(s: &str) -> Vec<i32> {
    s.split_whitespace()
        .map(|s| s.parse().expect("should be a valid number"))
        .collect()
}

fn parse_input(s: &str) -> Vec<(i32, i32)> {
    match s.split_once('\n') {
        Some((time_line, distance_line)) => {
            let times_str = time_line
                .strip_prefix("Time:")
                .expect("expected Times: prefix");
            let dist_str = distance_line
                .strip_prefix("Distance:")
                .expect("expected Distance: prefix");
            let times = parse_numbers(times_str);
            let distances = parse_numbers(dist_str);
            times
                .iter()
                .copied()
                .zip(distances.iter().copied())
                .collect()
        }
        _ => {
            panic!("expected 2 lines");
        }
    }
}

#[cfg(test)]
fn get_example() -> &'static str {
    concat!("Time:      7  15   30\n", "Distance:  9  40  200\n",)
}

#[test]
fn test_parse_input() {
    assert_eq!(
        parse_input(get_example()),
        vec![(7, 9), (15, 40), (30, 200)]
    );
}

fn part1(input: &[(i32, i32)]) -> i32 {
    input
        .iter()
        .map(|(time, record)| count_ways_to_win(*time, *record))
        .product()
}

/// Reads the puzzle input.
fn get_input() -> Vec<(i32, i32)> {
    let input = str::from_utf8(include_bytes!("input.txt")).unwrap();
    parse_input(input)
}

fn main() {
    let times_records = get_input();
    println!("day 06 part 1: {}", part1(&times_records));
}
