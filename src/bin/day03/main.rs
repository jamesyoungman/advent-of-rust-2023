use std::collections::HashMap;
use std::collections::HashSet;
use std::str;

use lib::grid::Position;

#[derive(Eq, PartialEq, Clone, Copy)]
enum Symbol {
    Gear(Position),
    Other,
}

impl Symbol {
    fn is_gear(&self) -> bool {
        matches!(self, Symbol::Gear(_))
    }
}

fn symbol_type(ch: char, pos: Position) -> Option<Symbol> {
    if ch == '*' {
        Some(Symbol::Gear(pos))
    } else if ch == '.' || ch.is_ascii_digit() {
        None
    } else {
        Some(Symbol::Other)
    }
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

fn symbol_neighbour(p: &Position, schematic: &HashMap<Position, char>) -> Option<Symbol> {
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
                let symtype = symbol_type(*ch, neighbour);
                if symtype.is_some() {
                    return symtype;
                }
            }
        }
    }
    None
}

fn extract_part_numbers(
    schematic: &HashMap<Position, char>,
) -> (Vec<i64>, HashMap<Position, HashSet<usize>>) {
    let mut result = Vec::new();
    let mut gears: HashMap<Position, HashSet<usize>> = HashMap::new();
    let mut current_num: Option<i64> = None;
    let mut associated_part: Option<Symbol> = None;

    for y in 0.. {
        for x in 0.. {
            let p = Position { x, y };
            match schematic.get(&p) {
                None => {
                    if x == 0 {
                        return (result, gears);
                    } else {
                        break;
                    }
                }
                Some(ch) => {
                    if let Some(digit_value) = ch.to_digit(10) {
                        current_num = Some(current_num.unwrap_or(0) * 10 + i64::from(digit_value));
                        let neighbour = symbol_neighbour(&p, schematic);
                        match &neighbour {
                            Some(Symbol::Gear(_)) => {
                                associated_part = neighbour;
                            }
                            Some(Symbol::Other)
                                if !associated_part.map(|sym| sym.is_gear()).unwrap_or(false) =>
                            {
                                associated_part = Some(Symbol::Other);
                            }
                            _ => (),
                        }
                    } else if let Some(n) = current_num {
                        match associated_part {
                            Some(Symbol::Other) => {
                                result.push(n);
                            }
                            Some(Symbol::Gear(gear_location)) => {
                                let part_num_index = result.len();
                                gears
                                    .entry(gear_location)
                                    .and_modify(|partnum_indices| {
                                        partnum_indices.insert(part_num_index);
                                    })
                                    .or_insert({
                                        let mut h = HashSet::new();
                                        h.insert(part_num_index);
                                        h
                                    });
                                result.push(n);
                            }
                            None => (),
                        }
                        current_num = None;
                        associated_part = None;
                    }
                }
            }
        }
    }
    let gears = gears
        .into_iter()
        .filter(|(_, indices)| indices.len() > 1)
        .collect();
    (result, gears)
}

fn part1(schematic: &HashMap<Position, char>) -> i64 {
    let (part_numbers, _) = extract_part_numbers(schematic);
    part_numbers.iter().sum()
}

#[cfg(test)]
fn get_example() -> String {
    concat!(
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
    )
    .to_string()
}

#[test]
fn test_part1() {
    let example = get_example();
    assert_eq!(part1(&parse_input(&example)), 4361);
}

fn part2(schematic: &HashMap<Position, char>) -> i64 {
    let (part_numbers, gear_locations) = extract_part_numbers(schematic);
    gear_locations
        .values()
        .filter(|partnum_indices| partnum_indices.len() > 1)
        .map(|partnum_indices| {
            partnum_indices
                .iter()
                .map(|index: &usize| part_numbers[*index])
                .product::<i64>()
        })
        .sum()
}

#[test]
fn test_part2() {
    let example = get_example();
    assert_eq!(part2(&parse_input(&example)), 467835);
}

fn get_input() -> HashMap<Position, char> {
    let input = str::from_utf8(include_bytes!("input.txt")).unwrap();
    parse_input(input)
}

fn main() {
    println!("day 03 part 1: {}", part1(&get_input()));
    println!("day 03 part 2: {}", part2(&get_input()));
}
