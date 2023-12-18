use std::collections::BTreeSet;
use std::fmt::{Display, Formatter, Write};
//use std::str;

use lib::grid::{BoundingBox, CompassDirection, Position};

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

fn execute_dig_plan(plan: &[Instruction]) -> Grid {
    let mut grid: Grid = Grid::new(Position { x: 0, y: 0 });
    for instruction in plan.iter() {
        grid.dig(instruction.direction, instruction.distance);
    }
    grid
}

#[test]
fn test_example_part1_dig() {
    let plan = parse_input(get_example()).expect("example should be valid");
    let grid = execute_dig_plan(&plan);
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

fn main() {}
