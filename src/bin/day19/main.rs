use std::collections::HashMap;
use std::str;

use lazy_static::lazy_static;
use regex::Regex;

use lib::error::Fail;

#[cfg(test)]
fn get_example() -> &'static str {
    concat!(
        "px{a<2006:qkq,m>2090:A,rfg}\n",
        "pv{a>1716:R,A}\n",
        "lnx{m>1548:A,A}\n",
        "rfg{s<537:gd,x>2440:R,A}\n",
        "qs{s>3448:A,lnx}\n",
        "qkq{x<1416:A,crn}\n",
        "crn{x>2662:A,R}\n",
        "in{s<1351:px,qqz}\n",
        "qqz{s>2770:qs,m<1801:hdj,R}\n",
        "gd{a>3333:R,R}\n",
        "hdj{m>838:A,pv}\n",
        "\n",
        "{x=787,m=2655,a=1222,s=2876}\n",
        "{x=1679,m=44,a=2067,s=496}\n",
        "{x=2036,m=264,a=79,s=2244}\n",
        "{x=2461,m=1339,a=466,s=291}\n",
        "{x=2127,m=1623,a=2188,s=1013}\n",
    )
}

#[derive(Debug, PartialEq, Eq)]
struct Item {
    attributes: HashMap<String, i64>,
}

impl Item {
    fn total_rating(&self) -> i64 {
        self.attributes.values().sum()
    }
}

fn parse_integer(s: &str) -> Result<i64, Fail> {
    match s.parse() {
        Err(e) => Err(Fail(format!("{s} is not a valid integer: {e}"))),
        Ok(n) => Ok(n),
    }
}

fn parse_item(s: &str) -> Result<Item, Fail> {
    lazy_static! {
        static ref RE: Regex = Regex::new("([a-zA-Z0-9]+)=([0-9]+)").unwrap();
    }
    Ok(Item {
        attributes: RE
            .captures_iter(s)
            .map(|c| {
                let name = c.get(1).unwrap().as_str().to_string();
                let value = parse_integer(c.get(2).unwrap().as_str())?;
                Ok((name, value))
            })
            .collect::<Result<HashMap<String, i64>, Fail>>()?,
    })
}

#[test]
fn test_parse_item() {
    let item = parse_item("{x=2461,m=1339,a=466,s=291}").expect("test input is valid");
    assert_eq!(item.attributes.get("m"), Some(&1339_i64));
}

#[derive(Debug, PartialEq, Eq)]
enum Next {
    Stop(bool),
    Goto(String),
}

#[derive(Debug, PartialEq, Eq)]
enum Check {
    Condition {
        attribute: String,
        comparison: char,
        boundary: i64,
        next_if_met: Next,
    },
    Always(Next),
}

fn parse_check(s: &str) -> Result<Check, Fail> {
    fn parse_next(s: &str) -> Next {
        match s {
            "A" => Next::Stop(true),
            "R" => Next::Stop(false),
            target => Next::Goto(target.to_string()),
        }
    }

    lazy_static! {
        static ref RE: Regex = Regex::new("^([a-zA-Z0-9]+)([<>])([0-9]+):([a-zA-Z]+)$").unwrap();
    }
    let result: Result<Check, Fail> = match RE.captures(s) {
        Some(caps) => {
            let attribute = caps.get(1).unwrap().as_str().to_string();
            let comparison = match caps.get(2).unwrap().as_str().chars().next() {
                Some(ch) => ch,
                None => {
                    return Err(Fail("comparison should not be an empty string".to_string()));
                }
            };
            let boundary = match caps.get(3) {
                Some(m) => parse_integer(m.as_str())?,
                None => {
                    return Err(Fail("missing boundary".to_string()));
                }
            };
            let next_if_met = match caps.get(4) {
                Some(m) => parse_next(m.as_str()),
                None => {
                    return Err(Fail("missing next step".to_string()));
                }
            };
            Ok(Check::Condition {
                attribute,
                comparison,
                boundary,
                next_if_met,
            })
        }
        None => Ok(Check::Always(parse_next(s))),
    };
    match result {
        Ok(r) => Ok(r),
        Err(e) => Err(Fail(format!("{s} is not a valid check: {e}"))),
    }
}

#[test]
fn test_parse_check() {
    let check = parse_check("a<2006:qkq").expect("test input should be valid");
    match check {
        Check::Condition {
            attribute,
            comparison,
            boundary,
            next_if_met,
        } => {
            assert_eq!(attribute.as_str(), "a");
            assert_eq!(comparison, '<');
            assert_eq!(boundary, 2006);
            assert_eq!(next_if_met, Next::Goto("qkq".to_string()));
        }
        _ => {
            panic!("expected conditinal check");
        }
    }
}

impl Check {
    fn next_step_for_item(&self, item: &Item) -> Option<&Next> {
        match self {
            Check::Always(decision) => Some(decision),
            Check::Condition {
                attribute,
                comparison,
                boundary,
                next_if_met,
            } => match item.attributes.get(attribute) {
                Some(value) => {
                    if match comparison {
                        '>' => value > boundary,
                        '<' => value < boundary,
                        _ => {
                            panic!("don't know how to perform comparison {comparison}");
                        }
                    } {
                        Some(next_if_met)
                    } else {
                        None
                    }
                }
                None => {
                    panic!("item lacks attribute {attribute}");
                }
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Rule {
    checks: Vec<Check>,
    default_next: Next,
}

fn parse_rule(s: &str) -> Result<(String, Rule), Fail> {
    lazy_static! {
        static ref RE: Regex = Regex::new(concat!(
            "^",
        "([^{]+)",		// rule name
            "[{]",		// opening delimiter for checks
            "([^}]+)",		// the checks
            "[}]",		// closing delimiter for checks
        "$",
        )).expect("regex should be valid");
    }
    let (name, mut checks) = match RE.captures(s) {
        Some(caps) => {
            let name = caps.get(1).expect("name").as_str().to_string();
            let checks = caps.get(2).expect("checks").as_str();
            let checks = checks
                .split(',')
                .map(parse_check)
                .collect::<Result<Vec<Check>, Fail>>()?;
            (name, checks)
        }
        None => {
            return Err(Fail("expected to see a rule name and checks".to_string()));
        }
    };
    match checks.pop() {
        Some(Check::Always(default_next)) => Ok((
            name,
            Rule {
                checks,
                default_next,
            },
        )),
        Some(Check::Condition { .. }) => {
            Err(Fail("final check should not be conditional".to_string()))
        }
        None => Err(Fail(
            "there should be at least one check in every rule".to_string(),
        )),
    }
}

impl Rule {
    fn examine(&self, item: &Item) -> &Next {
        self.checks
            .iter()
            .find_map(|check| check.next_step_for_item(item))
            .unwrap_or(&self.default_next)
    }
}

#[test]
fn test_parse_rule() {
    let (name, rule) = parse_rule("qqz{s>2770:qs,m<1801:hdj,R}").expect("test input is valid");
    assert_eq!(name.as_str(), "qqz");
    assert_eq!(rule.checks.len(), 2);
    assert_eq!(
        rule.checks[0],
        Check::Condition {
            attribute: "s".to_string(),
            comparison: '>',
            boundary: 2770,
            next_if_met: Next::Goto("qs".to_string()),
        }
    );
    assert_eq!(
        rule.checks[1],
        Check::Condition {
            attribute: "m".to_string(),
            comparison: '<',
            boundary: 1801,
            next_if_met: Next::Goto("hdj".to_string()),
        }
    );
    assert_eq!(rule.default_next, Next::Stop(false));
}

fn parse_input(s: &str) -> Result<(HashMap<String, Rule>, Vec<Item>), Fail> {
    match s.split_once("\n\n") {
        Some((first, second)) => Ok((
            first
                .split_terminator('\n')
                .map(parse_rule)
                .collect::<Result<HashMap<String, Rule>, Fail>>()?,
            second
                .split_terminator('\n')
                .map(parse_item)
                .collect::<Result<Vec<Item>, Fail>>()?,
        )),
        None => Err(Fail(
            "expected blank line between the rules and the items".to_string(),
        )),
    }
}

#[test]
fn test_parse_input() {
    let example = get_example();
    let (rules, items) = parse_input(example).expect("input is valid");
    assert_eq!(rules.len(), 11);
    assert_eq!(
        rules["pv"],
        Rule {
            checks: vec![Check::Condition {
                attribute: "a".to_string(),
                comparison: '>',
                boundary: 1716,
                next_if_met: Next::Stop(false),
            },],
            default_next: Next::Stop(true),
        }
    );
    assert_eq!(items.len(), 5);
}

fn accept(item: &Item, rules: &HashMap<String, Rule>) -> bool {
    let mut rule_name = "in";
    while let Some(next) = rules.get(rule_name).map(|rule| rule.examine(item)) {
        rule_name = match next {
            Next::Stop(decision) => {
                return *decision;
            }
            Next::Goto(name) => name.as_str(),
        };
    }
    panic!("cannot find rule {rule_name}");
}

fn part1(rules: &HashMap<String, Rule>, items: &[Item]) -> i64 {
    items
        .iter()
        .filter(|item| accept(item, rules))
        .map(Item::total_rating)
        .sum()
}

#[test]
fn test_part1() {
    let (rules, items) = parse_input(get_example()).expect("example input is valid");
    assert_eq!(part1(&rules, &items), 19114);
}

/// Reads the puzzle input.
fn get_input() -> &'static str {
    str::from_utf8(include_bytes!("input.txt")).unwrap()
}

fn main() {
    let (rules, items) = parse_input(get_input()).expect("puzzle input is valid");
    println!("day 19 part 1: {}", part1(&rules, &items));
}
