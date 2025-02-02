use bevy::prelude::{Entity, Resource};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Automaton {
    entity: Entity,
    life: isize,
}

impl Automaton {
    pub fn new(entity: Entity, life: isize) -> Self {
        Self { entity, life }
    }
    pub fn new_zero(entity: Entity) -> Self {
        Self {
            entity,
            life: 0isize,
        }
    }
    pub fn change_life(&mut self, step: isize) {
        self.life += step;
    }
    pub fn life(&self) -> isize {
        self.life
    }
    pub fn entity(&self) -> Entity {
        self.entity
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct SystemDims {
    x: usize,
    y: usize,
    z: usize,
}

impl SystemDims {
    pub fn new_from_tuple(dims: (usize, usize, usize)) -> Self {
        Self {
            x: dims.0,
            y: dims.1,
            z: dims.2,
        }
    }
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }
    pub fn new_cube_clamped(min: usize, max: usize, value: usize) -> Self {
        let l = value.max(min).min(max);
        Self { x: l, y: l, z: l }
    }
    pub fn x(&self) -> usize {
        self.x
    }
    pub fn y(&self) -> usize {
        self.y
    }
    pub fn z(&self) -> usize {
        self.z
    }
    pub fn range_x(&self) -> std::ops::Range<usize> {
        create_range(self.x)
    }
    pub fn range_y(&self) -> std::ops::Range<usize> {
        create_range(self.y)
    }
    pub fn range_z(&self) -> std::ops::Range<usize> {
        create_range(self.z)
    }
    pub fn core_range_x(&self, fract: usize) -> std::ops::Range<usize> {
        fract_range(self.x, fract)
    }
    pub fn core_range_y(&self, fract: usize) -> std::ops::Range<usize> {
        fract_range(self.y, fract)
    }
    pub fn core_range_z(&self, fract: usize) -> std::ops::Range<usize> {
        fract_range(self.z, fract)
    }
    pub fn max_amount(&self) -> usize {
        self.x * self.y * self.z
    }
}

fn create_range(x: usize) -> std::ops::Range<usize> {
    std::ops::Range::<usize> {
        start: 0usize,
        end: x,
    }
}

fn fract_range(x: usize, fract: usize) -> std::ops::Range<usize> {
    let part = x / fract + 1;
    let start = x / 2 - part / 2;
    let end = start + part;
    std::ops::Range::<usize> { start, end }
}

// contains a three dimensional vector of Option<Automaton>
// the minimum number of possible states is two:
// Some(Automaton) and None
#[derive(Debug, Clone, Resource)]
pub struct AutoSystem3d {
    data: Vec<Vec<Vec<Option<Automaton>>>>,
}
// defines how to deal with the borders of the system
// trying to access position -1,0,64 in a 64x64x64
// system will result in accessing position 63,0,0
// so we actually jump back to the opposite wall
fn rem_euclid_3d(ixyz: (isize, isize, isize), dims: &SystemDims) -> (usize, usize, usize) {
    (
        ixyz.0.rem_euclid(dims.x as isize) as usize,
        ixyz.1.rem_euclid(dims.y as isize) as usize,
        ixyz.2.rem_euclid(dims.z as isize) as usize,
    )
}

impl AutoSystem3d {
    pub fn new_from_dims(dims: &SystemDims) -> Self {
        Self {
            data: vec![vec![vec![Option::<Automaton>::None; dims.z]; dims.y]; dims.x],
        }
    }
    pub fn access_xyz(&mut self, xyz: (usize, usize, usize), value: Option<Automaton>) {
        self.data[xyz.0][xyz.1][xyz.2] = value;
    }
    pub fn set_xyz(&mut self, xyz: (usize, usize, usize), value: Automaton) {
        self.data[xyz.0][xyz.1][xyz.2] = Some(value);
    }
    pub fn delete_xyz(&mut self, xyz: (usize, usize, usize)) {
        self.data[xyz.0][xyz.1][xyz.2] = Option::<Automaton>::None;
    }
    pub fn rem_euclid_bool(&self, xyz: (isize, isize, isize), dims: &SystemDims) -> bool {
        let (x, y, z) = rem_euclid_3d(xyz, dims);
        self.data[x][y][z].is_some()
    }
    pub fn count_neighbours_moore(&self, uxyz: (usize, usize, usize), dims: &SystemDims) -> usize {
        let mut count = 0usize;
        for ixyz in neighbours_moore_3d(uxyz) {
            if self.rem_euclid_bool(ixyz, dims) {
                count += 1;
            }
        }
        count
    }
    pub fn count_neighbours_von_neumann(
        &self,
        uxyz: (usize, usize, usize),
        dims: &SystemDims,
    ) -> usize {
        let mut count = 0usize;
        for ixyz in neighbours_von_neumann_3d(uxyz) {
            if self.rem_euclid_bool(ixyz, dims) {
                count += 1;
            }
        }
        count
    }
    pub fn get_at_xyz(&self, uxyz: (usize, usize, usize)) -> Option<Automaton> {
        self.data[uxyz.0][uxyz.1][uxyz.2]
    }
    pub fn apply_changes(&mut self, changes: &Vec<SysChange>) {
        for ele in changes {
            self.data[ele.x()][ele.y()][ele.z()] = ele.element();
        }
    }
    pub fn debug(&self) {
        eprintln!("{:?}", &self);
    }
}

// I've decided to just write all possible neighbours out
// nested loops would haven been probably much smarter, but
// honestly I'm not sure how far compiler optimisations go
fn neighbours_moore_3d(uxyz: (usize, usize, usize)) -> [(isize, isize, isize); 26] {
    let xyz = (uxyz.0 as isize, uxyz.1 as isize, uxyz.2 as isize);
    [
        (xyz.0 - 1, xyz.1 - 1, xyz.2 - 1),
        (xyz.0 - 1, xyz.1 - 1, xyz.2),
        (xyz.0 - 1, xyz.1 - 1, xyz.2 + 1),
        (xyz.0 - 1, xyz.1, xyz.2 - 1),
        (xyz.0 - 1, xyz.1, xyz.2),
        (xyz.0 - 1, xyz.1, xyz.2 + 1),
        (xyz.0 - 1, xyz.1 + 1, xyz.2 - 1),
        (xyz.0 - 1, xyz.1 + 1, xyz.2),
        (xyz.0 - 1, xyz.1 + 1, xyz.2 + 1),
        (xyz.0, xyz.1 - 1, xyz.2 - 1),
        (xyz.0, xyz.1 - 1, xyz.2),
        (xyz.0, xyz.1 - 1, xyz.2 + 1),
        (xyz.0, xyz.1, xyz.2 - 1),
        (xyz.0, xyz.1, xyz.2 + 1),
        (xyz.0, xyz.1 + 1, xyz.2 - 1),
        (xyz.0, xyz.1 + 1, xyz.2),
        (xyz.0, xyz.1 + 1, xyz.2 + 1),
        (xyz.0 + 1, xyz.1 - 1, xyz.2 - 1),
        (xyz.0 + 1, xyz.1 - 1, xyz.2),
        (xyz.0 + 1, xyz.1 - 1, xyz.2 + 1),
        (xyz.0 + 1, xyz.1, xyz.2 - 1),
        (xyz.0 + 1, xyz.1, xyz.2),
        (xyz.0 + 1, xyz.1, xyz.2 + 1),
        (xyz.0 + 1, xyz.1 + 1, xyz.2 - 1),
        (xyz.0 + 1, xyz.1 + 1, xyz.2),
        (xyz.0 + 1, xyz.1 + 1, xyz.2 + 1),
    ]
}

fn neighbours_von_neumann_3d(uxyz: (usize, usize, usize)) -> [(isize, isize, isize); 6] {
    let xyz = (uxyz.0 as isize, uxyz.1 as isize, uxyz.2 as isize);
    [
        (xyz.0, xyz.1 - 1, xyz.2),
        (xyz.0, xyz.1 + 1, xyz.2),
        (xyz.0 - 1, xyz.1, xyz.2),
        (xyz.0 + 1, xyz.1, xyz.2),
        (xyz.0, xyz.1, xyz.2 - 1),
        (xyz.0, xyz.1, xyz.2 + 1),
    ]
}

// during each step we keep track of the changes
// the new state of the system is represented as
// a Vec<SysChange>, that can be applied to the AutoSystem3d
// resource
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SysChange {
    uxyz: (usize, usize, usize),
    element: Option<Automaton>,
}

impl SysChange {
    pub fn spawn(x: usize, y: usize, z: usize, automaton: Automaton) -> Self {
        Self {
            uxyz: (x, y, z),
            element: Some(automaton),
        }
    }
    pub fn empty(x: usize, y: usize, z: usize) -> Self {
        Self {
            uxyz: (x, y, z),
            element: Option::<Automaton>::None,
        }
    }
    pub fn change_life(
        x: usize,
        y: usize,
        z: usize,
        automaton: Automaton,
        life_change: isize,
    ) -> Self {
        Self {
            uxyz: (x, y, z),
            element: Some(Automaton::new(
                automaton.entity,
                automaton.life + life_change,
            )),
        }
    }
    pub fn x(&self) -> usize {
        self.uxyz.0
    }
    pub fn y(&self) -> usize {
        self.uxyz.1
    }
    pub fn z(&self) -> usize {
        self.uxyz.2
    }
    pub fn element(&self) -> Option<Automaton> {
        self.element
    }
}

// define rules for the 3d automata
// if despawn_lower < number of neighbours < despawn_upper
// then the cube will survive, else life will be reduced
// by one, if life is already zero, the cube will be despawned
// if spawn_lower < number of neighbours < spawn_upper
// and the spot is empty a new cube will spawn
// note that a life of 0 still results in two possible
// states: Some(Entity) and None
#[derive(Debug, Copy, Clone, Resource)]
pub struct Rules {
    spawn_lower: usize,
    spawn_upper: usize,
    despawn_lower: usize,
    despawn_upper: usize,
    life: isize,
}

impl Rules {
    pub fn new(
        spawn_lower: usize,
        spawn_upper: usize,
        despawn_lower: usize,
        despawn_upper: usize,
        life: isize,
    ) -> Self {
        Self {
            spawn_lower,
            spawn_upper,
            despawn_lower,
            despawn_upper,
            life,
        }
    }
    
    pub fn spawn_lower(&self) -> usize {
        self.spawn_lower
    }
    pub fn spawn_upper(&self) -> usize {
        self.spawn_upper
    }
    pub fn despawn_lower(&self) -> usize {
        self.despawn_lower
    }
    pub fn despawn_upper(&self) -> usize {
        self.despawn_upper
    }
    pub fn life(&self) -> isize {
        self.life
    }
}
// should correspond to '5-6/5-6/5/M' in standard notation
impl Default for Rules {
    fn default() -> Self {
        Self {
            spawn_lower: 4,
            spawn_upper: 7,
            despawn_lower: 5,
            despawn_upper: 6,
            life: 3,
        }
    }
}
