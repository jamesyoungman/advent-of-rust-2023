use std::collections::BTreeSet;
use std::fmt::{Display, Formatter, Write};
use std::str;

use lib::grid::{BoundingBox, CompassDirection, Position, ALL_MOVE_OPTIONS};

use lib::error::Fail;

#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
struct Instruction {
    direction: CompassDirection,
    distance: i64,
}

fn parse_integer(s: &str) -> Result<i64, Fail> {
    match s.parse() {
        Ok(n) => Ok(n),
        Err(e) => Err(Fail(e.to_string())),
    }
}

fn parse_direction(s: &str) -> Result<CompassDirection, Fail> {
    use CompassDirection::*;
    match s {
        "U" => Ok(North),
        "D" => Ok(South),
        "L" => Ok(West),
        "R" => Ok(East),
        _ => Err(Fail(format!("unknown direction {s}"))),
    }
}

fn parse_line(s: &str) -> Result<Instruction, Fail> {
    match s.split_once(' ') {
        Some((dir, dist_and_colour)) => match dist_and_colour.split_once(' ') {
            Some((dist, _colour)) => Ok(Instruction {
                direction: parse_direction(dir)?,
                distance: parse_integer(dist)?,
            }),
            None => Err(Fail("colour field is missing".to_string())),
        },
        None => Err(Fail("line should contain spaces".to_string())),
    }
}

fn parse_input(s: &str) -> Result<Vec<Instruction>, Fail> {
    s.split_terminator('\n')
        .map(parse_line)
        .collect::<Result<Vec<Instruction>, Fail>>()
}

#[cfg(test)]
fn get_example() -> &'static str {
    concat!(
        "R 6 (#70c710)\n",
        "D 5 (#0dc571)\n",
        "L 2 (#5713f0)\n",
        "D 2 (#d2c081)\n",
        "R 2 (#59c680)\n",
        "D 2 (#411b91)\n",
        "L 5 (#8ceee2)\n",
        "U 2 (#caa173)\n",
        "L 1 (#1b58a2)\n",
        "U 2 (#caa171)\n",
        "R 2 (#7807d2)\n",
        "U 3 (#a77fa3)\n",
        "L 2 (#015232)\n",
        "U 2 (#7a21e3)\n",
    )
}

#[test]
fn test_parse_example() {
    let plan = parse_input(get_example()).expect("example should be valid");
    assert_eq!(plan.len(), 14);
    assert_eq!(
        plan[0],
        Instruction {
            direction: CompassDirection::East,
            distance: 6,
        }
    );
}

fn flood(
    start: &Position,
    bbox: &BoundingBox,
    cells: &mut BTreeSet<Position>,
    forbidden: &BTreeSet<Position>,
) {
    let mut iteration_count = 0;
    let iteration_limit = bbox.area() * 4;
    let mut frontier = Vec::new();
    frontier.push(*start);
    while let Some(pos) = frontier.pop() {
        iteration_count += 1;
        if iteration_count > iteration_limit {
            panic!("infinite loop in flood");
        }
        cells.insert(pos);
        for direction in ALL_MOVE_OPTIONS.iter() {
            let n = pos.move_direction(direction);
            if bbox.contains(&n) && !cells.contains(&n) && !forbidden.contains(&n) {
                frontier.push(n);
            }
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq)]
struct Grid {
    pos: Position,
    cubes: BTreeSet<Position>,
    bbox: BoundingBox,
}

impl Grid {
    fn new(start: Position) -> Grid {
        let mut cubes = BTreeSet::new();
        cubes.insert(start);
        Grid {
            bbox: BoundingBox::new(&start),
            pos: start,
            cubes,
        }
    }

    fn capacity(&self) -> i64 {
        self.cubes.len() as i64
    }

    fn dig_at(&mut self, pos: Position) {
        self.bbox.update(&pos);
        self.cubes.insert(pos);
        self.pos = pos;
    }

    fn dig(&mut self, direction: CompassDirection, dist: i64) {
        for _ in 0..dist {
            self.dig_at(self.pos.move_direction(&direction))
        }
    }

    fn find_interior(&self) -> BTreeSet<Position> {
        let enlarged_bbox = BoundingBox {
            top_left: Position {
                x: self.bbox.top_left.x - 1,
                y: self.bbox.top_left.y - 1,
            },
            bottom_right: Position {
                x: self.bbox.bottom_right.x + 1,
                y: self.bbox.bottom_right.y + 1,
            },
        };
        let mut exterior = BTreeSet::new();
        flood(
            &enlarged_bbox.top_left,
            &enlarged_bbox,
            &mut exterior,
            &self.cubes,
        );
        self.bbox
            .surface()
            .filter(|pos| !exterior.contains(pos))
            .collect()
    }

    fn excavate_interior(&mut self) {
        // changes to the interior will not affect the bounding box.
        self.cubes.extend(self.find_interior());
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        for y in self.bbox.top_left.y..=self.bbox.bottom_right.y {
            for x in self.bbox.top_left.x..=self.bbox.bottom_right.x {
                let here = Position { x, y };
                f.write_char(if self.cubes.contains(&here) { '#' } else { '.' })?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

fn dig_trenches(plan: &[Instruction]) -> Grid {
    let mut grid: Grid = Grid::new(Position { x: 0, y: 0 });
    for instruction in plan.iter() {
        grid.dig(instruction.direction, instruction.distance);
    }
    grid
}

#[test]
fn test_example_part1_dig() {
    let plan = parse_input(get_example()).expect("example should be valid");
    let grid = dig_trenches(&plan);
    assert_eq!(
        grid.to_string(),
        concat!(
            "#######\n",
            "#.....#\n",
            "###...#\n",
            "..#...#\n",
            "..#...#\n",
            "###.###\n",
            "#...#..\n",
            "##..###\n",
            ".#....#\n",
            ".######\n",
        )
    );
}

#[test]
fn test_example_part1_excavate_interior() {
    let plan = parse_input(get_example()).expect("example should be valid");
    let mut grid = dig_trenches(&plan);
    grid.excavate_interior();
    assert_eq!(
        grid.to_string(),
        concat!(
            "#######\n",
            "#######\n",
            "#######\n",
            "..#####\n",
            "..#####\n",
            "#######\n",
            "#####..\n",
            "#######\n",
            ".######\n",
            ".######\n",
        )
    );
}

fn part1(plan: &[Instruction]) -> i64 {
    let mut grid = dig_trenches(plan);
    grid.excavate_interior();
    grid.capacity()
}

#[test]
fn test_example_part1() {
    let plan = parse_input(get_example()).expect("example should be valid");
    assert_eq!(part1(&plan), 62);
}

fn main() {
    let input = str::from_utf8(include_bytes!("input.txt")).unwrap();
    let plan = parse_input(input).expect("input should be valid");
    println!("day 16 part 1: {}", part1(&plan));
}
