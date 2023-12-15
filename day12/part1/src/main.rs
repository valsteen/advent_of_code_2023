use std::error::Error;
use std::io::{stdin, BufRead};
use std::num::ParseIntError;

fn parse_lines<I, E>(lines: I) -> impl Iterator<Item = Result<(Vec<Option<bool>>, Vec<usize>), Box<dyn Error>>>
where
    I: Iterator<Item = Result<String, E>>,
    E: Error + 'static,
{
    lines.map(|line| {
        let line = line?;
        let characters = &mut line.chars();

        let conditions = parse_conditions(characters)?;
        let damages = parse_damages(characters.as_str())?;

        Ok((conditions, damages))
    })
}

fn parse_conditions(characters: &mut std::str::Chars) -> Result<Vec<Option<bool>>, Box<dyn Error>> {
    characters
        .map_while(|c| (c != ' ').then_some(c))
        .map(|c| match c {
            '#' => Ok(Some(true)),
            '.' => Ok(Some(false)),
            '?' => Ok(None),
            _ => Err(format!("unknown character '{c}'").into()),
        })
        .collect::<Result<_, Box<dyn Error>>>()
}

fn parse_damages(line: &str) -> Result<Vec<usize>, ParseIntError> {
    line.split(',').map(str::parse).collect::<Result<Vec<_>, _>>()
}

fn calculate_combinations((condition_vec, damage_vec): (Vec<Option<bool>>, Vec<usize>)) -> usize {
    let mut queue = Vec::from([(condition_vec, damage_vec, None)]);
    let mut count = 0;
    while let Some((mut line, mut damages, mut expected)) = queue.pop() {
        if expected == Some(0) && line.last() == Some(&Some(true)) {
            continue;
        }

        if expected.unwrap_or_default() == 0 {
            while line.last() == Some(&Some(false)) {
                expected = None;
                line.pop();
            }

            if damages.is_empty() {
                if !line.iter().any(|&c| c == Some(true)) {
                    count += 1;
                }
                continue;
            }
        }

        match line.pop() {
            None | Some(Some(false)) => (),
            Some(Some(true)) => match expected {
                Some(0) => (),
                None => {
                    if let Some(expected) = damages.pop() {
                        queue.push((line, damages, Some(expected - 1)));
                    }
                }
                Some(expected) => {
                    queue.push((line, damages, Some(expected - 1)));
                }
            },
            Some(None) => match expected {
                Some(0) => {
                    queue.push((line, damages, None));
                }
                Some(expected) => {
                    queue.push((line, damages, Some(expected - 1)));
                }
                None => {
                    queue.push((line.clone(), damages.clone(), None));
                    if let Some(damage) = damages.pop() {
                        queue.push((line, damages, Some(damage - 1)));
                    }
                }
            },
        }
    }

    count
}

fn main() -> Result<(), Box<dyn Error>> {
    let parsed_lines = parse_lines(stdin().lock().lines());

    let result = parsed_lines
        .map(|line| Ok(calculate_combinations(line?)))
        .sum::<Result<usize, Box<dyn Error>>>()?;

    println!("{result}");
    Ok::<_, Box<dyn Error>>(())
}
