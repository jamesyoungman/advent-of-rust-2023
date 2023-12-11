use std::cmp::{max, Ordering};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter, Write};
use std::str;

use bimap::BiMap;

use lib::error::Fail;
use lib::grid::{manhattan, BoundingBox, Position};

#[derive(Debug)]
struct Image {
    occupied_rows: BTreeMap<i64, BTreeSet<i64>>,
    occupied_cols: BTreeMap<i64, BTreeSet<i64>>,
    bounds: BoundingBox,
}

impl Display for Image {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        for y in (self.bounds.top_left.y)..=(self.bounds.bottom_right.y) {
            match self.occupied_rows.get(&y) {
                Some(row) => {
                    for x in (self.bounds.top_left.x)..=(self.bounds.bottom_right.x) {
                        if row.contains(&x) {
                            f.write_char('#')?;
                        } else {
                            f.write_char('.')?;
                        }
                    }
                }
                None => {
                    for _ in (self.bounds.top_left.x)..=(self.bounds.bottom_right.x) {
                        f.write_char('.')?;
                    }
                }
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

impl Image {
    fn popcount(&self) -> usize {
        let answer_row_wise: usize = self.occupied_rows.values().map(|row| row.len()).sum();
        let answer_col_wise: usize = self.occupied_cols.values().map(|col| col.len()).sum();
        assert_eq!(answer_col_wise, answer_row_wise);
        answer_col_wise
    }

    #[cfg(test)]
    fn unoccupied_cols(&self) -> Vec<i64> {
        ((self.bounds.top_left.x)..=(self.bounds.bottom_right.x))
            .filter(|x| !self.occupied_cols.contains_key(&x))
            .collect()
    }

    #[cfg(test)]
    fn unoccupied_rows(&self) -> Vec<i64> {
        ((self.bounds.top_left.y)..=(self.bounds.bottom_right.y))
            .filter(|y| !self.occupied_rows.contains_key(&y))
            .collect()
    }
}

fn parse_input(s: &str) -> Result<Image, Fail> {
    let mut bbox: Option<BoundingBox> = None;
    let mut occupied_rows: BTreeMap<i64, BTreeSet<i64>> = BTreeMap::new();
    let mut occupied_cols: BTreeMap<i64, BTreeSet<i64>> = BTreeMap::new();
    for (y, line) in s.split_terminator('\n').enumerate() {
        for (x, ch) in line.chars().enumerate() {
            let pos = Position {
                x: x as i64,
                y: y as i64,
            };
            match bbox.as_mut() {
                None => {
                    bbox = Some(BoundingBox {
                        top_left: pos.clone(),
                        bottom_right: pos.clone(),
                    });
                }
                Some(b) => {
                    b.update(&pos);
                }
            }
            fn just(z: i64) -> BTreeSet<i64> {
                let mut result = BTreeSet::new();
                result.insert(z);
                result
            }
            if ch == '#' {
                occupied_cols
                    .entry(pos.x)
                    .and_modify(|col| {
                        col.insert(pos.y);
                    })
                    .or_insert_with(|| just(pos.y));
                occupied_rows
                    .entry(pos.y)
                    .and_modify(|row| {
                        row.insert(pos.x);
                    })
                    .or_insert_with(|| just(pos.x));
            }
        }
    }
    match bbox {
        Some(bounds) => Ok(Image {
            occupied_rows,
            occupied_cols,
            bounds,
        }),
        None => Err(Fail("empty input".to_string())),
    }
}

#[cfg(test)]
fn get_example() -> &'static str {
    concat!(
        "...#......\n",
        ".......#..\n",
        "#.........\n",
        "..........\n",
        "......#...\n",
        ".#........\n",
        ".........#\n",
        "..........\n",
        ".......#..\n",
        "#...#.....\n",
    )
}

#[cfg(test)]
fn get_example_image() -> Image {
    parse_input(get_example()).expect("example should be valid")
}

#[test]
fn test_parse() {
    let image =
        parse_input(concat!("....\n", ".##.\n", "..#.\n")).expect("test input should be valid");
    assert_eq!(image.popcount(), 3);
    assert_eq!(
        image.bounds,
        BoundingBox {
            top_left: Position { x: 0, y: 0 },
            bottom_right: Position { x: 3, y: 2 },
        }
    );
}

#[test]
fn test_image_display() {
    let representation = get_example();
    let img = parse_input(representation).expect("example should be valid");
    assert_eq!(&img.to_string(), representation);
}

#[test]
fn test_col_occupation() {
    let img = get_example_image();
    assert_eq!(img.unoccupied_cols(), vec![2, 5, 8]);
}

#[test]
fn test_row_occupation() {
    let img = get_example_image();
    assert_eq!(img.unoccupied_rows(), vec![3, 7]);
}

struct ExpandedImage<'a> {
    original: &'a Image,
    // x_map maps x values from the expanded coordinate system to the
    // original coordinate system.
    x_map: BiMap<i64, i64>,
    // y_map maps y values from the expanded coordinate system to the
    // original coordinate system.
    y_map: BiMap<i64, i64>,
    bounds: BoundingBox,
}

fn expand<'a>(img: &'a Image, expandby: i64) -> ExpandedImage<'a> {
    let extra_rows_or_columns = expandby - 1;
    let (x_map, max_x) = {
        let mut empty_col_count: i64 = 0;
        let mut x_map: BiMap<i64, i64> = Default::default();
        let mut max_x = img.bounds.top_left.x;
        for orig_x in (img.bounds.top_left.x)..=(img.bounds.bottom_right.x) {
            let expanded_x = empty_col_count + orig_x;
            max_x = max(max_x, expanded_x);
            if img.occupied_cols.contains_key(&orig_x) {
                x_map.insert(expanded_x, orig_x);
            } else {
                empty_col_count += extra_rows_or_columns;
            }
        }
        (x_map, max_x)
    };

    let (y_map, max_y) = {
        let mut empty_row_count: i64 = 0;
        let mut y_map: BiMap<i64, i64> = Default::default();
        let mut max_y = img.bounds.top_left.y;

        for orig_y in (img.bounds.top_left.y)..=(img.bounds.bottom_right.y) {
            let expanded_y = empty_row_count + orig_y;
            max_y = max(max_y, expanded_y);
            if img.occupied_rows.contains_key(&orig_y) {
                y_map.insert(expanded_y, orig_y);
            } else {
                empty_row_count += extra_rows_or_columns;
            }
        }
        (y_map, max_y)
    };

    ExpandedImage {
        original: img,
        x_map,
        y_map,
        bounds: BoundingBox {
            top_left: Position {
                x: img.bounds.top_left.x,
                y: img.bounds.top_left.y,
            },
            bottom_right: Position { x: max_x, y: max_y },
        },
    }
}

fn print_empty_row(f: &mut Formatter<'_>, len: i64) -> Result<(), std::fmt::Error> {
    for _ in 0..len {
        f.write_char('.')?;
    }
    Ok(())
}

impl<'a> Display for ExpandedImage<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        for y in (self.bounds.top_left.y)..=(self.bounds.bottom_right.y) {
            match self
                .y_map
                .get_by_left(&y)
                .map(|orig_y| self.original.occupied_rows.get(&orig_y))
                .flatten()
            {
                Some(row) => {
                    for x in (self.bounds.top_left.x)..=(self.bounds.bottom_right.x) {
                        match self.x_map.get_by_left(&x) {
                            Some(orig_x) => {
                                if row.contains(&orig_x) {
                                    f.write_char('#')?;
                                } else {
                                    f.write_char('.')?;
                                }
                            }
                            None => {
                                f.write_char('.')?;
                            }
                        }
                    }
                }
                None => {
                    let len = 1 + self.bounds.bottom_right.x - self.bounds.top_left.x;
                    print_empty_row(f, len)?;
                }
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

impl<'a> ExpandedImage<'a> {
    fn galaxies(&self) -> Vec<Position> {
        let mut result = Vec::with_capacity(self.original.popcount());
        for (y, xs) in self.original.occupied_rows.iter() {
            let expanded_y = self
                .y_map
                .get_by_right(y)
                .expect("y_map entry should exist");
            for x in xs.iter() {
                let expanded_x = self
                    .x_map
                    .get_by_right(x)
                    .expect("x_map entry should exist");
                result.push(Position {
                    x: *expanded_x,
                    y: *expanded_y,
                });
            }
        }
        result
    }

    fn galaxy_pairs(&self) -> Vec<(Position, Position)> {
        let mut result = Vec::new();
        let v = self.galaxies();
        fn galaxy_cmp(left: &Position, right: &Position) -> Ordering {
            left.x.cmp(&right.x).then_with(|| left.y.cmp(&right.y))
        }
        for first in v.iter() {
            for second in v.iter() {
                if galaxy_cmp(first, second) == Ordering::Less {
                    result.push((first.clone(), second.clone()));
                }
            }
        }
        result
    }
}

#[test]
fn test_expand() {
    let img = get_example_image();
    let expanded = expand(&img, 2);
    let expected = concat!(
        "....#........\n",
        ".........#...\n",
        "#............\n",
        ".............\n",
        ".............\n",
        "........#....\n",
        ".#...........\n",
        "............#\n",
        ".............\n",
        ".............\n",
        ".........#...\n",
        "#....#.......\n",
    );
    let got = expanded.to_string();
    println!("got:\n{got}");
    println!("expected:\n{expected}");
    assert_eq!(expanded.to_string(), expected);
}

fn sum_distances(expanded: &ExpandedImage<'_>) -> i64 {
    expanded
        .galaxy_pairs()
        .iter()
        .map(|(first, second)| manhattan(first, second))
        .sum()
}

fn part1(img: &Image) -> i64 {
    sum_distances(&expand(img, 2))
}

#[test]
fn test_part1() {
    let img = get_example_image();
    assert_eq!(part1(&img), 374);
}

fn part2(img: &Image) -> i64 {
    sum_distances(&expand(img, 1_000_000))
}

#[test]
fn test_expand_10_100() {
    let img = get_example_image();
    assert_eq!(sum_distances(&expand(&img, 10)), 1030);
    assert_eq!(sum_distances(&expand(&img, 100)), 8410);
}

fn main() {
    let input = str::from_utf8(include_bytes!("input.txt")).unwrap();
    let img = parse_input(input).expect("input should be valid");
    println!("day 11 part 1: {}", part1(&img));
    println!("day 11 part 2: {}", part2(&img));
}
