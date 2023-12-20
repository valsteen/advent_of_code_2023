use fnv::FnvHashMap;
use sort_by_derive::EnumAccessor;
use std::collections::VecDeque;
use std::error::Error;
use std::io::{stdin, BufRead};
use std::iter::repeat_with;
use std::rc::Rc;
use std::str::FromStr;

#[derive(Debug)]
struct Broadcaster {
    name: Rc<str>,
    destinations: Vec<Rc<str>>,
}

impl Broadcaster {
    fn pulse(&self, pulse: bool) -> impl Iterator<Item = (Rc<str>, Rc<str>, bool)> + '_ {
        self.destinations
            .iter()
            .map(move |destination| (self.name.clone(), destination.clone(), pulse))
    }
}

#[derive(Debug)]
struct FlipFlop {
    name: Rc<str>,
    destinations: Vec<Rc<str>>,
    state: bool,
}

impl FlipFlop {
    fn pulse(&mut self, pulse: bool) -> Option<impl Iterator<Item = (Rc<str>, Rc<str>, bool)> + '_> {
        if pulse {
            None
        } else {
            self.state = !self.state;
            Some(
                self.destinations
                    .iter()
                    .map(|destination| (self.name.clone(), destination.clone(), self.state)),
            )
        }
    }
}

#[derive(Debug)]
struct Conjunction {
    name: Rc<str>,
    destinations: Vec<Rc<str>>,
    states: FnvHashMap<Rc<str>, bool>,
}

impl Conjunction {
    fn pulse(
        &mut self,
        pulse: bool,
        origin: &Rc<str>,
    ) -> Result<impl Iterator<Item = (Rc<str>, Rc<str>, bool)> + '_, Box<dyn Error>> {
        let state = self
            .states
            .get_mut(origin)
            .ok_or_else(|| format!("state {origin} was not found"))?;
        *state = pulse;

        let output = !self.states.iter().all(|(_, state)| *state);

        let name = &self.name;
        Ok(self
            .destinations
            .iter()
            .cloned()
            .map(move |destination| (name.clone(), destination, output)))
    }
}

#[derive(EnumAccessor)]
#[accessor(name: Rc<str>)]
#[accessor(destinations: Vec<Rc<str>>)]
enum Module {
    Broadcaster(Broadcaster),
    FlipFlop(FlipFlop),
    Conjunction(Conjunction),
    Output(Output),
}

struct Output {
    name: Rc<str>,
    state: bool,
    destinations: Vec<Rc<str>>,
    high_pulses: usize,
    low_pulses: usize,
}

impl Output {
    fn pulse(&mut self, pulse: bool) {
        self.state = pulse;

        if pulse {
            self.high_pulses += 1;
        } else {
            self.low_pulses += 1;
        }
    }
}

struct State {
    modules: FnvHashMap<Rc<str>, Module>,
}

impl State {
    fn press(&mut self) -> Result<(usize, usize), Box<dyn Error>> {
        let mut pulses = VecDeque::from([("".into(), "broadcaster".into(), false)]);
        let mut high_counter = 0;
        let mut low_counter = 0;

        while let Some((origin, destination, pulse)) = pulses.pop_front() {
            if pulse {
                high_counter += 1;
            } else {
                low_counter += 1;
            }
            let destination = self
                .modules
                .get_mut(&destination)
                .ok_or_else(|| format!("no such destination: {destination}"))?;

            match destination {
                Module::Broadcaster(broadcaster) => pulses.extend(broadcaster.pulse(pulse)),
                Module::FlipFlop(flip_flop) => pulses.extend(flip_flop.pulse(pulse).into_iter().flatten()),
                Module::Conjunction(conjunction) => {
                    pulses.extend(conjunction.pulse(pulse, &origin)?);
                }
                Module::Output(output) => output.pulse(pulse),
            }
        }

        Ok((low_counter, high_counter))
    }
}

impl FromStr for Module {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split([' ', ',']).filter(|s| !s.is_empty());
        let type_name = parts.next().ok_or_else(|| format!("type and name not found {s}"))?;
        parts.next();
        let destinations = parts.map(<&str>::into).collect::<Vec<_>>();

        Ok(match (&type_name[0..1], &type_name[1..]) {
            ("b", "roadcaster") => Module::Broadcaster(Broadcaster {
                name: "broadcaster".into(),
                destinations,
            }),
            ("%", name) => Module::FlipFlop(FlipFlop {
                name: name.into(),
                destinations,
                state: false,
            }),
            ("&", name) => Module::Conjunction(Conjunction {
                name: name.into(),
                states: FnvHashMap::default(),
                destinations,
            }),
            _ => Err(format!("invalid module type in {s}"))?,
        })
    }
}

impl State {
    fn new(mut modules: FnvHashMap<Rc<str>, Module>) -> Self {
        let sources = modules
            .iter()
            .map(|(name, module)| (name.clone(), module.destinations().clone()))
            .collect::<FnvHashMap<_, _>>();

        for destination in sources.values().flatten() {
            if !modules.contains_key(destination) {
                modules.insert(
                    destination.clone(),
                    Module::Output(Output {
                        name: destination.clone(),
                        state: false,
                        destinations: vec![],
                        high_pulses: 0,
                        low_pulses: 0,
                    }),
                );
            }
        }

        for module in modules.values_mut() {
            if let Module::Conjunction(module) = module {
                module.states = sources
                    .iter()
                    .filter_map(|(name, destinations)| {
                        destinations
                            .iter()
                            .any(|destination_name| destination_name.as_ref() == module.name.as_ref())
                            .then_some((name.clone(), false))
                    })
                    .collect();
            }
        }

        Self { modules }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = stdin().lock().lines();
    let mut state = State::new(
        input
            .map(|line| {
                let module: Module = line?.parse()?;
                Ok::<_, Box<dyn Error>>((module.name().clone(), module))
            })
            .collect::<Result<FnvHashMap<Rc<str>, Module>, Box<dyn Error>>>()?,
    );

    let (low, high) = repeat_with(|| state.press())
        .take(1000)
        .try_fold((0, 0), |(low, high), result| {
            let (inc_low, inc_high) = result?;
            Ok::<_, Box<dyn Error>>((low + inc_low, high + inc_high))
        })?;

    println!("{}", low * high);

    Ok::<_, Box<dyn Error>>(())
}
