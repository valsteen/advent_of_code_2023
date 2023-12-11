use std::error::Error;
use std::io::{stdin, BufRead};
use std::iter::{Copied, Zip};
use std::slice::{Iter, IterMut};

trait MapTrait<'a>: Sized {
    type Iterator: Iterator<Item = (i64, i64)>;
    type MutIterator: Iterator<Item = (&'a mut i64, &'a mut i64)>;
    fn try_from_input<I, E>(iterator: I) -> Result<(Self, i64, i64, i64, i64), E>
    where
        I: Iterator<Item = Result<String, E>>,
        E: Error;

    fn len(&self) -> usize;
    fn contains_x(&self, x: i64) -> bool;
    fn contains_y(&self, y: i64) -> bool;
    fn iter(&'a self) -> Self::Iterator;
    fn iter_mut(&'a mut self) -> Self::MutIterator;
    fn get(&self, i: usize) -> Option<(i64, i64)>;
}

impl<'a> MapTrait<'a> for (Vec<i64>, Vec<i64>) {
    type Iterator = Zip<Copied<Iter<'a, i64>>, Copied<Iter<'a, i64>>>;
    type MutIterator = Zip<IterMut<'a, i64>, IterMut<'a, i64>>;

    fn try_from_input<I, E>(iterator: I) -> Result<(Self, i64, i64, i64, i64), E>
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
            min_x,
            min_y,
            max_x,
            max_y,
        ))
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn contains_x(&self, x: i64) -> bool {
        self.0.contains(&x)
    }

    fn contains_y(&self, y: i64) -> bool {
        self.1.contains(&y)
    }

    fn iter(&'a self) -> Self::Iterator {
        self.0.iter().copied().zip(self.1.iter().copied())
    }

    fn iter_mut(&'a mut self) -> Self::MutIterator {
        self.0.iter_mut().zip(self.1.iter_mut())
    }

    fn get(&self, i: usize) -> Option<(i64, i64)> {
        Some((*self.0.get(i)?, *self.1.get(i)?))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let (mut map, min_x, min_y, mut max_x, mut max_y) = <(Vec<_>, Vec<_>)>::try_from_input(stdin().lock().lines())?;

    let mut x = min_x;
    while x <= max_x {
        if !map.contains_x(x) {
            for (px, _) in map.iter_mut() {
                if *px > x {
                    *px += 1;
                    max_x = max_x.max(*px);
                }
            }
            x += 1;
        }
        x += 1;
    }

    let mut y = min_y;
    while y <= max_y {
        if !map.contains_y(y) {
            for (_, py) in map.iter_mut() {
                if *py > y {
                    *py += 1;
                    max_y = max_y.max(*py);
                }
            }
            y += 1;
        }
        y += 1;
    }

    let sum = (0..map.len() - 1)
        .flat_map(|start| (start + 1..map.len()).map(move |end| (start, end)))
        .map(|(start, end)| {
            let p1 = map.get(start).unwrap();
            let p2 = map.get(end).unwrap();
            (p1.0 - p2.0).abs() + (p1.1 - p2.1).abs()
        })
        .sum::<i64>();
    println!("{sum}");
    Ok::<_, Box<dyn Error>>(())
}
