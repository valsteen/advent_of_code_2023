use std::error::Error;
use std::io::{stdin, BufRead};

struct CategoryMap {
    source_start: u64,
    destination_start: u64,
    len: u64,
}

impl CategoryMap {
    fn convert(&self, number: u64) -> Option<u64> {
        (self.source_start..self.source_start + self.len)
            .contains(&number)
            .then(|| self.destination_start + number - self.source_start)
    }
}

trait CategoryMapTrait {
    fn convert(&self, number: u64) -> u64;
}

impl CategoryMapTrait for Vec<CategoryMap> {
    fn convert(&self, number: u64) -> u64 {
        self.iter().find_map(|c| c.convert(number)).unwrap_or(number)
    }
}

type ConversionMap = Vec<Vec<CategoryMap>>;

trait Converter {
    fn convert(&self, number: u64) -> u64;
}

impl Converter for ConversionMap {
    fn convert(&self, number: u64) -> u64 {
        self.iter().fold(number, |number, vec| vec.convert(number))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    let mut lines = lines.map_while(Result::ok);
    let seeds = lines
        .next()
        .ok_or("missing first line")?
        .split_once(": ")
        .ok_or("invalid first line")?
        .1
        .split(' ')
        .map(str::parse::<u64>)
        .collect::<Result<Vec<u64>, _>>()?;

    let mut conversions = ConversionMap::new();

    if !lines.next().ok_or("expected next blank line")?.is_empty() {
        return Err("blank second line expected".into());
    }

    'main: loop {
        lines.next().ok_or("Unexpected end of file")?;
        let mut this_conversion = Vec::new();
        loop {
            let line = match lines.next() {
                None => {
                    conversions.push(this_conversion);
                    break 'main;
                }
                Some(line) if line.is_empty() => {
                    conversions.push(this_conversion);
                    break;
                }
                Some(line) => line,
            };
            let mut parts = line.splitn(3, ' ').map(str::parse::<u64>);
            let destination_start = parts
                .next()
                .ok_or_else(|| format!("expected destination in {line}"))??;
            let source_start = parts.next().ok_or_else(|| format!("expected source in {line}"))??;
            let len = parts.next().ok_or_else(|| format!("expected length in {line}"))??;
            this_conversion.push(CategoryMap {
                source_start,
                destination_start,
                len,
            });
        }
    }

    println!(
        "{}",
        seeds
            .into_iter()
            .map(|seed| conversions.convert(seed))
            .min()
            .ok_or("no minimum found")?
    );
    Ok(())
}
