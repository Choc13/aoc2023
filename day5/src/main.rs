use core::panic;
use std::fs::File;
use std::io::{BufRead, BufReader};

use mapping::{MergeResult, MergeSource};

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mapping {
    length: u64,
    source_start: u64,
    dest_start: u64,
}

pub mod mapping {
    use crate::Mapping;

    #[derive(Debug, PartialEq, Eq)]
    pub enum MergeSource {
        Input(Mapping),
        Output(Mapping),
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct MergeResult {
        pub left: Option<MergeSource>,
        pub intersection: Option<Mapping>,
        pub right: Option<MergeSource>,
    }
    impl MergeResult {
        pub(crate) fn left_mapping(&self) -> Option<Mapping> {
            self.left.as_ref().map(|s| match s {
                MergeSource::Input(m) | MergeSource::Output(m) => m.to_owned(),
            })
        }
    }
}

impl Mapping {
    fn new(dest_start: u64, source_start: u64, length: u64) -> Self {
        Self {
            length,
            source_start,
            dest_start,
        }
    }

    fn source_end(&self) -> u64 {
        self.source_start + self.length
    }

    fn dest_end(&self) -> u64 {
        self.dest_start + self.length
    }

    fn try_map_dest(&self, source: u64) -> Option<u64> {
        if self.source_start <= source && source < (self.source_start + self.length) {
            Some(source - self.source_start + self.dest_start)
        } else {
            None
        }
    }

    fn truncate_end(&self, length: u64) -> Self {
        Self {
            length: self.length.min(length),
            ..*self
        }
    }

    fn truncate_start(&self, length: u64) -> Self {
        let length = self.length.min(length);
        let delta = self.length - length;
        Self {
            length,
            source_start: self.source_start + delta,
            dest_start: self.dest_start + delta,
        }
    }

    fn merge(&self, output: &Self) -> MergeResult {
        MergeResult {
            left: if self.dest_start < output.source_start {
                let length = self.length.min(output.source_start - self.dest_start);
                Some(MergeSource::Input(self.truncate_end(length)))
            } else if output.source_start < self.dest_start {
                let length = output.length.min(self.dest_start - output.source_start);
                Some(MergeSource::Output(output.truncate_end(length)))
            } else {
                None
            },
            intersection: {
                let start = self.dest_start.max(output.source_start);
                let end = self.dest_end().min(output.source_end());
                if end > start {
                    Some(Mapping {
                        length: end - start,
                        source_start: self.source_start + (start - self.dest_start),
                        dest_start: output.dest_start + (start - output.source_start),
                    })
                } else {
                    None
                }
            },
            right: if self.dest_end() > output.source_end() {
                let length = self.length.min(self.dest_end() - output.source_end());
                Some(MergeSource::Input(self.truncate_start(length)))
            } else if output.source_end() > self.dest_end() {
                let length = output.length.min(output.source_end() - self.dest_end());
                Some(MergeSource::Output(output.truncate_start(length)))
            } else {
                None
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Map {
    ranges: Vec<Mapping>,
}

impl Map {
    fn lookup_dest(&self, source: u64) -> u64 {
        self.ranges
            .iter()
            .find_map(|m| m.try_map_dest(source))
            .unwrap_or(source)
    }

    fn merge(&self, output: &Map) -> Map {
        let mut inputs = self.ranges.to_owned();
        inputs.sort_by_key(|m| m.dest_start);
        let mut outputs = output.ranges.to_owned();
        outputs.sort_by_key(|m| m.source_start);
        let ranges = unfold((inputs, outputs), |(inputs, outputs)| {
            match (&inputs[..], &outputs[..]) {
                ([input, inputs @ ..], [output, outputs @ ..]) => {
                    let merge_result = input.merge(output);
                    let merged = &[merge_result.left_mapping(), merge_result.intersection]
                        .iter()
                        .filter_map(|x| x.to_owned())
                        .collect::<Vec<_>>();
                    let state = match merge_result.right {
                        Some(MergeSource::Input(input)) => {
                            let mut x = vec![input];
                            x.extend(inputs.to_vec());
                            (x.to_owned(), outputs.to_owned())
                        }
                        Some(MergeSource::Output(output)) => {
                            let mut x = vec![output];
                            x.extend(outputs.to_vec());
                            (inputs.to_owned(), x.to_owned())
                        }
                        None => (inputs.to_owned(), outputs.to_owned()),
                    };
                    Some((state.to_owned(), merged.to_owned()))
                }
                ([], [output, outputs @ ..]) => {
                    Some(((Vec::new(), outputs.to_owned()), vec![output.to_owned()]))
                }
                ([input, inputs @ ..], []) => {
                    Some(((inputs.to_owned(), Vec::new()), vec![input.to_owned()]))
                }
                (&[], &[]) => None,
            }
        })
        .flat_map(|m| m)
        .collect();
        Map { ranges }
    }
}

#[derive(Debug)]
struct Almanac {
    seeds: Vec<u64>,
    seed_to_soil: Map,
    soil_to_fert: Map,
    fert_to_water: Map,
    water_to_light: Map,
    light_to_temp: Map,
    temp_to_hum: Map,
    hum_to_location: Map,
}

impl Almanac {
    fn seed_to_location(&self) -> Map {
        self.seed_to_soil
            .merge(&self.soil_to_fert)
            .merge(&self.fert_to_water)
            .merge(&self.water_to_light)
            .merge(&self.light_to_temp)
            .merge(&self.temp_to_hum)
            .merge(&self.hum_to_location)
    }

    fn lookup_seed_location(&self, seed: u64) -> u64 {
        self.seed_to_location().lookup_dest(seed)
    }

    fn closest_seed_location(&self) -> u64 {
        self.seeds
            .iter()
            .map(|s| self.lookup_seed_location(*s))
            .min()
            .unwrap()
    }
}

fn parse_almanac<'a, T: std::io::Read>(reader: BufReader<T>) -> Almanac {
    fn parse_seeds(
        mut lines: impl Iterator<Item = String>,
    ) -> (Vec<u64>, impl Iterator<Item = String>) {
        let first = lines.next().unwrap();
        let seeds = first
            .trim_start_matches("seeds: ")
            .split_ascii_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        (seeds, lines)
    }

    fn parse_maps(lines: impl Iterator<Item = String>) -> Vec<Map> {
        lines
            .filter(|l| !l.contains("map:"))
            .skip_while(|l| l.is_empty())
            .fold(
                (Vec::new(), Vec::new()),
                |(mut maps, mut curr_map), line| {
                    if line.is_empty() {
                        maps.push(Map { ranges: curr_map });
                        (maps, Vec::new())
                    } else {
                        let mapping = match line
                            .split_ascii_whitespace()
                            .map(|s| s.parse::<u64>().unwrap())
                            .collect::<Vec<_>>()[..]
                        {
                            [dest_start, source_start, length] => {
                                Mapping::new(dest_start, source_start, length)
                            }
                            _ => panic!("Invalid mapping line '{}'.", line),
                        };
                        curr_map.push(mapping);
                        (maps, curr_map)
                    }
                },
            )
            .0
    }

    let lines = reader.lines().map(|l| l.unwrap());
    let (seeds, lines) = parse_seeds(lines);
    match &parse_maps(lines)[..] {
        [seed_to_soil, soil_to_fert, fert_to_water, water_to_light, light_to_temp, temp_to_hum, hum_to_location] => {
            Almanac {
                seeds,
                seed_to_soil: seed_to_soil.to_owned(),
                soil_to_fert: soil_to_fert.to_owned(),
                fert_to_water: fert_to_water.to_owned(),
                water_to_light: water_to_light.to_owned(),
                light_to_temp: light_to_temp.to_owned(),
                temp_to_hum: temp_to_hum.to_owned(),
                hum_to_location: hum_to_location.to_owned(),
            }
        }
        _ => panic!("Incorrect number of mappings found."),
    }
}

fn answer_a<T: std::io::Read>(reader: BufReader<T>) -> u64 {
    let almanac = parse_almanac(reader);
    almanac.closest_seed_location()
}

fn answer_b<T: std::io::Read>(reader: BufReader<T>) -> Option<u64> {
    let almanac: Almanac = parse_almanac(reader);
    let seed_to_location = almanac.seed_to_location();
    almanac
        .seeds
        .chunks_exact(2)
        .map(|p| (p.get(0).unwrap(), p.get(1).unwrap()))
        .flat_map(|(range_start, length)| {
            let range_end = range_start.checked_add(*length).unwrap();
            seed_to_location.ranges.iter().filter_map(move |r| {
                let range_end = range_end.min(r.source_end());
                let range_start = *range_start.max(&r.source_start);
                if range_start < range_end {
                    Some(range_start)
                } else {
                    None
                }
            })
        })
        .map(|s| seed_to_location.lookup_dest(s))
        .min()
}

fn main() -> std::io::Result<()> {
    let file = File::open("day5/input.txt")?;
    let reader = BufReader::new(file);
    let result = answer_b(reader);
    println!("{:?}", result);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use crate::{
        answer_a, answer_b,
        mapping::{MergeResult, MergeSource},
        parse_almanac, Map, Mapping,
    };

    #[test]
    fn sample_a() {
        let input = include_str!("../test.txt");
        let reader = BufReader::new(input.as_bytes());
        let result = answer_a(reader);
        assert!(result == 35);
    }

    #[test]
    fn test_seed_locations() {
        let input = include_str!("../test.txt");
        let reader = BufReader::new(input.as_bytes());
        let almanac = parse_almanac(reader);

        assert!(almanac.lookup_seed_location(79) == 82);
        assert!(almanac.lookup_seed_location(14) == 43);
        assert!(almanac.lookup_seed_location(55) == 86);
        assert!(almanac.lookup_seed_location(13) == 35);
    }

    #[test]
    fn test_merge_mapping_with_self() {
        let mapping = Mapping {
            length: 1,
            source_start: 1,
            dest_start: 1,
        };
        assert!(
            mapping.merge(&mapping)
                == MergeResult {
                    left: None,
                    intersection: Some(mapping),
                    right: None
                }
        );
    }

    #[test]
    fn test_merge_mapping_with_input_left() {
        let input = Mapping {
            length: 1,
            source_start: 1,
            dest_start: 1,
        };
        let output = Mapping {
            length: 1,
            source_start: 2,
            dest_start: 2,
        };
        assert!(
            input.merge(&output)
                == MergeResult {
                    left: Some(MergeSource::Input(input)),
                    intersection: None,
                    right: Some(MergeSource::Output(output))
                }
        );
    }

    #[test]
    fn test_merge_mapping_with_input_right() {
        let input = Mapping {
            length: 1,
            source_start: 3,
            dest_start: 3,
        };
        let output = Mapping {
            length: 1,
            source_start: 2,
            dest_start: 2,
        };
        assert!(
            input.merge(&output)
                == MergeResult {
                    left: Some(MergeSource::Output(output)),
                    intersection: None,
                    right: Some(MergeSource::Input(input))
                }
        );
    }

    #[test]
    fn test_merge_mapping_input_intersects_output_left() {
        let input = Mapping {
            length: 2,
            source_start: 0,
            dest_start: 10,
        };
        let output = Mapping {
            length: 3,
            source_start: 11,
            dest_start: 20,
        };
        assert!(
            input.merge(&output)
                == MergeResult {
                    left: Some(MergeSource::Input(Mapping {
                        length: 1,
                        source_start: 0,
                        dest_start: 10
                    })),
                    intersection: Some(Mapping {
                        length: 1,
                        source_start: 1,
                        dest_start: 20
                    }),
                    right: Some(MergeSource::Output(Mapping {
                        length: 2,
                        source_start: 12,
                        dest_start: 21
                    }))
                }
        );
    }

    #[test]
    fn test_merge_mapping_failing_example() {
        let input = Mapping {
            length: 2,
            source_start: 98,
            dest_start: 50,
        };
        let output = Mapping {
            length: 37,
            source_start: 15,
            dest_start: 0,
        };
        let result = input.merge(&output);
        assert!(
            result
                == MergeResult {
                    left: Some(MergeSource::Output(Mapping {
                        length: 35,
                        source_start: 15,
                        dest_start: 0
                    })),
                    intersection: Some(Mapping {
                        length: 2,
                        source_start: 98,
                        dest_start: 35
                    }),
                    right: None
                }
        );
    }

    #[test]
    fn test_merge_maps() {
        let input = Map {
            ranges: vec![
                Mapping {
                    length: 2,
                    source_start: 98,
                    dest_start: 50,
                },
                Mapping {
                    length: 48,
                    source_start: 50,
                    dest_start: 52,
                },
            ],
        };
        let output = Map {
            ranges: vec![
                Mapping {
                    length: 37,
                    source_start: 15,
                    dest_start: 0,
                },
                Mapping {
                    length: 2,
                    source_start: 52,
                    dest_start: 37,
                },
                Mapping {
                    length: 15,
                    source_start: 0,
                    dest_start: 39,
                },
            ],
        };
        let merged = input.merge(&output);
        assert!(
            merged
                == Map {
                    ranges: vec![
                        Mapping {
                            length: 15,
                            source_start: 0,
                            dest_start: 39,
                        },
                        Mapping {
                            length: 35,
                            source_start: 15,
                            dest_start: 0,
                        },
                        Mapping {
                            length: 2,
                            source_start: 98,
                            dest_start: 35,
                        },
                        Mapping {
                            length: 2,
                            source_start: 50,
                            dest_start: 37,
                        },
                        Mapping {
                            length: 46,
                            source_start: 52,
                            dest_start: 54,
                        },
                    ]
                }
        );
    }

    #[test]
    fn sample_b() {
        let input = include_str!("../test.txt");
        let reader = BufReader::new(input.as_bytes());
        let result = answer_b(reader);
        println!("{:?}", result);
        assert!(result == Some(46));
    }

    #[test]
    fn test_seed_to_location() {
        let input = include_str!("../test.txt");
        let reader = BufReader::new(input.as_bytes());
        let almanac = parse_almanac(reader);
        let result = almanac.seed_to_location().lookup_dest(82);
        println!("{:?}", result);
        assert!(result == 46);
    }
}
