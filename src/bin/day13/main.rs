use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
enum Symmetry {
    Horizontal(i64),
    Vertical(i64),
}

impl Debug for Symmetry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Symmetry::Horizontal(i) => write!(f, "Horizontal({i})"),
            Symmetry::Vertical(i) => write!(f, "Vertical({i})"),
        }
    }
}

impl Symmetry {
    fn score(&self) -> i64 {
        match self {
            Symmetry::Horizontal(x) => 1 + *x,
            Symmetry::Vertical(y) => 100 * (1 + *y),
        }
    }

    #[cfg(test)]
    fn reflection_point(&self) -> i64 {
        match self {
            Symmetry::Horizontal(n) | Symmetry::Vertical(n) => *n,
        }
    }

    #[cfg(test)]
    fn reflection_point_as_string(&self) -> String {
        let pos = self.reflection_point() as usize;
        format!("{:pos$}><", "")
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
enum SmudgeFix {
    Single(Position),
    Multiple,
}

impl TryFrom<Vec<Position>> for SmudgeFix {
    type Error = Fail;
    fn try_from(v: Vec<Position>) -> Result<SmudgeFix, Fail> {
        match v.as_slice() {
            [single] => Ok(SmudgeFix::Single(*single)),
            [_first, ..] => Ok(SmudgeFix::Multiple),
            [] => Err(Fail("mismatch vector should not be empty".to_string())),
        }
    }
}

impl From<Position> for SmudgeFix {
    fn from(fix: Position) -> SmudgeFix {
        SmudgeFix::Single(fix)
    }
}

impl SmudgeFix {
    fn merge(&mut self, _edits: SmudgeFix) {
        *self = SmudgeFix::Multiple;
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
struct SmudgeFixesNeeded {
    changes_needed: BTreeMap<Symmetry, SmudgeFix>,
}

impl SmudgeFixesNeeded {
    fn score_if_fixed(&self) -> i64 {
        self.changes_needed
            .iter()
            .map(|(sym, fix)| match fix {
                SmudgeFix::Single(_) => sym.score(),
                _ => 0,
            })
            .sum()
    }

    fn mismatches_at(axis: Symmetry, locations: Vec<Position>) -> SmudgeFixesNeeded {
        let changes_needed: BTreeMap<Symmetry, SmudgeFix> = [(
            axis,
            SmudgeFix::try_from(locations).expect("locations should not be empty"),
        )]
        .into_iter()
        .collect();
        SmudgeFixesNeeded { changes_needed }
    }

    fn new() -> SmudgeFixesNeeded {
        SmudgeFixesNeeded {
            changes_needed: Default::default(),
        }
    }

    fn union(mut self, other: SmudgeFixesNeeded) -> SmudgeFixesNeeded {
        for (axis, edits) in other.changes_needed.into_iter() {
            self.changes_needed
                .entry(axis)
                .and_modify(|fix| fix.merge(edits.clone()))
                .or_insert_with(|| edits.clone());
        }
        self
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SymmetryAssessment {
    Mismatch(SmudgeFixesNeeded),
    Symmetrical(BTreeSet<Symmetry>, SmudgeFixesNeeded),
    AllAxes,
}

impl SymmetryAssessment {
    fn symmetrical_about(axis: Symmetry) -> SymmetryAssessment {
        let symmetries = {
            let mut v = BTreeSet::new();
            v.insert(axis);
            v
        };
        SymmetryAssessment::Symmetrical(symmetries, SmudgeFixesNeeded::new())
    }

    fn empty_mismatch() -> SymmetryAssessment {
        SymmetryAssessment::Mismatch(SmudgeFixesNeeded::new())
    }

    fn smudge_summary_score(&self) -> i64 {
        match self {
            SymmetryAssessment::AllAxes => {
                panic!("it looks like your input pattern was had zero area");
            }
            SymmetryAssessment::Mismatch(smudge_fixes) => smudge_fixes.score_if_fixed(),
            SymmetryAssessment::Symmetrical(_, fixes) => fixes.score_if_fixed(),
        }
    }

    fn summary_score(&self) -> i64 {
        match self {
            SymmetryAssessment::AllAxes => {
                panic!("it looks like your input pattern was had zero area");
            }
            SymmetryAssessment::Mismatch(_) => 0,
            SymmetryAssessment::Symmetrical(symmetries, _) => {
                symmetries.iter().map(|sym| sym.score()).sum()
            }
        }
    }

    fn union(self, other: SymmetryAssessment) -> SymmetryAssessment {
        use SymmetryAssessment::*;
        match (self, other) {
            (AllAxes, _) | (_, AllAxes) => AllAxes,
            (Mismatch(smudges1), Mismatch(smudges2)) => Mismatch(smudges1.union(smudges2)),
            (Mismatch(smudges1), Symmetrical(v, smudges2))
            | (Symmetrical(v, smudges1), Mismatch(smudges2)) => {
                Symmetrical(v, smudges1.union(smudges2))
            }
            (Symmetrical(mut v1, smudges1), Symmetrical(mut v2, smudges2)) => {
                v1.append(&mut v2);
                Symmetrical(v1, smudges1.union(smudges2))
            }
        }
    }
}

#[cfg(test)]
fn show_line_of_reflection(pat: &Pattern, line: &Symmetry, show_at: i64) -> String {
    let terrain = match line {
        Symmetry::Horizontal(_) => pat.row_string(show_at),
        Symmetry::Vertical(_) => pat.column_string(show_at),
    };
    format!("{terrain}\n{}\n", line.reflection_point_as_string())
}

fn consistently_pick_one<'a>(p1: &'a Position, p2: &'a Position) -> &'a Position {
    match p1.x.cmp(&p2.x).then_with(|| p1.y.cmp(&p2.y)) {
        Ordering::Less => p1,
        Ordering::Greater => p2,
        Ordering::Equal => {
            // In this case it doesn't matter which we pick.
            p2
        }
    }
}

impl Pattern {
    fn reflection_area_would_be_empty(&self, axis: &Symmetry) -> bool {
        match axis {
            Symmetry::Horizontal(x) => *x < self.bbox.top_left.x || *x >= self.bbox.bottom_right.x,
            Symmetry::Vertical(y) => *y < self.bbox.top_left.y || *y >= self.bbox.bottom_right.y,
        }
    }

    #[cfg(test)]
    fn row_string(&self, y: i64) -> String {
        self.columns()
            .map(|x| self.get_marker(&Position { x, y }).unwrap_or('?'))
            .collect()
    }

    #[cfg(test)]
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

    fn symmetries(&self) -> SymmetryAssessment {
        self.horizontal_symmetries()
            .union(self.vertical_symmetries())
    }

    fn horizontal_symmetries(&self) -> SymmetryAssessment {
        self.columns()
            .rev()
            .skip(1)
            .map(|x| {
                let axis = Symmetry::Horizontal(x);
                let edits = self.horizontal_symmetry_mismatches(&axis);
                if edits.is_empty() {
                    SymmetryAssessment::symmetrical_about(axis)
                } else {
                    SymmetryAssessment::Mismatch(SmudgeFixesNeeded::mismatches_at(axis, edits))
                }
            })
            .fold(
                SymmetryAssessment::empty_mismatch(),
                SymmetryAssessment::union,
            )
    }

    fn vertical_symmetries(&self) -> SymmetryAssessment {
        self.rows()
            .rev()
            .skip(1)
            .map(|y| {
                let axis = Symmetry::Vertical(y);
                let edits = self.vertical_symmetry_mismatches(&axis);
                if edits.is_empty() {
                    SymmetryAssessment::symmetrical_about(axis)
                } else {
                    SymmetryAssessment::Mismatch(SmudgeFixesNeeded::mismatches_at(axis, edits))
                }
            })
            .fold(
                SymmetryAssessment::empty_mismatch(),
                SymmetryAssessment::union,
            )
    }

    fn check_point_pair_match(&self, p1: &Position, p2: &Position) -> bool {
        let marker1 = self.get_marker(p1);
        let marker2 = self.get_marker(p2);
        match (marker1, marker2) {
            (None, _) | (_, None) => true,
            (Some(m1), Some(m2)) => m1 == m2,
        }
    }

    fn row_symmetry_mismatches_for_axis(&self, axis: &Symmetry, y: i64) -> Vec<Position> {
        // axis is the possible reflection axis, y identifies the row
        // to check.
        let x_about = match axis {
            Symmetry::Horizontal(x) => *x,
            Symmetry::Vertical(_) => {
                panic!("is_row_symmetrical_about_axis: should only be called to check horizontal symmetry: {axis:?}");
            }
        };

        (0..=x_about)
            .map(|distance| (x_about - distance, 1 + x_about + distance))
            .map(|(x1, x2)| (Position { x: x1, y }, Position { x: x2, y }))
            .filter(|(p1, p2)| self.bbox.contains(p1) && self.bbox.contains(p2))
            .filter(|(p1, p2)| !self.check_point_pair_match(p1, p2))
            .map(|(p1, p2)| *consistently_pick_one(&p1, &p2))
            .collect()
    }

    fn column_symmetry_mismatches_for_axis(&self, x: i64, axis: &Symmetry) -> Vec<Position> {
        // axis is the possible vertical reflection axis, x identifies
        // the column to check.
        #[cfg(test)]
        println!(
            "checking for reflection axis at {axis:?} over column {x:3}: {}",
            self.column_string(x)
        );
        let y_about = match axis {
            Symmetry::Vertical(y) => *y,
            Symmetry::Horizontal(_) => {
                panic!("is_column_symmetrical_about_axis: should only be called on possible vertical symmetries: {axis:?}");
            }
        };
        (0..=y_about)
            .map(|distance| (y_about - distance, 1 + y_about + distance))
            .map(|(y1, y2)| (Position { x, y: y1 }, Position { x, y: y2 }))
            .filter(|(p1, p2)| self.bbox.contains(p1) && self.bbox.contains(p2))
            .filter(|(p1, p2)| !self.check_point_pair_match(p1, p2))
            .map(|(p1, p2)| *consistently_pick_one(&p1, &p2))
            .collect()
    }

    fn horizontal_symmetry_mismatches(&self, axis: &Symmetry) -> Vec<Position> {
        assert!(!self.reflection_area_would_be_empty(axis));
        self.rows()
            .flat_map(|y| self.row_symmetry_mismatches_for_axis(axis, y))
            .collect()
    }

    fn vertical_symmetry_mismatches(&self, axis: &Symmetry) -> Vec<Position> {
        assert!(!self.reflection_area_would_be_empty(axis));
        self.columns()
            .flat_map(|x| self.column_symmetry_mismatches_for_axis(x, axis))
            .collect()
    }
}

#[test]
fn test_horizontal_symmetry_first_pattern() {
    let first_pattern = get_examples()[0].clone();

    assert_eq!(
        first_pattern.horizontal_symmetry_mismatches(&Symmetry::Horizontal(4)),
        vec![]
    );

    assert!(!first_pattern
        .horizontal_symmetry_mismatches(&Symmetry::Horizontal(0))
        .is_empty());

    match first_pattern.horizontal_symmetries() {
        SymmetryAssessment::Symmetrical(syms, _fixes) => {
            assert!(syms.contains(&Symmetry::Horizontal(4)));
            assert_eq!(syms.len(), 1);
        }
        other => {
            panic!("expected first_pattern to be symmetrical about 4, but instead got: {other:?}");
        }
    }
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
        .horizontal_symmetry_mismatches(&axis_second_pattern_is_not_symmetrical_about)
        .is_empty());

    assert!(matches!(
        second_pattern.horizontal_symmetries(),
        SymmetryAssessment::Mismatch(_)
    ));
}

#[test]
fn test_vertical_symmetry_first_pattern() {
    let first_pattern = get_examples()[0].clone();
    // The first pattern cannot be symmetrical about row
    // 2 because Position{x:0,y:0) (#) should be reflected
    // at Position{x:0,y:5} (.) but it isn't.
    assert_eq!(
        first_pattern.column_symmetry_mismatches_for_axis(0, &Symmetry::Vertical(2)),
        vec![Position { x: 0, y: 0 }]
    );

    assert!(matches!(
        first_pattern.vertical_symmetries(),
        SymmetryAssessment::Mismatch(_)
    ));
}

#[test]
fn test_first_pattern_all_reflections() {
    let examples = get_examples();
    let first_pattern = examples[0].clone();
    match first_pattern.horizontal_symmetries() {
        SymmetryAssessment::Symmetrical(symmetries, _) => {
            assert_eq!(symmetries.len(), 1);
            assert!(symmetries.contains(&Symmetry::Horizontal(4)));
        }
        _ => {
            panic!("expected to see horizontal symmetry");
        }
    };

    assert!(matches!(
        first_pattern.vertical_symmetries(),
        SymmetryAssessment::Mismatch(_)
    ));

    match first_pattern.symmetries() {
        SymmetryAssessment::Symmetrical(syms, _) => {
            assert_eq!(syms.len(), 1);
            assert!(syms.contains(&Symmetry::Horizontal(4)));
        }
        _ => {
            panic!("expected to see horizontal symmetry");
        }
    }
}

#[test]
fn test_second_pattern_all_reflections() {
    let examples = get_examples();
    let second_pattern = examples[1].clone();
    match second_pattern.symmetries() {
        SymmetryAssessment::Symmetrical(syms, _) => {
            assert_eq!(syms.len(), 1);
            assert!(syms.contains(&Symmetry::Vertical(3)));
        }
        _ => {
            panic!("expected to see vertical symmetry");
        }
    }
}

fn part1(patterns: &[Pattern]) -> i64 {
    patterns
        .iter()
        .map(|pat| pat.symmetries().summary_score())
        .sum()
}

fn part2(patterns: &[Pattern]) -> i64 {
    patterns
        .iter()
        .map(|pat| pat.symmetries().smudge_summary_score())
        .sum()
}

#[test]
fn test_part1() {
    let examples = get_examples();

    let first_pattern = examples[0].clone();
    assert_eq!(part1(&[first_pattern]), 5);

    let second_pattern = examples[1].clone();
    assert_eq!(part1(&[second_pattern]), 400);

    assert_eq!(part1(&examples), 405);
}

#[test]
fn test_part2() {
    let examples = get_examples();
    assert_eq!(part2(&examples), 400);
}

#[test]
fn test_part2_first_pattern_all_reflections() {
    let examples = get_examples();
    let first_pattern = examples[0].clone();

    assert_eq!(
        first_pattern.column_symmetry_mismatches_for_axis(0, &Symmetry::Vertical(2)),
        vec![Position { x: 0, y: 0 }]
    );

    assert_eq!(
        first_pattern.vertical_symmetry_mismatches(&Symmetry::Vertical(2)),
        vec![Position { x: 0, y: 0 }]
    );

    match first_pattern.vertical_symmetries() {
        SymmetryAssessment::Mismatch(SmudgeFixesNeeded { changes_needed }) => {
            assert_eq!(
                changes_needed.get(&Symmetry::Vertical(2)),
                Some(&SmudgeFix::Single(Position { x: 0, y: 0 }))
            );
        }
        _ => {
            panic!("expected to see no vertical symmetries");
        }
    }
}

fn get_input() -> &'static str {
    str::from_utf8(include_bytes!("input.txt")).unwrap()
}

fn main() {
    let patterns = parse_input(get_input()).expect("puzzle input should be valid");
    println!("day 13 part 1: {}", part1(&patterns));
    println!("day 13 part 2: {}", part2(&patterns));
}
