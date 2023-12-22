use std::cmp::{max, min};
use std::collections::{BTreeMap, HashSet};
use std::fmt::{Debug, Display};
use std::str;

use lib::error::Fail;
use lib::grid::{BoundingBox, Position};

#[derive(PartialEq, Eq, Hash, Clone)]
struct Position3 {
    x: i64,
    y: i64,
    z: i64,
}

/// Position3 values sort by z first, so that we can order them by height-above-ground.
impl Ord for Position3 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.z
            .cmp(&other.z)
            .then(self.x.cmp(&other.x))
            .then(self.y.cmp(&other.y))
    }
}

impl PartialOrd for Position3 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for Position3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{},{}", self.x, self.y, self.z)
    }
}
impl Debug for Position3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (x, y, z) = (self.x, self.y, self.z);
        write!(f, "Position3{{x:{x},y:{y},z:{z}}}")
    }
}

impl TryFrom<&str> for Position3 {
    type Error = Fail;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if let [x, y, z] = s
            .split(',')
            .map(|s| {
                s.parse::<i64>()
                    .map_err(|e| Fail(format!("{s} is not a valid 3D point: {e}")))
            })
            .collect::<Result<Vec<i64>, Fail>>()?
            .as_slice()
        {
            Ok(Position3 {
                x: *x,
                y: *y,
                z: *z,
            })
        } else {
            Err(Fail(format!("not a valid 3D point: {s}")))
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct Brick {
    lower: Position3,
    upper: Position3,
    label: Option<String>,
}

impl Display for Brick {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.label.as_ref() {
            Some(label) => write!(f, "{}~{} <- {label}", self.lower, self.upper),
            None => write!(f, "{}~{}", self.lower, self.upper),
        }
    }
}

impl Debug for Brick {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl TryFrom<&str> for Brick {
    type Error = Fail;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if let Some((left, right)) = s.split_once('~') {
            let (right, label) = match right.split_once("<-") {
                Some((r, s)) => (r.trim(), Some(s.trim().to_string())),
                None => (right, None),
            };
            let left = Position3::try_from(left)?;
            let right = Position3::try_from(right)?;
            if left.z <= right.z {
                Ok(Brick {
                    lower: left,
                    upper: right,
                    label,
                })
            } else {
                Ok(Brick {
                    lower: right,
                    upper: left,
                    label,
                })
            }
        } else {
            Err(Fail(format!("expected '~' in {s}")))
        }
    }
}

impl Ord for Brick {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.lower
            .cmp(&other.lower)
            .then(self.upper.cmp(&other.upper))
    }
}

impl PartialOrd for Brick {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[test]
fn brick_comarison() {
    // From the example:
    //
    // ```
    // 1,0,1~1,2,1   <- A
    // 0,0,2~2,0,2   <- B
    // 0,2,3~2,2,3   <- C
    // 0,0,4~0,2,4   <- D
    // 2,0,5~2,2,5   <- E
    // 0,1,6~2,1,6   <- F
    // 1,1,8~1,1,9   <- G
    // ```
    //
    // Looking along the y axis, these are arranged:
    //
    // ```
    //  x
    // 012
    // .G. 9
    // .G. 8
    // ... 7
    // FFF 6
    // ..E 5 z
    // D.. 4
    // CCC 3
    // BBB 2
    // .A. 1
    // --- 0
    // ```
    let e = Brick::try_from("2,0,5~2,2,5").expect("Brick E should be valid");
    let f = Brick::try_from("0,1,6~2,1,6").expect("Brick F should be valid");
    assert!(e < f);
    assert!(e <= f);
    assert!(f > e);
    assert!(f >= e);

    assert!(!(e > f));
    assert!(!(e >= f));

    assert!(!(f < e));
    assert!(!(f <= e));
}

#[test]
fn brick_equality() {
    let e = Brick::try_from("2,0,5~2,2,5").expect("Brick E should be valid");
    let f = Brick::try_from("0,1,6~2,1,6").expect("Brick F should be valid");
    let f_twin = Brick::try_from("0,1,6~2,1,6").expect("Brick F (twin) should be valid");
    assert_eq!(e, e);
    assert_eq!(f, f);

    assert_eq!(f, f_twin);
    assert_eq!(f_twin, f);

    assert!(!(e == f));
    assert!(!(f == e));
}

impl Brick {
    fn plan(&self) -> BoundingBox {
        BoundingBox {
            top_left: Position {
                x: min(self.lower.x, self.upper.x),
                y: min(self.lower.y, self.upper.y),
            },
            bottom_right: Position {
                x: max(self.lower.x, self.upper.x),
                y: max(self.lower.y, self.upper.y),
            },
        }
    }
}

#[test]
fn brick_plan() {
    let brick = Brick::try_from("2,0,5~2,2,5").expect("Brick E should be valid");
    assert_eq!(
        brick.plan(),
        BoundingBox {
            top_left: Position { x: 2, y: 0 },
            bottom_right: Position { x: 2, y: 2 }
        }
    );
}

#[cfg(test)]
fn get_unlabeled_example() -> &'static str {
    concat!(
        "1,0,1~1,2,1\n",
        "0,0,2~2,0,2\n",
        "0,2,3~2,2,3\n",
        "0,0,4~0,2,4\n",
        "2,0,5~2,2,5\n",
        "0,1,6~2,1,6\n",
        "1,1,8~1,1,9\n",
    )
}

#[cfg(test)]
fn get_labeled_example() -> &'static str {
    concat!(
        "1,0,1~1,2,1   <- A\n",
        "0,0,2~2,0,2   <- B\n",
        "0,2,3~2,2,3   <- C\n",
        "0,0,4~0,2,4   <- D\n",
        "2,0,5~2,2,5   <- E\n",
        "0,1,6~2,1,6   <- F\n",
        "1,1,8~1,1,9   <- G\n",
    )
}

fn parse_input(s: &str) -> Result<Vec<Brick>, Fail> {
    s.split_terminator('\n')
        .map(Brick::try_from)
        .collect::<Result<Vec<Brick>, Fail>>()
}

#[test]
fn test_parse_unlabeled_example() {
    let bricks = parse_input(get_unlabeled_example()).expect("example should be valid");
    assert_eq!(bricks.len(), 7);
    assert_eq!(
        &bricks[0],
        &Brick {
            lower: Position3 { x: 1, y: 0, z: 1 },
            upper: Position3 { x: 1, y: 2, z: 1 },
            label: None,
        }
    );
}

#[test]
fn test_parse_labeled_example() {
    let bricks = parse_input(get_labeled_example()).expect("labeled example should be valid");
    assert_eq!(bricks.len(), 7);
    assert_eq!(
        &bricks[0],
        &Brick {
            lower: Position3 { x: 1, y: 0, z: 1 },
            upper: Position3 { x: 1, y: 2, z: 1 },
            label: Some("A".to_string()),
        }
    );
}

#[derive(Debug, Default)]
struct Surface {
    heightmap: BTreeMap<Position, (i64, usize)>,
}

impl Surface {
    fn get(&self, pos: &Position) -> (i64, Option<usize>) {
        match self.heightmap.get(pos) {
            Some((h, index)) => (*h, Some(*index)),
            None => (0, None),
        }
    }

    fn set_height(&mut self, bbox: &BoundingBox, z: i64, index: usize) {
        for pos in bbox.surface() {
            self.heightmap
                .entry(pos)
                .and_modify(|(existing_height, existing_index)| {
                    if *existing_height >= z {
                        panic!("shape with bottom at {z} fell too far at {pos}");
                    } else {
                        //println!("updated height at {pos} is {z}");
                        *existing_height = z;
                        *existing_index = index;
                    }
                })
                .or_insert_with(|| {
                    //println!("new height at {pos} is {z}");
                    (z, index)
                });
        }
    }
}

#[test]
fn test_surface_default_height() {
    let surface = Surface::default();
    assert_eq!(surface.get(&Position { x: 1000, y: 22 }), (0, None));
}

#[test]
fn test_surface_set_height() {
    let mut surface = Surface::default();
    let brick = Brick::try_from("2,0,5~2,2,5").expect("brick should be valid");
    // The brick would fall from z=5 to z=1.  The brick itself, once
    // fallen, has height 1, extending from z=1 to z=1.
    surface.set_height(&brick.plan(), 1, 200);
    assert_eq!(surface.get(&Position { x: 1000, y: 22 }), (0, None));
    assert_eq!(surface.get(&Position { x: 2, y: 0 }), (1, Some(200)));
    assert_eq!(surface.get(&Position { x: 2, y: 1 }), (1, Some(200)));
    assert_eq!(surface.get(&Position { x: 2, y: 2 }), (1, Some(200)));
    assert_eq!(surface.get(&Position { x: 2, y: 3 }), (0, None));
}

fn just(ix: Option<usize>) -> HashSet<usize> {
    let mut result = HashSet::new();
    if let Some(i) = ix {
        result.insert(i);
    }
    result
}

fn identify_supporting_bricks(
    acc: Option<(i64, HashSet<usize>)>,
    h: i64,
    maybe_index: Option<usize>,
) -> Option<(i64, HashSet<usize>)> {
    match acc {
        None => Some((h, just(maybe_index))),
        Some((existing_height, mut bricks)) => {
            if existing_height < h {
                Some((h, just(maybe_index)))
            } else if existing_height == h {
                if let Some(i) = maybe_index {
                    bricks.insert(i);
                }
                Some((h, bricks))
            } else {
                Some((existing_height, bricks))
            }
        }
    }
}

fn compute_fallen_brick_positions(bricks: &[Brick]) -> (Vec<Brick>, HashSet<usize>) {
    //let labels: Vec<String> = bricks
    //    .iter()
    //    .enumerate()
    //    .map(|(ix, brick)| match brick.label.as_ref() {
    //        Some(label) => label.to_string(),
    //        None => format!("{}", ix),
    //    })
    //    .collect();
    let mut indexed_bricks: Vec<(Brick, usize)> = bricks
        .iter()
        .enumerate()
        .map(|(index, brick)| (brick.clone(), index))
        .collect();
    let mut can_disintegrate: HashSet<usize> = HashSet::new();
    indexed_bricks.sort(); // by z-height
    let mut heightmap = Surface::default();
    for (brick, index) in indexed_bricks.iter_mut() {
        can_disintegrate.insert(*index);
        let brick_xy_bbox = brick.plan();
        //println!();
        //println!("brick {brick:?} is falling; its xy bounding box is {brick_xy_bbox:?}");
        if let Some((highest_ground, supporting_bricks)) =
            brick_xy_bbox.surface().fold(None, |acc, pos| {
                let (h, maybe_index) = heightmap.get(&pos);
                identify_supporting_bricks(acc, h, maybe_index)
            })
        {
            // Suppose the "ground" at this point has z=1.  Then,
            // the bottom of this brick will come to rest at z=2.
            let resting_z = highest_ground + 1;
            let fell_by = brick.lower.z - resting_z;

            // If the brick is 2 units high then the top of the
            // brick will be at z=3 (the brick occupying the
            // levels z=2 and z=3).
            brick.lower.z -= fell_by;
            brick.upper.z -= fell_by;
            //println!("the highest existing surface level within that box is {highest_ground} so it comes to rest after a fall of {fell_by} at z={}: {brick:?}", brick.lower.z);
            heightmap.set_height(&brick_xy_bbox, brick.upper.z, *index);
            //let supporting_labels: Vec<&str> = supporting_bricks
            //    .iter()
            //    .map(|ix| labels[*ix].as_str())
            //    .collect();
            match supporting_bricks.len() {
                0 => (),
                1 => {
                    //println!("brick {supporting_labels:?} cannot be disintegrated as it is the only support for {brick}");
                    for supporting_brick_index in supporting_bricks.into_iter() {
                        can_disintegrate.remove(&supporting_brick_index);
                    }
                }
                _ => {
                    //println!(
                    //    "bricks {supporting_labels:?} can be disintegrated as {brick} is multiply suported"
                    //);
                }
            }
        } else {
            panic!("brick {brick} has zero area in the xy plane");
        }
    }
    let fallen_bricks: Vec<Brick> = indexed_bricks.into_iter().map(|(brick, _)| brick).collect();
    //println!(
    //    "We can disintegrate {:?}",
    //    can_disintegrate
    //        .iter()
    //        .map(|ix| labels[*ix].as_str())
    //        .collect::<Vec<&str>>()
    //);
    (fallen_bricks, can_disintegrate)
}

#[test]
fn example_compute_fallen_brick_positions() {
    let (bricks, can_disintegrate) = compute_fallen_brick_positions(
        &parse_input(get_labeled_example()).expect("example should be valid"),
    );
    assert!(bricks.contains(&Brick {
        // A (which didn't move)
        lower: Position3 { x: 1, y: 0, z: 1 },
        upper: Position3 { x: 1, y: 2, z: 1 },
        label: Some("A".to_string()),
    }));
    assert!(bricks.contains(&Brick {
        // B (which didn't move), resting on A.
        lower: Position3 { x: 0, y: 0, z: 2 },
        upper: Position3 { x: 2, y: 0, z: 2 },
        label: Some("B".to_string()),
    }));
    assert!(bricks.contains(&Brick {
        // C (which did move), resting on A and B with z=2.
        lower: Position3 { x: 0, y: 2, z: 2 },
        upper: Position3 { x: 2, y: 2, z: 2 },
        label: Some("C".to_string()),
    }));
    assert!(bricks.contains(&Brick {
        // D fell from z=4 to z=3.
        lower: Position3 { x: 0, y: 0, z: 3 },
        upper: Position3 { x: 0, y: 2, z: 3 },
        label: Some("D".to_string()),
    }));
    assert!(bricks.contains(&Brick {
        // E fell from z=5 to z=3.
        lower: Position3 { x: 2, y: 0, z: 3 },
        upper: Position3 { x: 2, y: 2, z: 3 },
        label: Some("E".to_string()),
    }));
    assert!(bricks.contains(&Brick {
        // F fell from z=6 to z=4.
        lower: Position3 { x: 0, y: 1, z: 4 },
        upper: Position3 { x: 2, y: 1, z: 4 },
        label: Some("F".to_string()),
    }));
    dbg!(&bricks);
    assert!(bricks.contains(&Brick {
        // G fell from z=8 (top being z=9) to z=5
        lower: Position3 { x: 1, y: 1, z: 5 },
        upper: Position3 { x: 1, y: 1, z: 6 },
        label: Some("G".to_string()),
    }));

    assert_eq!(can_disintegrate.len(), 5);
}

fn part1(bricks: &[Brick]) -> usize {
    compute_fallen_brick_positions(bricks).1.len()
}

#[test]
fn test_part1() {
    let bricks = parse_input(get_labeled_example()).expect("example should be valid");
    assert_eq!(part1(&bricks), 5);
}

fn main() {
    let input = str::from_utf8(include_bytes!("input.txt")).unwrap();
    let bricks = parse_input(input).expect("puzz input should be valid");
    println!("day 22 part 1: {}", part1(&bricks));
}
