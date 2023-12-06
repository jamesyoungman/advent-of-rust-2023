use std::str;

enum Part {
    One,
    Two,
}

fn win(charge_time: i64, race_time: i64, record: i64) -> bool {
    // The charge_time is also equal to the speed we get.
    // The vehicle moves for (race_time - charge_time).
    //
    // Hence the distance moved is charge_time * (race_time - charge_time).
    //
    // We win if this is greater than `record`.  So the win condition is
    //
    // charge_time * (race_time - charge_time) > record
    charge_time > record / (race_time - charge_time)
}

fn charge_time_for_max_distance(race_time: i64) -> f64 {
    // The maximum distance we can travel in the race time is simply
    // the maximum of the distance-travelled function, and so we
    // differentiate it to find the maximum.  The maximum is at T/2.
    (race_time as f64) / 2.0
}

fn approx_win_region_width(race_time: i64, record: i64) -> f64 {
    let r = record as f64;
    let t = race_time as f64;
    // We use (and then simplify) the quadratic formula togive us the
    // distance between the roots of the quadratic equation
    // determining the win condition.
    (t * t - 4.0 * r).sqrt()
}

fn exact_win_region(race_time: i64, record: i64) -> (i64, i64) {
    // We use approx_win_region_width to determine the width of the win region,
    // rounding down on the left and up on the right.
    let halfwidth = approx_win_region_width(race_time, record) / 2.0;
    let lower = ((charge_time_for_max_distance(race_time) - halfwidth).floor()) as i64;
    let upper = (charge_time_for_max_distance(race_time) + halfwidth).ceil() as i64;
    let is_win = |x: &i64| win(*x, race_time, record);

    // The values for `lower` and `upper` are approximations, so we
    // check the nearby points to find the lowest and the highest
    // winning charge time.
    let lower = (lower..)
        .find(is_win)
        .expect("should be able to find lower bound");
    let upper = (0..upper)
        .rev() // searching right-to-left
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

fn count_ways_to_win(race_time: i64, record: i64) -> i64 {
    let (lower, upper) = exact_win_region(race_time, record);
    1 + upper - lower
}

#[test]
fn test_count_ways_to_win() {
    assert_eq!(count_ways_to_win(7, 9), 4);
    assert_eq!(count_ways_to_win(15, 40), 8);
    assert_eq!(count_ways_to_win(30, 200), 9);
}

fn parse_numbers_part1(s: &str) -> Vec<i64> {
    s.split_whitespace()
        .map(|s| s.parse().expect("should be a valid number"))
        .collect()
}

fn parse_numbers_part2(s: &str) -> Vec<i64> {
    let s: String = s.chars().filter(|ch| !ch.is_ascii_whitespace()).collect();
    match s.parse() {
        Ok(n) => vec![n],
        Err(e) => {
            panic!("{s} should be a valid number: {e}");
        }
    }
}

fn parse_numbers(s: &str, part: &Part) -> Vec<i64> {
    match part {
        Part::One => parse_numbers_part1(s),
        Part::Two => parse_numbers_part2(s),
    }
}

fn parse_input(s: &str, part: &Part) -> Vec<(i64, i64)> {
    match s.split_once('\n') {
        Some((time_line, distance_line)) => {
            let times_str = time_line
                .strip_prefix("Time:")
                .expect("expected Times: prefix");
            let dist_str = distance_line
                .strip_prefix("Distance:")
                .expect("expected Distance: prefix");
            let times = parse_numbers(times_str, part);
            let distances = parse_numbers(dist_str, part);
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
        parse_input(get_example(), &Part::One),
        vec![(7, 9), (15, 40), (30, 200)]
    );
    assert_eq!(
        parse_input(get_example(), &Part::Two),
        vec![(71530, 940200)]
    );
}

fn solve(input: &[(i64, i64)]) -> i64 {
    input
        .iter()
        .map(|(time, record)| count_ways_to_win(*time, *record))
        .product()
}

#[test]
fn test_part1() {
    let part1_times_records = parse_input(get_example(), &Part::One);
    assert_eq!(solve(&part1_times_records), 288);
}

#[test]
fn test_part2() {
    let part2_times_records = parse_input(get_example(), &Part::Two);
    assert_eq!(solve(&part2_times_records), 71503);
}

fn main() {
    let input = str::from_utf8(include_bytes!("input.txt")).unwrap();
    let part1_times_records = parse_input(input, &Part::One);
    println!("day 06 part 1: {}", solve(&part1_times_records));
    let part2_times_records = parse_input(input, &Part::Two);
    println!("day 06 part 2: {}", solve(&part2_times_records));
}
