use aho_corasick::AhoCorasick;
use std::borrow::Cow;

use std::error::Error;
use std::io::{stdin, BufRead};

const VALID_NUMBERS: [&str; 9] = ["one", "two", "three", "four", "five", "six", "seven", "eight", "nine"];

fn main() -> Result<(), Box<dyn Error>> {
    let mut numbers = Vec::with_capacity(18);

    for i in 1..=9 {
        numbers.push((Cow::from(i.to_string()), i));
    }
    for (i, number) in VALID_NUMBERS.iter().enumerate() {
        numbers.push((Cow::from(*number), i + 1));
    }

    let matcher = AhoCorasick::new(numbers.iter().map(|(s, _)| s.as_bytes()))?;

    let lines = stdin().lock().lines();
    let sum = lines
        .flatten()
        .try_fold(0, |acc, line| {
            let mut patterns = matcher.find_overlapping_iter::<&[u8]>(line.as_ref());
            let first = numbers[patterns.next()?.pattern().as_usize()].1;
            let last = patterns
                .last()
                .map_or(first, |found| numbers[found.pattern().as_usize()].1);
            Some(acc + first * 10 + last)
        })
        .ok_or("invalid input")?;

    println!("{sum}");
    Ok(())
}
