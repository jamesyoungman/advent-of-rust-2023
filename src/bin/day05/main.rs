use std::collections::HashMap;
use std::str;

use regex::Regex;

use lib::error::{fail_from_error, Fail};

type Id = u32;

#[derive(Debug)]
struct MappingRange {
    dest_start: Id,
    source_start: Id,
    len: Id,
}

impl MappingRange {
    fn get(&self, id: Id) -> Option<Id> {
        if id < self.source_start {
            None
        } else {
            let offset = id - self.source_start;
            if offset >= self.len {
                None
            } else {
                Some(self.dest_start + offset)
            }
        }
    }
}

#[test]
fn test_mapping_range_lookup() {
    let example = MappingRange {
        dest_start: 50,
        source_start: 98,
        len: 2,
    };
    assert_eq!(example.get(97), None);
    assert_eq!(example.get(98), Some(50));
    assert_eq!(example.get(99), Some(51));
    assert_eq!(example.get(100), None);
}

impl TryFrom<&str> for MappingRange {
    type Error = Fail;

    fn try_from(s: &str) -> Result<MappingRange, Self::Error> {
        let fields: Vec<Id> = s
            .split_whitespace()
            .map(|s| s.parse().map_err(|e| fail_from_error(&e)))
            .collect::<Result<Vec<Id>, Self::Error>>()?;
        match fields.as_slice() {
            [dest_start, source_start, len] => Ok(MappingRange {
                dest_start: *dest_start,
                source_start: *source_start,
                len: *len,
            }),
            _ => Err(Fail(format!("expected 3 fields, got {s:?}"))),
        }
    }
}

#[derive(Debug)]
struct Mapping {
    entries: Vec<MappingRange>,
}

impl Mapping {
    fn get(&self, id: Id) -> Id {
        for maprange in self.entries.iter() {
            if let Some(result) = maprange.get(id) {
                return result;
            }
        }
        id
    }
}

impl TryFrom<&str> for Mapping {
    type Error = Fail;

    fn try_from(s: &str) -> Result<Mapping, Self::Error> {
        let entries: Vec<MappingRange> = s
            .split_terminator('\n')
            .map(MappingRange::try_from)
            .collect::<Result<Vec<MappingRange>, Fail>>()?;
        Ok(Mapping { entries })
    }
}

#[test]
fn test_mapping_lookup() {
    let mapping =
        Mapping::try_from(concat!("50 98 2\n", "52 50 48\n")).expect("example should be valid");
    assert_eq!(mapping.get(0), 0);
    assert_eq!(mapping.get(1), 1);
    assert_eq!(mapping.get(48), 48);
    assert_eq!(mapping.get(49), 49);
    assert_eq!(mapping.get(50), 52);
    assert_eq!(mapping.get(51), 53);
    assert_eq!(mapping.get(96), 98);
    assert_eq!(mapping.get(97), 99);
    assert_eq!(mapping.get(98), 50);
    assert_eq!(mapping.get(99), 51);
}

#[derive(Debug)]
struct Almanac {
    seeds: Vec<Id>,
    mappings: HashMap<String, Mapping>,
}

impl TryFrom<&str> for Almanac {
    type Error = Fail;

    fn try_from(s: &str) -> Result<Almanac, Self::Error> {
        let map_re = Regex::new("^(.*) map:\n(?s)(.*)$").unwrap();
        let seeds_re = Regex::new("^seeds: (.*)$").unwrap();
        let chunks = s.split("\n\n");
        let mut seeds: Vec<Id> = Vec::new();
        let mut mappings: HashMap<String, Mapping> = HashMap::new();
        for chunk in chunks {
            match seeds_re.captures(chunk) {
                Some(caps) => {
                    seeds = caps[1]
                        .split_whitespace()
                        .map(|s| s.parse())
                        .collect::<Result<Vec<Id>, _>>()
                        .map_err(|e| fail_from_error(&e))?;
                }
                None => match map_re.captures(chunk) {
                    Some(caps) => {
                        let name = caps[1].to_string();
                        let mapping = Mapping::try_from(&caps[2])?;
                        mappings.insert(name, mapping);
                    }
                    None => {
                        return Err(Fail(format!(
                            "unable to parse a chunk (it's not a seeds entry or a mapping: {chunk}"
                        )));
                    }
                },
            }
        }
        Ok(Almanac { seeds, mappings })
    }
}

impl Almanac {
    fn map(&self, map_name: &str, id: Id) -> Id {
        match self.mappings.get(map_name) {
            Some(mapping) => mapping.get(id),
            None => {
                panic!("mapping {map_name} does not contain an entry for {id}");
            }
        }
    }

    fn get_location_number_for_seed(&self, seed: Id) -> Id {
        self.map(
            "humidity-to-location",
            self.map(
                "temperature-to-humidity",
                self.map(
                    "light-to-temperature",
                    self.map(
                        "water-to-light",
                        self.map(
                            "fertilizer-to-water",
                            self.map("soil-to-fertilizer", self.map("seed-to-soil", seed)),
                        ),
                    ),
                ),
            ),
        )
    }

    fn get_lowest_location(&self) -> Option<Id> {
        self.seeds
            .iter()
            .map(|seed| self.get_location_number_for_seed(*seed))
            .min()
    }
}

#[cfg(test)]
fn get_example() -> &'static str {
    concat!(
        "seeds: 79 14 55 13\n",
        "\n",
        "seed-to-soil map:\n",
        "50 98 2\n",
        "52 50 48\n",
        "\n",
        "soil-to-fertilizer map:\n",
        "0 15 37\n",
        "37 52 2\n",
        "39 0 15\n",
        "\n",
        "fertilizer-to-water map:\n",
        "49 53 8\n",
        "0 11 42\n",
        "42 0 7\n",
        "57 7 4\n",
        "\n",
        "water-to-light map:\n",
        "88 18 7\n",
        "18 25 70\n",
        "\n",
        "light-to-temperature map:\n",
        "45 77 23\n",
        "81 45 19\n",
        "68 64 13\n",
        "\n",
        "temperature-to-humidity map:\n",
        "0 69 1\n",
        "1 0 69\n",
        "\n",
        "humidity-to-location map:\n",
        "60 56 37\n",
        "56 93 4\n",
    )
}

#[test]
fn test_parse_example() {
    let almanac = Almanac::try_from(get_example()).expect("example should be valid");
    assert_eq!(almanac.seeds.len(), 4);
    assert_eq!(almanac.mappings.len(), 7);
    for mapping_name in [
        "seed-to-soil",
        "soil-to-fertilizer",
        "fertilizer-to-water",
        "water-to-light",
        "light-to-temperature",
        "temperature-to-humidity",
        "humidity-to-location",
    ] {
        if !almanac.mappings.contains_key(mapping_name) {
            dbg!(almanac.mappings.keys());
            panic!("Almanac lacks mapping {mapping_name}");
        }
    }
}

#[test]
fn test_example_mappings() {
    let almanac = Almanac::try_from(get_example()).expect("example should be valid");
    assert_eq!(almanac.get_location_number_for_seed(79), 82);
    assert_eq!(almanac.get_location_number_for_seed(14), 43);
    assert_eq!(almanac.get_location_number_for_seed(55), 86);
    assert_eq!(almanac.get_location_number_for_seed(13), 35);
}

#[test]
fn test_get_lowest_location() {
    let almanac = Almanac::try_from(get_example()).expect("example should be valid");
    assert_eq!(almanac.get_lowest_location(), Some(35));
}

/// Reads the puzzle input.
fn get_input() -> String {
    let input = str::from_utf8(include_bytes!("input.txt")).unwrap();
    input.to_string()
}

fn main() {
    let input = get_input();
    let almanac = Almanac::try_from(input.as_str()).expect("input should be valid");
    match almanac.get_lowest_location() {
        Some(loc) => {
            println!("day 07 part 1: {loc}");
        }
        None => {
            eprintln!("day 07 part 1: almanac has no seeds!");
        }
    }
}
