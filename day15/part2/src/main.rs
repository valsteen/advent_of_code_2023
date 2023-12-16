use std::error::Error;
use std::io::{stdin, BufRead};

#[derive(Default, Debug, Clone)]
struct Lens {
    focal: usize,
    label: String,
}

#[derive(Default, Debug, Clone)]
struct LensBox {
    lenses: Vec<Lens>,
}

struct Game {
    boxes: [LensBox; 256],
}

impl Default for Game {
    fn default() -> Self {
        Game {
            boxes: vec![LensBox::default(); 256].try_into().unwrap(),
        }
    }
}

enum Instruction {
    Remove(usize, String),
    Add(usize, String, usize),
}

impl TryFrom<&str> for Instruction {
    type Error = Box<dyn Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.bytes().last() == Some(b'-') {
            return Ok(Instruction::Remove(
                hash(&value[0..value.len() - 1]),
                value[0..value.len() - 1].to_string(),
            ));
        }
        let mut parts = value.splitn(2, '=');
        let label = parts.next().ok_or("empty string")?.to_string();
        Ok(Instruction::Add(
            hash(label.as_str()),
            label,
            parts
                .next()
                .ok_or_else(|| format!("missing focal in {value}"))?
                .parse()?,
        ))
    }
}

impl Game {
    fn process(&mut self, instruction: Instruction) -> (usize, &LensBox) {
        match instruction {
            Instruction::Remove(destination, label) => {
                self.boxes[destination].lenses.retain(|l| l.label != label);
                (destination, &self.boxes[destination])
            }
            Instruction::Add(destination, label, focal) => {
                if let Some(lens) = self.boxes[destination].lenses.iter_mut().find(|l| l.label == label) {
                    lens.focal = focal;
                } else {
                    self.boxes[destination].lenses.push(Lens { focal, label });
                }
                (destination, &self.boxes[destination])
            }
        }
    }
}

fn hash(s: &str) -> usize {
    let mut result = 0;

    for c in s.bytes() {
        result += usize::from(c);
        result *= 17;
        result %= 256;
    }
    result
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut game = Game::default();
    for line in stdin().lock().lines() {
        for lens in line?.split(',') {
            let instruction = Instruction::try_from(lens)?;
            game.process(instruction);
        }
    }

    let mut sum = 0usize;
    for (i, lensbox) in game.boxes.iter().enumerate() {
        for (j, lens) in lensbox.lenses.iter().enumerate() {
            sum += (i + 1) * (j + 1) * lens.focal;
        }
    }
    println!("{sum}");

    Ok::<_, Box<dyn Error>>(())
}
