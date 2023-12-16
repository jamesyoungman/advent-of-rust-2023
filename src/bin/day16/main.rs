use lib::error::Fail;
use std::collections::{HashMap, HashSet};
use std::str;

use lib::grid::{BoundingBox, CompassDirection, Position};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Tile {
    Empty,
    DashSplitter,    // -
    PipeSplitter,    // |
    BackslashMirror, // \
    SlashMirror,     // /
}

impl TryFrom<char> for Tile {
    type Error = Fail;
    fn try_from(ch: char) -> Result<Tile, Self::Error> {
        use Tile::*;
        match ch {
            '-' => Ok(DashSplitter),
            '|' => Ok(PipeSplitter),
            '/' => Ok(SlashMirror),
            '\\' => Ok(BackslashMirror),
            '.' => Ok(Empty),
            other => Err(Fail(format!("unexpected character {other}"))),
        }
    }
}

#[derive(Debug, Clone)]
struct Grid {
    cells: HashMap<Position, Tile>,
    bbox: BoundingBox,
}

impl Grid {
    fn possible_start_points(&self) -> impl Iterator<Item = Beam> + '_ {
        use CompassDirection::*;
        let top = (self.bbox.top_left.x..self.bbox.bottom_right.x).map(|x| Beam {
            pos: Position {
                x,
                y: self.bbox.top_left.y,
            },
            direction: South,
        });
        let bottom = (self.bbox.top_left.x..self.bbox.bottom_right.x).map(|x| Beam {
            pos: Position {
                x,
                y: self.bbox.bottom_right.y,
            },
            direction: North,
        });
        let left = (self.bbox.top_left.y..self.bbox.bottom_right.y).map(|y| Beam {
            pos: Position {
                x: self.bbox.top_left.x,
                y,
            },
            direction: East,
        });
        let right = (self.bbox.top_left.y..self.bbox.bottom_right.y).map(|y| Beam {
            pos: Position {
                x: self.bbox.bottom_right.x,
                y,
            },
            direction: West,
        });
        left.chain(right).chain(top).chain(bottom)
    }
}

fn parse_grid(s: &str) -> Result<Grid, Fail> {
    let mut here = Position { x: 0, y: 0 };
    let mut cells = HashMap::new();
    let mut bbox = BoundingBox::new(&here);
    for ch in s.chars() {
        if ch == '\n' {
            if here.y == 0 && here.x == 0 {
                // Ignore so that the bounding box stays correct.
                continue;
            }
            here.x = 0;
            here.y += 1;
        } else {
            cells.insert(here, Tile::try_from(ch)?);
            bbox.update(&here);
            here.x += 1;
        }
    }
    Ok(Grid { cells, bbox })
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Beam {
    pos: Position,
    direction: CompassDirection,
}

impl Beam {
    fn next(self, tile: &Tile) -> (Beam, Option<Beam>) {
        use CompassDirection::*;
        let (updated_direction, new_beam_direction): (CompassDirection, Option<CompassDirection>) =
            match tile {
                Tile::Empty => (self.direction, None),
                Tile::DashSplitter => match self.direction {
                    East | West => (self.direction, None),
                    North | South => (East, Some(West)),
                },
                Tile::PipeSplitter => match self.direction {
                    North | South => (self.direction, None),
                    East | West => (North, Some(South)),
                },
                Tile::SlashMirror => (
                    match self.direction {
                        North => East,
                        East => North,
                        South => West,
                        West => South,
                    },
                    None,
                ),
                Tile::BackslashMirror => (
                    match self.direction {
                        North => West,
                        East => South,
                        South => East,
                        West => North,
                    },
                    None,
                ),
            };
        (
            Beam {
                pos: self.pos.move_direction(&updated_direction),
                direction: updated_direction,
            },
            new_beam_direction.map(|direction| Beam {
                pos: self.pos.move_direction(&direction),
                direction,
            }),
        )
    }
}

fn trace_beams(initial: Beam, grid: &Grid) -> HashSet<Position> {
    let mut energised = HashSet::new();
    let mut cycle_detector: HashSet<Beam> = HashSet::new();
    let mut todo = vec![initial];
    while let Some(mut beam) = todo.pop() {
        while let Some(tile) = grid.cells.get(&beam.pos) {
            //eprintln!("beam is now at {}", &beam.pos);
            if !cycle_detector.insert(beam.clone()) {
                // We have a cycle
                break;
            }
            energised.insert(beam.pos);
            beam = match beam.next(tile) {
                (b, None) => b,
                (b, Some(split_beam)) => {
                    if !cycle_detector.contains(&split_beam) {
                        todo.push(split_beam);
                    }
                    b
                }
            };
        }
        // The current beam has now left the grid, so we are done with
        // it.
    }
    energised
}

fn count_energised_squares(initial: Beam, grid: &Grid) -> usize {
    trace_beams(initial, grid).len()
}

fn part1(grid: &Grid) -> usize {
    count_energised_squares(
        Beam {
            direction: CompassDirection::East,
            pos: grid.bbox.top_left,
        },
        grid,
    )
}

#[cfg(test)]
fn get_example() -> &'static str {
    concat!(
        r".|...\....",
        "\n",
        r"|.-.\.....",
        "\n",
        r".....|-...",
        "\n",
        r"........|.",
        "\n",
        r"..........",
        "\n",
        r".........\",
        "\n",
        r"..../.\\..",
        "\n",
        r".-.-/..|..",
        "\n",
        r".|....-|.\",
        "\n",
        r"..//.|....",
        "\n",
    )
}

#[test]
fn test_part1() {
    let grid = parse_grid(get_example()).expect("example should be valid");
    assert_eq!(part1(&grid), 46);
}

fn part2(grid: &Grid) -> usize {
    grid.possible_start_points()
        .map(|start| count_energised_squares(start, grid))
        .max()
        .unwrap_or(0)
}

#[test]
fn test_part2() {
    let grid = parse_grid(get_example()).expect("example should be valid");
    assert_eq!(part2(&grid), 51);
}

fn get_input() -> &'static str {
    str::from_utf8(include_bytes!("input.txt")).unwrap()
}

fn main() {
    let grid = parse_grid(get_input()).expect("input should be valid");
    println!("day 16 part 1: {}", part1(&grid));
    println!("day 16 part 2: {}", part2(&grid));
}
