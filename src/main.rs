use rand;
use std::io;

// =======================================================================
// Macros
// =======================================================================
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

// -----------------------------------------------------------------------
// Direction
// -----------------------------------------------------------------------
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
impl Direction {
    fn parse(s: &str) -> Self {
        match s {
            "N" => Self::N,
            "E" => Self::E,
            "S" => Self::S,
            "W" => Self::W,
            x => panic!("{}", x),
        }
    }
}

// -----------------------------------------------------------------------
// System
// -----------------------------------------------------------------------
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
impl System {
    fn parse(s: &str) -> Self {
        match s {
            "TORPEDO" => Self::Torpedo,
            "SONAR" => Self::Sonar,
            "SILENCE" => Self::Silence,
            "MINE" => Self::Mine,
            s => panic!("{}", s),
        }
    }
}

// =======================================================================
// Tools
// =======================================================================
// -----------------------------------------------------------------------
// Map
// -----------------------------------------------------------------------
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

    fn eq_shift(&mut self, dir: &Direction) {
        match dir {
            Direction::N => {
                for i in self.w..self.data.len() {
                    self.data[i - self.w] = self.data[i];
                }
                for i in (self.data.len() - self.w)..self.data.len() {
                    self.data[i] = false;
                }
            }
            Direction::S => {
                for i in (0..self.data.len() - self.w).rev() {
                    self.data[i + self.w] = self.data[i];
                }
                for i in 0..self.w {
                    self.data[i] = false;
                }
            }
            Direction::E => {
                for y in 0..self.h {
                    for x in (0..self.w - 1).rev() {
                        self.data[y * self.w + x + 1] = self.data[y * self.w + x];
                    }
                    self.data[y * self.w] = false;
                }
            }
            Direction::W => {
                for y in 0..self.h {
                    for x in 0..self.w - 1 {
                        self.data[y * self.w + x] = self.data[y * self.w + x + 1];
                    }
                    self.data[y * self.w + self.w - 1] = false;
                }
            }
        }
    }

    fn sector(&self, pos: &Pos) -> usize {
        let sector_w = self.w / 3;
        let sector_h = self.h / 3;
        let sector_x = pos.x / sector_w;
        let sector_y = pos.y / sector_h;
        sector_y * 3 + sector_x + 1
    }

    fn sector_mask(&self, sector: usize) -> Self {
        let mut map = Self::new(self.h, self.w);
        let sector_w = self.w / 3;
        let sector_h = self.h / 3;
        let sector_x = (sector - 1) % 3;
        let sector_y = (sector - 1) / 3;
        for y in sector_y * sector_h..(sector_y + 1) * sector_h {
            for x in sector_x * sector_w..(sector_x + 1) * sector_w {
                map.set(y, x, true);
            }
        }
        map
    }

    fn torpedo_mask(&self, pos: &Pos) -> Self {
        let mut map = Self::new(self.h, self.w);
        for y in 0..map.h as isize {
            for x in 0..map.w as isize {
                if dist1(pos.y as isize - y, pos.x as isize - x) < 4 {
                    map.set(y as usize, x as usize, true);
                }
            }
        }
        map
    }

    fn expand(&mut self, size: usize) {
        let mut map = self.clone();
        for i in 0..size {
            let mut tmp = map.clone();
            for y in 0..self.h {
                for x in 0..self.w {
                    let has_good_neigh = (x > 0 && map.get(y, x - 1))
                        || (x < self.w - 1 && map.get(y, x + 1))
                        || (y > 0 && map.get(y - 1, x))
                        || (y < self.h - 1 && map.get(y + 1, x));
                    if has_good_neigh {
                        tmp.set(y, x, true);
                    }
                }
            }
            map = tmp;
        }
        self.data.copy_from_slice(&map.data);
    }

    fn square(&mut self, pos: &Pos) -> Map {
        let min_x = if pos.x > 0 { pos.x - 1 } else { 0 };
        let min_y = if pos.y > 0 { pos.y - 1 } else { 0 };
        let max_x = if pos.x < self.w - 1 { pos.x + 1 } else { 0 };
        let max_y = if pos.y < self.h - 1 { pos.y + 1 } else { 0 };
        let mut map = Map::new(self.h, self.w);
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                map.set(y, x, true);
            }
        }
        map
    }

    fn nb_false(&self) -> usize {
        self.data.iter().map(|v| (!*v) as usize).sum()
    }

    fn first_false(&self) -> Option<Pos> {
        for (i, v) in self.data.iter().enumerate() {
            if !v {
                return Some(Pos {
                    y: i / self.w,
                    x: i % self.w,
                });
            }
        }
        None
    }
}
fn dist1(dy: isize, dx: isize) -> usize {
    dy.abs() as usize + dx.abs() as usize
}

// Tests -----------------------------------------------------------------
#[test]
fn test_shift() {
    let mut map_a = Map::new(3, 3);
    map_a.set(1, 1, true);
    let mut map_n = Map::new(3, 3);
    map_n.set(0, 1, true);

    map_a.eq_shift(&Direction::N);
    assert_eq!(map_a, map_n);

    let mut map_middle = Map::new(3, 3);
    map_middle.set(1, 1, true);

    map_a.eq_shift(&Direction::S);
    assert_eq!(map_a, map_middle);

    let mut map_e = Map::new(3, 3);
    map_e.set(1, 2, true);

    map_a.eq_shift(&Direction::E);
    assert_eq!(map_a, map_e);

    map_a.eq_shift(&Direction::W);
    assert_eq!(map_a, map_middle);
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

// =======================================================================
// Game data
// =======================================================================
// -----------------------------------------------------------------------
// Pos
// -----------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
struct Pos {
    x: usize,
    y: usize,
}
impl std::fmt::Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{};{}]", self.x, self.y)
    }
}

// -----------------------------------------------------------------------
// MePlayer
// -----------------------------------------------------------------------
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

// -----------------------------------------------------------------------
// OppPos
// -----------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
enum OppPos {
    Exact(Pos),
    Area(Map),
}
impl std::fmt::Display for OppPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OppPos::Exact(pos) => format!("Exact: {}", pos),
                OppPos::Area(map) => format!("Area:\n{}", map),
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
struct OppPlayer {
    life: i32,
    // TODO All those fields should be fuzzy values
    forbidden_map: Option<Map>,
    pos: OppPos,
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
            forbidden_map: None,
            pos: OppPos::Area(pos_map),
            life: MAX_LIFE,
            torpedo: cooldown::TORPEDO,
            sonar: cooldown::SONAR,
            silence: cooldown::SILENCE,
            mine: cooldown::MINE,
        }
    }
}

// -----------------------------------------------------------------------
// Action
// -----------------------------------------------------------------------
// TODO There should be a Me action and an Opp action because the parameter differ too much
#[derive(Debug, Clone, PartialEq)]
enum Action {
    Move(Direction, System),
    Surface(usize),
    Torpedo(Pos),
    Sonar(usize),
    Silence(Direction, usize),
}
impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Action::Move(dir, sys) => format!("MOVE {} {}", dir, sys),
                Action::Surface(_) => format!("SURFACE"),
                Action::Torpedo(Pos { y, x }) => format!("TORPEDO {} {}", x, y),
                Action::Sonar(sector) => format!("SONAR {}", sector),
                Action::Silence(dir, dist) => format!("SILENCE {} {}", dir, dist),
            }
        )
    }
}
impl Action {
    fn parse(s: &str) -> Self {
        eprintln!("XXX: Action parse: {}", s);
        let mut words: Vec<_> = s.split(' ').collect();
        let cmd = words.remove(0);
        match cmd {
            "MOVE" => Action::Move(Direction::parse(words[0]), System::Torpedo),
            "SURFACE" => Action::Surface(parse_input!(words[0], usize)),
            "TORPEDO" => Action::Torpedo(Pos {
                x: parse_input!(words[0], usize),
                y: parse_input!(words[1], usize),
            }),
            "SONAR" => Action::Sonar(parse_input!(words[0], usize)),
            "SILENCE" => Action::Silence(Direction::N, 0),
            x => panic!("{}", x),
        }
    }
}

fn parse_action_list(line: &str) -> Vec<Action> {
    let mut ret = vec![];
    for act_str in line.split('|') {
        ret.push(Action::parse(act_str));
    }
    ret
}

// -----------------------------------------------------------------------
// LastTurn
// -----------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
struct LastTurn {
    opp_life: i32,
    torpedo: Option<Pos>,
}

impl LastTurn {
    fn new() -> Self {
        Self {
            opp_life: MAX_LIFE,
            torpedo: None,
        }
    }
}

// -----------------------------------------------------------------------
// Game
// -----------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
struct Game {
    // Static
    map: Map,
    my_id: usize,

    // Dynamic
    me: MePlayer,
    opp: OppPlayer,

    // Next action
    actions: Vec<Action>,

    // Last turn state
    last_turn: LastTurn,
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
            actions: vec![],
            last_turn: LastTurn::new(),
        }
    }

    fn start_at(&self, y: usize, x: usize) {
        println!("{} {}", x, y);
    }
}

// Gameplay
impl Game {
    fn update_opponent(&mut self, line: &str) {
        if line != "NA" {
            for action in parse_action_list(line) {
                let mut new_pos = None;
                match &mut self.opp.pos {
                    OppPos::Area(map) => {
                        match &action {
                            Action::Move(dir, _) => {
                                map.eq_shift(&dir);
                                map.eq_and_not(&self.map);
                            }
                            Action::Surface(sector) => {
                                let mask = map.sector_mask(*sector);
                                map.eq_and(&mask);
                            }
                            Action::Torpedo(pos) => {
                                let mask = map.torpedo_mask(pos);
                                map.eq_and(&mask);
                            }
                            Action::Silence(_, _) => {
                                map.expand(4);
                                map.eq_and_not(&self.map);
                            }
                            Action::Sonar(_) => (),
                        }
                        if map.nb_false() == 0 {
                            new_pos = Some(OppPos::Exact(map.first_false().unwrap()));
                        }
                    }
                    OppPos::Exact(pos) => match &action {
                        Action::Move(dir, _) => {
                            let new_pos = dir.apply(&pos).unwrap();
                            pos.y = new_pos.y;
                            pos.x = new_pos.x;
                        }
                        Action::Surface(_) => (),
                        Action::Torpedo(_) => (),
                        Action::Silence(_, _) => {
                            let mut map = self.map.clone();
                            self.map.set(pos.y, pos.x, true);
                            map.expand(4);
                            map.eq_and_not(&self.map);
                            new_pos = Some(OppPos::Area(map));
                        }
                        Action::Sonar(_) => (),
                    },
                }
                if let Some(pos) = new_pos {
                    self.opp.pos = pos;
                }
            }
            if let Some(pos) = &self.last_turn.torpedo {
                if self.last_turn.opp_life != self.opp.life {
                    match self.last_turn.opp_life - self.opp.life {
                        1 => {
                            if let OppPos::Area(map) = &mut self.opp.pos {
                                let mut square_map = self.map.square(&pos);
                                square_map.set(pos.y, pos.x, false);
                                map.eq_and(&square_map);
                            }
                        }
                        _ => self.opp.pos = OppPos::Exact(pos.clone()),
                    }
                }
            }
        }
        eprintln!("Opponent position:\n{}", self.opp.pos);
    }
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
        self.me.sonar = parse_input!(inputs[5], usize);
        self.me.silence = parse_input!(inputs[6], usize);
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
        self.update_opponent(&opponent_orders);

        self.actions = vec![];
        self.last_turn = LastTurn::new();
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
        self.actions
            .push(Action::Move(direction.clone(), system.clone()));
        Ok(())
    }

    fn surface(&mut self) {
        self.actions
            .push(Action::Surface(self.map.sector(&self.me.pos)));
        self.me.forbidden_map.copy_from(&self.map);
    }

    fn torpedo(&mut self, pos: Pos) -> Result<(), ()> {
        if self.me.torpedo > 0 {
            return Err(());
        }
        self.last_turn.torpedo = Some(pos.clone());
        self.actions.push(Action::Torpedo(pos));
        Ok(())
    }

    fn silence(&mut self, dir: &Direction, dist: usize) -> Result<(), ()> {
        // TODO Check errors by iterating on positions
        let mut me_pos = self.me.pos.clone();
        for _ in 0..dist {
            me_pos = dir.apply(&me_pos)?;
            if me_pos.x >= self.map.w || me_pos.y >= self.map.h {
                return Err(());
            }
            if self.me.forbidden_map.get(me_pos.y, me_pos.x) {
                return Err(());
            }
        }
        self.actions.push(Action::Silence(dir.clone(), dist));
        Ok(())
    }

    fn commit(&mut self) {
        println!(
            "{}",
            self.actions
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>()
                .join(" | ")
        );
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

    fn plan_move(&mut self, game: &mut Game) -> Option<Direction> {
        let dirs = vec![Direction::E, Direction::N, Direction::W, Direction::S];
        let good_dirs: Vec<_> = dirs
            .into_iter()
            .filter(|d| game.can_move_to(d).is_ok())
            .collect();
        eprintln!("Possible directions: {:?}", good_dirs);

        Some(match good_dirs.len() {
            0 => {
                return None;
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
        })
    }

    fn plan_actions(&mut self, game: &mut Game) {
        let dir = match self.plan_move(game) {
            Some(dir) => dir,
            None => {
                self.dir = None;
                game.surface();
                return;
            }
        };
        self.dir = Some(dir.clone());

        if game.me.silence == 0 {
            game.silence(&dir, 1).unwrap();
        } else {
            game.move_to(&dir, &System::Silence).unwrap();
        }
    }
}

// =======================================================================
// main
// =======================================================================
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
