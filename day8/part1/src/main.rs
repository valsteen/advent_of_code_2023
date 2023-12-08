use std::collections::HashMap;
use std::error::Error;
use std::io::{stdin, BufRead};

#[derive(Debug, Copy, Clone)]
enum Direction {
    L,
    R,
}

#[derive(Debug)]
struct Destinations {
    l: String,
    r: String,
}

impl From<String> for Destinations {
    fn from(value: String) -> Self {
        Destinations {
            l: value[7..=9].to_string(),
            r: value[12..=14].to_string(),
        }
    }
}

impl TryFrom<char> for Direction {
    type Error = &'static str;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(match value {
            'L' => Direction::L,
            'R' => Direction::R,
            _ => Err("invalid direction")?,
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    let mut lines = lines.map_while(Result::ok);

    let directions_list = lines
        .next()
        .ok_or("expected directions line")?
        .chars()
        .map(Direction::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    lines.next();

    let destinations = lines
        .map(|line| {
            let source = line[0..3].to_string();
            let destinations = Destinations::try_from(line)?;
            Ok::<_, Box<dyn Error>>((source, destinations))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;

    let mut step = 0;
    let mut origin = "AAA".to_string();
    let mut cycle = directions_list.iter().cycle();

    while origin != "ZZZ" {
        let direction = cycle.next().unwrap();
        let destination = destinations.get(origin.as_str()).unwrap();
        origin = match direction {
            Direction::L => destination.l.clone(),
            Direction::R => destination.r.clone(),
        };
        step += 1;
    }

    println!("{step}");

    Ok::<_, Box<dyn Error>>(())
}
