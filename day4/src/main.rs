use core::panic;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};

struct Card {
    id: u64,
    winning_numbers: HashSet<u64>,
    revealed_numbers: HashSet<u64>,
}

impl Card {
    fn matches(&self) -> u64 {
        self.revealed_numbers
            .intersection(&self.winning_numbers)
            .count() as u64
    }

    fn score(&self) -> u64 {
        self.matches()
            .checked_sub(1)
            .map(|n| 2u64.pow(n.try_into().unwrap()))
            .unwrap_or(0)
    }
}

fn parse_cards<T: std::io::Read>(reader: BufReader<T>) -> impl Iterator<Item = Card> {
    reader
        .lines()
        .map(|l| l.unwrap())
        .map(|l| match l.split(':').collect::<Vec<_>>()[..] {
            [card_name, card_data] => match card_data.split("|").collect::<Vec<_>>()[..] {
                [winning_numbers, revealed_numbers] => Card {
                    id: card_name.trim_start_matches("Card").trim().parse().unwrap(),
                    winning_numbers: winning_numbers
                        .split_ascii_whitespace()
                        .map(|s| s.parse().unwrap())
                        .collect(),
                    revealed_numbers: revealed_numbers
                        .split_ascii_whitespace()
                        .map(|s| s.parse().unwrap())
                        .collect(),
                },
                _ => panic!("Card data was invalid, expected a |, but got {}", card_data),
            },
            _ => panic!("Card was invalid, expected to find a ':', but got '{}'.", l),
        })
}

fn answer_a<T: std::io::Read>(reader: BufReader<T>) -> u64 {
    parse_cards(reader).map(|c| c.score()).sum()
}

fn answer_b<T: std::io::Read>(reader: BufReader<T>) -> u64 {
    parse_cards(reader)
        .fold(HashMap::new(), |mut card_counts, card| {
            let num_cards = card_counts.get(&card.id).unwrap_or(&0) + 1;
            card_counts.insert(card.id, num_cards);
            let matches = card.matches();
            for id in (card.id + 1)..=(card.id + matches) {
                card_counts.insert(id, card_counts.get(&id).unwrap_or(&0) + num_cards);
            }
            card_counts
        })
        .values()
        .sum()
}

fn main() -> std::io::Result<()> {
    let file = File::open("day4/input.txt")?;
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
        assert!(result == 13);
    }

    #[test]
    fn sample_b() {
        let input = include_str!("../test.txt");
        let reader = BufReader::new(input.as_bytes());
        let result = answer_b(reader);
        assert!(result == 30);
    }
}
