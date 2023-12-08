use std::collections::HashMap;
use std::str;

use num::integer::lcm;
use regex::Regex;

use lib::error::Fail;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct Node {
    left: String,
    right: String,
}

#[derive(Debug, Eq, PartialEq)]
struct Network {
    nodes: HashMap<String, Node>,
}

impl Network {
    fn step(&self, here: &String, step: char) -> Result<&String, Fail> {
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
    let line_re = Regex::new(r"^([A-Z0-9]{3}) = \(([A-Z0-9]{3}), ([A-Z0-9]{3})\)$").unwrap();
    match s.split_once("\n\n") {
        Some((instructions, mappings)) => Ok((
            instructions.to_string(),
            Network {
                nodes: mappings
                    .split_terminator('\n')
                    .map(|line| match line_re.captures(line) {
                        Some(caps) => Ok((
                            String::from(&caps[1]),
                            Node {
                                left: String::from(&caps[2]),
                                right: String::from(&caps[3]),
                            },
                        )),
                        None => Err(Fail(format!("line has incorrect format: {line}"))),
                    })
                    .collect::<Result<HashMap<String, Node>, Fail>>()?,
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
                    String::from(name),
                    Node {
                        left: String::from(l),
                        right: String::from(r),
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

fn count_steps<F>(instructions: &str, network: &Network, start: &str, is_target: F) -> usize
where
    F: Fn(&str) -> bool,
{
    let mut here = &String::from(start);
    for (steps_taken, instruction) in instructions.chars().cycle().enumerate() {
        here = network.step(here, instruction).expect("remain in network");
        if is_target(here) {
            return steps_taken + 1;
        }
    }
    unreachable!()
}

fn part1(instructions: &str, network: &Network) -> usize {
    let done = |name: &str| name == "ZZZ";
    count_steps(instructions, network, "AAA", done)
}

#[test]
fn test_part1_example1() {
    let (instructions, network) = get_example_1();
    assert_eq!(part1(&instructions, &network), 2);
}

#[test]
fn test_part1_example2() {
    let (instructions, network) = parse_input(concat!(
        "LLR\n",
        "\n",
        "AAA = (BBB, BBB)\n",
        "BBB = (AAA, ZZZ)\n",
        "ZZZ = (ZZZ, ZZZ)\n",
    ))
    .expect("example should be valid");
    assert_eq!(part1(&instructions, &network), 6);
}

fn part2(instructions: &str, network: &Network) -> usize {
    fn is_target(name: &str) -> bool {
        name.ends_with('Z')
    }

    network
        .nodes
        .keys()
        // Identify start nodes.
        .filter(|node| node.ends_with('A'))
        // Measure the length of the cycle starting at each start node.
        .map(|start| count_steps(instructions, network, start, is_target))
        // Find the lowest common multiple of all the cycle lengths.
        .fold(None, |acc, n| match acc {
            None => Some(n),
            Some(acc) => Some(lcm(acc, n)),
        })
        .expect("there must be at least one start node")
}

#[test]
fn test_part2_example3() {
    const INPUT: &str = concat!(
        "LR\n",
        "\n",
        "11A = (11B, XXX)\n",
        "11B = (XXX, 11Z)\n",
        "11Z = (11B, XXX)\n",
        "22A = (22B, XXX)\n",
        "22B = (22C, 22C)\n",
        "22C = (22Z, 22Z)\n",
        "22Z = (22B, 22B)\n",
        "XXX = (XXX, XXX)\n",
    );
    let (instructions, network) = parse_input(INPUT).expect("example input should be valid");
    assert_eq!(part2(&instructions, &network), 6);
}

fn main() {
    let input = str::from_utf8(include_bytes!("input.txt")).unwrap();
    let (instructions, network) = parse_input(input).expect("puzzle input should be valid");
    println!("day 08 part 1: {}", part1(&instructions, &network));
    println!("day 08 part 2: {}", part2(&instructions, &network));
}
