use std::collections::HashMap;
use std::str;

use lib::grid::Position;

fn is_symbol(ch: &char) -> bool {
    *ch != '.' && !ch.is_ascii_digit()
}

fn parse_input(input: &str) -> HashMap<Position, char> {
    let mut result = HashMap::new();
    for (y, line) in input.split_terminator('\n').enumerate() {
        for (x, ch) in line.chars().enumerate() {
            result.insert(
                Position {
                    x: x as i64,
                    y: y as i64,
                },
                ch,
            );
        }
    }
    result
}

fn has_symbol_neighbour(p: &Position, schematic: &HashMap<Position, char>) -> bool {
    for dy in [-1, 0, 1] {
        for dx in [-1, 0, 1] {
            if dx == 0 && dy == 0 {
                continue;
            }
            let neighbour = Position {
                x: p.x + dx,
                y: p.y + dy,
            };
            if let Some(ch) = schematic.get(&neighbour) {
                if is_symbol(ch) {
                    return true;
                }
            }
        }
    }
    false
}

fn extract_part_numbers(schematic: &HashMap<Position, char>) -> Vec<i64> {
    let mut result = Vec::new();
    let mut current_num: Option<i64> = None;
    let mut is_part_num: bool = false;
    for y in 0.. {
        for x in 0.. {
            let p = Position { x, y };
            match schematic.get(&p) {
                None => {
                    if x == 0 {
                        return result;
                    } else {
                        break;
                    }
                }
                Some(ch) => {
                    if let Some(digit_value) = ch.to_digit(10) {
                        current_num = Some(current_num.unwrap_or(0) * 10 + i64::from(digit_value));
                        if has_symbol_neighbour(&p, schematic) {
                            is_part_num = true;
                        }
                    } else if let Some(n) = current_num {
                        if is_part_num {
                            result.push(n);
                        }
                        current_num = None;
                        is_part_num = false;
                    }
                }
            }
        }
    }
    result
}

fn part1(schematic: &HashMap<Position, char>) -> i64 {
    extract_part_numbers(schematic).iter().sum()
}

#[test]
fn test_part1() {
    let example = concat!(
        "467..114..\n",
        "...*......\n",
        "..35..633.\n",
        "......#...\n",
        "617*......\n",
        ".....+.58.\n",
        "..592.....\n",
        "......755.\n",
        "...$.*....\n",
        ".664.598..\n",
    );
    assert_eq!(part1(&parse_input(example)), 4361);
}

fn get_input() -> HashMap<Position, char> {
    let input = str::from_utf8(include_bytes!("input.txt")).unwrap();
    parse_input(input)
}

fn main() {
    println!("day 03 part 1: {}", part1(&get_input()));
}
