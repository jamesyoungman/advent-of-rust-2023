use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;
use std::str;

use regex::Regex;

use lib::error::Fail;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct Name {
    label: String,
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.write_str(&self.label)
    }
}

impl From<&str> for Name {
    fn from(s: &str) -> Name {
        Name {
            label: s.to_string(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct Node {
    left: Name,
    right: Name,
}

#[derive(Debug, Eq, PartialEq)]
struct Network {
    nodes: HashMap<Name, Node>,
}

impl Network {
    fn step(&self, here: &Name, step: char) -> Result<&Name, Fail> {
        let go_left = match step {
            'L' => Ok(true),
            'R' => Ok(false),
            other => Err(Fail(format!("invalid step {other}"))),
        }?;
        match self.nodes.get(here) {
            Some(Node { left, right }) => Ok({
                if go_left {
                    left
                } else {
                    right
                }
            }),
            None => Err(Fail(format!("{here} is not a known location"))),
        }
    }
}

fn parse_input(s: &str) -> Result<(String, Network), Fail> {
    let line_re = Regex::new(r"^([A-Z]{3}) = \(([A-Z]{3}), ([A-Z]{3})\)$").unwrap();
    match s.split_once("\n\n") {
        Some((instructions, mappings)) => Ok((
            instructions.to_string(),
            Network {
                nodes: mappings
                    .split_terminator('\n')
                    .map(|line| match line_re.captures(line) {
                        Some(caps) => Ok((
                            Name::from(&caps[1]),
                            Node {
                                left: Name::from(&caps[2]),
                                right: Name::from(&caps[3]),
                            },
                        )),
                        None => Err(Fail(format!("line has incorrect format: {line}"))),
                    })
                    .collect::<Result<HashMap<Name, Node>, Fail>>()?,
            },
        )),
        None => Err(Fail(format!("input did not contain a double newline: {s}"))),
    }
}

#[cfg(test)]
fn build_network(nw: &[(&str, (&str, &str))]) -> Network {
    Network {
        nodes: nw
            .into_iter()
            .map(|&(name, (l, r))| {
                (
                    Name::from(name),
                    Node {
                        left: Name::from(l),
                        right: Name::from(r),
                    },
                )
            })
            .collect(),
    }
}

#[cfg(test)]
fn get_example_1() -> (String, Network) {
    const INPUT: &str = concat!(
        "RL\n",
        "\n",
        "AAA = (BBB, CCC)\n",
        "BBB = (DDD, EEE)\n",
        "CCC = (ZZZ, GGG)\n",
        "DDD = (DDD, DDD)\n",
        "EEE = (EEE, EEE)\n",
        "GGG = (GGG, GGG)\n",
        "ZZZ = (ZZZ, ZZZ)\n",
    );
    parse_input(INPUT).expect("example 1 should be valid")
}

#[cfg(test)]
fn get_example_2() -> (String, Network) {
    const INPUT: &str = concat!(
        "LLR\n",
        "\n",
        "AAA = (BBB, BBB)\n",
        "BBB = (AAA, ZZZ)\n",
        "ZZZ = (ZZZ, ZZZ)\n",
    );
    parse_input(INPUT).expect("example 2 should be valid")
}

#[test]
fn test_parser() {
    let expected = [
        ("AAA", ("BBB", "CCC")),
        ("BBB", ("DDD", "EEE")),
        ("CCC", ("ZZZ", "GGG")),
        ("DDD", ("DDD", "DDD")),
        ("EEE", ("EEE", "EEE")),
        ("GGG", ("GGG", "GGG")),
        ("ZZZ", ("ZZZ", "ZZZ")),
    ];
    let expected_network = build_network(&expected);
    assert_eq!(get_example_1(), ("RL".to_string(), expected_network,));
}

#[test]
fn test_stepping() {
    let (_instructions, network) = get_example_1();
    assert_eq!(
        network.step(&Name::from("AAA"), 'L'),
        Ok(&Name::from("BBB"))
    );
    assert_eq!(
        network.step(&Name::from("AAA"), 'R'),
        Ok(&Name::from("CCC"))
    );
    assert_eq!(
        network.step(&Name::from("BBB"), 'L'),
        Ok(&Name::from("DDD"))
    );
}

fn count_steps_to_target(
    instructions: &str,
    network: &Network,
    start: &str,
    target: &str,
) -> usize {
    let mut seen: HashSet<(usize, Name)> = HashSet::new();
    let mut here = &Name::from(start);
    let target = &Name::from(target);
    for (steps_taken, (cycle_pos, instruction)) in
        instructions.chars().enumerate().cycle().enumerate()
    {
        let marker: (usize, Name) = (cycle_pos, here.clone());
        if seen.contains(&marker) {
            panic!("infinite loop");
        }
        seen.insert(marker);
        here = network.step(here, instruction).expect("remain in network");
        if here == target {
            return steps_taken + 1;
        }
    }
    unreachable!()
}

#[test]
fn test_count_steps_to_target_example1() {
    let (instructions, network) = get_example_1();
    assert_eq!(
        count_steps_to_target(&instructions, &network, "AAA", "ZZZ"),
        2
    );
}

#[test]
fn test_count_steps_to_target_example2() {
    let (instructions, network) = get_example_2();
    assert_eq!(
        count_steps_to_target(&instructions, &network, "AAA", "ZZZ"),
        6
    );
}

fn get_input() -> &'static str {
    str::from_utf8(include_bytes!("input.txt")).unwrap()
}

fn get_parsed_input() -> (String, Network) {
    parse_input(get_input()).expect("puzzle input should be valid")
}

fn part1(instructions: &str, network: &Network) -> usize {
    count_steps_to_target(instructions, network, "AAA", "ZZZ")
}

fn main() {
    let (instructions, network) = get_parsed_input();
    println!("day 08 part 1: {}", part1(&instructions, &network));
}
