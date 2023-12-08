use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;
use std::str;

use num::integer::lcm;
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
    fn start_nodes(&self) -> Vec<&Name> {
        fn is_start_node(name: &Name) -> bool {
            name.label.ends_with('A')
        }
        self.nodes
            .keys()
            .filter(|node| is_start_node(node))
            .collect()
    }

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
    let line_re = Regex::new(r"^([A-Z0-9]{3}) = \(([A-Z0-9]{3}), ([A-Z0-9]{3})\)$").unwrap();
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

#[cfg(test)]
fn get_example_3() -> (String, Network) {
    // This is the example from part 2.
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
    parse_input(INPUT).expect("example 3 should be valid")
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

fn count_steps_to_target_part1<F>(
    instructions: &str,
    network: &Network,
    start: &str,
    is_target: F,
) -> usize
where
    F: Fn(&Name) -> bool,
{
    let mut seen: HashSet<(usize, Name)> = HashSet::new();
    let mut here = &Name::from(start);
    for (steps_taken, (cycle_pos, instruction)) in
        instructions.chars().enumerate().cycle().enumerate()
    {
        let marker: (usize, Name) = (cycle_pos, here.clone());
        if seen.contains(&marker) {
            panic!("infinite loop");
        }
        seen.insert(marker);
        here = network.step(here, instruction).expect("remain in network");
        if is_target(here) {
            return steps_taken + 1;
        }
    }
    unreachable!()
}

fn count_steps_to_target_part2(instructions: &str, network: &Network) -> usize {
    fn is_parallel_target(name: &Name) -> bool {
        name.label.ends_with('Z')
    }

    fn lcm_of_all(items: &[usize]) -> Option<usize> {
        match items {
            [initial, rest @ ..] => Some(rest.iter().fold(*initial, |acc, n| lcm(acc, *n))),
            [] => None,
        }
    }

    let cycle_lengths: Vec<usize> = network
        .start_nodes()
        .iter()
        .map(|start| {
            count_steps_to_target_part1(instructions, network, &start.label, is_parallel_target)
        })
        .collect();
    lcm_of_all(&cycle_lengths).expect("there must be at least one start node")
}

fn get_input() -> &'static str {
    str::from_utf8(include_bytes!("input.txt")).unwrap()
}

fn get_parsed_input() -> (String, Network) {
    parse_input(get_input()).expect("puzzle input should be valid")
}

fn part1(instructions: &str, network: &Network) -> usize {
    let done = |name: &Name| name.label == "ZZZ";
    count_steps_to_target_part1(instructions, network, "AAA", done)
}

#[test]
fn test_part1_example1() {
    let (instructions, network) = get_example_1();
    assert_eq!(part1(&instructions, &network), 2);
}

#[test]
fn test_part1_example2() {
    let (instructions, network) = get_example_2();
    assert_eq!(part1(&instructions, &network), 6);
}

fn part2(instructions: &str, network: &Network) -> usize {
    count_steps_to_target_part2(instructions, network)
}

#[test]
fn test_part2_example3() {
    let (instructions, network) = get_example_3();
    assert_eq!(part2(&instructions, &network), 6);
}

fn main() {
    let (instructions, network) = get_parsed_input();
    println!("day 08 part 1: {}", part1(&instructions, &network));
    println!("day 08 part 2: {}", part2(&instructions, &network));
}
