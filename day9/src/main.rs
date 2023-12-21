use std::fs::File;
use std::io::{BufRead, BufReader};

struct MeasurementHistory(pub Vec<i64>);

impl MeasurementHistory {
    fn difference_series(&self) -> Self {
        let pairs = self.0.iter().skip(1).zip(self.0.clone());
        MeasurementHistory(pairs.map(|(next, prev)| next - prev).collect())
    }

    fn predict_next(&self) -> i64 {
        let last = self.0.last().expect("Measurement history cannot be empty");
        *last
            + (if self.0.iter().all(|m| *m == *last) {
                0
            } else {
                self.difference_series().predict_next()
            })
    }

    fn predict_prev(&self) -> i64 {
        let first = self.0.first().expect("Measurement history cannot be empty");
        *first
            - (if self.0.iter().all(|m| *m == *first) {
                0
            } else {
                self.difference_series().predict_prev()
            })
    }
}

fn parse_measurements<'a, T: std::io::Read>(reader: BufReader<T>) -> Vec<MeasurementHistory> {
    reader
        .lines()
        .map(|l| l.unwrap())
        .map(|line| {
            MeasurementHistory(
                line.split_ascii_whitespace()
                    .map(|s| s.parse::<i64>().unwrap())
                    .collect(),
            )
        })
        .collect()
}

fn answer_a<T: std::io::Read>(reader: BufReader<T>) -> i64 {
    let measurements = parse_measurements(reader);
    measurements.iter().map(|m| m.predict_next()).sum()
}

fn answer_b<T: std::io::Read>(reader: BufReader<T>) -> i64 {
    let measurements = parse_measurements(reader);
    measurements.iter().map(|m| m.predict_prev()).sum()
}

fn main() -> std::io::Result<()> {
    let file = File::open("day9/input.txt")?;
    let reader = BufReader::new(file);
    let result = answer_a(reader);
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
        assert!(result == 114);
    }

    #[test]
    fn input_a() {
        let input = include_str!("../input.txt");
        let reader = BufReader::new(input.as_bytes());
        let result = answer_a(reader);
        println!("{:?}", result);
        assert!(result == 2105961943);
    }

    #[test]
    fn sample_b() {
        let input = include_str!("../test.txt");
        let reader = BufReader::new(input.as_bytes());
        let result = answer_b(reader);
        println!("{:?}", result);
        assert!(result == 2);
    }

    #[test]
    fn input_b() {
        let input = include_str!("../input.txt");
        let reader = BufReader::new(input.as_bytes());
        let result = answer_b(reader);
        println!("{:?}", result);
        assert!(result == 1019);
    }
}
