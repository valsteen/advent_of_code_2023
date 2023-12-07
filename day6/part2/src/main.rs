use std::error::Error;
use std::io::{stdin, BufRead};

struct Race {
    time: u64,
    record: u64,
}

impl Race {
    fn runs(&self) -> usize {
        (0..=self.time)
            .filter(|time| (self.time - time) * time > self.record)
            .count()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    let mut lines = lines.map_while(Result::ok);

    let time = lines.next().ok_or("no times given")?.chars().fold(0u64, |acc, c| {
        if let Some(digit) = c.to_digit(10) {
            acc * 10 + u64::from(digit)
        } else {
            acc
        }
    });
    let record = lines.next().ok_or("no distance given")?.chars().fold(0u64, |acc, c| {
        if let Some(digit) = c.to_digit(10) {
            acc * 10 + u64::from(digit)
        } else {
            acc
        }
    });

    let possibilities = Race { time, record }.runs();

    println!("{possibilities}");

    Ok::<_, Box<dyn Error>>(())
}
