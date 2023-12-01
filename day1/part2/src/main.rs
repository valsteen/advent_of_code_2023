use aho_corasick::AhoCorasick;
use std::error::Error;
use std::io::{stdin, BufRead};

const VALID_NUMBERS: [&str; 9] = ["one", "two", "three", "four", "five", "six", "seven", "eight", "nine"];

fn main() -> Result<(), Box<dyn Error>> {
    let numbers = (1..=9)
        .map(|i| i.to_string().as_bytes().to_owned())
        .chain(VALID_NUMBERS.into_iter().map(|number| number.as_bytes().to_owned()));

    let matcher = AhoCorasick::new(numbers)?;

    let lines = stdin().lock().lines();
    let sum = lines
        .flatten()
        .try_fold(0, |acc, line| {
            let mut patterns = matcher.find_overlapping_iter::<&[u8]>(line.as_ref());
            let first = patterns.next()?.pattern().as_usize() % 9 + 1;
            let last = patterns
                .last()
                .map_or(first, |found| found.pattern().as_usize() % 9 + 1);
            Some(acc + first * 10 + last)
        })
        .ok_or("invalid input")?;

    println!("{sum}");
    Ok(())
}
