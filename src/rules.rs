use bevy::prelude::Resource;
use std::fmt;

#[derive(Debug, Clone, Copy, Resource)]
pub enum Neighbourhood {
    Moore,
    VonNeumann,
}

impl Neighbourhood {
    pub fn parse_from_str(input: &str) -> Self {
        match input {
            "VN" | "vn" => Self::VonNeumann,
            _ => Self::Moore,
        }
    }
}

impl fmt::Display for Neighbourhood {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Neighbourhood::Moore => write!(f, "Moore"),
            Neighbourhood::VonNeumann => write!(f, "Von Neumann"),
        }
    }
}

#[derive(Debug, Clone, Resource)]
pub struct Rules {
    survive: Vec<(usize, usize)>,
    spawn: Vec<(usize, usize)>,
    life: isize,
    neighbourhood: Neighbourhood,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            survive: vec![(5, 10)],
            spawn: vec![(8, 8)],
            life: 5,
            neighbourhood: Neighbourhood::Moore,
        }
    }
}

impl Rules {
    pub fn parse_from_str(input: &str) -> Option<Self> {
        let x: Vec<_> = input.split('/').collect();
        if let (Some(first), Some(second), Some(third), Some(fourth)) =
            (x.get(0), x.get(1), x.get(2), x.get(3))
        {
            if let Some(life) = third.parse::<isize>().ok() {
                if life > 1 {
                    Some(Self {
                        survive: parse_condis(first),
                        spawn: parse_condis(second),
                        // subtracting 2 because we already start with two states: Some and None
                        // the value provided here resembles additional states
                        life: life - 2,
                        neighbourhood: Neighbourhood::parse_from_str(fourth),
                    })
                } else {
                    eprintln!("there must be at least 2 states");
                    None
                }
            } else {
                eprintln!("failed to parse number of states");
                None
            }
        } else {
            eprintln!("even the basic parsing failed");
            None
        }
    }

    pub fn check_despawn(&self, n: usize) -> bool {
        check_exclusive(n, &self.survive)
    }

    pub fn check_spawn(&self, n: usize) -> bool {
        check_inclusive(n, &self.spawn)
    }

    pub fn life(&self) -> isize {
        self.life
    }

    pub fn neighbourhood(&self) -> Neighbourhood {
        self.neighbourhood
    }

    pub fn default_warn() -> Self {
        eprintln!("WARNING: Parsing the rules failed, using default values.");
        Self::default()
    }
}

impl fmt::Display for Rules {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Survival:")?;
        for rule in &self.survive {
            write!(f, " {}-{}", rule.0, rule.1)?;
        }
        write!(f, "\nSpawn:")?;
        for rule in &self.spawn {
            write!(f, " {}-{}", rule.0, rule.1)?;
        }
        write!(f, "\nExtra life: {}", self.life)?;
        write!(f, "\nNeighbourhood: {}", self.neighbourhood)?;
        Ok(())
    }
}

pub fn parse_condis(input: &str) -> Vec<(usize, usize)> {
    let mut output = Vec::<(usize, usize)>::new();
    for part in input.split(',') {
        let x: Vec<_> = part.split('-').collect();
        if x.len() == 1 {
            match part.parse::<usize>() {
                Ok(a) => output.push((a, a)),
                _ => continue,
            }
        } else {
            match (x[0].parse::<usize>(), x[1].parse::<usize>()) {
                (Ok(a), Ok(b)) => output.push((a, b)),
                _ => continue,
            }
        }
    }
    output
}

fn check_exclusive(n: usize, condis: &[(usize, usize)]) -> bool {
    let mut b = true;
    for c in condis {
        b = b && ((n < c.0) || (n > c.1));
    }
    b
}

fn check_inclusive(n: usize, condis: &[(usize, usize)]) -> bool {
    let mut b = false;
    for c in condis {
        b = b || ((n >= c.0) && (n <= c.1));
    }
    b
}
