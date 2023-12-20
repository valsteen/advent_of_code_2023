use fnv::FnvHashMap;
use num_integer::gcd;
use sort_by_derive::EnumAccessor;
use std::{
    cell::RefCell,
    collections::VecDeque,
    error::Error,
    fmt,
    fmt::{Debug, Display},
    io::{stdin, BufRead},
    iter::repeat_with,
    rc::Rc,
};

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

struct Conjunction<O> {
    name: Rc<str>,
    destinations: Vec<Rc<str>>,
    states: FnvHashMap<Rc<str>, bool>,
    observer: O,
}

struct Done;

impl Debug for Done {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt("all cycles have been found", f)
    }
}

impl Display for Done {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt("all cycles have been found", f)
    }
}

impl Error for Done {}

impl<O> Conjunction<O>
where
    O: Observer,
{
    fn pulse(
        &mut self,
        pulse: bool,
        origin: &Rc<str>,
        counter: usize,
    ) -> Result<impl Iterator<Item = (Rc<str>, Rc<str>, bool)> + '_, Box<dyn Error>> {
        let state = self
            .states
            .get_mut(origin)
            .ok_or_else(|| format!("state {origin} was not found"))?;

        if *state != pulse && self.observer.observe(counter, &self.name, origin, pulse) {
            return Err(Done)?;
        }

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
#[accessor(destinations: Vec<Rc<str>>, except(Output))]
enum Module<O> {
    Broadcaster(Broadcaster),
    FlipFlop(FlipFlop),
    Conjunction(Conjunction<O>),
    Output(Output),
}

struct Output {
    name: Rc<str>,
}

struct State<O> {
    modules: FnvHashMap<Rc<str>, Module<O>>,
}

impl<O> State<O>
where
    O: Observer,
{
    fn press(&mut self, counter: &mut usize) -> Result<(), Box<dyn Error>> {
        let mut pulses = VecDeque::from([("".into(), "broadcaster".into(), false)]);

        while let Some((origin, destination, pulse)) = pulses.pop_front() {
            let destination = self
                .modules
                .get_mut(&destination)
                .ok_or_else(|| format!("no such destination: {destination}"))?;

            match destination {
                Module::Broadcaster(broadcaster) => pulses.extend(broadcaster.pulse(pulse)),
                Module::FlipFlop(flip_flop) => pulses.extend(flip_flop.pulse(pulse).into_iter().flatten()),
                Module::Conjunction(conjunction) => {
                    pulses.extend(conjunction.pulse(pulse, &origin, *counter)?);
                }
                Module::Output(_) => (),
            }
        }

        Ok(())
    }
}

enum ModuleType {
    Broadcaster,
    FlipFlop,
    Conjunction,
}

impl Module<()> {
    #[allow(clippy::type_complexity)]
    fn parse(s: &str) -> Result<(ModuleType, Rc<str>, Vec<Rc<str>>), Box<dyn Error>> {
        let mut parts = s.split([' ', ',']).filter(|s| !s.is_empty());
        let type_name = parts.next().ok_or_else(|| format!("type and name not found {s}"))?;
        parts.next();
        let destinations = parts.map(<&str>::into).collect::<Vec<_>>();

        let (module_type, name) = match (&type_name[0..1], &type_name[1..]) {
            ("b", "roadcaster") => (ModuleType::Broadcaster, "broadcaster".into()),
            ("%", name) => (ModuleType::FlipFlop, name.into()),
            ("&", name) => (ModuleType::Conjunction, name.into()),
            _ => Err(format!("invalid module type in {type_name}"))?,
        };

        Ok((module_type, name, destinations))
    }
}

impl<O> Module<O> {
    fn new(module_type: &ModuleType, name: Rc<str>, destinations: Vec<Rc<str>>, callback: O) -> Module<O> {
        match module_type {
            ModuleType::Broadcaster => Module::Broadcaster(Broadcaster { name, destinations }),
            ModuleType::FlipFlop => Module::FlipFlop(FlipFlop {
                name,
                destinations,
                state: false,
            }),
            ModuleType::Conjunction => Module::Conjunction(Conjunction {
                name,
                states: FnvHashMap::default(),
                destinations,
                observer: callback,
            }),
        }
    }
}

impl<O> State<O> {
    fn new(mut modules: FnvHashMap<Rc<str>, Module<O>>) -> Self {
        let sources = modules
            .iter()
            .filter_map(|(name, module)| {
                module
                    .destinations()
                    .map(|destinations| (name.clone(), destinations.clone()))
            })
            .collect::<FnvHashMap<_, _>>();

        for destination in sources.values().flatten() {
            if !modules.contains_key(destination) {
                modules.insert(
                    destination.clone(),
                    Module::Output(Output {
                        name: destination.clone(),
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

trait Observer {
    fn observe(&self, tick: usize, module_name: &str, state_name: &str, pulse: bool) -> bool;
}

struct CycleObserver {
    module_name: Rc<str>,
    origins: Vec<Rc<str>>,
    cycle: Vec<Option<usize>>,
}

impl CycleObserver {
    fn record(&mut self, press_counter: usize, module_name: &str, origin: &str, pulse: bool) -> bool {
        if module_name != self.module_name.as_ref() {
            return false;
        }
        let input = self.origins.iter().position(|n| n.as_ref() == origin).unwrap();

        if pulse {
            let cycle = self.cycle.get_mut(input).unwrap();

            if cycle.is_none() {
                *cycle = Some(press_counter);
            }
        }

        self.cycle.iter().all(Option::is_some)
    }

    fn new(module_name: Rc<str>) -> Self {
        Self {
            module_name,
            origins: vec![],
            cycle: vec![],
        }
    }
    fn add_state(&mut self, name: &Rc<str>, _state: bool) {
        self.cycle.push(None);
        self.origins.push(name.clone());
    }
}

impl Observer for &RefCell<CycleObserver> {
    fn observe(&self, tick: usize, module_name: &str, origin: &str, pulse: bool) -> bool {
        self.borrow_mut().record(tick, module_name, origin, pulse)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = stdin().lock().lines();

    let module_types = input
        .map(|line| Module::parse(line?.as_str()))
        .collect::<Result<Vec<_>, _>>()?;

    let conjunction_name = if let Some((_, name, _destinations)) = module_types
        .iter()
        .find(|(_, _, destinations)| destinations.iter().any(|d| d.as_ref() == "rx"))
    {
        name.clone()
    } else {
        return Err("cannot find rx module".into());
    };

    let time_series = CycleObserver::new(conjunction_name.clone());

    let cycles = RefCell::new(time_series);

    let mut state = State::new(
        module_types
            .into_iter()
            .map(|(module_type, name, destinations)| {
                let module = Module::new(&module_type, name, destinations, &cycles);
                Ok::<_, Box<dyn Error>>((module.name().clone(), module))
            })
            .collect::<Result<FnvHashMap<Rc<str>, _>, Box<dyn Error>>>()?,
    );

    {
        let mut time_series = cycles.borrow_mut();
        let Some(Module::Conjunction(conjunction)) = state.modules.get(&conjunction_name) else {
            return Err("module to observe not found".into());
        };
        for (name, state) in &conjunction.states {
            time_series.add_state(name, *state);
        }
    }

    let mut counter = 0;

    match repeat_with(|| {
        counter += 1;
        state.press(&mut counter)
    })
    .try_fold((), |(), result| {
        result?;
        Ok::<_, Box<dyn Error>>(())
    }) {
        Ok(()) => unreachable!(),
        Err(e) if e.is::<Done>() => Ok::<_, Box<dyn Error>>(()),
        Err(e) => return Err(e),
    }?;

    let cycles = cycles.into_inner();

    let result = cycles.cycle.into_iter().try_fold(1, |a, b| {
        let b = b.ok_or("not all cycles were detected")?;
        Ok::<_, Box<dyn Error>>(a * b / gcd(a, b))
    })?;

    println!("{result}");

    Ok::<_, Box<dyn Error>>(())
}
