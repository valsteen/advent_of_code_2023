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
    fn reverse_convert(&self, number: u64) -> Option<u64> {
        (self.destination_start..self.destination_start + self.len)
            .contains(&number)
            .then(|| self.source_start + number - self.destination_start)
    }
}

trait CategoryMapTrait {
    fn convert(&self, number: u64) -> u64;
    fn reverse_convert(&self, number: u64) -> u64;

    fn merge(self, other: &Self) -> Self;
}

impl CategoryMapTrait for Vec<CategoryMap> {
    fn convert(&self, number: u64) -> u64 {
        self.iter().find_map(|c| c.convert(number)).unwrap_or(number)
    }

    fn reverse_convert(&self, number: u64) -> u64 {
        self.iter().find_map(|c| c.reverse_convert(number)).unwrap_or(number)
    }

    fn merge(self, other: &Self) -> Self {
        let mut simplified = Vec::new();

        let mut starting_points = self
            .iter()
            .flat_map(|self_item| [self_item.source_start, self_item.source_start + self_item.len])
            .chain(other.iter().flat_map(|other_item| {
                [
                    self.reverse_convert(other_item.source_start),
                    self.reverse_convert(other_item.source_start + other_item.len),
                ]
            }))
            .collect::<Vec<_>>();
        starting_points.sort_unstable();
        starting_points.dedup();

        let mut starting_points = starting_points.into_iter();
        let mut this_start = starting_points.next().unwrap();
        loop {
            let Some(next) = starting_points.next() else {
                break;
            };
            let len = next - this_start;

            simplified.push(CategoryMap {
                source_start: this_start,
                destination_start: other.convert(self.convert(this_start)),
                len,
            });

            this_start = next;
        }
        simplified
    }
}

trait Converter {
    fn convert(&self, number: u64) -> u64;
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

    let mut conversions: Vec<CategoryMap> = Vec::new();

    if !lines.next().ok_or("expected next blank line")?.is_empty() {
        return Err("blank second line expected".into());
    }

    'main: loop {
        lines.next().ok_or("Unexpected end of file")?;
        let mut this_conversion = Vec::new();
        loop {
            let line = match lines.next() {
                None => {
                    // ¯\_(ツ)_/¯
                    conversions = conversions.merge(&this_conversion);
                    break 'main;
                }
                Some(line) if line.is_empty() => {
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
        conversions = conversions.merge(&this_conversion);
    }

    let ranges = seeds.chunks(2).try_fold(Vec::new(), |mut res, tuple| {
        let mut tuple = tuple.iter();
        let start = *tuple.next().ok_or("tuple start expected")?;
        let len = *tuple.next().ok_or("tuple end expected")?;

        res.push(start..start + len);
        Ok::<_, &'static str>(res)
    })?;

    let as_conversion = ranges
        .iter()
        .map(|range| CategoryMap {
            source_start: range.start,
            destination_start: range.start,
            len: range.end - range.start,
        })
        .collect::<Vec<_>>();

    let min = as_conversion
        .merge(&conversions)
        .into_iter()
        .filter_map(|conversion| {
            ranges
                .iter()
                .any(|range| range.contains(&conversion.source_start))
                .then(|| conversions.convert(conversion.source_start))
        })
        .min()
        .ok_or("cannot find minimum")?;

    println!("{min}");
    Ok(())
}
