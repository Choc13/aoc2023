use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Div;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Race {
    time: u64,
    best_distance: u64,
}

impl Race {
    fn distance(&self, length_of_press: u64) -> u64 {
        length_of_press * (self.time - length_of_press)
    }

    fn margin_of_error(&self) -> u64 {
        // The function relating distance, d, to the length of the button press, x, is f(d) = x(T - x)
        // This function is symmetrical and convex with it's maxima at the midpoint.
        // So in order to do the root finding we can just binary search down from the midpoint
        // to find the highest position that results in a distance worse than the record.
        // Then we can double this and add on the mid-point (taking care to add 2 for odd T) to get the result.

        fn binary_search_down(race: &Race, start: u64, end: u64) -> u64 {
            if start == end {
                return start;
            }

            let midpoint = start + (end.checked_sub(start).unwrap().div_ceil(2));
            if race.distance(midpoint) > race.best_distance {
                binary_search_down(race, start, midpoint.checked_sub(1).unwrap())
            } else {
                binary_search_down(race, midpoint, end)
            }
        }

        let is_even = self.time % 2 == 0;
        let midpoint = self.time / 2; // Rounds down in the odd case

        let lh_root = binary_search_down(self, 0, midpoint);
        ((midpoint - lh_root) * 2) - if is_even { 1 } else { 0 }
    }
}

fn parse_races<'a, T: std::io::Read>(reader: BufReader<T>) -> Vec<Race> {
    fn parse_line(lines: &Vec<String>, index: usize, prefix: &str) -> Vec<u64> {
        lines
            .get(index)
            .unwrap()
            .trim_start_matches(prefix)
            .split_ascii_whitespace()
            .map(|s| s.trim().parse::<u64>().unwrap())
            .collect()
    }

    let lines = reader.lines().map(|l| l.unwrap()).collect::<Vec<_>>();
    let times = parse_line(&lines, 0, "Time:");
    let distances = parse_line(&lines, 1, "Distance:");
    times
        .iter()
        .zip(distances)
        .map(|(time, best_distance)| Race {
            time: *time,
            best_distance,
        })
        .collect()
}

fn parse_race_b<'a, T: std::io::Read>(reader: BufReader<T>) -> Race {
    fn parse_line(lines: &Vec<String>, index: usize, prefix: &str) -> u64 {
        lines
            .get(index)
            .unwrap()
            .trim_start_matches(prefix)
            .replace(" ", "")
            .trim()
            .parse::<u64>()
            .unwrap()
    }

    let lines = reader.lines().map(|l| l.unwrap()).collect::<Vec<_>>();
    let time = parse_line(&lines, 0, "Time:");
    let best_distance = parse_line(&lines, 1, "Distance:");
    Race {
        time,
        best_distance,
    }
}

fn answer_a<T: std::io::Read>(reader: BufReader<T>) -> u64 {
    let races = parse_races(reader);
    races.iter().map(|r| r.margin_of_error()).product()
}

fn answer_b<T: std::io::Read>(reader: BufReader<T>) -> u64 {
    let race = parse_race_b(reader);
    race.margin_of_error()
}

fn main() -> std::io::Result<()> {
    let file = File::open("day6/input.txt")?;
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
        assert!(result == 288);
    }

    #[test]
    fn sample_b() {
        let input = include_str!("../test.txt");
        let reader = BufReader::new(input.as_bytes());
        let result = answer_b(reader);
        println!("{:?}", result);
        assert!(result == 71503);
    }
}
