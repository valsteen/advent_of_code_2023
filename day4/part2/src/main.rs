use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use std::io::{stdin, BufRead};

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();

    let mut winning = HashMap::new();
    let mut hands = VecDeque::new();
    let mut n = 1;

    for line in lines.map_while(Result::ok) {}.try_fold(VecDeque::new(), |mut hands, (n, line)| {
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
            .try_fold(None, |acc, n| {
                let n = n.map_err(|_| format!("Malformed line: {line}"))?;
                Ok::<_, Box<dyn Error>>(
                    winning
                        .contains(&n)
                        .then(|| Some(acc.map_or(1, |acc| acc * 2)))
                        .unwrap_or(acc),
                )
            })?
            .unwrap_or_default();
        if score > 0 {
            hands.push_back((n + 1, score));
        }
        Ok::<_, Box<dyn Error>>(hands)
    })?;
    let mut wins = HashMap::new();

    while let Some(hand) = hands.pop_front() {}

    println!("{hands:?}");
    Ok(())
}
