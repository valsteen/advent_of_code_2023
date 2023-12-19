use fnv::FnvHashMap;
use std::convert::Infallible;
use std::error::Error;
use std::io::{stdin, BufRead};
use std::str::FromStr;

#[derive(Debug, Copy, Clone)]
enum RuleType {
    Less,
    Greater,
}

impl TryFrom<u8> for RuleType {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            b'<' => Self::Less,
            b'>' => Self::Greater,
            _ => Err(format!("no such rule type {}", char::from(value)))?,
        })
    }
}

#[derive(Debug, Clone)]
enum Destination {
    WorkflowResult(WorkflowResult),
    Routed(String),
}

#[derive(Debug, Clone)]
enum WorkflowResult {
    Accepted,
    Rejected,
}

impl FromStr for Destination {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "A" => Self::WorkflowResult(WorkflowResult::Accepted),
            "R" => Self::WorkflowResult(WorkflowResult::Rejected),
            _ => Self::Routed(s.to_string()),
        })
    }
}

#[derive(Debug, Clone)]
struct Rule {
    name: char,
    rule_type: RuleType,
    number: usize,
    destination: Destination,
}
#[derive(Debug)]
struct Workflow {
    name: String,
    rules: Vec<Rule>,
    default: Destination,
}

enum ParsedRule {
    Rule(Rule),
    Default(String),
}

impl FromStr for ParsedRule {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(':');

        let condition = parts.next().ok_or_else(|| format!("missing condition in {s}"))?;

        let Some(destination) = parts.next().map(str::to_string) else {
            return Ok(ParsedRule::Default(s.to_string()));
        };

        let part = condition.chars().next().ok_or_else(|| format!("missing part in {s}"))?;
        let rule_type = RuleType::try_from(
            s.as_bytes()
                .get(1)
                .copied()
                .ok_or_else(|| format!("missing rule type in {s}"))?,
        )?;
        let number = condition
            .get(2..)
            .ok_or_else(|| format!("missing number in {s}"))?
            .parse()?;

        Ok(ParsedRule::Rule(Rule {
            name: part,
            rule_type,
            number,
            destination: destination.parse()?,
        }))
    }
}
impl FromStr for Workflow {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s[..s.len() - 1].split('{');
        let name = parts
            .next()
            .ok_or_else(|| format!("no workflow name found in {s}"))?
            .to_string();

        let mut rules_part = parts
            .next()
            .ok_or_else(|| format!("no workflow rules found in {s}"))?
            .split(',');

        let mut rules = Vec::new();

        let default = loop {
            let Some(rule) = rules_part.next() else { break None };
            match ParsedRule::from_str(rule)? {
                ParsedRule::Rule(rule) => rules.push(rule),
                ParsedRule::Default(default) => {
                    break Some(default);
                }
            }
        };
        let default = default.ok_or_else(|| format!("missing default in {s}"))?;
        Ok(Workflow {
            name,
            rules,
            default: default.parse()?,
        })
    }
}

impl Workflow {
    fn process(&self, rating: &Rating) -> Result<&Destination, Box<dyn Error>> {
        let destinations = self.rules.iter().map(|rule| {
            let value = *rating
                .0
                .get(&rule.name)
                .ok_or_else(|| format!("name not found: {}", rule.name))?;
            Ok::<_, Box<dyn Error>>(match rule.rule_type {
                RuleType::Less => (value < rule.number).then_some(&rule.destination),
                RuleType::Greater => (value > rule.number).then_some(&rule.destination),
            })
        });

        for destination in destinations {
            if let Some(destination) = destination? {
                return Ok(destination);
            }
        }
        Ok(&self.default)
    }
}

#[derive(Debug)]
struct Rating(FnvHashMap<char, usize>);

impl FromStr for Rating {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s[1..s.len() - 1]
                .split(',')
                .map(|s| {
                    let mut parts = s.split('=');
                    let name = parts.next().ok_or_else(|| format!("missing name in {s}"))?.to_string();
                    if name.len() > 1 {
                        Err(format!("only single letter criteria supported ( {s} )"))?;
                    }
                    let name = name
                        .chars()
                        .next()
                        .ok_or_else(|| format!("one character required {s}"))?;
                    let number = parts.next().ok_or_else(|| format!("missing number in {s}"))?.parse()?;
                    Ok::<_, Box<dyn Error>>((name, number))
                })
                .collect::<Result<_, _>>()?,
        ))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut input = stdin().lock().lines();
    let mut workflows = FnvHashMap::default();

    for line in input.by_ref() {
        let line = line?;
        if line.is_empty() {
            break;
        }
        let workflow = Workflow::from_str(line.as_str())?;
        workflows.insert(workflow.name.clone(), workflow);
    }

    let workflow_in = workflows.get("in").ok_or("workflow 'in' missing")?;

    let ratings = input
        .filter_map(|line| {
            let rating: Rating = match line.map_err(Box::from).and_then(|s| s.parse().map_err(Box::from)) {
                Ok(rating) => rating,
                Err(err) => return Some(Err(err)),
            };
            let mut workflow = workflow_in;

            loop {
                let destination = match workflow.process(&rating) {
                    Ok(destination) => destination,
                    Err(err) => return Some(Err(err)),
                };

                workflow = match destination {
                    Destination::WorkflowResult(WorkflowResult::Accepted) => return Some(Ok(rating)),
                    Destination::WorkflowResult(WorkflowResult::Rejected) => return None,
                    Destination::Routed(destination) => match workflows.get(destination) {
                        None => return Some(Err(format!("no such destination {destination}").into())),
                        Some(workflow) => workflow,
                    },
                };
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    let sum = ratings
        .into_iter()
        .map(|rating| {
            ['x', 'm', 'a', 's']
                .into_iter()
                .map(|c| rating.0.get(&c).ok_or_else(|| format!("rating for {c} not found")))
                .sum::<Result<usize, _>>()
        })
        .sum::<Result<usize, _>>()?;

    println!("{sum}");

    Ok::<_, Box<dyn Error>>(())
}
