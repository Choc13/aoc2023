use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::marker::PhantomData;

trait JackVariant: Copy {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct RegularJack {}
impl JackVariant for RegularJack {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Joker {}
impl JackVariant for Joker {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Card<J: JackVariant> {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack(PhantomData<J>),
    Queen,
    King,
    Ace,
}

impl Card<RegularJack> {
    fn rank(&self) -> u64 {
        match self {
            Card::Two => 0,
            Card::Three => 1,
            Card::Four => 2,
            Card::Five => 3,
            Card::Six => 4,
            Card::Seven => 5,
            Card::Eight => 6,
            Card::Nine => 7,
            Card::Ten => 8,
            Card::Jack(PhantomData) => 9,
            Card::Queen => 10,
            Card::King => 11,
            Card::Ace => 12,
        }
    }
}

impl Ord for Card<RegularJack> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl PartialOrd for Card<RegularJack> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Card<Joker> {
    fn rank(&self) -> u64 {
        match self {
            Card::Jack(PhantomData) => 0,
            Card::Two => 1,
            Card::Three => 2,
            Card::Four => 3,
            Card::Five => 4,
            Card::Six => 5,
            Card::Seven => 6,
            Card::Eight => 7,
            Card::Nine => 8,
            Card::Ten => 9,
            Card::Queen => 10,
            Card::King => 11,
            Card::Ace => 12,
        }
    }
}

impl Ord for Card<Joker> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl PartialOrd for Card<Joker> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HandType {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

impl HandType {
    fn rank(&self) -> u64 {
        match self {
            HandType::HighCard => 0,
            HandType::OnePair => 1,
            HandType::TwoPair => 2,
            HandType::ThreeOfAKind => 3,
            HandType::FullHouse => 4,
            HandType::FourOfAKind => 5,
            HandType::FiveOfAKind => 6,
        }
    }
}

impl Ord for HandType {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl PartialOrd for HandType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Hand<J: JackVariant> {
    cards: [Card<J>; 5],
}

impl Hand<RegularJack> {
    fn typ(&self) -> HandType {
        let counts = self.cards.iter().fold(HashMap::new(), |mut s, c| {
            s.entry(*c).and_modify(|e| *e += 1).or_insert(1);
            s
        });
        let mut sorted_counts = counts.values().collect::<Vec<_>>();
        sorted_counts.sort();
        sorted_counts.reverse();
        match &sorted_counts[..] {
            [5] => HandType::FiveOfAKind,
            [4, 1] => HandType::FourOfAKind,
            [3, 2] => HandType::FullHouse,
            [3, 1, 1] => HandType::ThreeOfAKind,
            [2, 2, 1] => HandType::TwoPair,
            [2, 1, 1, 1] => HandType::OnePair,
            [1, 1, 1, 1, 1] => HandType::HighCard,
            _ => panic!("Unknown hand type '{:?}", self),
        }
    }
}

impl Ord for Hand<RegularJack> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.typ().cmp(&other.typ()) {
            Ordering::Equal => self.cards.cmp(&other.cards),
            x => x,
        }
    }
}

impl PartialOrd for Hand<RegularJack> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hand<Joker> {
    fn typ(&self) -> HandType {
        let mut counts = self.cards.iter().fold(HashMap::new(), |mut s, c| {
            s.entry(*c).and_modify(|e| *e += 1).or_insert(1);
            s
        });
        let jacks = counts
            .remove_entry(&Card::Jack(PhantomData::<Joker>))
            .map(|x| x.1)
            .unwrap_or(0);

        let mut sorted_counts = counts.values().collect::<Vec<_>>();
        sorted_counts.sort();
        sorted_counts.reverse();
        if sorted_counts.is_empty() {
            sorted_counts = vec![&0]
        }
        let max = *sorted_counts[0] + jacks;
        sorted_counts[0] = &max;

        match &sorted_counts[..] {
            [5] => HandType::FiveOfAKind,
            [4, 1] => HandType::FourOfAKind,
            [3, 2] => HandType::FullHouse,
            [3, 1, 1] => HandType::ThreeOfAKind,
            [2, 2, 1] => HandType::TwoPair,
            [2, 1, 1, 1] => HandType::OnePair,
            [1, 1, 1, 1, 1] => HandType::HighCard,
            _ => panic!("Unknown hand type '{:?}", self),
        }
    }
}

impl Ord for Hand<Joker> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.typ().cmp(&other.typ()) {
            Ordering::Equal => self.cards.cmp(&other.cards),
            x => x,
        }
    }
}

impl PartialOrd for Hand<Joker> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn parse_game<'a, T: std::io::Read, J: JackVariant>(reader: BufReader<T>) -> Vec<(Hand<J>, u64)> {
    fn parse_card<J: JackVariant>(c: char) -> Card<J> {
        match c {
            '2' => Card::Two,
            '3' => Card::Three,
            '4' => Card::Four,
            '5' => Card::Five,
            '6' => Card::Six,
            '7' => Card::Seven,
            '8' => Card::Eight,
            '9' => Card::Nine,
            'T' => Card::Ten,
            'J' => Card::Jack(PhantomData),
            'Q' => Card::Queen,
            'K' => Card::King,
            'A' => Card::Ace,
            x => panic!("Unknown card '{:?}'", x),
        }
    }

    fn parse_hand<J: JackVariant>(str: &str) -> Hand<J> {
        match str.chars().map(parse_card).collect::<Vec<_>>()[..] {
            [a, b, c, d, e] => Hand {
                cards: [a, b, c, d, e],
            },
            _ => panic!("Expected only 5 cards in a hand, but got {:?}", str.len()),
        }
    }

    fn parse_line<J: JackVariant>(line: String) -> (Hand<J>, u64) {
        match &line.split_ascii_whitespace().collect::<Vec<_>>()[..] {
            [hand, bid] => (parse_hand(hand), bid.parse().unwrap()),
            x => panic!("Invalid line, {:?}", x),
        }
    }

    reader.lines().map(|l| l.unwrap()).map(parse_line).collect()
}

fn answer_a<T: std::io::Read>(reader: BufReader<T>) -> u64 {
    let mut game = parse_game::<_, RegularJack>(reader);
    game.sort_by_key(|x| x.0);
    game.iter()
        .enumerate()
        .map(|(rank, g)| (rank as u64 + 1) * g.1)
        .sum()
}

fn answer_b<T: std::io::Read>(reader: BufReader<T>) -> u64 {
    let mut game = parse_game::<_, Joker>(reader);
    game.sort_by_key(|x| x.0);
    game.iter()
        .enumerate()
        .map(|(rank, g)| (rank as u64 + 1) * g.1)
        .sum()
}

fn main() -> std::io::Result<()> {
    let file = File::open("day7/input.txt")?;
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
        assert!(result == 6440);
    }

    #[test]
    fn sample_b() {
        let input = include_str!("../test.txt");
        let reader = BufReader::new(input.as_bytes());
        let result = answer_b(reader);
        println!("{:?}", result);
        assert!(result == 5905);
    }
}
