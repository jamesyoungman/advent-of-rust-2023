use std::cmp::{max, min};
use std::fmt::Display;
use std::str;

use lib::error::Fail;
use lib::grid::{BoundingBox, Position};

#[derive(Debug, PartialEq, Eq, Hash)]
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

impl TryFrom<&str> for Position3 {
    type Error = Fail;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if let [x, y, z] = s
            .split(',')
            .map(|s| s.parse::<i64>().map_err(|e| Fail(e.to_string())))
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

#[derive(Debug, PartialEq, Eq, Hash)]
struct Brick {
    lower: Position3,
    upper: Position3,
}

impl Display for Brick {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}~{}", self.lower, self.upper)
    }
}

impl TryFrom<&str> for Brick {
    type Error = Fail;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if let Some((left, right)) = s.split_once('~') {
            let left = Position3::try_from(left)?;
            let right = Position3::try_from(right)?;
            if left.z <= right.z {
                Ok(Brick {
                    lower: left,
                    upper: right,
                })
            } else {
                Ok(Brick {
                    lower: right,
                    upper: left,
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
fn get_example() -> &'static str {
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

fn parse_input(s: &str) -> Result<Vec<Brick>, Fail> {
    s.split_terminator('\n')
        .map(Brick::try_from)
        .collect::<Result<Vec<Brick>, Fail>>()
}

#[test]
fn test_parse_example() {
    let bricks = parse_input(get_example()).expect("example should be valid");
    assert_eq!(bricks.len(), 7);
    assert_eq!(
        &bricks[0],
        &Brick {
            lower: Position3 { x: 1, y: 0, z: 1 },
            upper: Position3 { x: 1, y: 2, z: 1 }
        }
    );
}

fn main() {}
