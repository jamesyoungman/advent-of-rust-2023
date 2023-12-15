use std::fmt::{Display, Formatter, Write};
use std::str;

use lib::days::day15;
use lib::error::Fail;

#[inline]
fn hash(s: &str) -> u8 {
    day15::hash_generic::<u32>(s)
}

fn part1(s: &str) -> u64 {
    s.split(',').map(|s| u64::from(hash(s))).sum()
}

#[test]
fn test_part1() {
    assert_eq!(
        part1("rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7"),
        1320
    );
}

#[derive(Debug, Default, PartialEq, Eq)]
struct LabeledLens {
    label: String,
    focal_length: u8,
}

impl Display for LabeledLens {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "[{0} {1}]", self.label, self.focal_length)
    }
}

impl LabeledLens {
    fn power(&self, pos: usize) -> u64 {
        (pos as u64) * u64::from(self.focal_length)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Instruction {
    Remove(String),
    Insert(String, u8),
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Instruction::Remove(label) => write!(f, "{label}-"),
            Instruction::Insert(label, focal_length) => write!(f, "{label}={focal_length}"),
        }
    }
}

impl Instruction {
    fn label(&self) -> &str {
        match self {
            Instruction::Remove(label) | Instruction::Insert(label, _) => label,
        }
    }

    fn target(&self) -> usize {
        usize::from(hash(self.label()))
    }
}

impl TryFrom<&str> for Instruction {
    type Error = Fail;
    fn try_from(instruction: &str) -> Result<Instruction, Self::Error> {
        match instruction.split_once('=') {
            Some((label, fl)) => match fl.parse() {
                Ok(focal_length) => Ok(Instruction::Insert(label.to_string(), focal_length)),
                Err(e) => Err(Fail(format!("{fl} is not a valid focal length: {e}"))),
            },
            None => match instruction.strip_suffix('-') {
                Some(label) => Ok(Instruction::Remove(label.to_string())),
                None => Err(Fail(format!(
                    "don't know how to interpret instruction {instruction}"
                ))),
            },
        }
    }
}

#[test]
fn test_instruction_from_str() {
    assert_eq!(
        Instruction::try_from("rn=1"),
        Ok(Instruction::Insert("rn".to_string(), 1))
    );
    assert_eq!(
        Instruction::try_from("cm=2"),
        Ok(Instruction::Insert("cm".to_string(), 2))
    );
    assert_eq!(
        Instruction::try_from("qp-"),
        Ok(Instruction::Remove("qp".to_string()))
    );
}

#[test]
fn test_instruction_target() {
    assert_eq!(Instruction::try_from("rn=1").map(|ins| ins.target()), Ok(0));
    assert_eq!(Instruction::try_from("cm-").map(|ins| ins.target()), Ok(0));
    assert_eq!(Instruction::try_from("pc=4").map(|ins| ins.target()), Ok(3));
}

#[derive(Debug, Default, PartialEq, Eq)]
struct LensBox {
    // In the context of the instructions, "behind" means "at a
    // greater index".
    lenses: Vec<LabeledLens>,
}

impl Display for LensBox {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut first = true;
        for lens in self.lenses.iter() {
            if first {
                first = false;
            } else {
                f.write_char(' ')?;
            }
            write!(f, "{lens}")?;
        }
        Ok(())
    }
}

impl LensBox {
    fn is_empty(&self) -> bool {
        self.lenses.is_empty()
    }

    fn remove(&mut self, label_to_remove: &str) {
        self.lenses.retain(|lens| lens.label != label_to_remove);
    }

    fn insert(&mut self, label_to_add: &str, focal_length: u8) {
        if let Some(existing_lens) = self
            .lenses
            .iter_mut()
            .find(|lens| lens.label == label_to_add)
        {
            existing_lens.focal_length = focal_length;
        } else {
            self.lenses.push(LabeledLens {
                label: label_to_add.to_string(),
                focal_length,
            });
        }
    }
    fn perform(&mut self, instruction: &Instruction) {
        match instruction {
            Instruction::Remove(label) => {
                self.remove(label);
            }
            Instruction::Insert(label, focal_length) => {
                self.insert(label, *focal_length);
            }
        }
    }

    fn power(&self, boxnum: usize) -> u64 {
        self.lenses
            .iter()
            .enumerate()
            .map(|(i, lens)| (boxnum as u64) * lens.power(i + 1))
            .sum()
    }
}

#[test]
fn test_lens_box_perform() {
    let mut b = LensBox::default();
    b.perform(&Instruction::Insert("rn".to_string(), 1));
    assert_eq!(
        b,
        LensBox {
            lenses: vec![LabeledLens {
                label: "rn".to_string(),
                focal_length: 1,
            }]
        }
    );
    b.perform(&Instruction::Insert("cm".to_string(), 2));
    assert_eq!(
        b,
        LensBox {
            lenses: vec![
                LabeledLens {
                    label: "rn".to_string(),
                    focal_length: 1,
                },
                LabeledLens {
                    label: "cm".to_string(),
                    focal_length: 2,
                }
            ]
        }
    );
    b.perform(&Instruction::Remove("cm".to_string()));
    assert_eq!(
        b,
        LensBox {
            lenses: vec![LabeledLens {
                label: "rn".to_string(),
                focal_length: 1,
            },]
        }
    );
}

#[derive(Debug)]
struct LensArray {
    lens_boxes: Vec<LensBox>,
}

impl Default for LensArray {
    fn default() -> LensArray {
        const LEN: usize = 256;
        let mut lens_boxes = Vec::with_capacity(LEN);
        lens_boxes.resize_with(LEN, LensBox::default);
        LensArray { lens_boxes }
    }
}

impl LensArray {
    fn perform(&mut self, instruction: &Instruction, verbose: bool) {
        let box_index = instruction.target();
        if let Some(target) = self.lens_boxes.get_mut(box_index) {
            target.perform(instruction);
            if verbose {
                eprintln!("After \"{instruction}\":\n{self}");
            }
        } else {
            panic!("we don't have a box {box_index}");
        }
    }

    fn perform_sequence(&mut self, instructions: &[Instruction], verbose: bool) {
        for instruction in instructions.iter() {
            if verbose {
                eprintln!("applying instruction {instruction}");
            }
            self.perform(instruction, verbose);
        }
    }

    fn power(&self) -> u64 {
        self.lens_boxes
            .iter()
            .enumerate()
            .map(|(i, lens_box)| lens_box.power(i + 1))
            .sum()
    }
}

impl Display for LensArray {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        for (index, lens_box) in self.lens_boxes.iter().enumerate() {
            if !lens_box.is_empty() {
                writeln!(f, "Box {index}: {lens_box}")?;
            }
        }
        Ok(())
    }
}

#[test]
fn test_lens_array_power() {
    let mut array = LensArray::default();
    array.lens_boxes[0].lenses = vec![
        LabeledLens {
            label: "rn".to_string(),
            focal_length: 1,
        },
        LabeledLens {
            label: "cm".to_string(),
            focal_length: 2,
        },
    ];
    array.lens_boxes[3].lenses = vec![
        LabeledLens {
            label: "ot".to_string(),
            focal_length: 7,
        },
        LabeledLens {
            label: "ab".to_string(),
            focal_length: 5,
        },
        LabeledLens {
            label: "pc".to_string(),
            focal_length: 6,
        },
    ];
    assert_eq!(array.lens_boxes[0].power(1), 1 + 4);
    assert_eq!(array.lens_boxes[3].power(4), 28 + 40 + 72);
}

fn parse_instructions(s: &str) -> Result<Vec<Instruction>, Fail> {
    s.split(',').map(Instruction::try_from).collect()
}

#[test]
fn test_lens_array_perform_sequence() {
    const EXAMPLE: &str = "rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7";
    let instructions: Vec<Instruction> =
        parse_instructions(EXAMPLE).expect("example instructions should be valid");
    let mut array = LensArray::default();
    array.perform_sequence(&instructions, true);
    //dbg!(&array);
    assert_eq!(
        array.lens_boxes[0],
        LensBox {
            lenses: vec![
                LabeledLens {
                    label: "rn".to_string(),
                    focal_length: 1,
                },
                LabeledLens {
                    label: "cm".to_string(),
                    focal_length: 2,
                },
            ]
        }
    );
}

fn part2(s: &'static str, verbose: bool) -> u64 {
    let instructions = parse_instructions(s).expect("input should be valid");
    let mut array = LensArray::default();
    array.perform_sequence(&instructions, verbose);
    array.power()
}

#[test]
fn test_part2() {
    const EXAMPLE: &str = "rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7";
    assert_eq!(part2(EXAMPLE, true), 145);
}

fn get_input() -> &'static str {
    str::from_utf8(include_bytes!("input.txt")).unwrap().trim()
}

fn main() {
    let input = get_input();
    println!("day 15 part 1: {}", part1(input));
    println!("day 15 part 2: {}", part2(input, false));
}
