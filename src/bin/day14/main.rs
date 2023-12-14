use std::collections::BTreeMap;
use std::fmt::{Display, Write};
use std::str;

use lib::error::Fail;

use lib::grid::{BoundingBox, CompassDirection, Position};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Rock {
    Round,
    Cube,
}

impl Rock {
    fn as_char(&self) -> char {
        match self {
            Rock::Round => 'O',
            Rock::Cube => '#',
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Platform {
    rocks: BTreeMap<Position, Rock>,
    bbox: BoundingBox,
}

impl Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in self.bbox.top_left.y..=self.bbox.bottom_right.y {
            for x in self.bbox.top_left.x..=self.bbox.bottom_right.x {
                let here = Position { x, y };
                let ch = self
                    .rocks
                    .get(&here)
                    .map(|rock| rock.as_char())
                    .unwrap_or('.');
                f.write_char(ch)?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

fn parse_input(s: &str) -> Result<Platform, Fail> {
    let mut rocks = BTreeMap::new();
    let mut bbox: Option<BoundingBox> = None;
    for (y, line) in s.split_terminator('\n').enumerate() {
        for (x, ch) in line.chars().enumerate() {
            let here = Position {
                x: x as i64,
                y: y as i64,
            };
            match bbox.as_mut() {
                None => {
                    bbox = Some(BoundingBox::new(&here));
                }
                Some(b) => {
                    b.update(&here);
                }
            }
            match ch {
                '#' => {
                    rocks.insert(here, Rock::Cube);
                }
                'O' => {
                    rocks.insert(here, Rock::Round);
                }
                '.' => (),
                other => {
                    return Err(Fail(format!("unexpected input char {other}")));
                }
            }
        }
    }
    if let Some(bbox) = bbox {
        Ok(Platform { rocks, bbox })
    } else {
        Err(Fail("empty patterns are not allowed".to_string()))
    }
}

#[cfg(test)]
fn get_example() -> &'static str {
    concat!(
        "OOOO.#.O..\n",
        "OO..#....#\n",
        "OO..O##..O\n",
        "O..#.OO...\n",
        "........#.\n",
        "..#....#.#\n",
        "..O..#.O.O\n",
        "..O.......\n",
        "#....###..\n",
        "#....#....\n",
    )
}

#[cfg(test)]
fn get_parsed_example() -> Platform {
    parse_input(get_example()).expect("example should be valid")
}

#[test]
fn test_parse() {
    get_parsed_example();
}

#[test]
fn test_tilt() {
    let expected = tilted_north_example();
    let got = get_parsed_example().tilt(CompassDirection::North);
    println!("expected:\n{expected}\ngot:\n{got}");
    assert_eq!(got, expected);
}

fn direction_vector(direction: &CompassDirection) -> (i64, i64) {
    use CompassDirection::*;
    match direction {
        North => (0, -1),
        East => (1, 0),
        South => (0, 1),
        West => (-1, 0),
    }
}

fn next_pos(pos: &Position, (dx, dy): (i64, i64)) -> Position {
    Position {
        x: pos.x + dx,
        y: pos.y + dy,
    }
}

fn compute_final_position(
    mut pos: Position,
    direction: &CompassDirection,
    occupied: &BTreeMap<Position, Rock>,
    bounds: &BoundingBox,
) -> Position {
    let vector = direction_vector(direction);
    loop {
        let newpos = next_pos(&pos, vector);
        if (!bounds.contains(&newpos)) || occupied.contains_key(&newpos) {
            return pos;
        } else {
            pos = newpos;
        }
    }
}

impl Platform {
    fn popcount(&self, rock: &Rock) -> usize {
        self.rocks.values().filter(|r| *r == rock).count()
    }

    fn tilt(&self, direction: CompassDirection) -> Platform {
        // We need to order the rocks such that those closest to the
        // edge (in the direction of tilt) appear first.  This
        // simplifies the process of moving them.  This lambda
        // computes the order in which we should deal with the round
        // rocks.
        let rank = |pos: &Position| {
            match direction {
                // For tilting to the North or West, order the rocks
                // in increasing (respectively) y or x value.  For
                // tilting to the South or East, order the rocks in
                // the opposite sense (so that we deal with high
                // ordinate values dirst).
                CompassDirection::North => pos.y,
                CompassDirection::East => -pos.x,
                CompassDirection::South => -pos.y,
                CompassDirection::West => pos.x,
            }
        };

        let round_rocks_by_original_pos: BTreeMap<i64, Vec<Position>> = self
            .rocks
            .iter()
            .filter_map(|(pos, rock)| {
                if *rock == Rock::Round {
                    Some((rank(pos), *pos))
                } else {
                    None
                }
            })
            .fold(BTreeMap::new(), |mut acc, (rank, pos)| {
                acc.entry(rank)
                    .and_modify(|v| v.push(pos))
                    .or_insert_with(|| vec![pos]);
                acc
            });
        assert_eq!(
            self.popcount(&Rock::Round),
            round_rocks_by_original_pos.values().map(|v| v.len()).sum(),
            "We lost or gained some round rocks"
        );

        // Preserve the existing position of the cube rocks.
        let mut new_positions: BTreeMap<Position, Rock> = self
            .rocks
            .iter()
            .filter(|(_, rock)| **rock == Rock::Cube)
            .map(|(p, r)| (*p, *r))
            .collect();

        // Move the rounded rocks in the correct direction.
        for pos in round_rocks_by_original_pos
            .iter()
            .flat_map(|(_, pos)| pos.iter())
        {
            let newpos =
                compute_final_position(pos.clone(), &direction, &new_positions, &self.bbox);
            new_positions.insert(newpos, Rock::Round);
        }

        Platform {
            rocks: new_positions,
            bbox: self.bbox,
        }
    }

    fn rock_load(&self, pos: &Position, rock: &Rock, tilt_direction: CompassDirection) -> i64 {
        match rock {
            Rock::Cube => 0,
            Rock::Round => match tilt_direction {
                CompassDirection::North => 1 + self.bbox.bottom_right.y - pos.y,
                CompassDirection::South => 1 + pos.y - self.bbox.top_left.y,
                CompassDirection::East => 1 + pos.x - self.bbox.top_left.x,
                CompassDirection::West => 1 + self.bbox.bottom_right.x - pos.x,
            },
        }
    }

    fn loading(&self, direction: CompassDirection) -> i64 {
        self.rocks
            .iter()
            .map(|(pos, rock)| self.rock_load(pos, rock, direction))
            .sum()
    }
}

#[cfg(test)]
fn tilted_north_example() -> Platform {
    parse_input(concat!(
        "OOOO.#.O..\n", //  10
        "OO..#....#\n", //   9
        "OO..O##..O\n", //   8
        "O..#.OO...\n", //   7
        "........#.\n", //   6
        "..#....#.#\n", //   5
        "..O..#.O.O\n", //   4
        "..O.......\n", //   3
        "#....###..\n", //   2
        "#....#....\n", //   1
    ))
    .expect("tilted example should be valid")
}

#[test]
fn test_rock_load_cube() {
    let tilted_platform = tilted_north_example();
    for y in (tilted_platform.bbox.top_left.y)..=(tilted_platform.bbox.bottom_right.y) {
        for x in (tilted_platform.bbox.top_left.x)..=(tilted_platform.bbox.bottom_right.x) {
            assert_eq!(
                tilted_platform.rock_load(&Position { x, y }, &Rock::Cube, CompassDirection::North),
                0
            );
        }
    }
}

#[test]
fn test_rock_load_round() {
    let tilted_platform = tilted_north_example();
    assert_eq!(
        tilted_platform.rock_load(
            &Position { x: 0, y: 0 },
            &Rock::Round,
            CompassDirection::North
        ),
        10
    );
    assert_eq!(
        tilted_platform.rock_load(
            &Position { x: 0, y: 1 },
            &Rock::Round,
            CompassDirection::North
        ),
        9
    );
}

#[test]
fn test_loading() {
    let tilted_platform = tilted_north_example();
    assert_eq!(tilted_platform.loading(CompassDirection::North), 136);
}

fn part1(platform: &Platform) -> i64 {
    platform
        .tilt(CompassDirection::North)
        .loading(CompassDirection::North)
}

#[test]
fn test_part1() {
    let platform = get_parsed_example();
    assert_eq!(
        platform
            .tilt(CompassDirection::North)
            .loading(CompassDirection::North),
        136
    );
}

fn get_input() -> &'static str {
    str::from_utf8(include_bytes!("input.txt")).unwrap()
}

fn main() {
    let input = parse_input(get_input()).expect("puzzle input should be valid");
    println!("day 14 part 1: {}", part1(&input));
}
