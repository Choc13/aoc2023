use std::fs::File;
use std::io::{BufRead, BufReader};

struct Reveal {
    red: u32,
    green: u32,
    blue: u32,
}

impl Reveal {
    fn empty() -> Self {
        Reveal {
            red: 0u32,
            green: 0u32,
            blue: 0u32,
        }
    }

    fn red(red: u32) -> Self {
        Reveal {
            red,
            ..Self::empty()
        }
    }

    fn green(green: u32) -> Self {
        Reveal {
            green,
            ..Self::empty()
        }
    }

    fn blue(blue: u32) -> Self {
        Reveal {
            blue,
            ..Self::empty()
        }
    }

    fn add(&self, other: &Self) -> Self {
        Reveal {
            red: self.red + other.red,
            green: self.green + other.green,
            blue: self.blue + other.blue,
        }
    }

    fn power(&self) -> u32 {
        self.red * self.green * self.blue
    }
}

struct Game {
    id: u32,
    reveals: Vec<Reveal>,
}

impl Game {
    fn min_possible_reveal(&self) -> Reveal {
        Reveal {
            red: self.reveals.iter().map(|r| r.red).max().unwrap(),
            green: self.reveals.iter().map(|r| r.green).max().unwrap(),
            blue: self.reveals.iter().map(|r| r.blue).max().unwrap(),
        }
    }
}

fn parse_reveal(str: &str) -> Reveal {
    str.split(',')
        .map(|s| s.trim())
        .fold(Reveal::empty(), |r, s| {
            let split = s.split_ascii_whitespace().collect::<Vec<_>>();
            let count: u32 = split.first().unwrap().parse().unwrap();
            let second = split.get(1).unwrap();
            match *second {
                "red" => Reveal::red(count).add(&r),
                "green" => Reveal::green(count).add(&r),
                "blue" => Reveal::blue(count).add(&r),
                x => panic!("{:?}", x),
            }
        })
}

fn parse_game_id(str: &str) -> u32 {
    str.trim_start_matches("Game ").parse().unwrap()
}

fn parse_game(str: &str) -> Game {
    let split = str.split(':').map(|s| s.trim()).collect::<Vec<_>>();
    Game {
        id: parse_game_id(&split.first().unwrap()),
        reveals: split
            .last()
            .unwrap()
            .split(";")
            .map(|s| s.trim())
            .map(parse_reveal)
            .collect(),
    }
}

fn answer_a(file: File) -> u32 {
    BufReader::new(file)
        .lines()
        .filter_map(|s| s.ok())
        .map(|s| parse_game(&s))
        .filter(|g| {
            g.reveals
                .iter()
                .all(|r| r.red <= 12 && r.green <= 13 && r.blue <= 14)
        })
        .map(|g| g.id)
        .sum::<u32>()
}

fn answer_b(file: File) -> u32 {
    BufReader::new(file)
        .lines()
        .filter_map(|s| s.ok())
        .map(|s| parse_game(&s))
        .map(|g| g.min_possible_reveal())
        .map(|r| r.power())
        .sum::<u32>()
}

fn main() -> std::io::Result<()> {
    let file = File::open("day2-a/input.txt")?;
    let result = answer_b(file);
    println!("{:?}", result);
    Ok(())
}
