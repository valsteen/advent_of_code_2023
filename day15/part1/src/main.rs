use std::error::Error;
use std::io::{stdin, BufRead};

fn main() -> Result<(), Box<dyn Error>> {
    let sum = stdin()
        .lock()
        .lines()
        .map(|line| {
            Ok(line?
                .split(',')
                .map(|s| {
                    let mut result = 0usize;

                    for c in s.bytes() {
                        result += usize::from(c);
                        result *= 17;
                        result %= 256;
                    }
                    result
                })
                .sum::<usize>())
        })
        .sum::<Result<usize, Box<dyn Error>>>()?;

    println!("{sum}");

    Ok::<_, Box<dyn Error>>(())
}
