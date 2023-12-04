use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
struct Point {
    x: i64,
    y: i64,
}

#[derive(Debug, Copy, Clone)]
struct Number {
    value: u64,
    origin: Point,
    length: usize,
}

impl Number {
    fn surrounding_points(&self) -> impl Iterator<Item = Point> + '_ {
        let start = self.origin.x - 1;
        let end = self
            .origin
            .x
            .checked_add_unsigned(self.length as u64)
            .unwrap();
        let mut points = Vec::new();
        points.push(Point {
            x: start,
            ..self.origin
        });
        for x in start..=end {
            points.push(Point {
                x,
                y: self.origin.y + 1,
            });
            points.push(Point {
                x,
                y: self.origin.y - 1,
            });
        }

        points.push(Point {
            x: end,
            ..self.origin
        });
        points.into_iter()
    }
}

#[derive(Debug)]
struct Schematic {
    symbols: HashMap<Point, char>,
    numbers: Vec<Number>,
}

impl Schematic {
    fn new() -> Self {
        Self {
            symbols: HashMap::<Point, char>::new(),
            numbers: Vec::new(),
        }
    }

    fn part_numbers(&self) -> impl Iterator<Item = u64> + '_ {
        self.numbers
            .iter()
            .filter(|n| {
                n.surrounding_points()
                    .any(|p| self.symbols.contains_key(&p))
            })
            .map(|n| n.value)
    }

    fn add_symbol(mut self, symbol: Point, char: char) -> Self {
        self.symbols.insert(symbol, char);
        self
    }

    fn add_number(mut self, number: Number) -> Self {
        self.numbers.push(number);
        self
    }

    fn adjacent_parts(&self) -> HashMap<Point, (Number, Number)> {
        let mut adjacent_point_count = HashMap::new();
        for (p, n) in self
            .numbers
            .iter()
            .flat_map(|n| n.surrounding_points().map(move |p| (p, n)))
        {
            let mut parts = adjacent_point_count
                .get(&p)
                .unwrap_or(&Vec::new())
                .to_owned();
            parts.push(*n);
            adjacent_point_count.insert(p, parts);
        }
        adjacent_point_count
            .iter()
            .filter(|(_, c)| c.len() == 2)
            .map(|(p, parts)| (*p, (*parts.get(0).unwrap(), *parts.get(1).unwrap())))
            .collect()
    }

    fn gear_ratios(&self) -> Vec<u64> {
        let adjacent_parts = self.adjacent_parts();
        self.symbols
            .iter()
            .filter(|(_, c)| **c == '*')
            .filter_map(|(p, _)| adjacent_parts.get(p))
            .map(|(a, b)| a.value * b.value)
            .collect()
    }

    fn print(&self) -> String {
        let max_x = self
            .numbers
            .iter()
            .map(|n| n.origin.x + (n.length as i64))
            .max()
            .unwrap();
        let max_y = self.numbers.iter().map(|n| n.origin.y).max().unwrap();
        let number_map = self
            .numbers
            .iter()
            .flat_map(|n| {
                n.value
                    .to_string()
                    .chars()
                    .enumerate()
                    .map(|(i, c)| {
                        (
                            Point {
                                x: n.origin.x + i as i64,
                                ..n.origin
                            },
                            c,
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<HashMap<_, _>>();
        let mut result = String::new();
        for y in 0..=max_y {
            let mut line = String::new();
            for x in 0..=max_x {
                let point = Point { x, y };
                if self.symbols.contains_key(&point) {
                    line.push(*self.symbols.get(&point).unwrap());
                } else if number_map.contains_key(&point) {
                    line.push(*number_map.get(&point).unwrap());
                } else {
                    line.push('.');
                }
            }
            result.push_str(&line);
            result.push('\n');
        }
        result
    }
}

fn parse_schematic(file: &File) -> Schematic {
    BufReader::new(file)
        .lines()
        .map(|l| l.unwrap())
        .enumerate()
        .fold(Schematic::new(), |schematic, (y, line)| {
            line.chars()
                .chain(['.'])
                .enumerate()
                .fold(
                    (schematic, String::new()),
                    |(schematic, mut digits), (x, c)| {
                        if c.is_digit(10) {
                            digits.push(c);
                            (schematic, digits)
                        } else {
                            let point = Point {
                                x: i64::try_from(x).unwrap(),
                                y: i64::try_from(y).unwrap(),
                            };
                            let schematic = if c == '.' {
                                schematic
                            } else {
                                schematic.add_symbol(point, c)
                            };
                            let schematic = if digits.is_empty() {
                                schematic
                            } else {
                                let length = digits.chars().count();
                                schematic.add_number(Number {
                                    value: digits.parse().unwrap(),
                                    origin: Point {
                                        x: point.x.checked_sub(length as i64).unwrap(),
                                        ..point
                                    },
                                    length,
                                })
                            };
                            (schematic, String::new())
                        }
                    },
                )
                .0
        })
}

fn answer_a(file: &File) -> u64 {
    let schematic = parse_schematic(&file);
    schematic.part_numbers().sum()
}

fn answer_b(file: &File) -> u64 {
    let schematic = parse_schematic(&file);
    let gear_ratios = schematic.gear_ratios();
    gear_ratios.iter().sum()
}

fn main() -> std::io::Result<()> {
    let file = File::open("day3/input.txt")?;
    let result = answer_b(&file);
    println!("{:?}", result);
    Ok(())
}
