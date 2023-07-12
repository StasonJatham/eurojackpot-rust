use std::collections::{BinaryHeap, HashMap};
use rand::Rng;
use std::time::{Duration, Instant};
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::io::{Write, BufWriter};



const TOP_COMBINATIONS_COUNT: usize = 5;
const SAVE_FREQUENCY: u64 = 10_000_000;
const FILE_PATH: &str = "top_combinations.txt";

#[derive(Debug, Eq, PartialEq, Clone)]
struct CombinationCount(Vec<u8>, u64);

impl Ord for CombinationCount {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.1.cmp(&other.1)
    }
}

impl PartialOrd for CombinationCount {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}




fn main() {
    let mut frequency_map: HashMap<Vec<u8>, u64> = HashMap::new();
    let mut number_counts: HashMap<u8, u64> = HashMap::new();
    let mut combinations: Vec<Vec<u8>> = Vec::new();
    let mut iteration_count: u64 = 0;

    // Read the file if it exists to continue counting from where it left off
    if let Ok(file) = File::open(FILE_PATH) {
        let reader = BufReader::new(file);
        for line in reader.lines() {
            if let Ok(combination) = line {
                if let Some(numbers) = parse_combination(&combination) {
                    let count = frequency_map.entry(numbers.clone()).or_insert(0);
                    *count += 1;
                    update_number_counts(&mut number_counts, &numbers);
                }
            }
        }
        combinations = find_top_combinations(&frequency_map);
        iteration_count = (combinations.len() as u64 * SAVE_FREQUENCY) as u64;
        let loaded_combinations = load_top_combinations();
        println!("Loaded combinations: {:?}", loaded_combinations);
    }

    let mut rng = rand::thread_rng();
    let mut start_time = Instant::now();

    loop {
        let combination = generate_combination(&mut rng);
        let count = frequency_map.entry(combination.clone()).or_insert(0);
        *count += 1;
        update_number_counts(&mut number_counts, &combination);

        if iteration_count % SAVE_FREQUENCY == 0 {
            save_top_combinations(&combinations);
            println!("Saved top combinations:");
            for (index, combination) in combinations.iter().enumerate() {
                println!("{}: {:?}", index + 1, combination);
            }
        }

        combinations = find_top_combinations_parallel(&frequency_map);

        iteration_count += 1;

        // Print the top combinations and current iteration count every 10 seconds
        if start_time.elapsed() >= Duration::from_secs(10) {
            println!("Top 5 combinations: {:?}", combinations);
            println!("Current iteration count: {}", iteration_count);
            start_time = Instant::now();
        }
    }
}

fn generate_combination(rng: &mut impl Rng) -> Vec<u8> {
    let mut combination = Vec::new();
    while combination.len() < 5 {
        let num = rng.gen_range(1..51);
        if !combination.contains(&num) {
            combination.push(num);
        }
    }
    combination.sort();
    let euro_num_1 = rng.gen_range(1..11);
    let mut euro_num_2 = rng.gen_range(1..11);
    while euro_num_2 == euro_num_1 {
        euro_num_2 = rng.gen_range(1..11);
    }
    combination.push(euro_num_1);
    combination.push(euro_num_2);
    combination
}


fn update_number_counts(number_counts: &mut HashMap<u8, u64>, combination: &[u8]) {
    for num in combination {
        *number_counts.entry(*num).or_insert(0) += 1;
    }
}

fn find_top_combinations(frequency_map: &HashMap<Vec<u8>, u64>) -> Vec<Vec<u8>> {
    let mut combinations: BinaryHeap<CombinationCount> = BinaryHeap::new();
    for (combination, count) in frequency_map.iter() {
        if combinations.len() < TOP_COMBINATIONS_COUNT {
            combinations.push(CombinationCount(combination.clone(), *count));
        } else if let Some(min_combination) = combinations.peek() {
            if count > &min_combination.1 {
                combinations.pop();
                combinations.push(CombinationCount(combination.clone(), *count));
            }
        }
    }
    combinations.iter().map(|cc| cc.0.clone()).collect()
}

fn find_top_combinations_parallel(frequency_map: &HashMap<Vec<u8>, u64>) -> Vec<Vec<u8>> {
    frequency_map
        .par_iter()
        .fold(
            || Vec::new(),
            |mut acc, (combination, count)| {
                if acc.len() < TOP_COMBINATIONS_COUNT {
                    acc.push(CombinationCount(combination.clone(), *count));
                } else if let Some(min_combination) = acc.iter().min() {
                    if count > &min_combination.1 {
                        let min_index = acc.iter().position(|cc| cc == min_combination).unwrap();
                        acc.remove(min_index);
                        acc.push(CombinationCount(combination.clone(), *count));
                    }
                }
                acc
            },
        )
        .reduce(
            || Vec::new(),
            |mut acc1, acc2| {
                acc1.extend(acc2);
                if acc1.len() > TOP_COMBINATIONS_COUNT {
                    acc1.sort_unstable_by(|a, b| b.1.cmp(&a.1));
                    acc1.truncate(TOP_COMBINATIONS_COUNT);
                }
                acc1
            },
        )
        .iter()
        .map(|cc| cc.0.clone())
        .collect()
}


fn save_top_combinations(combinations: &[Vec<u8>]) {
    let file = File::create(FILE_PATH)
        .expect("Failed to create file for saving top combinations");
    let mut writer = BufWriter::new(file);

    for combination in combinations {
        let combination_str = combination
            .iter()
            .map(|num| num.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        writeln!(writer, "{}", combination_str)
            .expect("Failed to write to file");
    }

    writer.flush().expect("Failed to flush file");
}

fn load_top_combinations() -> Vec<Vec<u8>> {
    let file = File::open(FILE_PATH)
        .expect("Failed to open file for loading top combinations");
    let reader = BufReader::new(file);

    let mut combinations = Vec::new();
    for line in reader.lines() {
        if let Ok(combination_str) = line {
            let combination = parse_combination(&combination_str);
            if let Some(combination) = combination {
                combinations.push(combination);
            }
        }
    }

    combinations
}


fn parse_combination(combination_str: &str) -> Option<Vec<u8>> {
    let numbers = combination_str
        .split_whitespace()
        .filter_map(|num_str| num_str.parse().ok())
        .collect::<Vec<u8>>();
    if numbers.len() == 7 {
        Some(numbers)
    } else {
        None
    }
}
