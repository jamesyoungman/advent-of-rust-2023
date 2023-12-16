use itertools::Itertools;
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

impl Tile {
    fn as_char(&self) -> char {
        use Tile::*;
        match self {
            Empty => '.',
            DashSplitter => '-',
            PipeSplitter => '|',
            BackslashMirror => '\\',
            SlashMirror => '/',
        }
    }
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
            cells.insert(here.clone(), Tile::try_from(ch)?);
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

fn trace_beams(initial: Beam, grid: &Grid) -> HashMap<Position, usize> {
    let mut beam_counts = HashMap::new();
    let mut cycle_detector: HashSet<Beam> = HashSet::new();
    let mut todo = vec![initial];
    while let Some(mut beam) = todo.pop() {
        //eprintln!(
        //    "tracing a new beam {beam:?}: there are {} left to do after this one",
        //    todo.len()
        //);
        while let Some(tile) = grid.cells.get(&beam.pos) {
            //eprintln!("beam is now at {}", &beam.pos);
            if !cycle_detector.insert(beam.clone()) {
                // We have a cycle
                break;
            }
            let oldpos = beam.pos;
            beam_counts
                .entry(beam.pos)
                .and_modify(|n| *n += 1)
                .or_insert(1);
            beam = match beam.next(tile) {
                (b, None) => b,
                (b, Some(split_beam)) => {
                    if !cycle_detector.contains(&split_beam) {
                        todo.push(split_beam);
                    }
                    b
                }
            };
            assert!(beam.pos != oldpos);
        }
        // The current beam has now left the grid, so we are done with
        // it.
    }
    beam_counts
}

fn beam_counts_to_string(counts: &HashMap<Position, usize>, bbox: &BoundingBox) -> String {
    let mut result = String::with_capacity((bbox.area() + bbox.height()) as usize);
    for y in bbox.top_left.y..=bbox.bottom_right.y {
        for x in bbox.top_left.x..=bbox.bottom_right.x {
            let here = Position { x, y };
            if counts.contains_key(&here) {
                result.push('#');
            } else {
                result.push('.');
            }
        }
        result.push('\n');
    }
    result
}

fn part1(grid: &Grid) -> usize {
    let initial = Beam {
        direction: CompassDirection::East,
        pos: grid.bbox.top_left,
    };
    let beam_counts = trace_beams(initial, grid);
    eprintln!(
        "part 1: energised squares:\n{}",
        beam_counts_to_string(&beam_counts, &grid.bbox)
    );
    beam_counts.len()
}

#[test]
fn test_part1() {
    //let example = concat!(
    //    "######....\n",
    //    ".#...#....\n",
    //    ".#...#####\n",
    //    ".#...##...\n",
    //    ".#...##...\n",
    //    ".#...##...\n",
    //    ".#..####..\n",
    //    "########..\n",
    //    ".#######..\n",
    //    ".#...#.#..\n",
    //);
    let example = concat!(
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
    );
    let grid = parse_grid(example).expect("example should be valid");
    assert_eq!(part1(&grid), 46);
}

fn get_input() -> &'static str {
    str::from_utf8(include_bytes!("input.txt")).unwrap()
}

fn main() {
    let grid = parse_grid(get_input()).expect("input should be valid");
    println!("day 16 part 1: {}", part1(&grid));
}
