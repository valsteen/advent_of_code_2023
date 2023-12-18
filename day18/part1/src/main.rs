use geo::algorithm::Area;
use geo::{BooleanOps, BoundingRect, Coord, LineString, Polygon};
use geo_types::MultiPolygon;
use std::error::Error;
use std::io::{stdin, BufRead};
use std::str::FromStr;

fn parse_lines<I, E>(lines: I) -> impl Iterator<Item = Result<Instruction, Box<dyn Error>>>
where
    I: Iterator<Item = Result<String, E>>,
    E: Error + 'static,
{
    lines.map(|line| line?.parse())
}

#[derive(Debug, Copy, Clone)]
struct Instruction {
    direction: Direction,
    amount: f64,
}

impl FromStr for Instruction {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || format!("invalid line : {s}");
        let mut parts = s.split(' ');
        Ok(Instruction {
            direction: Direction::try_from(
                parts
                    .next()
                    .ok_or_else(err)?
                    .as_bytes()
                    .first()
                    .copied()
                    .ok_or_else(err)?,
            )?,
            amount: parts.next().ok_or_else(err)?.parse()?,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Direction {
    Horizontal(HorizontalDirection),
    Vertical(VerticalDirection),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum HorizontalDirection {
    L,
    R,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum VerticalDirection {
    U,
    D,
}

impl TryFrom<u8> for Direction {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            b'U' => Self::Vertical(VerticalDirection::U),
            b'D' => Self::Vertical(VerticalDirection::D),
            b'L' => Self::Horizontal(HorizontalDirection::L),
            b'R' => Self::Horizontal(HorizontalDirection::R),
            _ => Err(format!("invalid direction: {}", char::from(value)))?,
        })
    }
}

fn coords_from_instruction(prev: Coord, instruction: Instruction) -> Coord {
    match instruction.direction {
        Direction::Horizontal(HorizontalDirection::L) => Coord::from((prev.x - instruction.amount, prev.y)),
        Direction::Horizontal(HorizontalDirection::R) => Coord::from((prev.x + instruction.amount, prev.y)),
        Direction::Vertical(VerticalDirection::U) => Coord::from((prev.x, prev.y - instruction.amount)),
        Direction::Vertical(VerticalDirection::D) => Coord::from((prev.x, prev.y + instruction.amount)),
    }
}

#[allow(forbidden_lint_groups)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_precision_loss)]
fn make_polygon<I: Iterator<Item = Result<Instruction, Box<dyn Error>>>>(
    iterator: I,
) -> Result<Polygon, Box<dyn Error>> {
    let mut prev = Coord::from((0.0, 0.0));
    let mut multi_polygon = MultiPolygon::new([].to_vec());

    for instruction in iterator {
        let instruction = instruction?;
        let destination = coords_from_instruction(prev, instruction);
        let mut rect = LineString::from([prev, destination].to_vec())
            .bounding_rect()
            .ok_or("unable to create a shape")?;
        let mut max = rect.max();
        max.x += 1.0;
        max.y += 1.0;
        rect.set_max(max);

        let add = MultiPolygon::new([rect.to_polygon()].to_vec()).difference(&multi_polygon);
        multi_polygon.0.extend(add);
        prev = destination;
    }

    Ok(simplify(&multi_polygon))
}

fn simplify(multi_polygon: &MultiPolygon) -> Polygon {
    let boundaries = MultiPolygon::from([multi_polygon.bounding_rect().unwrap()].to_vec());
    let diff = boundaries.difference(multi_polygon);
    let mut result = boundaries.difference(&diff);
    assert_eq!(result.0.len(), 1);
    result.0.pop().unwrap()
}
fn main() -> Result<(), Box<dyn Error>> {
    let polygon = make_polygon(parse_lines(stdin().lock().lines()))?;
    assert_eq!(polygon.interiors().len(), 1);
    let interior = Polygon::new(polygon.interiors()[0].clone(), vec![]);
    println!("{}", polygon.unsigned_area() + interior.unsigned_area());

    Ok::<_, Box<dyn Error>>(())
}
