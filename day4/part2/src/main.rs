use std::collections::{HashSet, VecDeque};
use std::error::Error;
use std::io::{stdin, BufRead};
use std::iter::repeat;

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();

    let (_, score) = lines
        .map_while(Result::ok)
        .try_fold((VecDeque::new(), 0), |(mut copies, total), line| {
            let instances = copies.pop_front().unwrap_or(1);

            let (_, line) = line.split_once(':').ok_or_else(|| format!("Malformed line: {line}"))?;
            let (part1, part2) = line.split_once('|').ok_or_else(|| format!("Malformed line: {line}"))?;

            let winning = part1
                .split(' ')
                .filter(|s| !s.is_empty())
                .map(str::parse::<u64>)
                .collect::<Result<HashSet<_>, _>>()
                .map_err(|_| format!("Malformed line: {line}"))?;
            let score = part2
                .split(' ')
                .filter(|s| !s.is_empty())
                .map(str::parse::<u64>)
                .try_fold(0, |acc, n| {
                    let n = n.map_err(|_| format!("Malformed line: {line}"))?;
                    if winning.contains(&n) {
                        Ok::<_, Box<dyn Error>>(acc + 1)
                    } else {
                        Ok(acc)
                    }
                })?;

            let mut index = 0;
            while index < score {
                if let Some(n) = copies.get_mut(index) {
                    *n += instances;
                    index += 1;
                    continue;
                }
                break;
            }
            copies.extend(repeat(1 + instances).take(score - index));
            Ok::<_, Box<dyn Error>>((copies, instances + total))
        })?;
    println!("{score}");
    Ok(())
}
