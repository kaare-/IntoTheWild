//! Material properties and attempt parameters. Loaded from TOML at the edge
//! of the process; the core only sees this struct.

use crate::Material;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tuning {
    pub rain_starts_at: u64,
    #[serde(default = "default_season_at")]
    pub season_at: u64,
    #[serde(default = "default_rot_ticks")]
    pub rot_ticks: u64,
    pub drag_ticks: u64,
    pub frame_phase_ticks: u64,
    pub frame_phases: u8,
    pub bark_ticks: u64,
    #[serde(default = "default_fire_ticks")]
    pub fire_ticks: u64,
    /// Ticks of leftover heat after a body leaves a fire or the fire goes out.
    /// During them the sky's cold is halved. Open wind still cools.
    #[serde(default = "default_warmth_hold")]
    pub warmth_hold: u64,
    pub body_rain: u32,
    pub brook_rain: u32,
    pub wade: u32,
    pub damp_at: u32,
    pub soaked_at: u32,
    pub thirst_per_tick: u32,
    pub drink_relief: u32,
    pub thirsty_at: u32,
    pub parched_at: u32,
    #[serde(default = "default_hunger_tick")]
    pub hunger_per_tick: u32,
    #[serde(default = "default_cook_relief")]
    pub cook_relief: u32,
    #[serde(default = "default_hungry_at")]
    pub hungry_at: u32,
    #[serde(default = "default_starving_at")]
    pub starving_at: u32,
    #[serde(default = "default_illness_chance")]
    pub illness_chance: u32,
    #[serde(default = "default_still_chance")]
    pub still_chance: u32,
    #[serde(default = "default_hunger_tick")]
    pub energy_per_tick: u32,
    #[serde(default = "default_hungry_at")]
    pub tiring_at: u32,
    #[serde(default = "default_starving_at")]
    pub spent_at: u32,
    #[serde(default = "default_cook_relief")]
    pub eat_relief: u32,
    #[serde(default = "default_chill_at")]
    pub chill_at: u64,
    #[serde(default = "default_cold_tick")]
    pub cold_per_tick: u32,
    #[serde(default = "default_hungry_at")]
    pub chilly_at: u32,
    #[serde(default = "default_starving_at")]
    pub cold_at: u32,
    /// Cold a body loses each tick while standing on a burning fire.
    /// That tick does not also add the sky's cold.
    #[serde(default = "default_fire_warmth")]
    pub fire_warmth: u32,
    #[serde(default = "default_sun_tick")]
    pub sun_per_tick: u32,
    #[serde(default = "default_hungry_at")]
    pub pink_at: u32,
    #[serde(default = "default_starving_at")]
    pub burned_at: u32,
    #[serde(default = "default_smear_at")]
    pub smear_at: u32,
    #[serde(default = "default_mud_fade")]
    pub mud_fade: u32,
    #[serde(default = "default_mud_wash")]
    pub mud_wash: u32,
    #[serde(default = "default_mud_sun")]
    pub mud_sun: u32,
    #[serde(default = "default_illness_hit")]
    pub illness_hit: u32,
    #[serde(default = "default_ill_at")]
    pub ill_at: u32,
    #[serde(default = "default_illness_recovery")]
    pub illness_recovery: u32,
    pub carry_slots: u32,
    pub sight: i32,
    pub drag_needed: u32,
    pub rock_roof: u32,
    pub dry_bark_roof: u32,
    pub wet_bark_roof: u32,
    pub branch_rain: u32,
    pub bark_rain: u32,
    pub log_rain: u32,
    #[serde(default = "default_berry_rain")]
    pub berry_rain: u32,
    #[serde(default = "default_water_rain")]
    pub water_rain: u32,
    #[serde(default = "default_berry_rain")]
    pub mushroom_rain: u32,
    #[serde(default = "default_berry_rain")]
    pub bitter_rain: u32,
    #[serde(default = "default_water_rain")]
    pub pot_rain: u32,
    #[serde(default = "default_water_rain")]
    pub mud_rain: u32,
}

fn default_fire_ticks() -> u64 {
    8
}
fn default_warmth_hold() -> u64 {
    4
}
fn default_season_at() -> u64 {
    18
}
fn default_rot_ticks() -> u64 {
    8
}
fn default_hunger_tick() -> u32 {
    10
}
fn default_cook_relief() -> u32 {
    450
}
fn default_hungry_at() -> u32 {
    250
}
fn default_starving_at() -> u32 {
    600
}
fn default_berry_rain() -> u32 {
    160
}
fn default_illness_chance() -> u32 {
    500
}
fn default_still_chance() -> u32 {
    900
}
fn default_chill_at() -> u64 {
    12
}
fn default_cold_tick() -> u32 {
    40
}
fn default_fire_warmth() -> u32 {
    160
}
fn default_sun_tick() -> u32 {
    50
}
fn default_smear_at() -> u32 {
    500
}
fn default_mud_fade() -> u32 {
    40
}
fn default_mud_wash() -> u32 {
    200
}
fn default_mud_sun() -> u32 {
    250
}
fn default_illness_hit() -> u32 {
    600
}
fn default_ill_at() -> u32 {
    250
}
fn default_illness_recovery() -> u32 {
    100
}
fn default_water_rain() -> u32 {
    0
}

#[derive(Deserialize)]
struct RawFile {
    time: RawTime,
    body: RawBody,
    forces: RawForces,
    roof: RawRoof,
    materials: BTreeMap<String, RawMat>,
}

#[derive(Deserialize)]
struct RawTime {
    rain_starts_at: u64,
    season_at: u64,
    rot_ticks: u64,
    drag_ticks: u64,
    frame_phase_ticks: u64,
    frame_phases: u8,
    bark_ticks: u64,
    fire_ticks: u64,
    warmth_hold: u64,
    chill_at: u64,
}

#[derive(Deserialize)]
struct RawBody {
    rain_gain: f64,
    brook_rain: f64,
    wade: f64,
    damp_at: f64,
    soaked_at: f64,
    thirst_per_tick: f64,
    drink_relief: f64,
    thirsty_at: f64,
    parched_at: f64,
    hunger_per_tick: f64,
    cook_relief: f64,
    hungry_at: f64,
    starving_at: f64,
    illness_chance: f64,
    still_chance: f64,
    illness_hit: f64,
    ill_at: f64,
    illness_recovery: f64,
    cold_per_tick: f64,
    chilly_at: f64,
    cold_at: f64,
    fire_warmth: f64,
    sun_per_tick: f64,
    pink_at: f64,
    burned_at: f64,
    smear_at: f64,
    mud_fade: f64,
    mud_wash: f64,
    mud_sun: f64,
    carry_slots: u32,
    sight: i32,
}

#[derive(Deserialize)]
struct RawForces {
    drag_needed: u32,
}

#[derive(Deserialize)]
struct RawRoof {
    rock: f64,
    dry_bark: f64,
    wet_bark: f64,
}

#[derive(Deserialize)]
struct RawMat {
    rain_gain: f64,
}

fn millis(value: f64, name: &str) -> Result<u32, String> {
    if !(0.0..=1.0).contains(&value) {
        return Err(format!("{name} must be between 0 and 1"));
    }
    Ok((value * 1000.0).round() as u32)
}

impl Tuning {
    pub fn from_toml(text: &str) -> Result<Self, String> {
        let raw: RawFile = toml::from_str(text).map_err(|err| err.to_string())?;
        if raw.time.drag_ticks == 0
            || raw.time.frame_phase_ticks == 0
            || raw.time.bark_ticks == 0
            || raw.time.fire_ticks == 0
            || raw.time.rot_ticks == 0
            || raw.time.warmth_hold == 0
        {
            return Err("work durations must be at least 1 tick".into());
        }
        if raw.time.frame_phases == 0 {
            return Err("frame_phases must be at least 1".into());
        }
        if raw.body.carry_slots == 0 {
            return Err("carry_slots must be at least 1".into());
        }
        if raw.body.sight < 1 {
            return Err("sight must be at least 1".into());
        }
        if raw.forces.drag_needed == 0 {
            return Err("drag_needed must be at least 1".into());
        }
        let branch = raw
            .materials
            .get("branch")
            .ok_or("missing material branch")?;
        let bark = raw.materials.get("bark").ok_or("missing material bark")?;
        let log = raw.materials.get("log").ok_or("missing material log")?;
        let berry = raw.materials.get("berry").ok_or("missing material berry")?;
        let water = raw.materials.get("water").ok_or("missing material water")?;
        let mushroom = raw
            .materials
            .get("mushroom")
            .ok_or("missing material mushroom")?;
        let bitter = raw
            .materials
            .get("bitter berry")
            .ok_or("missing material bitter berry")?;
        let pot = raw.materials.get("pot").ok_or("missing material pot")?;
        let mud = raw.materials.get("mud").ok_or("missing material mud")?;
        let hunger_per_tick = millis(raw.body.hunger_per_tick, "body.hunger_per_tick")?;
        let cook_relief = millis(raw.body.cook_relief, "body.cook_relief")?;
        let hungry_at = millis(raw.body.hungry_at, "body.hungry_at")?;
        let starving_at = millis(raw.body.starving_at, "body.starving_at")?;
        Ok(Tuning {
            rain_starts_at: raw.time.rain_starts_at,
            season_at: raw.time.season_at,
            rot_ticks: raw.time.rot_ticks,
            drag_ticks: raw.time.drag_ticks,
            frame_phase_ticks: raw.time.frame_phase_ticks,
            frame_phases: raw.time.frame_phases,
            bark_ticks: raw.time.bark_ticks,
            fire_ticks: raw.time.fire_ticks,
            warmth_hold: raw.time.warmth_hold,
            chill_at: raw.time.chill_at,
            body_rain: millis(raw.body.rain_gain, "body.rain_gain")?,
            brook_rain: millis(raw.body.brook_rain, "body.brook_rain")?,
            wade: millis(raw.body.wade, "body.wade")?,
            damp_at: millis(raw.body.damp_at, "body.damp_at")?,
            soaked_at: millis(raw.body.soaked_at, "body.soaked_at")?,
            thirst_per_tick: millis(raw.body.thirst_per_tick, "body.thirst_per_tick")?,
            drink_relief: millis(raw.body.drink_relief, "body.drink_relief")?,
            thirsty_at: millis(raw.body.thirsty_at, "body.thirsty_at")?,
            parched_at: millis(raw.body.parched_at, "body.parched_at")?,
            hunger_per_tick,
            cook_relief,
            hungry_at,
            starving_at,
            illness_chance: millis(raw.body.illness_chance, "body.illness_chance")?,
            still_chance: millis(raw.body.still_chance, "body.still_chance")?,
            energy_per_tick: hunger_per_tick,
            tiring_at: hungry_at,
            spent_at: starving_at,
            eat_relief: cook_relief,
            cold_per_tick: millis(raw.body.cold_per_tick, "body.cold_per_tick")?,
            chilly_at: millis(raw.body.chilly_at, "body.chilly_at")?,
            cold_at: millis(raw.body.cold_at, "body.cold_at")?,
            fire_warmth: millis(raw.body.fire_warmth, "body.fire_warmth")?,
            sun_per_tick: millis(raw.body.sun_per_tick, "body.sun_per_tick")?,
            pink_at: millis(raw.body.pink_at, "body.pink_at")?,
            burned_at: millis(raw.body.burned_at, "body.burned_at")?,
            smear_at: millis(raw.body.smear_at, "body.smear_at")?,
            mud_fade: millis(raw.body.mud_fade, "body.mud_fade")?,
            mud_wash: millis(raw.body.mud_wash, "body.mud_wash")?,
            mud_sun: millis(raw.body.mud_sun, "body.mud_sun")?,
            illness_hit: millis(raw.body.illness_hit, "body.illness_hit")?,
            ill_at: millis(raw.body.ill_at, "body.ill_at")?,
            illness_recovery: millis(raw.body.illness_recovery, "body.illness_recovery")?,
            carry_slots: raw.body.carry_slots,
            sight: raw.body.sight,
            drag_needed: raw.forces.drag_needed,
            rock_roof: millis(raw.roof.rock, "roof.rock")?,
            dry_bark_roof: millis(raw.roof.dry_bark, "roof.dry_bark")?,
            wet_bark_roof: millis(raw.roof.wet_bark, "roof.wet_bark")?,
            branch_rain: millis(branch.rain_gain, "branch.rain_gain")?,
            bark_rain: millis(bark.rain_gain, "bark.rain_gain")?,
            log_rain: millis(log.rain_gain, "log.rain_gain")?,
            berry_rain: millis(berry.rain_gain, "berry.rain_gain")?,
            water_rain: millis(water.rain_gain, "water.rain_gain")?,
            mushroom_rain: millis(mushroom.rain_gain, "mushroom.rain_gain")?,
            bitter_rain: millis(bitter.rain_gain, "bitter berry.rain_gain")?,
            pot_rain: millis(pot.rain_gain, "pot.rain_gain")?,
            mud_rain: millis(mud.rain_gain, "mud.rain_gain")?,
        })
    }

    pub fn evening() -> Self {
        Self::from_toml(include_str!("../../config/evening.toml"))
            .expect("config/evening.toml is valid")
    }

    pub fn rain_gain(&self, material: Material) -> u32 {
        match material {
            Material::Branch => self.branch_rain,
            Material::Bark => self.bark_rain,
            Material::Log => self.log_rain,
            Material::Berry => self.berry_rain,
            Material::Water => self.water_rain,
            Material::Mushroom => self.mushroom_rain,
            Material::BitterBerry => self.bitter_rain,
            Material::Pot => self.pot_rain,
            Material::Mud => self.mud_rain,
            Material::Stone => 0,
        }
    }
}
