use std::str;

fn differences(v: &[i32]) -> Vec<i32> {
    v.windows(2).map(|w| w[1] - w[0]).collect()
}

#[test]
fn test_differences() {
    for (input, expected) in [
        (vec![0, 3, 6, 9, 12, 15], vec![3, 3, 3, 3, 3]),
        (vec![10, 13, 16, 21, 30, 45, 68], vec![3, 3, 5, 9, 15, 23]),
    ] {
        assert_eq!(differences(&input), expected);
    }
}

fn all_zero(v: &[i32]) -> bool {
    v.iter().all(|&n| n == 0)
}

fn compute_successive_diffs(input: Vec<i32>) -> Vec<Vec<i32>> {
    let mut result = Vec::new();
    result.push(input);
    // Compute the diffs
    while {
        let last = result.pop().unwrap();
        let diffs = differences(&last);
        let done = all_zero(&diffs);
        result.push(last);
        result.push(diffs);
        !done
    } {}
    result
}

#[test]
fn test_compute_successive_diffs() {
    assert_eq!(
        compute_successive_diffs(vec![10, 13, 16, 21, 30, 45, 68]),
        vec![
            vec![10, 13, 16, 21, 30, 45, 68],
            vec![3, 3, 5, 9, 15, 23],
            vec![0, 2, 4, 6, 8],
            vec![2, 2, 2, 2],
            vec![0, 0, 0]
        ]
    );
}

fn extrapolate(input: Vec<Vec<i32>>) -> Vec<Vec<i32>> {
    let mut endval = 0_i32;
    let mut result = Vec::with_capacity(input.len());
    for mut v in input.into_iter().rev() {
        endval += *v.last().unwrap();
        v.push(endval);
        result.push(v);
    }
    result.into_iter().rev().collect()
}

#[test]
fn test_extrapolate() {
    assert_eq!(
        extrapolate(vec![
            vec![10, 13, 16, 21, 30, 45],
            vec![3, 3, 5, 9, 15],
            vec![0, 2, 4, 6],
            vec![2, 2, 2],
            vec![0, 0]
        ],),
        vec![
            vec![10, 13, 16, 21, 30, 45, 68],
            vec![3, 3, 5, 9, 15, 23],
            vec![0, 2, 4, 6, 8],
            vec![2, 2, 2, 2],
            vec![0, 0, 0]
        ]
    );
}

fn predict_next_value(v: Vec<i32>) -> i32 {
    *extrapolate(compute_successive_diffs(v))
        .first()
        .unwrap()
        .last()
        .expect("input should not be empty")
}

#[test]
fn test_predict_next_value() {
    assert_eq!(predict_next_value(vec![10, 13, 16, 21, 30, 45]), 68);
}

fn part1(vv: Vec<Vec<i32>>) -> i32 {
    vv.into_iter().map(|v| predict_next_value(v)).sum()
}

#[test]
fn test_part1() {
    assert_eq!(
        part1(vec![
            vec![0, 3, 6, 9, 12, 15],
            vec![1, 3, 6, 10, 15, 21],
            vec![10, 13, 16, 21, 30, 45]
        ]),
        114
    );
}

fn number_seq(s: &str) -> Vec<i32> {
    s.split_whitespace()
        .map(|num| num.parse().expect("should be a valid number"))
        .collect()
}

#[test]
fn test_number_seq() {
    assert_eq!(
        number_seq("10 13 16 21 30 45"),
        vec![10, 13, 16, 21, 30, 45]
    );
}

fn main() {
    let input = str::from_utf8(include_bytes!("input.txt")).unwrap();
    let parsed_input: Vec<Vec<i32>> = input
        .split_terminator('\n')
        .map(|line| number_seq(line))
        .collect();
    println!("day 09 part 1: {}", part1(parsed_input));
}
