use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt::{self, Debug, Display, Formatter, Write};
use std::str;

use lib::error::Fail;
use lib::grid::{bounds, CompassDirection, Position};

#[derive(Debug, PartialEq, Eq)]
struct Delta {
    x: i64,
    y: i64,
}

#[derive(Debug, PartialEq, Eq)]
enum Pipe {
    PipeF,
    PipeJ,
    PipeL,
    Pipe7,
    PipeH,
    PipeV,
}

impl TryFrom<char> for Pipe {
    type Error = Fail;
    fn try_from(ch: char) -> Result<Pipe, Self::Error> {
        match ch {
            '-' => Ok(Pipe::PipeH),
            '|' => Ok(Pipe::PipeV),
            'F' => Ok(Pipe::PipeF),
            'J' => Ok(Pipe::PipeJ),
            'L' => Ok(Pipe::PipeL),
            '7' => Ok(Pipe::Pipe7),
            _ => Err(Fail(format!("not a pipe character: {ch}"))),
        }
    }
}

impl Display for Pipe {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_char(match self {
            Pipe::PipeF => 'F',
            Pipe::PipeJ => 'J',
            Pipe::PipeL => 'L',
            Pipe::Pipe7 => '7',
            Pipe::PipeH => '-',
            Pipe::PipeV => '|',
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Grid {
    cells: HashMap<Position, Pipe>,
    start: Position,
}

impl Grid {
    fn neighbours(&self, pos: &Position) -> Vec<Position> {
        //dbg!(pos);
        let mut neighbours = match self.cells.get(pos) {
            None => vec![],
            Some(pipe) => match pipe {
                Pipe::PipeJ => vec![
                    pos.move_direction(&CompassDirection::North),
                    pos.move_direction(&CompassDirection::West),
                ],
                Pipe::PipeF => vec![
                    pos.move_direction(&CompassDirection::East),
                    pos.move_direction(&CompassDirection::South),
                ],
                Pipe::Pipe7 => vec![
                    pos.move_direction(&CompassDirection::West),
                    pos.move_direction(&CompassDirection::South),
                ],
                Pipe::PipeL => vec![
                    pos.move_direction(&CompassDirection::East),
                    pos.move_direction(&CompassDirection::North),
                ],
                Pipe::PipeH => vec![
                    pos.move_direction(&CompassDirection::East),
                    pos.move_direction(&CompassDirection::West),
                ],
                Pipe::PipeV => vec![
                    pos.move_direction(&CompassDirection::North),
                    pos.move_direction(&CompassDirection::South),
                ],
            },
        };
        neighbours.retain(|pos| self.cells.contains_key(pos));
        neighbours
    }

    fn identify_start_pos_pipe(&mut self) -> Result<(), Fail> {
        // Decide if the start pipe has an exit in each of the cardinal directions.
        let north = match self.cells.get(&Position {
            x: self.start.x,
            y: self.start.y - 1,
        }) {
            Some(Pipe::Pipe7) | Some(Pipe::PipeF) | Some(Pipe::PipeV) => true,
            _ => false,
        };
        let east = match self.cells.get(&Position {
            x: self.start.x + 1,
            y: self.start.y,
        }) {
            Some(Pipe::PipeJ) | Some(Pipe::Pipe7) | Some(Pipe::PipeH) => true,
            _ => false,
        };
        let south = match self.cells.get(&Position {
            x: self.start.x,
            y: self.start.y + 1,
        }) {
            Some(Pipe::PipeJ) | Some(Pipe::PipeL) | Some(Pipe::PipeV) => true,
            _ => false,
        };
        let west = match self.cells.get(&Position {
            x: self.start.x - 1,
            y: self.start.y,
        }) {
            Some(Pipe::PipeL) | Some(Pipe::PipeF) | Some(Pipe::PipeH) => true,
            _ => false,
        };
        let insufficient =
            || Fail("cannot determine start pipe type: insufficient exits".to_string());
        let toomany = || Fail("cannot determine start pipe type: too many exits".to_string());
        const F: bool = false;
        const T: bool = true;
        let pipe: Pipe = match (north, east, south, west) {
            (F, F, F, F) => Err(insufficient()),
            (F, F, F, T) => Err(insufficient()),
            (F, F, T, F) => Err(insufficient()),
            (F, F, T, T) => Ok(Pipe::Pipe7),
            (F, T, F, F) => Err(insufficient()),
            (F, T, F, T) => Ok(Pipe::PipeH),
            (F, T, T, F) => Ok(Pipe::PipeF),
            (F, T, T, T) => Err(toomany()),
            (T, F, F, F) => Err(insufficient()),
            (T, F, F, T) => Ok(Pipe::PipeJ),
            (T, F, T, F) => Ok(Pipe::PipeV),
            (T, F, T, T) => Err(toomany()),
            (T, T, F, F) => Ok(Pipe::PipeL),
            (T, T, F, T) => Err(toomany()),
            (T, T, T, F) => Err(toomany()),
            (T, T, T, T) => Err(toomany()),
        }?;
        self.cells.insert(self.start, pipe);
        Ok(())
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(bounds) = bounds(self.cells.keys()) {
            for y in (bounds.top_left.y)..=(bounds.bottom_right.y) {
                for x in (bounds.top_left.x)..=(bounds.bottom_right.x) {
                    let pos = Position { x, y };
                    if pos == self.start {
                        f.write_char('S')?;
                    } else {
                        match self.cells.get(&Position { x, y }) {
                            Some(pipe) => {
                                write!(f, "{pipe}")?;
                            }
                            None => {
                                f.write_char('.')?;
                            }
                        }
                    }
                }
                f.write_char('\n')?;
            }
        }
        Ok(())
    }
}

fn parse_input(s: &str) -> Result<Grid, Fail> {
    let mut cells = HashMap::new();
    let mut start: Option<Position> = None;
    for (y, line) in s.split_terminator('\n').enumerate() {
        for (x, ch) in line.chars().enumerate() {
            match ch {
                '.' => {
                    continue;
                }
                'S' => {
                    start = Some(Position {
                        x: x as i64,
                        y: y as i64,
                    });
                }
                'J' | 'L' | 'F' | '7' | '-' | '|' => {
                    let pipe = Pipe::try_from(ch)?;
                    cells.insert(
                        Position {
                            x: x as i64,
                            y: y as i64,
                        },
                        pipe,
                    );
                }
                _ => return Err(Fail(format!("unrecognised character {ch}"))),
            }
        }
    }
    if let Some(start) = start {
        let mut grid = Grid { cells, start };
        grid.identify_start_pos_pipe()?;
        Ok(grid)
    } else {
        Err(Fail("no known start position".to_string()))
    }
}

#[test]
fn test_parse_input() {
    let grid = parse_input(concat!(
        "7-F7-\n", ".FJ|7\n", "SJLL7\n", "|F--J\n", "LJ.LJ\n",
    ))
    .expect("test input is valid");
    let expected_start = Position { x: 0, y: 2 };
    assert_eq!(grid.start, expected_start);
    assert_eq!(grid.cells.get(&expected_start), Some(&Pipe::PipeF))
}

fn measure_distances(grid: &Grid) -> HashMap<Position, usize> {
    let mut frontier: VecDeque<(Position, usize)> = VecDeque::from([(grid.start, 0)]);
    let mut result: HashMap<Position, usize> = HashMap::new();
    result.insert(grid.start, 0);
    while let Some((pos, steps)) = frontier.pop_front() {
        for n in grid.neighbours(&pos) {
            result.entry(n).or_insert_with(|| {
                frontier.push_back((n, steps + 1));
                steps + 1
            });
        }
    }
    result
}

fn show_distances(distances: &HashMap<Position, usize>) {
    let mut inverted: BTreeMap<usize, Vec<Position>> = BTreeMap::new();
    for (pos, steps) in distances.iter() {
        inverted
            .entry(*steps)
            .and_modify(|v| v.push(*pos))
            .or_insert_with(|| vec![*pos]);
    }
    //dbg!(inverted);
}

fn part1(s: &str) -> Option<usize> {
    let grid = parse_input(s).expect("input should be valid");
    println!("{}", &grid);
    let distances: HashMap<Position, usize> = measure_distances(&grid);
    show_distances(&distances);
    distances.values().max().copied()
}

#[test]
fn test_part1() {
    let input = concat!("7-F7-\n", ".FJ|7\n", "SJLL7\n", "|F--J\n", "LJ.LJ\n",);
    assert_eq!(part1(input), Some(8));
}

fn main() {
    let input = str::from_utf8(include_bytes!("input.txt")).unwrap();
    println!(
        "day 10 part 1: {}",
        part1(input).expect("part 1 should have a solution")
    );
}
