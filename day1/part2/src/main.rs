use std::error::Error;
use std::io::{stdin, BufRead};

const VALID_NUMBERS: [&str; 9] = ["one", "two", "three", "four", "five", "six", "seven", "eight", "nine"];

struct FoundString {
    len: usize,
    index: usize,
}

fn find_all<T: AsRef<[u8]>, I: AsRef<[u8]>, U: AsRef<[I]>>(needles: U, haystack: T) -> Vec<usize> {
    let needles = needles.as_ref();
    let mut progress = Vec::<FoundString>::new();
    let mut result = Vec::new();

    for c in haystack.as_ref() {
        progress.retain_mut(|found| {
            let needle = needles[found.index].as_ref();
            if needle[found.len] == *c {
                found.len += 1;
                if found.len == needle.len() {
                    result.push(found.index);
                    false
                } else {
                    true
                }
            } else {
                false
            }
        });
        for (index, needle) in needles.iter().map(AsRef::as_ref).enumerate() {
            if &needle[0] == c {
                if needle.len() == 1 {
                    result.push(index);
                } else {
                    progress.push(FoundString { len: 1, index });
                }
            }
        }
    }
    result
}

fn main() -> Result<(), Box<dyn Error>> {
    let numbers: Vec<Vec<u8>> = (1..=9)
        .map(|i| i.to_string().as_bytes().to_owned())
        .chain(VALID_NUMBERS.into_iter().map(|number| number.as_bytes().to_owned()))
        .collect();

    let lines = stdin().lock().lines();
    let sum = lines.flatten().try_fold(0, |acc, line| {
        let mut patterns = find_all(&numbers, line.as_bytes()).into_iter();
        let first = patterns.next().ok_or_else(|| format!("No match in '{line}'"))? % 9 + 1;
        let last = patterns.last().map_or(first, |found| found % 9 + 1);
        Ok::<_, String>(acc + first * 10 + last)
    })?;

    println!("{sum}");
    Ok(())
}
