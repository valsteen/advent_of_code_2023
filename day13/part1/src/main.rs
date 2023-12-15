use std::error::Error;
use std::io::{stdin, BufRead};

fn parse_map<I, E>(lines: I) -> Result<Vec<Vec<Vec<bool>>>, Box<dyn Error>>
where
    I: Iterator<Item = Result<String, E>>,
    E: Error + 'static,
{
    let mut maps = Vec::new();
    let mut map = Vec::new();

    for line in lines {
        let line = line?;
        if line.is_empty() {
            maps.push(map);
            map = Vec::new();
            continue;
        }

        map.push(
            line.into_bytes()
                .into_iter()
                .map(|c| {
                    Ok::<_, Box<dyn Error>>(match c {
                        b'#' => true,
                        b'.' => false,
                        _ => Err("invalid tile")?,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
    }
    maps.push(map);

    Ok(maps)
}

fn find_mirror_x(map: &Vec<Vec<bool>>) -> Option<usize> {
    for mid_x in 1..map[0].len() {
        let mut mirror_at = None;
        for x in 0..map[0].len() {
            let mirror_x = mid_x + mid_x - x - 1;
            if mirror_x >= map[0].len() {
                continue;
            }
            let is_mirror = (0..map.len()).all(|y| map[y][x] == map[y][mirror_x]);
            if !is_mirror {
                mirror_at = None;
                break;
            }
            mirror_at = Some(mid_x);
        }
        if let Some(x) = mirror_at {
            return Some(x);
        }
    }
    None
}

fn find_mirror_y(map: &Vec<Vec<bool>>) -> Option<usize> {
    for mid_y in 1..map.len() {
        let mut mirror_at = None;
        for y in 0..map.len() {
            let mirror_y = mid_y + mid_y - y - 1;
            if mirror_y >= map.len() {
                continue;
            }
            if map[y] != map[mirror_y] {
                mirror_at = None;
                break;
            }
            mirror_at = Some(mid_y);
        }
        if let Some(y) = mirror_at {
            return Some(y);
        }
    }
    None
}

fn main() -> Result<(), Box<dyn Error>> {
    let maps = parse_map(stdin().lock().lines())?;

    let mut sum_x = 0usize;
    let mut sum_y = 0usize;

    'main: for map in maps {
        if let Some(score) = find_mirror_y(&map) {
            sum_y += score;
            continue 'main;
        } else if let Some(score) = find_mirror_x(&map) {
            sum_x += score;
            continue 'main;
        }
    }

    println!("{}", sum_x + 100 * sum_y);

    Ok::<_, Box<dyn Error>>(())
}
