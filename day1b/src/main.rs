use std::fs::File;
use std::io::{BufRead, BufReader};

const NUMBER_STRS: [(&str, u32); 20] = [
    ("0", 0u32),
    ("zero", 0u32),
    ("1", 1u32),
    ("one", 1u32),
    ("2", 2u32),
    ("two", 2u32),
    ("3", 3u32),
    ("three", 3u32),
    ("4", 4u32),
    ("four", 4u32),
    ("5", 5u32),
    ("five", 5u32),
    ("6", 6u32),
    ("six", 6u32),
    ("7", 7u32),
    ("seven", 7u32),
    ("8", 8u32),
    ("eight", 8u32),
    ("9", 9u32),
    ("nine", 9u32),
];

fn parse_digits2(str: &str) -> Vec<u32> {
    (0..str.len())
        .map(|i| &str[i..])
        .flat_map(|s| {
            NUMBER_STRS
                .iter()
                .filter_map(|(nstr, n)| if s.starts_with(nstr) { Some(*n) } else { None })
        })
        .collect()
}

fn main() -> std::io::Result<()> {
    let file = File::open("day1-b/input.txt")?;
    let result: u32 = BufReader::new(file)
        .lines()
        .filter_map(|l| {
            l.map(|l| {
                let digits = parse_digits2(&l);
                let first = digits.first().unwrap();
                let last = digits.last().unwrap();
                (10 * first) + last
            })
            .ok()
        })
        .sum();
    println!("{:?}", result);
    Ok(())
}
