use std::collections::VecDeque;
use std::error::Error;
use std::io::{stdin, BufRead};
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;

const VALID_NUMBERS: [&str; 9] = ["one", "two", "three", "four", "five", "six", "seven", "eight", "nine"];

struct FoundString {
    len: usize,
    index: usize,
}

struct Finder<'s, I: Deref<Target = [u8]>, U: Deref<Target = [I]>> {
    needles: U,
    haystack: &'s [u8],
    phantom_data: PhantomData<I>,
    progress: VecDeque<FoundString>,
    next_progress: VecDeque<FoundString>,
    haystack_index: usize,
    needle_progress: Option<usize>,
}

impl<'s, I: Deref<Target = [u8]>, U: Deref<Target = [I]>> Finder<'s, I, U> {
    fn new(needles: U, haystack: &'s [u8]) -> Self {
        Self {
            needles,
            haystack,
            phantom_data: PhantomData,
            progress: VecDeque::new(),
            next_progress: VecDeque::new(),
            haystack_index: 0,
            needle_progress: Some(0),
        }
    }
}

impl<'s, I: Deref<Target = [u8]>, U: Deref<Target = [I]>> Iterator for Finder<'s, I, U> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let c = self.haystack.get(self.haystack_index)?;

            if let Some(needle_index) = &mut self.needle_progress {
                while *needle_index < self.needles.len() {
                    let needle = &self.needles[*needle_index];

                    if needle[0] == *c {
                        if needle.len() == 1 {
                            let index = *needle_index;
                            *needle_index += 1;
                            return Some(index);
                        }
                        self.next_progress.push_back(FoundString {
                            len: 1,
                            index: *needle_index,
                        });
                    }
                    *needle_index += 1;
                }
                self.needle_progress = None;
            }

            while let Some(mut found) = self.progress.pop_front() {
                let needle = &self.needles[found.index];

                if needle[found.len] == *c {
                    found.len += 1;
                    if found.len == needle.len() {
                        return Some(found.index);
                    }
                    self.next_progress.push_back(found);
                }
            }

            self.haystack_index += 1;
            mem::swap(&mut self.next_progress, &mut self.progress);
            self.needle_progress = Some(0);
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let numbers: Vec<Vec<u8>> = (1..=9)
        .map(|i| i.to_string().as_bytes().to_owned())
        .chain(VALID_NUMBERS.into_iter().map(|number| number.as_bytes().to_owned()))
        .collect();

    let lines = stdin().lock().lines();
    let sum = lines.flatten().try_fold(0, |acc, line| {
        let mut patterns = Finder::<Vec<u8>, _>::new(numbers.as_slice(), line.as_bytes());
        let first = patterns.next().ok_or_else(|| format!("No match in '{line}'"))? % 9 + 1;
        let last = patterns.last().map_or(first, |found| found % 9 + 1);
        Ok::<_, String>(acc + first * 10 + last)
    })?;

    println!("{sum}");
    Ok(())
}
