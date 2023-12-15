use rayon::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::io::{stdin, BufRead};
use std::num::ParseIntError;
use std::sync::{Arc, RwLock};

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

type CacheLock = Arc<RwLock<HashMap<(Vec<Option<bool>>, Vec<usize>, Option<usize>), usize>>>;

fn get_cache_value(cache: &CacheLock, key: &(Vec<Option<bool>>, Vec<usize>, Option<usize>)) -> Option<usize> {
    cache.read().unwrap().get(key).copied()
}

fn insert_into_cache(cache: &CacheLock, key: (Vec<Option<bool>>, Vec<usize>, Option<usize>), value: usize) {
    cache.write().unwrap().insert(key, value);
}

fn calculate_combinations(
    mut conditions: Vec<Option<bool>>,
    mut damages: Vec<usize>,
    mut expected_damage: Option<usize>,
    cache: CacheLock,
) -> usize {
    let key = (conditions.clone(), damages.clone(), expected_damage);
    if let Some(result) = get_cache_value(&cache, &key) {
        return result;
    }

    if (expected_damage == Some(0) && conditions.last() == Some(&Some(true)))
        || conditions
            .iter()
            .filter(|&condition| condition == &Some(true) || condition.is_none())
            .count()
            < damages.iter().sum::<usize>() + expected_damage.unwrap_or(0)
    {
        insert_into_cache(&cache, key, 0);
        return 0;
    }

    if expected_damage.unwrap_or_default() == 0 {
        while conditions.last() == Some(&Some(false)) {
            expected_damage = None;
            conditions.pop();
        }

        if damages.is_empty() {
            if !conditions.iter().any(|&c| c == Some(true)) {
                insert_into_cache(&cache, key, 1);
                return 1;
            }
            insert_into_cache(&cache, key, 0);
            return 0;
        }
    }

    match conditions.pop() {
        None | Some(Some(false)) => (),
        Some(Some(true)) => match expected_damage {
            Some(0) => (),
            None => {
                if let Some(expected) = damages.pop() {
                    return calculate_combinations(conditions, damages, Some(expected - 1), cache);
                }
            }
            Some(expected) => {
                return calculate_combinations(conditions, damages, Some(expected - 1), cache);
            }
        },
        Some(None) => match expected_damage {
            Some(0) => {
                return calculate_combinations(conditions, damages, None, cache);
            }
            Some(expected) => {
                return calculate_combinations(conditions, damages, Some(expected - 1), cache);
            }
            None => {
                let mut tasks = Vec::from([(conditions.clone(), damages.clone(), None)]);
                if let Some(damage) = damages.pop() {
                    tasks.push((conditions.clone(), damages.clone(), Some(damage - 1)));
                }
                let result = tasks
                    .into_par_iter()
                    .map(|args| calculate_combinations(args.0, args.1, args.2, cache.clone()))
                    .sum::<usize>();
                insert_into_cache(&cache, key, result);
                return result;
            }
        },
    }

    insert_into_cache(&cache, key, 0);
    0
}

fn main() -> Result<(), Box<dyn Error>> {
    let parsed_lines = parse_lines(stdin().lock().lines());
    let result = parsed_lines
        .into_iter()
        .collect::<Result<Vec<(Vec<Option<bool>>, Vec<usize>)>, _>>()?
        .into_par_iter()
        .map(|line| {
            let (mut records, mut damages) = line;

            let records_copy = records.clone();
            let damages_copy = damages.clone();

            for _ in 0..4 {
                records.push(None);
                records.extend(records_copy.clone());
                damages.extend(damages_copy.clone());
            }

            calculate_combinations(records, damages, None, Arc::new(RwLock::new(HashMap::new())))
        })
        .sum::<usize>();

    println!("{result}");
    Ok::<_, Box<dyn Error>>(())
}
