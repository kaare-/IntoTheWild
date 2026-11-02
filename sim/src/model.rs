//! Valley state. Quantities stay piles until a person lifts them or builds
//! them into a frame; after that they are items with stable ids.

use crate::serde_map;
use crate::tuning::Tuning;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const WIDTH: i32 = 7;
pub const HEIGHT: i32 = 5;
const EVENT_CAP: usize = 20_000;

pub type ActorId = u32;
pub type ItemId = u32;
pub type ConstructionId = u32;
pub type WorkId = u32;
pub type EventId = u64;
pub type CellId = u16;

pub fn cell_id(x: i32, y: i32) -> Option<CellId> {
    if x < 0 || y < 0 || x >= WIDTH || y >= HEIGHT {
        None
    } else {
        Some((y * WIDTH + x) as u16)
    }
}

pub fn cell_xy(id: CellId) -> (i32, i32) {
    let n = id as i32;
    (n % WIDTH, n / WIDTH)
}

pub fn chebyshev(a: CellId, b: CellId) -> i32 {
    let (x1, y1) = cell_xy(a);
    let (x2, y2) = cell_xy(b);
    (x1 - x2).abs().max((y1 - y2).abs())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum Material {
    Branch,
    Bark,
    Log,
}

impl Material {
    pub fn word(self) -> &'static str {
        match self {
            Material::Branch => "branch",
            Material::Bark => "bark",
            Material::Log => "log",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Loose,
    Support,
    Beam,
    BarkCover,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Region {
    Slope,
    Brook,
    Floor,
}

impl Region {
    pub fn word(self) -> &'static str {
        match self {
            Region::Slope => "slope",
            Region::Brook => "brook",
            Region::Floor => "valley floor",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Dir {
    North,
    East,
    South,
    West,
}

impl Dir {
    pub fn delta(self) -> (i32, i32) {
        match self {
            Dir::North => (0, -1),
            Dir::East => (1, 0),
            Dir::South => (0, 1),
            Dir::West => (-1, 0),
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            Dir::North => "north",
            Dir::East => "east",
            Dir::South => "south",
            Dir::West => "west",
        }
    }

    pub fn all() -> [Dir; 4] {
        [Dir::North, Dir::East, Dir::South, Dir::West]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExamineTarget {
    Here,
    SelfBody,
    Wood,
    Bark,
    Log,
    Frame,
    Person(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Attempt {
    Look,
    Examine(ExamineTarget),
    Say(String),
    Move(Dir),
    Gather(Material),
    Drop(Material),
    Grab,
    Drag(Dir),
    Release,
    Interrupt,
    RaiseFrame,
    LayBark,
    Drink,
}

impl Attempt {
    pub fn uses_body(&self) -> bool {
        matches!(
            self,
            Attempt::Move(_)
                | Attempt::Gather(_)
                | Attempt::Drop(_)
                | Attempt::Grab
                | Attempt::Drag(_)
                | Attempt::RaiseFrame
                | Attempt::LayBark
                | Attempt::Drink
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Intent {
    pub command_id: String,
    pub actor_id: ActorId,
    pub attempt: Attempt,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandResult {
    pub command_id: String,
    pub actor_id: ActorId,
    pub success: bool,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepReport {
    pub time: u64,
    pub results: Vec<CommandResult>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    pub id: CellId,
    pub x: i32,
    pub y: i32,
    pub region: Region,
    pub rock: bool,
    pub trail: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pile {
    pub cell: CellId,
    pub material: Material,
    pub count: u32,
    pub moisture: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Actor {
    pub id: ActorId,
    pub name: String,
    pub token: String,
    pub cell: CellId,
    pub wetness: u32,
    #[serde(default)]
    pub thirst: u32,
    pub busy: Option<WorkId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub id: ItemId,
    pub material: Material,
    pub moisture: u32,
    pub cell: CellId,
    pub carrier: Option<ActorId>,
    pub role: Role,
    pub construction: Option<ConstructionId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Construction {
    pub id: ConstructionId,
    pub cell: CellId,
    pub support_a: Option<ItemId>,
    pub support_b: Option<ItemId>,
    pub beam: Option<ItemId>,
    pub bark: Option<ItemId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkKind {
    Drag,
    RaiseFrame,
    LayBark,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Work {
    pub id: WorkId,
    pub kind: WorkKind,
    pub actors: Vec<ActorId>,
    pub completes_at: u64,
    pub cause_events: Vec<EventId>,
    pub item: Option<ItemId>,
    pub dir: Option<Dir>,
    pub phase_done: u8,
    pub phases: u8,
    pub construction: Option<ConstructionId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grip {
    pub actor: ActorId,
    pub item: ItemId,
    pub since_event: EventId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Wake {
    Rain,
    Thirst,
    Work(WorkId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scheduled {
    pub at: u64,
    pub seq: u32,
    pub wake: Wake,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scope {
    Personal(ActorId),
    Cell(CellId),
    Valley,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Draw {
    pub tag: String,
    pub subject: String,
    pub value: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub time: u64,
    pub text: String,
    pub scope: Scope,
    pub affected_actors: Vec<ActorId>,
    pub affected_cells: Vec<CellId>,
    pub affected_items: Vec<ItemId>,
    pub causes: Vec<EventId>,
    pub draw: Option<Draw>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct World {
    pub tuning: Tuning,
    pub seed: u64,
    pub time: u64,
    pub raining: bool,
    pub first_rain: Option<EventId>,
    pub next_actor: ActorId,
    pub next_item: ItemId,
    pub next_construction: ConstructionId,
    pub next_work: WorkId,
    pub next_event: EventId,
    pub next_seq: u32,
    pub cells: Vec<Cell>,
    #[serde(with = "serde_map")]
    pub(crate) actors: BTreeMap<ActorId, Actor>,
    #[serde(with = "serde_map")]
    pub(crate) items: BTreeMap<ItemId, Item>,
    #[serde(with = "serde_map")]
    pub(crate) constructions: BTreeMap<ConstructionId, Construction>,
    #[serde(with = "serde_map")]
    pub(crate) works: BTreeMap<WorkId, Work>,
    pub(crate) piles: Vec<Pile>,
    pub(crate) grips: Vec<Grip>,
    pub(crate) schedule: Vec<Scheduled>,
    pub(crate) events: Vec<Event>,
    pub(crate) applied: BTreeMap<String, CommandResult>,
}

const SPAWNS: [(i32, i32); 7] = [(4, 3), (3, 3), (5, 3), (4, 4), (2, 3), (6, 3), (3, 4)];

impl World {
    pub fn valley(tuning: Tuning, seed: u64) -> Self {
        let mut cells = Vec::with_capacity((WIDTH * HEIGHT) as usize);
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let rock = x == 6 && y == 1;
                let region = if y == 0 {
                    Region::Slope
                } else if x <= 1 && (y == 1 || y == 2) {
                    Region::Brook
                } else {
                    Region::Floor
                };
                let trail = matches!((x, y), (2, 3) | (3, 3) | (4, 3) | (4, 2) | (5, 2));
                cells.push(Cell {
                    id: cell_id(x, y).unwrap(),
                    x,
                    y,
                    region,
                    rock,
                    trail,
                });
            }
        }
        let mut world = World {
            tuning,
            seed,
            time: 0,
            raining: false,
            first_rain: None,
            next_actor: 1,
            next_item: 1,
            next_construction: 1,
            next_work: 1,
            next_event: 1,
            next_seq: 1,
            cells,
            actors: BTreeMap::new(),
            items: BTreeMap::new(),
            constructions: BTreeMap::new(),
            works: BTreeMap::new(),
            piles: Vec::new(),
            grips: Vec::new(),
            schedule: Vec::new(),
            events: Vec::new(),
            applied: BTreeMap::new(),
        };
        let rain_at = world.tuning.rain_starts_at;
        world.schedule(rain_at, Wake::Rain);
        world.schedule(0, Wake::Thirst);
        world.add_pile(cell_id(6, 1).unwrap(), Material::Branch, 4, 0);
        world.add_pile(cell_id(6, 1).unwrap(), Material::Bark, 2, 0);
        world.add_pile(cell_id(5, 2).unwrap(), Material::Branch, 4, 0);
        world.add_pile(cell_id(1, 0).unwrap(), Material::Bark, 2, 0);
        world.add_pile(cell_id(3, 2).unwrap(), Material::Log, 1, 0);
        world.normalize();
        world
    }

    pub fn join(&mut self, name: &str, token: &str) -> Result<ActorId, String> {
        let name = clean_name(name)?;
        if token.is_empty() || token.len() > 80 {
            return Err("missing token".into());
        }
        if self.actors.values().any(|a| a.token == token) {
            return Err("that token is already in the valley".into());
        }
        if self
            .actors
            .values()
            .any(|a| a.name.eq_ignore_ascii_case(&name))
        {
            return Err(format!("{name} is already a name in the valley"));
        }
        let n = self.actors.len();
        let (x, y) = if n < SPAWNS.len() {
            SPAWNS[n]
        } else {
            let i = n as i32;
            (i % WIDTH, (i / WIDTH) % HEIGHT)
        };
        self.spawn_at(&name, token, x, y)
    }

    pub fn spawn_at(&mut self, name: &str, token: &str, x: i32, y: i32) -> Result<ActorId, String> {
        let name = clean_name(name)?;
        let cell = cell_id(x, y).ok_or("that place is outside the valley")?;
        if self.actors.values().any(|a| a.token == token) {
            return Err("that token is already in the valley".into());
        }
        if self
            .actors
            .values()
            .any(|a| a.name.eq_ignore_ascii_case(&name))
        {
            return Err(format!("{name} is already a name in the valley"));
        }
        let id = self.next_actor;
        self.next_actor += 1;
        self.actors.insert(
            id,
            Actor {
                id,
                name,
                token: token.to_string(),
                cell,
                wetness: 0,
                thirst: 0,
                busy: None,
            },
        );
        Ok(id)
    }

    pub fn actor_by_token(&self, token: &str) -> Option<ActorId> {
        self.actors
            .values()
            .find(|a| a.token == token)
            .map(|a| a.id)
    }

    pub fn actor_cell(&self, id: ActorId) -> Option<(i32, i32)> {
        self.actor(id).map(|actor| cell_xy(actor.cell))
    }

    pub fn stored_result(&self, command_id: &str) -> Option<CommandResult> {
        self.applied.get(command_id).cloned()
    }

    pub fn sim_time(&self) -> u64 {
        self.time
    }

    pub fn is_raining(&self) -> bool {
        self.raining
    }

    pub fn event_log(&self) -> &[Event] {
        &self.events
    }

    pub fn material_tally(&self, material: Material) -> u32 {
        let piles: u32 = self
            .piles
            .iter()
            .filter(|p| p.material == material)
            .map(|p| p.count)
            .sum();
        let items = self
            .items
            .values()
            .filter(|i| i.material == material)
            .count() as u32;
        piles + items
    }

    pub fn log_cell(&self) -> Option<(i32, i32)> {
        if let Some(item) = self.items.values().find(|i| i.material == Material::Log) {
            return Some(cell_xy(item.cell));
        }
        self.piles
            .iter()
            .find(|p| p.material == Material::Log && p.count > 0)
            .map(|p| cell_xy(p.cell))
    }

    pub fn log_gripped_by(&self) -> Vec<String> {
        let Some(item) = self.items.values().find(|i| i.material == Material::Log) else {
            return Vec::new();
        };
        let mut names: Vec<String> = self
            .grips
            .iter()
            .filter(|g| g.item == item.id)
            .filter_map(|g| self.actors.get(&g.actor).map(|a| a.name.clone()))
            .collect();
        names.sort();
        names
    }

    pub fn exposure_at(&self, x: i32, y: i32) -> Option<u32> {
        cell_id(x, y).map(|id| self.exposure_millis(id))
    }

    pub fn step(&mut self, intents: &[Intent]) -> StepReport {
        let time = self.time;
        let results = self.resolve(intents);
        self.fire_due();
        self.normalize();
        self.trim_events();
        self.time += 1;
        StepReport { time, results }
    }

    pub(crate) fn schedule(&mut self, at: u64, wake: Wake) {
        if self.schedule.iter().any(|s| s.at == at && s.wake == wake) {
            return;
        }
        let seq = self.next_seq;
        self.next_seq += 1;
        self.schedule.push(Scheduled { at, seq, wake });
    }

    pub(crate) fn actor(&self, id: ActorId) -> Option<&Actor> {
        self.actors.get(&id)
    }

    pub(crate) fn cell(&self, id: CellId) -> &Cell {
        &self.cells[id as usize]
    }

    pub(crate) fn item(&self, id: ItemId) -> Option<&Item> {
        self.items.get(&id)
    }

    pub(crate) fn pile_count(&self, cell: CellId, material: Material) -> u32 {
        self.piles
            .iter()
            .find(|p| p.cell == cell && p.material == material)
            .map(|p| p.count)
            .unwrap_or(0)
    }

    pub(crate) fn pile_moisture(&self, cell: CellId, material: Material) -> u32 {
        self.piles
            .iter()
            .find(|p| p.cell == cell && p.material == material)
            .map(|p| p.moisture)
            .unwrap_or(0)
    }

    pub(crate) fn add_pile(&mut self, cell: CellId, material: Material, count: u32, moisture: u32) {
        if count == 0 {
            return;
        }
        if let Some(pile) = self
            .piles
            .iter_mut()
            .find(|p| p.cell == cell && p.material == material)
        {
            let total = pile.count + count;
            pile.moisture = (pile.moisture * pile.count + moisture * count) / total.max(1);
            pile.count = total;
        } else {
            self.piles.push(Pile {
                cell,
                material,
                count,
                moisture,
            });
        }
    }

    pub(crate) fn take_from_pile(
        &mut self,
        cell: CellId,
        material: Material,
        n: u32,
    ) -> Option<u32> {
        let idx = self
            .piles
            .iter()
            .position(|p| p.cell == cell && p.material == material)?;
        if self.piles[idx].count < n {
            return None;
        }
        let moisture = self.piles[idx].moisture;
        self.piles[idx].count -= n;
        if self.piles[idx].count == 0 {
            self.piles.remove(idx);
        }
        Some(moisture)
    }

    pub(crate) fn mint_item(
        &mut self,
        material: Material,
        moisture: u32,
        cell: CellId,
        carrier: Option<ActorId>,
        role: Role,
    ) -> ItemId {
        let id = self.next_item;
        self.next_item += 1;
        self.items.insert(
            id,
            Item {
                id,
                material,
                moisture,
                cell,
                carrier,
                role,
                construction: None,
            },
        );
        id
    }

    pub(crate) fn carry_count(&self, actor: ActorId) -> u32 {
        self.items
            .values()
            .filter(|i| i.carrier == Some(actor))
            .count() as u32
    }

    pub(crate) fn loose_ids(&self, actor: ActorId, material: Material) -> Vec<ItemId> {
        let mut ids: Vec<ItemId> = self
            .items
            .values()
            .filter(|i| i.carrier == Some(actor) && i.material == material && i.role == Role::Loose)
            .map(|i| i.id)
            .collect();
        ids.sort();
        ids
    }

    pub(crate) fn gripping(&self, actor: ActorId) -> Option<ItemId> {
        self.grips.iter().find(|g| g.actor == actor).map(|g| g.item)
    }

    pub(crate) fn log_at(&self, cell: CellId) -> bool {
        self.pile_count(cell, Material::Log) > 0
            || self
                .items
                .values()
                .any(|i| i.material == Material::Log && i.cell == cell && i.carrier.is_none())
    }

    pub(crate) fn log_item(&self) -> Option<&Item> {
        self.items.values().find(|i| i.material == Material::Log)
    }

    pub(crate) fn construction_at(&self, cell: CellId) -> Option<&Construction> {
        self.constructions.values().find(|c| c.cell == cell)
    }

    pub(crate) fn condition_word(&self, moisture: u32) -> &'static str {
        if moisture >= self.tuning.soaked_at {
            "soaked"
        } else if moisture >= self.tuning.damp_at {
            "damp"
        } else {
            "dry"
        }
    }

    pub(crate) fn thirst_word(&self, thirst: u32) -> &'static str {
        if thirst >= self.tuning.parched_at {
            "parched"
        } else if thirst >= self.tuning.thirsty_at {
            "thirsty"
        } else {
            "not thirsty"
        }
    }

    pub(crate) fn thirst_sentence(&self, thirst: u32) -> &'static str {
        match self.thirst_word(thirst) {
            "parched" => "parched",
            "thirsty" => "thirsty",
            _ => "not thirsty",
        }
    }

    pub(crate) fn name_of(&self, id: ActorId) -> String {
        self.actors
            .get(&id)
            .map(|a| a.name.clone())
            .unwrap_or_else(|| "someone".into())
    }

    pub(crate) fn names(&self, ids: &[ActorId]) -> String {
        let mut names: Vec<String> = ids.iter().map(|id| self.name_of(*id)).collect();
        names.sort();
        match names.len() {
            0 => "nobody".into(),
            1 => names[0].clone(),
            2 => format!("{} and {}", names[0], names[1]),
            _ => {
                let last = names[names.len() - 1].clone();
                format!("{}, and {last}", names[..names.len() - 1].join(", "))
            }
        }
    }

    pub(crate) fn move_actor(&mut self, actor: ActorId, dest: CellId) {
        if let Some(person) = self.actors.get_mut(&actor) {
            person.cell = dest;
        }
        let carried: Vec<ItemId> = self
            .items
            .values()
            .filter(|i| i.carrier == Some(actor))
            .map(|i| i.id)
            .collect();
        for id in carried {
            if let Some(item) = self.items.get_mut(&id) {
                item.cell = dest;
            }
        }
    }

    pub(crate) fn emit(
        &mut self,
        text: impl Into<String>,
        scope: Scope,
        mut actors: Vec<ActorId>,
        mut cells: Vec<CellId>,
        mut items: Vec<ItemId>,
        mut causes: Vec<EventId>,
        draw: Option<Draw>,
    ) -> EventId {
        actors.sort_unstable();
        actors.dedup();
        cells.sort_unstable();
        cells.dedup();
        items.sort_unstable();
        items.dedup();
        causes.sort_unstable();
        causes.dedup();
        let id = self.next_event;
        self.next_event += 1;
        self.events.push(Event {
            id,
            time: self.time,
            text: text.into(),
            scope,
            affected_actors: actors,
            affected_cells: cells,
            affected_items: items,
            causes,
            draw,
        });
        id
    }

    pub(crate) fn finish(
        &mut self,
        command_id: String,
        actor_id: ActorId,
        success: bool,
        message: impl Into<String>,
        results: &mut Vec<CommandResult>,
    ) {
        let result = CommandResult {
            command_id,
            actor_id,
            success,
            message: message.into(),
        };
        self.applied
            .insert(result.command_id.clone(), result.clone());
        results.push(result);
    }

    pub(crate) fn relative(&self, from: CellId, to: CellId) -> String {
        if from == to {
            return "here".into();
        }
        let (x1, y1) = cell_xy(from);
        let (x2, y2) = cell_xy(to);
        let dx = x2 - x1;
        let dy = y2 - y1;
        let ns = if dy < 0 {
            "north"
        } else if dy > 0 {
            "south"
        } else {
            ""
        };
        let ew = if dx > 0 {
            "east"
        } else if dx < 0 {
            "west"
        } else {
            ""
        };
        match (ns, ew) {
            ("", "") => "here".into(),
            (ns, "") => ns.into(),
            ("", ew) => ew.into(),
            (ns, ew) => format!("{ns} and {ew}"),
        }
    }

    fn normalize(&mut self) {
        self.piles.sort_by_key(|p| (p.cell, p.material as u8));
        self.grips.sort_by_key(|g| (g.item, g.actor));
        self.schedule.sort_by_key(|s| (s.at, s.seq));
    }

    fn trim_events(&mut self) {
        if self.events.len() > EVENT_CAP {
            let extra = self.events.len() - EVENT_CAP;
            self.events.drain(0..extra);
        }
    }

    pub(crate) fn wet_add(&self, gain: u32, exposure: u32) -> u32 {
        ((gain as u64) * (exposure as u64) / 1000) as u32
    }
}

fn clean_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    let chars: Vec<char> = name.chars().collect();
    if chars.is_empty() || chars.len() > 24 {
        return Err("use a name of 1 to 24 characters".into());
    }
    if !chars[0].is_ascii_alphabetic() {
        return Err("a name starts with a letter".into());
    }
    if !chars
        .iter()
        .all(|c| c.is_ascii_alphanumeric() || *c == ' ' || *c == '-' || *c == '\'')
    {
        return Err("use letters, numbers, spaces, hyphens, or apostrophes".into());
    }
    Ok(name.to_string())
}

pub fn contest_hash(seed: u64, time: u64, tag: &str, subject: &str, actor: u32) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    feed(&mut hash, &seed.to_le_bytes());
    feed(&mut hash, &time.to_le_bytes());
    feed(&mut hash, tag.as_bytes());
    feed(&mut hash, subject.as_bytes());
    feed(&mut hash, &actor.to_le_bytes());
    hash
}

fn feed(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= *byte as u64;
        *hash = hash.wrapping_mul(0x100000001b3);
    }
    *hash ^= 0xff;
    *hash = hash.wrapping_mul(0x100000001b3);
}
