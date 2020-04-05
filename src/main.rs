use rand;
use std::io;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

// =======================================================================
// Defines
// =======================================================================
const MAP_NB_REGION: usize = 3;
const MAP_REGION_SIZE: usize = 5;
const MAP_SIZE: usize = MAP_NB_REGION * MAP_REGION_SIZE;
mod cooldown {
    pub const TORPEDO: usize = 3;
    pub const SONAR: usize = 4;
    pub const SILENCE: usize = 7;
    pub const MINE: usize = 3;
}
const MAX_LIFE: i32 = 6;

#[derive(Debug, Clone, PartialEq)]
enum Direction {
    N,
    E,
    S,
    W,
}
impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::N => 'N',
                Self::E => 'E',
                Self::S => 'S',
                Self::W => 'W',
            }
        )
    }
}
impl Direction {
    fn apply(&self, pos: &Pos) -> Result<Pos, ()> {
        match self {
            Self::N => {
                if pos.y == 0 {
                    Err(())
                } else {
                    Ok(Pos {
                        x: pos.x,
                        y: pos.y - 1,
                    })
                }
            }
            Self::S => Ok(Pos {
                x: pos.x,
                y: pos.y + 1,
            }),
            Self::W => {
                if pos.x == 0 {
                    Err(())
                } else {
                    Ok(Pos {
                        x: pos.x - 1,
                        y: pos.y,
                    })
                }
            }
            Self::E => Ok(Pos {
                x: pos.x + 1,
                y: pos.y,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum System {
    Torpedo,
    Sonar,
    Silence,
    Mine,
}
impl std::fmt::Display for System {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Torpedo => "TORPEDO",
                Self::Sonar => "SONAR",
                Self::Silence => "SILENCE",
                Self::Mine => "MINE",
            }
        )
    }
}

// =======================================================================
// Tools
// =======================================================================
struct WeightMap {
    h: usize,
    w: usize,
    data: Box<[bool]>,
}

impl WeightMap {}

// =======================================================================
// Game data
// =======================================================================
#[derive(Debug, Clone, PartialEq)]
struct Map {
    h: usize,
    w: usize,
    // true means blocked
    data: Box<[bool]>,
}
impl Map {
    fn new(h: usize, w: usize) -> Self {
        Self {
            data: vec![false; w * h].into_boxed_slice(),
            h,
            w,
        }
    }
    fn set(&mut self, y: usize, x: usize, value: bool) {
        self.data[y * self.w + x] = value;
    }
    fn get(&self, y: usize, x: usize) -> bool {
        self.data[y * self.w + x]
    }
    fn copy_from(&mut self, map: &Map) {
        self.data.copy_from_slice(&map.data);
    }

    fn rand_false_pos(&self) -> Result<Pos, ()> {
        if self.data.iter().all(|v| *v) {
            return Err(());
        }
        let mut n: usize = rand::random::<u16>() as usize % self.data.len();
        let mut i = 0;
        while n > 0 {
            i += 1;
            if i >= self.data.len() {
                i = 0;
            }
            if !self.data[i] {
                n -= 1;
            }
        }
        Ok(Pos {
            y: i / self.h,
            x: i % self.w,
        })
    }

    fn eq_and_not(&mut self, map: &Map) {
        for (v, forbidden) in self.data.iter_mut().zip(map.data.iter()) {
            *v = *v && !*forbidden;
        }
    }

    fn eq_and(&mut self, map: &Map) {
        for (v, allowed) in self.data.iter_mut().zip(map.data.iter()) {
            if !*allowed {
                *v = false;
            }
        }
    }

    fn eq_or(&mut self, map: &Map) {
        for (v, allowed) in self.data.iter_mut().zip(map.data.iter()) {
            *v = *v || *allowed;
        }
    }

    fn invert(&mut self) {
        self.data.iter_mut().for_each(|v| *v = !*v);
    }

    fn reset(&mut self) {
        self.data.iter_mut().for_each(|v| *v = false);
    }

    fn sub_false_area_size(&self, start: &Pos) -> usize {
        if self.get(start.y, start.x) {
            return 0;
        }
        let mut map = Map::new(self.h, self.w);
        map.set(start.y, start.x, true);

        loop {
            let mut changed = false;
            for y in 0..self.h {
                for x in 0..self.w {
                    if !self.get(y, x) && !map.get(y, x) {
                        let has_good_neigh = (x > 0 && map.get(y, x - 1))
                            || (x < self.w - 1 && map.get(y, x + 1))
                            || (y > 0 && map.get(y - 1, x))
                            || (y < self.h - 1 && map.get(y + 1, x));
                        if has_good_neigh {
                            map.set(y, x, true);
                            changed = true;
                        }
                    }
                }
            }

            if !changed {
                break;
            }
        }

        map.data
            .iter()
            .fold(0, |acc, i| if *i { acc + 1 } else { acc })
    }
}
#[test]
fn test_and() {
    let mut map_a = Map::new(3, 3);
    map_a.set(1, 1, true);
    map_a.set(0, 1, true);
    let mut map_b = Map::new(3, 3);
    map_b.set(0, 1, true);
    map_b.set(1, 2, true);
    let mut map_c = Map::new(3, 3);
    map_c.set(0, 1, true);
    let mut map_anb = map_a.clone();
    map_anb.eq_and(&map_b);
    assert_eq!(map_anb, map_c);
}
#[test]
fn test_and_not() {
    let mut map_a = Map::new(3, 3);
    map_a.set(1, 1, true);
    map_a.set(0, 1, true);
    let mut map_b = Map::new(3, 3);
    map_b.set(0, 1, true);
    map_b.set(1, 2, true);
    let mut map_c = Map::new(3, 3);
    map_c.set(1, 1, true);
    let mut map_anb = map_a.clone();
    map_anb.eq_and_not(&map_b);
    assert_eq!(map_anb, map_c);
}
#[test]
fn test_or() {
    let mut map_a = Map::new(3, 3);
    map_a.set(1, 1, true);
    map_a.set(0, 1, true);
    let mut map_b = Map::new(3, 3);
    map_b.set(0, 1, true);
    map_b.set(1, 2, true);
    let mut map_c = Map::new(3, 3);
    map_c.set(1, 1, true);
    map_c.set(0, 1, true);
    map_c.set(1, 2, true);
    let mut map_anb = map_a.clone();
    map_anb.eq_or(&map_b);
    assert_eq!(map_anb, map_c);
}
#[test]
fn test_sub_false_area_size_small() {
    let mut map = Map::new(3, 3);
    map.set(1, 0, true);
    assert_eq!(map.sub_false_area_size(&Pos { y: 0, x: 0 }), 8);
    assert_eq!(map.sub_false_area_size(&Pos { y: 1, x: 1 }), 8);
    map.set(1, 1, true);
    assert_eq!(map.sub_false_area_size(&Pos { y: 0, x: 0 }), 7);
    assert_eq!(map.sub_false_area_size(&Pos { y: 2, x: 1 }), 7);
    map.set(1, 2, true);
    assert_eq!(map.sub_false_area_size(&Pos { y: 0, x: 0 }), 3);
    assert_eq!(map.sub_false_area_size(&Pos { y: 2, x: 1 }), 3);
}
#[test]
fn test_sub_false_area_size() {
    let size = 15;
    let mut map_big = Map::new(size, size);
    for i in 0..size {
        map_big.set(size / 2, i, true);
    }
    assert_eq!(
        map_big.sub_false_area_size(&Pos { y: 0, x: 0 }),
        size / 2 * size
    );
    assert_eq!(
        map_big.sub_false_area_size(&Pos {
            y: size - 1,
            x: size - 1
        }),
        size / 2 * size
    );
    map_big.set(1, 1, true);
    map_big.set(1, 2, true);
    assert_eq!(
        map_big.sub_false_area_size(&Pos { y: 0, x: 0 }),
        size / 2 * size - 2
    );
}

impl std::fmt::Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ret = vec![];
        for y in 0..self.h {
            let mut tmp = vec![];
            for x in 0..self.w {
                if self.data[y * self.w + x] {
                    tmp.push('x');
                } else {
                    tmp.push('.');
                }
            }
            ret.push(tmp.into_iter().collect::<String>());
        }
        write!(f, "{}", ret.join("\n"))
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Pos {
    x: usize,
    y: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum PlayerPos {
    Exact(Pos),
    Area(Map),
}
#[derive(Debug, Clone, PartialEq)]
struct MePlayer {
    forbidden_map: Map,
    pos: Pos,
    life: i32,
    torpedo: usize,
    sonar: usize,
    silence: usize,
    mine: usize,
}

impl MePlayer {
    fn new(map: &Map) -> Self {
        let mut pos_map = map.clone();
        pos_map.invert();
        Self {
            forbidden_map: map.clone(),
            pos: Pos { y: 0, x: 0 },
            life: MAX_LIFE,
            torpedo: cooldown::TORPEDO,
            sonar: cooldown::SONAR,
            silence: cooldown::SILENCE,
            mine: cooldown::MINE,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct OppPlayer {
    life: i32,
    // TODO All those fields should be fuzzy values
    forbidden_map: Map,
    pos: PlayerPos,
    torpedo: usize,
    sonar: usize,
    silence: usize,
    mine: usize,
}

impl OppPlayer {
    fn new(map: &Map) -> Self {
        let mut pos_map = map.clone();
        pos_map.invert();
        Self {
            forbidden_map: map.clone(),
            pos: PlayerPos::Area(pos_map),
            life: MAX_LIFE,
            torpedo: cooldown::TORPEDO,
            sonar: cooldown::SONAR,
            silence: cooldown::SILENCE,
            mine: cooldown::MINE,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Action {
    mov: Option<(Direction, System)>,
    surface: bool,
    torpedo: Option<Pos>,
}
impl Action {
    fn new() -> Self {
        Self {
            mov: None,
            surface: false,
            torpedo: None,
        }
    }
}
impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ret = vec![];
        if let Some((dir, sys)) = &self.mov {
            ret.push(format!("MOVE {} {}", dir, sys));
        }
        if self.surface {
            ret.push("SURFACE".to_string());
        }
        if let Some(pos) = &self.torpedo {
            ret.push(format!("TORPEDO {} {}", pos.x, pos.y));
        }
        write!(f, "{}", ret.join(" | "))
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Game {
    // Static
    map: Map,
    my_id: usize,

    // Dynamic
    me: MePlayer,
    opp: OppPlayer,

    // Next action
    action: Action,
}

// =======================================================================
// Game rules
// =======================================================================
// Setup
impl Game {
    fn new(map: Map, my_id: usize) -> Self {
        Self {
            me: MePlayer::new(&map),
            opp: OppPlayer::new(&map),
            map,
            my_id,
            action: Action::new(),
        }
    }

    fn start_at(&self, y: usize, x: usize) {
        println!("{} {}", x, y);
    }
}

// Gameplay
impl Game {
    fn sync(&mut self) {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(' ').collect::<Vec<_>>();
        let x = parse_input!(inputs[0], usize);
        let y = parse_input!(inputs[1], usize);
        self.me.pos = Pos { y, x };
        self.me.life = parse_input!(inputs[2], i32);
        self.opp.life = parse_input!(inputs[3], i32);
        self.me.torpedo = parse_input!(inputs[4], usize);
        // self.me.sonar = parse_input!(inputs[5], usize);
        // self.me.silence = parse_input!(inputs[6], usize);
        // self.me.mine = parse_input!(inputs[7], usize);

        // Update path map
        self.me.forbidden_map.set(y, x, true);

        // TODO
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let sonar_result = input_line.trim().to_string();
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let opponent_orders = input_line.trim_end().to_string();

        self.action = Action::new();
    }

    fn can_move_to(&mut self, direction: &Direction) -> Result<(), ()> {
        let me_pos = &self.me.pos;
        let next_pos = direction.apply(me_pos)?;
        if next_pos.x >= self.map.w || next_pos.y >= self.map.h {
            return Err(());
        }
        if self.me.forbidden_map.get(next_pos.y, next_pos.x) {
            return Err(());
        }
        Ok(())
    }

    fn move_to(&mut self, direction: &Direction, system: &System) -> Result<(), ()> {
        self.can_move_to(direction)?;
        assert!(self.action.mov.is_none());
        self.action.mov = Some((direction.clone(), system.clone()));
        Ok(())
    }

    fn surface(&mut self) {
        self.action.surface = true;
    }

    fn torpedo(&mut self, pos: Pos) -> Result<(), ()> {
        if self.me.torpedo > 0 {
            return Err(());
        }
        assert!(self.action.torpedo.is_none());
        self.action.torpedo = Some(pos);
        Ok(())
    }

    fn commit(&mut self) {
        println!("{}", self.action);

        if self.action.surface {
            self.me.forbidden_map.copy_from(&self.map);
        }
    }
}

// =======================================================================
// IA
// =======================================================================
struct Ai {
    dir: Option<Direction>,
}

impl Ai {
    fn new() -> Self {
        Self { dir: None }
    }
}

impl Ai {
    fn select_start_point(&mut self, game: &mut Game) {
        let start_pos = game.map.rand_false_pos().unwrap();
        game.start_at(start_pos.y, start_pos.x);
    }
}
impl Ai {
    fn get_best_dir(&mut self, game: &mut Game, dirs: &[Direction]) -> Direction {
        let (best_index, _) = dirs
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let next_pos = d.apply(&game.me.pos).unwrap();
                let score = game.me.forbidden_map.sub_false_area_size(&next_pos);
                eprintln!("DIR {}, next_pos {:?}, score {}", d, next_pos, score);
                (i, score)
            })
            .max_by(|(_, max), (_, v)| max.cmp(v))
            .unwrap();
        eprintln!("{:?}, {}", dirs, best_index);
        dirs[best_index].clone()
    }

    fn plan_move(&mut self, game: &mut Game, recharge: &System) {
        let dirs = vec![Direction::E, Direction::N, Direction::W, Direction::S];
        let good_dirs: Vec<_> = dirs
            .into_iter()
            .filter(|d| game.can_move_to(d).is_ok())
            .collect();
        eprintln!("Possible directions: {:?}", good_dirs);

        let dir = match good_dirs.len() {
            0 => {
                self.dir = None;
                return game.surface();
            }
            1 => good_dirs[0].clone(),
            2 => self.get_best_dir(game, &good_dirs[..]),
            _ => {
                if let Some(dir) = &self.dir {
                    if good_dirs.contains(&dir) {
                        dir.clone()
                    } else {
                        good_dirs[0].clone()
                    }
                } else {
                    good_dirs[0].clone()
                }
            }
        };
        game.move_to(&dir, recharge).unwrap();
        self.dir = Some(dir);
    }

    fn plan_actions(&mut self, game: &mut Game) {
        self.plan_move(game, &System::Torpedo);

        // Manage shooting
        if game.me.torpedo == 0 {}
    }
}

fn main() {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let inputs = input_line.split(' ').collect::<Vec<_>>();
    let width = parse_input!(inputs[0], usize);
    let height = parse_input!(inputs[1], usize);
    let my_id = parse_input!(inputs[2], usize);
    let mut map = Map::new(height, width);
    for y in 0..height as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let line = input_line.trim_end().to_string();
        for (x, c) in line.chars().enumerate() {
            map.set(
                y,
                x,
                match c {
                    '.' => false,
                    'x' => true,
                    c => panic!("{}", c),
                },
            )
        }
    }
    let mut game = Game::new(map, my_id);
    let mut ai = Ai::new();

    // Choose position
    ai.select_start_point(&mut game);

    // game loop
    loop {
        game.sync();

        ai.plan_actions(&mut game);

        game.commit();
    }
}
