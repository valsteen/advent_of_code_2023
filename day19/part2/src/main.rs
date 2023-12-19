use ranges::Ranges;
use std::collections::Bound;
use std::error::Error;
use std::io::{stdin, BufRead};
use std::ops::RangeBounds;
use std::str::FromStr;

#[derive(Debug)]
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
    Routed(usize),
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum WorkflowResult {
    Accepted,
    Rejected,
}

impl Destination {
    fn from_str(s: &str, names: &mut Vec<String>) -> Self {
        match s {
            "A" => Self::WorkflowResult(WorkflowResult::Accepted),
            "R" => Self::WorkflowResult(WorkflowResult::Rejected),
            _ => Self::Routed(names.iter().position(|name| name == s).unwrap_or_else(|| {
                names.push(s.to_string());
                names.len() - 1
            })),
        }
    }
}

#[derive(Debug, Clone)]
struct Rule {
    xmas: usize,
    range: Ranges<usize>,
    destination: Destination,
}

#[derive(Debug, Clone)]
struct Workflow {
    rules: Vec<Rule>,
}

enum ParsedRule {
    Rule(Rule),
    Default(Destination),
}

impl ParsedRule {
    fn from_str(s: &str, names: &mut Vec<String>) -> Result<Self, Box<dyn Error>> {
        let mut parts = s.split(':');

        let condition = parts.next().ok_or_else(|| format!("missing condition in {s}"))?;

        let Some(destination) = parts.next().map(str::to_string) else {
            return Ok(ParsedRule::Default(Destination::from_str(s, names)));
        };

        let part = condition.chars().next().ok_or_else(|| format!("missing part in {s}"))?;
        let xmas = "xmas"
            .find(part)
            .ok_or_else(|| format!("'{part} is not part of 'xmas'"))?;
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

        let range = match rule_type {
            RuleType::Less => usize::MIN..number,
            RuleType::Greater => (number + 1)..usize::MAX,
        }
        .into();

        Ok(ParsedRule::Rule(Rule {
            xmas,
            range,
            destination: Destination::from_str(destination.as_str(), names),
        }))
    }
}

impl Workflow {
    fn from_str(s: &str, names: &mut Vec<String>) -> Result<(String, Self), Box<dyn Error>> {
        let mut parts = s[..s.len() - 1].split('{');
        let name = parts
            .next()
            .ok_or_else(|| format!("no workflow name found in {s}"))?
            .to_string();

        let rules_part = parts
            .next()
            .ok_or_else(|| format!("no workflow rules found in {s}"))?
            .split(',');

        let rules = rules_part
            .map(|rule| match ParsedRule::from_str(rule, names)? {
                ParsedRule::Rule(rule) => Ok::<_, Box<dyn Error>>(rule),
                ParsedRule::Default(default) => Ok(Rule {
                    xmas: 0,
                    range: (0..usize::MAX).into(),
                    destination: default,
                }),
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok((name, Workflow { rules }))
    }

    fn process_range(&self, mut ranges: [Ranges<usize>; 4], workflows: &Vec<Workflow>) -> Vec<[Ranges<usize>; 4]> {
        let mut results = Vec::new();

        for rule in &self.rules {
            let mut intersection = ranges.clone();
            intersection[rule.xmas] &= rule.range.clone();

            match rule.destination {
                Destination::WorkflowResult(WorkflowResult::Rejected) => (),
                Destination::WorkflowResult(WorkflowResult::Accepted) => results.push(intersection),
                Destination::Routed(destination) => {
                    results.extend(workflows[destination].process_range(intersection, workflows));
                }
            };
            ranges[rule.xmas] -= rule.range.clone();
        }

        results
    }
}

#[derive(Debug)]
struct Rating([usize; 4]);

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
                    let number = parts.next().ok_or_else(|| format!("missing number in {s}"))?.parse()?;
                    Ok::<_, Box<dyn Error>>(number)
                })
                .collect::<Result<Vec<_>, _>>()?
                .try_into()
                .map_err(|_| "unable to gather xmas scores")?,
        ))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut input = stdin().lock().lines();
    let mut workflows = Vec::new();
    let mut names = Vec::new();

    for line in input.by_ref() {
        let line = line?;
        if line.is_empty() {
            break;
        }
        let (name, workflow) = Workflow::from_str(line.as_str(), &mut names)?;

        if !names.iter().any(|s| s == &name) {
            names.push(name.clone());
        }
        workflows.push((name, workflow));
    }

    let workflows = names
        .iter()
        .map(|name| {
            workflows
                .iter()
                .find_map(|(target, workflow)| (name == target).then_some(workflow))
                .cloned()
                .ok_or_else(|| format!("cannot find workflow {name}"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let workflow_in = names
        .iter()
        .position(|name| name == "in")
        .ok_or("workflow 'in' missing")?;

    let result = workflows[workflow_in].process_range(
        [
            (1..=4000).into(),
            (1..=4000).into(),
            (1..=4000).into(),
            (1..=4000).into(),
        ],
        &workflows,
    );

    let sum = result
        .into_iter()
        .map(|ranges| {
            ranges
                .map(|ranges| {
                    ranges
                        .as_slice()
                        .iter()
                        .map(|range| {
                            let start = match range.start_bound() {
                                Bound::Included(x) => *x,
                                Bound::Excluded(x) => *x + 1,
                                Bound::Unbounded => unreachable!(),
                            };
                            let end = match range.end_bound() {
                                Bound::Included(x) => *x + 1,
                                Bound::Excluded(x) => *x,
                                Bound::Unbounded => unreachable!(),
                            };
                            end - start
                        })
                        .sum::<usize>()
                })
                .iter()
                .product::<usize>()
        })
        .sum::<usize>();

    println!("{sum}");

    Ok::<_, Box<dyn Error>>(())
}
