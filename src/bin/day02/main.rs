use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::str;

#[derive(Debug)]
struct Fail(String);

impl Display for Fail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed: {}", self.0)
    }
}

impl Error for Fail {}

#[cfg(test)]
fn part1_example() -> Vec<Game> {
    parse_input(concat!(
        "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green\n",
        "Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue\n",
        "Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red\n",
        "Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red\n",
        "Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green\n"
    ))
    .expect("example should be valid")
}

#[derive(Debug, Default)]
struct Stock {
    pub counts: HashMap<String, u32>,
}

impl Stock {
    fn power(&self) -> u32 {
        self.counts.values().product()
    }

    fn get(&self, colour: &str) -> &u32 {
        self.counts.get(colour).unwrap_or(&0)
    }

    fn update_requirement(&mut self, colour: &str, count: u32) {
        self.counts
            .entry(colour.to_string())
            .and_modify(|needed| {
                if *needed < count {
                    *needed = count
                }
            })
            .or_insert(count);
    }

    fn suffices_for(&self, required: &Stock) -> bool {
        required
            .counts
            .iter()
            .all(|(colour, needed)| self.get(colour) >= needed)
    }
}

#[derive(Debug)]
struct Turn {
    pub counts: HashMap<String, u32>,
}

impl Turn {
    fn update_requirement(&self, req: &mut Stock) {
        self.counts.iter().for_each(|(colour, count)| {
            req.update_requirement(colour, *count);
        });
    }
}

fn str_to_num(s: &str) -> Result<u32, Fail> {
    match s.parse() {
        Ok(n) => Ok(n),
        Err(e) => Err(Fail(format!("{s} is not a valid number: {e}"))),
    }
}

impl TryFrom<&str> for Turn {
    type Error = Fail;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(Turn {
            counts: s
                .split(", ")
                .map(|pair| match pair.split_once(' ') {
                    Some((ns, colour)) => match str_to_num(ns) {
                        Ok(n) => Ok((colour.to_string(), n)),
                        Err(e) => Err(e),
                    },
                    None => Err(Fail(format!("invalid pair: {pair}"))),
                })
                .collect::<Result<HashMap<String, u32>, Fail>>()?,
        })
    }
}

#[derive(Debug)]
struct Game {
    pub id: u32,
    pub turns: Vec<Turn>,
}

impl Game {
    fn min_requirement(&self) -> Stock {
        self.turns.iter().fold(Stock::default(), |mut acc, turn| {
            turn.update_requirement(&mut acc);
            acc
        })
    }
}

impl TryFrom<&str> for Game {
    type Error = Fail;

    fn try_from(line: &str) -> Result<Self, Self::Error> {
        match line.split_once(": ") {
            Some((prefix, counts_str)) => match prefix.strip_prefix("Game ") {
                None => Err(Fail(format!("prefix should start with 'Game ': {prefix}"))),
                Some(id_str) => Ok(Game {
                    id: str_to_num(id_str)?,
                    turns: counts_str
                        .split("; ")
                        .map(|turn| Turn::try_from(turn))
                        .collect::<Result<Vec<Turn>, Fail>>()?,
                }),
            },
            None => Err(Fail(format!("invalid line contains no id: {line}"))),
        }
    }
}

fn parse_input(input: &str) -> Result<Vec<Game>, Fail> {
    input.lines().map(Game::try_from).collect()
}

fn part1(games: &[Game], stock: &Stock) -> u32 {
    games
        .iter()
        .filter_map(|game| {
            if stock.suffices_for(&game.min_requirement()) {
                Some(game.id)
            } else {
                None
            }
        })
        .sum()
}

#[test]
fn test_part1() {
    let stock = Stock {
        counts: [
            ("red".to_string(), 12),
            ("green".to_string(), 13),
            ("blue".to_string(), 14),
        ]
        .into_iter()
        .collect(),
    };
    let got = part1(&part1_example(), &stock);
    assert_eq!(got, 8);
}

fn part2(games: &[Game]) -> u32 {
    games
        .iter()
        .map(|game| game.min_requirement().power())
        .sum()
}

#[test]
fn test_part2() {
    assert_eq!(part2(&part1_example()), 2286);
}

fn main() {
    let input = str::from_utf8(include_bytes!("input.txt")).unwrap();
    let part1_stock = Stock {
        counts: [
            ("red".to_string(), 12),
            ("green".to_string(), 13),
            ("blue".to_string(), 14),
        ]
        .into_iter()
        .collect(),
    };
    let games = parse_input(input).expect("input should be valid");

    println!("day 02 part 1: {}", part1(&games, &part1_stock));
    println!("day 02 part 2: {}", part2(&games));
}
