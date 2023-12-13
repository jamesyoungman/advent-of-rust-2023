use std::collections::BTreeSet;
use std::ops::RangeInclusive;
use std::str;

use lib::error::Fail;
use lib::grid::{BoundingBox, Position};

#[derive(Debug, Clone)]
struct Pattern {
    rocks: BTreeSet<Position>,
    bbox: BoundingBox,
}

fn parse_pattern(s: &str) -> Result<Pattern, Fail> {
    let mut rocks = BTreeSet::new();
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
                    rocks.insert(here);
                }
                '.' => (),
                other => {
                    return Err(Fail(format!("unexpected input char {other}")));
                }
            }
        }
    }
    if let Some(bbox) = bbox {
        Ok(Pattern { rocks, bbox })
    } else {
        Err(Fail("empty patterns are not allowed".to_string()))
    }
}

fn parse_input(s: &str) -> Result<Vec<Pattern>, Fail> {
    s.split("\n\n")
        .map(parse_pattern)
        .collect::<Result<Vec<Pattern>, Fail>>()
}

#[cfg(test)]
fn get_examples() -> Vec<Pattern> {
    let input = concat!(
        "#.##..##.\n",
        "..#.##.#.\n",
        "##......#\n",
        "##......#\n",
        "..#.##.#.\n",
        "..##..##.\n",
        "#.#.##.#.\n",
        "\n",
        "#...##..#\n",
        "#....#..#\n",
        "..##..###\n",
        "#####.##.\n",
        "#####.##.\n",
        "..##..###\n",
        "#....#..#\n",
    );
    parse_input(input).expect("example input should be valid")
}

#[test]
fn test_parse_input() {
    let examples = get_examples();
    match examples.as_slice() {
        [a, b] => {
            assert_eq!(a.columns(), 0..=8);
            assert_eq!(a.rows(), 0..=6);

            assert_eq!(b.columns(), 0..=8);
            assert_eq!(b.rows(), 0..=6);
        }
        _ => {
            panic!("expected 2 patterns, got {}", examples.len());
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Symmetry {
    Horizontal(i64),
    Vertical(i64),
}

impl Symmetry {
    fn score(&self) -> i64 {
        match self {
            Symmetry::Horizontal(x) => 1 + *x,
            Symmetry::Vertical(y) => 100 * (1 + *y),
        }
    }

    fn reflection_point(&self) -> i64 {
        match self {
            Symmetry::Horizontal(n) | Symmetry::Vertical(n) => *n,
        }
    }

    fn reflection_point_as_string(&self) -> String {
        let pos = self.reflection_point() as usize;
        format!("{:pos$}><", "")
    }
}

fn show_line_of_reflection(pat: &Pattern, line: &Symmetry, show_at: i64) -> String {
    let terrain = match line {
        Symmetry::Horizontal(_) => pat.row_string(show_at),
        Symmetry::Vertical(_) => pat.column_string(show_at),
    };
    format!("{terrain}\n{}\n", line.reflection_point_as_string())
}

impl Pattern {
    fn reflection_area_would_be_empty(&self, axis: &Symmetry) -> bool {
        match axis {
            Symmetry::Horizontal(x) => *x < self.bbox.top_left.x || *x >= self.bbox.bottom_right.x,
            Symmetry::Vertical(y) => *y < self.bbox.top_left.y || *y >= self.bbox.bottom_right.y,
        }
    }

    fn row_string(&self, y: i64) -> String {
        self.columns()
            .map(|x| self.get_marker(&Position { x, y }).unwrap_or('?'))
            .collect()
    }

    fn column_string(&self, x: i64) -> String {
        self.rows()
            .map(|y| self.get_marker(&Position { x, y }).unwrap_or('?'))
            .collect()
    }

    fn get_marker(&self, pos: &Position) -> Option<char> {
        if self.bbox.contains(pos) {
            if self.rocks.contains(pos) {
                Some('#')
            } else {
                Some('.')
            }
        } else {
            None
        }
    }

    fn columns(&self) -> RangeInclusive<i64> {
        (self.bbox.top_left.x)..=(self.bbox.bottom_right.x)
    }

    fn rows(&self) -> RangeInclusive<i64> {
        (self.bbox.top_left.y)..=(self.bbox.bottom_right.y)
    }

    fn symmetries(&self) -> Vec<Symmetry> {
        let mut result = Vec::new();
        result.extend(self.horizontal_symmetries());
        result.extend(self.vertical_symmetries());
        result
    }

    fn scored_symmetries(&self) -> Vec<i64> {
        self.symmetries().iter().map(|sym| sym.score()).collect()
    }

    fn horizontal_symmetries(&self) -> Vec<Symmetry> {
        self.columns()
            .map(|x| Symmetry::Horizontal(x))
            .filter(|axis| self.has_horizontal_symmetry(&axis))
            .collect()
    }

    fn vertical_symmetries(&self) -> Vec<Symmetry> {
        self.rows()
            .map(|y| Symmetry::Vertical(y))
            .filter(|axis| self.has_vertical_symmetry(axis))
            .collect()
    }

    fn is_row_symmetrical_about_axis(&self, axis: &Symmetry, y: i64) -> bool {
        // axis is the possible reflection axis, y identifies the row
        // to check.
        let x_about = match axis {
            Symmetry::Horizontal(x) => *x,
            Symmetry::Vertical(_) => {
                panic!("is_row_symmetrical_about_axis: should only be called to check horizontal symmetry: {axis:?}");
            }
        };

        dbg!(x_about);
        (0..=x_about)
            .map(|distance| dbg!((x_about - distance, 1 + x_about + distance)))
            .filter(|(x1, x2)| {
                self.bbox.contains(&Position { x: *x1, y })
                    && self.bbox.contains(&Position { x: *x2, y })
            })
            .all(|(x1, x2)| {
                let lpos = Position { x: x1, y };
                let rpos = Position { x: x2, y };
                let left = self.get_marker(&lpos);
                let right = self.get_marker(&rpos);
                dbg!(&lpos);
                dbg!(&left);

                dbg!(&rpos);
                dbg!(&right);

                match (left, right) {
                    (None, _) | (_, None) => true,
                    (Some(l), Some(r)) => l == r,
                }
            })
    }

    fn is_column_symmetrical_about_axis(&self, x: i64, axis: &Symmetry) -> bool {
        // axis is the possible vertical reflection axis, x identifies
        // the column to check.
        println!(
            "checking for reflection axis at {axis:?} over column {x}: {}",
            self.column_string(x)
        );
        let y_about = match axis {
            Symmetry::Vertical(y) => *y,
            Symmetry::Horizontal(_) => {
                panic!("is_column_symmetrical_about_axis: should only be called on possible vertical symmetries: {axis:?}");
            }
        };
        dbg!(y_about);
        (0..=y_about)
            .map(|distance| {
                dbg!(&distance);
                dbg!((y_about - distance, 1 + y_about + distance))
            })
            .filter(|(y1, y2)| {
                assert!(y1 != y2);
                self.bbox.contains(&Position { x, y: *y1 })
                    && self.bbox.contains(&Position { x, y: *y2 })
            })
            .all(|(y1, y2)| {
                assert!(y1 < y2);
                let upper_pos = Position { x, y: y1 };
                let lower_pos = Position { x, y: y2 };
                let upper = self.get_marker(&upper_pos);
                let lower = self.get_marker(&lower_pos);
                dbg!(&upper_pos);
                dbg!(&lower_pos);

                dbg!(&upper);
                dbg!(&lower);
                // TODO the None cases are probably not needed any more
                match (upper, lower) {
                    (None, _) | (_, None) => true,
                    (Some(l), Some(r)) => l == r,
                }
            })
    }

    fn has_horizontal_symmetry(&self, axis: &Symmetry) -> bool {
        (!self.reflection_area_would_be_empty(axis))
            && self
                .rows()
                .all(|y| dbg!(self.is_row_symmetrical_about_axis(&axis, dbg!(y))))
    }

    fn has_vertical_symmetry(&self, axis: &Symmetry) -> bool {
        (!self.reflection_area_would_be_empty(axis))
            && self
                .columns()
                .all(|x| self.is_column_symmetrical_about_axis(x, &axis))
    }
}

#[test]
fn test_horizontal_symmetry_first_pattern() {
    let first_pattern = get_examples()[0].clone();

    assert!(first_pattern.has_horizontal_symmetry(&Symmetry::Horizontal(4)));
    assert!(!first_pattern.has_horizontal_symmetry(&Symmetry::Horizontal(0)));

    assert_eq!(
        first_pattern.horizontal_symmetries(),
        vec![Symmetry::Horizontal(4)]
    );
}

#[test]
fn test_horizontal_reflection_second_pattern() {
    let second_pattern = get_examples()[1].clone();
    let axis_second_pattern_is_not_symmetrical_about = Symmetry::Horizontal(4);
    println!(
        "Second pattern should not be symmetrical about column 4:\n{}",
        show_line_of_reflection(
            &second_pattern,
            &axis_second_pattern_is_not_symmetrical_about,
            0
        )
    );
    assert!(!second_pattern
        .is_row_symmetrical_about_axis(&axis_second_pattern_is_not_symmetrical_about, 0));
    assert_eq!(second_pattern.horizontal_symmetries(), vec![]);
}

#[test]
fn test_vertical_symmetry_first_pattern() {
    let first_pattern = get_examples()[0].clone();
    assert!(
        // The first pattern cannot be symmetrical about row
        // 2 because Position{x:0,y:0) (#) should be reflected
        // at Position{x:0,y:5} (.) but it isn't.
        !first_pattern.is_column_symmetrical_about_axis(0, &Symmetry::Vertical(2))
    );
    assert_eq!(first_pattern.vertical_symmetries(), vec![]);
}

#[test]
fn test_first_pattern_all_reflections() {
    let examples = get_examples();
    let first_pattern = examples[0].clone();
    assert_eq!(
        first_pattern.horizontal_symmetries(),
        vec![Symmetry::Horizontal(4)]
    );
    assert_eq!(first_pattern.vertical_symmetries(), vec![]);
    assert_eq!(first_pattern.symmetries(), vec![Symmetry::Horizontal(4)]);
}

#[test]
fn test_second_pattern_all_reflections() {
    let examples = get_examples();
    let second_pattern = examples[1].clone();
    assert_eq!(second_pattern.symmetries(), vec![Symmetry::Vertical(3)]);
}

#[test]
fn test_scored_symmetries() {
    let examples = get_examples();

    let first_pattern = examples[0].clone();
    assert_eq!(first_pattern.scored_symmetries(), vec![5]);

    let second_pattern = examples[1].clone();
    assert_eq!(second_pattern.scored_symmetries(), vec![400]);
}

fn part1(patterns: &[Pattern]) -> i64 {
    patterns
        .iter()
        .flat_map(|pat| pat.scored_symmetries())
        .sum()
}

#[test]
fn test_part1() {
    let examples = get_examples();
    assert_eq!(part1(&examples), 405);
}

fn get_input() -> &'static str {
    str::from_utf8(include_bytes!("input.txt")).unwrap()
}

fn main() {
    let patterns = parse_input(get_input()).expect("puzzle input should be valid");
    println!("day 13 part 1: {}", part1(&patterns));
}
