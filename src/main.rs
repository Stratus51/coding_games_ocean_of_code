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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
        self.data[y * MAP_SIZE + x] = value;
    }
    fn get(&self, y: usize, x: usize) -> bool {
        self.data[y * MAP_SIZE + x]
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
            y: i / MAP_SIZE,
            x: i % MAP_SIZE,
        })
    }

    fn eqAndNot(&mut self, map: &Map) {
        for (v, forbidden) in self.data.iter_mut().zip(map.data.iter()) {
            if *forbidden {
                *v = false;
            }
        }
    }

    fn invert(&mut self) {
        self.data.iter_mut().for_each(|v| *v = !*v);
    }
}
impl std::fmt::Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ret = vec![];
        for y in 0..self.h {
            let mut tmp = vec![];
            for x in 0..self.w {
                if self.data[y * MAP_SIZE + x] {
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

#[derive(Debug, Clone)]
struct Pos {
    x: usize,
    y: usize,
}

#[derive(Debug, Clone)]
enum PlayerPos {
    Exact(Pos),
    Area(Map),
}
#[derive(Debug, Clone)]
struct Player {
    forbidden_map: Map,
    pos: PlayerPos,
    life: i32,
    torpedo: usize,
    sonar: usize,
    silence: usize,
    mine: usize,
}

impl Player {
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
struct Game {
    // Static
    map: Map,
    my_id: usize,

    // Dynamic
    me: Player,
    opp: Player,

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
            me: Player::new(&map),
            opp: Player::new(&map),
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
        self.me.pos = PlayerPos::Exact(Pos { y, x });
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

    fn move_to(&mut self, direction: &Direction, system: &System) -> Result<(), ()> {
        let me_pos = if let PlayerPos::Exact(pos) = &self.me.pos {
            pos
        } else {
            panic!()
        };
        let next_pos = direction.apply(me_pos)?;
        if next_pos.x >= self.map.w || next_pos.y >= self.map.h {
            return Err(());
        }
        if self.me.forbidden_map.get(next_pos.y, next_pos.x) {
            return Err(());
        }
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
    fn plan_actions(&mut self, game: &mut Game) {
        // Manage basic movement
        let dirs = [Direction::E, Direction::N, Direction::W, Direction::S];
        let recharge = System::Torpedo;
        let mut moved = false;
        for dir in dirs.iter() {
            if let Ok(_) = game.move_to(dir, &recharge) {
                moved = true;
                break;
            }
        }
        if !moved {
            game.surface();
        }

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
