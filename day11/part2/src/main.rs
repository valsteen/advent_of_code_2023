use std::convert::TryFrom;
use std::error::Error;
use std::io::{stdin, BufRead};

type MapResult = ((Vec<i64>, Vec<i64>), (i64, i64, i64, i64));

fn try_from_input<I, E>(iterator: I) -> Result<MapResult, E>
where
    I: Iterator<Item = Result<String, E>>,
    E: Error,
{
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (i64::MAX, i64::MAX, i64::MIN, i64::MIN);
    Ok((
        iterator
            .enumerate()
            .try_fold((Vec::new(), Vec::new()), |vecs, (y, line)| {
                Ok(line?
                    .into_bytes()
                    .into_iter()
                    .enumerate()
                    .filter(|&(_, c)| c == b'#')
                    .fold(vecs, |mut vecs, (x, _)| {
                        let x = i64::try_from(x).unwrap();
                        let y = i64::try_from(y).unwrap();
                        min_x = min_x.min(x);
                        min_y = min_y.min(y);
                        max_x = max_x.max(x);
                        max_y = min_x.max(y);
                        vecs.0.push(x);
                        vecs.1.push(y);
                        vecs
                    }))
            })?,
        (min_x, min_y, max_x, max_y),
    ))
}

fn update_values(value: &mut i64, max_value: &mut i64, map: &mut [i64]) {
    while *value <= *max_value {
        if !map.contains(value) {
            map.iter_mut().filter(|px| **px > *value).for_each(|px| *px += 999_999);
            *max_value = map.iter().max().copied().unwrap_or(*max_value);
            *value += 999_999;
        }
        *value += 1;
    }
}

fn calculate_sum(map: &[i64]) -> i64 {
    map.iter()
        .enumerate()
        .flat_map(|(i, &xi)| map[i + 1..].iter().map(move |&xj| (xi - xj).abs()))
        .sum()
}

fn main() -> Result<(), Box<dyn Error>> {
    let (mut map, (mut min_x, mut min_y, mut max_x, mut max_y)) = try_from_input(stdin().lock().lines())?;
    update_values(&mut min_x, &mut max_x, &mut map.0);
    update_values(&mut min_y, &mut max_y, &mut map.1);
    let sum = calculate_sum(&map.0) + calculate_sum(&map.1);
    println!("{sum}");
    Ok(())
}
