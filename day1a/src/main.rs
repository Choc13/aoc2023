use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> std::io::Result<()> {
    let file = File::open("day1a/input.txt")?;
    let result: u32 = BufReader::new(file)
        .lines()
        .map(|l| {
            l.map(|l| {
                let digits = l.chars().filter_map(|c| c.to_digit(10)).collect::<Vec<_>>();
                let first = digits.first().unwrap();
                let last = digits.last().unwrap();
                (10 * first) + last
            })
            .unwrap()
        })
        .sum();
    println!("{:?}", result);
    Ok(())
}
