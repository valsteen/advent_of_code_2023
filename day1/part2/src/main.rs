#![feature(coroutines)]
#![feature(iter_from_coroutine)]

use std::error::Error;
use std::io::{stdin, BufRead};
use std::ops::Deref;

const VALID_NUMBERS: [&str; 9] = ["one", "two", "three", "four", "five", "six", "seven", "eight", "nine"];

struct FoundString {
    len: usize,
    index: usize,
}

fn find_all<T: Deref<Target = [u8]>, I: Deref<Target = [u8]>, U: Deref<Target = [I]>>(
    needles: U,
    haystack: T,
) -> impl Iterator<Item = usize> {
    std::iter::from_coroutine(move || {
        let mut progress = Vec::<FoundString>::new();

        for haystack_index in 0..haystack.len() {
            let c = haystack[haystack_index];
            let mut i = 0usize;

            while i < progress.len() {
                let found = &mut progress[i];

                let needle = needles[found.index].as_ref();
                if needle[found.len] == c {
                    found.len += 1;
                    if found.len == needle.len() {
                        yield found.index;
                    } else {
                        i += 1;
                        continue;
                    }
                }
                progress.remove(i);
            }

            for index in 0..needles.len() {
                let needle = &needles[index];
                if needle[0] == c {
                    if needle.len() == 1 {
                        yield index;
                    } else {
                        progress.push(FoundString { len: 1, index });
                    }
                }
            }
        }
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let numbers: Vec<Vec<u8>> = (1..=9)
        .map(|i| i.to_string().as_bytes().to_owned())
        .chain(VALID_NUMBERS.into_iter().map(|number| number.as_bytes().to_owned()))
        .collect();

    let lines = stdin().lock().lines();
    let sum = lines.map_while(Result::ok).try_fold(0, |acc, line| {
        let mut patterns = find_all(numbers.as_slice(), line.as_bytes());
        let first = patterns.next().ok_or_else(|| format!("No match in '{line}'",))? % 9 + 1;
        let last = patterns.last().map_or(first, |found| found % 9 + 1);
        Ok::<_, String>(acc + first * 10 + last)
    })?;

    println!("{sum}");
    Ok(())
}
