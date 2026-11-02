//! Material properties and attempt parameters. Loaded from TOML at the edge
//! of the process; the core only sees this struct.

use crate::Material;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tuning {
    pub rain_starts_at: u64,
    pub drag_ticks: u64,
    pub frame_phase_ticks: u64,
    pub frame_phases: u8,
    pub bark_ticks: u64,
    #[serde(default = "default_fire_ticks")]
    pub fire_ticks: u64,
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
}

fn default_fire_ticks() -> u64 {
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
    drag_ticks: u64,
    frame_phase_ticks: u64,
    frame_phases: u8,
    bark_ticks: u64,
    fire_ticks: u64,
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
        Ok(Tuning {
            rain_starts_at: raw.time.rain_starts_at,
            drag_ticks: raw.time.drag_ticks,
            frame_phase_ticks: raw.time.frame_phase_ticks,
            frame_phases: raw.time.frame_phases,
            bark_ticks: raw.time.bark_ticks,
            fire_ticks: raw.time.fire_ticks,
            body_rain: millis(raw.body.rain_gain, "body.rain_gain")?,
            brook_rain: millis(raw.body.brook_rain, "body.brook_rain")?,
            wade: millis(raw.body.wade, "body.wade")?,
            damp_at: millis(raw.body.damp_at, "body.damp_at")?,
            soaked_at: millis(raw.body.soaked_at, "body.soaked_at")?,
            thirst_per_tick: millis(raw.body.thirst_per_tick, "body.thirst_per_tick")?,
            drink_relief: millis(raw.body.drink_relief, "body.drink_relief")?,
            thirsty_at: millis(raw.body.thirsty_at, "body.thirsty_at")?,
            parched_at: millis(raw.body.parched_at, "body.parched_at")?,
            hunger_per_tick: millis(raw.body.hunger_per_tick, "body.hunger_per_tick")?,
            cook_relief: millis(raw.body.cook_relief, "body.cook_relief")?,
            hungry_at: millis(raw.body.hungry_at, "body.hungry_at")?,
            starving_at: millis(raw.body.starving_at, "body.starving_at")?,
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
        }
    }
}
