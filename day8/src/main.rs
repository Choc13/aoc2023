use core::panic;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

struct Unfolder<F, S, U>(F, Option<S>)
where
    F: FnMut(S) -> Option<(S, U)>;

impl<F, S, U> Iterator for Unfolder<F, S, U>
where
    F: FnMut(S) -> Option<(S, U)>,
{
    type Item = U;
    fn next(&mut self) -> Option<U> {
        self.1
            .take()
            .and_then(|x| (&mut self.0)(x))
            .map(|(next_v, item)| {
                self.1 = Some(next_v);
                item
            })
    }
}

fn unfold<S, U, F>(state: S, f: F) -> impl Iterator<Item = U>
where
    F: FnMut(S) -> Option<(S, U)>,
{
    Unfolder(f, Some(state))
}

#[derive(Debug)]
enum Instruction {
    Left,
    Right,
}

#[derive(Debug, Clone)]
struct Node {
    label: String,
    left: String,
    right: String,
}

impl Node {
    fn lookup(&self, instruction: &Instruction) -> String {
        match instruction {
            Instruction::Left => self.left.to_owned(),
            Instruction::Right => self.right.to_owned(),
        }
    }
}

#[derive(Debug)]
struct Map {
    instructions: Vec<Instruction>,
    nodes: HashMap<String, Node>,
}

impl Map {
    fn new(instructions: Vec<Instruction>, nodes: Vec<Node>) -> Self {
        Self {
            instructions,
            nodes: nodes
                .iter()
                .map(|n| (n.label.to_owned(), n.to_owned()))
                .collect(),
        }
    }

    fn states(&self, start_label: &str) -> impl Iterator<Item = &Node> {
        let start = self.nodes.get(start_label);
        let instructions = unfold(&self.instructions[..], |state| match state {
            [head] => Some((&self.instructions[..], head)),
            [head, tail @ ..] => Some((tail, head)),
            [] => panic!("No instructions."),
        });
        instructions.scan(start, |s, instruction| {
            let output = s.to_owned();
            let next = s.and_then(|s| self.nodes.get(&s.lookup(instruction)));
            *s = next;
            output
        })
    }

    fn steps_to_exit<'a, F: Fn(&Node) -> bool + 'a>(
        &'a self,
        start_label: &str,
        is_exit: F,
    ) -> impl Iterator<Item = u64> + 'a {
        self.states(start_label)
            .enumerate()
            .filter(move |(_, s)| is_exit(s))
            .map(|(n, _)| n as u64)
    }
}

fn parse_map<'a, T: std::io::Read>(reader: BufReader<T>) -> Map {
    fn parse_instruction(c: char) -> Instruction {
        match c.to_ascii_uppercase() {
            'L' => Instruction::Left,
            'R' => Instruction::Right,
            _ => panic!("Unknown instruction '{:}'", c),
        }
    }

    fn parse_instructions(line: String) -> Vec<Instruction> {
        line.trim().chars().map(parse_instruction).collect()
    }

    fn parse_node(line: String) -> Node {
        match line.split('=').collect::<Vec<_>>()[..] {
            [label, body] => match body.split(',').collect::<Vec<_>>()[..] {
                [left, right] => Node {
                    label: label.trim().to_string(),
                    left: left
                        .trim()
                        .trim_matches(|c| !char::is_alphanumeric(c))
                        .to_string(),
                    right: right
                        .trim()
                        .trim_matches(|c| !char::is_alphanumeric(c))
                        .to_string(),
                },
                _ => panic!("Incorrect number of items in body, found '{:?}'", body),
            },
            _ => panic!("Incorrect number of items in node, found '{:?}", line),
        }
    }

    let mut lines = reader.lines().map(|l| l.unwrap());
    let instructions = parse_instructions(lines.next().unwrap());

    Map::new(
        instructions,
        lines.filter(|l| !l.is_empty()).map(parse_node).collect(),
    )
}

fn answer_a<T: std::io::Read>(reader: BufReader<T>) -> u64 {
    let map = parse_map(reader);
    *map.steps_to_exit("AAA", |s| s.label == "ZZZ")
        .take(1)
        .collect::<Vec<_>>()
        .first()
        .unwrap()
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

fn lcm(a: u64, b: u64) -> u64 {
    b * a / gcd(a, b)
}

fn answer_b<T: std::io::Read>(reader: BufReader<T>) -> u64 {
    let map = parse_map(reader);
    map.nodes
        .keys()
        .filter(|n| n.ends_with('A'))
        .map(|e| {
            let is_exit = |n: &Node| n.label.ends_with('Z');
            let steps_to_exit = map.steps_to_exit(e, is_exit).take(2).collect::<Vec<_>>();
            let first = *steps_to_exit.get(0).unwrap();
            (
                first,
                (*steps_to_exit.get(1).unwrap()).checked_sub(first).unwrap(),
            )
        })
        .fold(1, |s, x| lcm(s, x.0))
}

fn main() -> std::io::Result<()> {
    let file = File::open("day8/input.txt")?;
    let reader = BufReader::new(file);
    let result = answer_b(reader);
    println!("{:?}", result);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use crate::{answer_a, answer_b};

    #[test]
    fn sample_a() {
        let input = include_str!("../test.txt");
        let reader = BufReader::new(input.as_bytes());
        let result = answer_a(reader);
        println!("{:?}", result);
        assert!(result == 2);
    }

    #[test]
    fn sample2_a() {
        let input = include_str!("../test2.txt");
        let reader = BufReader::new(input.as_bytes());
        let result = answer_a(reader);
        println!("{:?}", result);
        assert!(result == 6);
    }

    #[test]
    fn input_a() {
        let input = include_str!("../input.txt");
        let reader = BufReader::new(input.as_bytes());
        let result = answer_a(reader);
        println!("{:?}", result);
        assert!(result == 19667);
    }

    #[test]
    fn sample_b() {
        let input = include_str!("../testb.txt");
        let reader = BufReader::new(input.as_bytes());
        let result = answer_b(reader);
        println!("{:?}", result);
        assert!(result == 6);
    }

    #[test]
    fn input_b() {
        let input = include_str!("../input.txt");
        let reader = BufReader::new(input.as_bytes());
        let result = answer_b(reader);
        println!("{:?}", result);
        assert!(result == 19185263738117);
    }
}
