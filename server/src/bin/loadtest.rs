//! Local batch measurement. This is not a promise about hosted play.
//!
//! It times [`World::step`] for scripted actors, then the same step plus one
//! SQLite transaction. It does not open sockets, parse HTTP, or fan out
//! WebSocket frames. Model latency is not involved.

use std::time::{Instant, SystemTime, UNIX_EPOCH};
use villmarks_server::{Record, Store};
use villmarks_sim::{Attempt, Dir, Intent, Tuning, World, WIDTH};

const WINDOWS: u32 = 20;

fn main() {
    let mut tuning = Tuning::evening();
    tuning.rain_starts_at = 10_000;
    println!("villmarks load test");
    println!(
        "windows={WINDOWS} rain_starts_at={} (rain does not fire during the sample)",
        tuning.rain_starts_at
    );
    println!("network=no  database column says whether sqlite is in the timed section");
    report("spread-move", 1000, false, tuning.clone());
    report("brook-say", 100, true, tuning);
}

fn report(name: &str, count: usize, brook: bool, tuning: Tuning) {
    let sim = measure_sim(count, brook, tuning.clone());
    let db = measure_db(count, brook, tuning);
    print_row(name, count, "sim", &sim);
    print_row(name, count, "sim+sqlite", &db);
}

fn print_row(name: &str, count: usize, mode: &str, samples: &[std::time::Duration]) {
    let total: u128 = samples.iter().map(|d| d.as_micros()).sum();
    let mean = total / samples.len() as u128;
    let max = samples.iter().map(|d| d.as_micros()).max().unwrap_or(0);
    let lag = samples.iter().any(|d| d.as_secs_f64() > 20.0);
    println!(
        "{name:12} actors={count:<4} mode={mode:<11} windows={} mean_us={mean} max_us={max} exceeded_20s_window={lag}",
        samples.len()
    );
}

fn measure_sim(count: usize, brook: bool, tuning: Tuning) -> Vec<std::time::Duration> {
    let mut world = populate(tuning, count, brook);
    let mut samples = Vec::new();
    for window in 0..WINDOWS {
        let intents = intents_for(&world, window, brook);
        let started = Instant::now();
        world.step(&intents);
        samples.push(started.elapsed());
    }
    samples
}

fn measure_db(count: usize, brook: bool, tuning: Tuning) -> Vec<std::time::Duration> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("villmarks-load-{nanos}"));
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("load.sqlite");
    let populated = populate(tuning, count, brook);
    let mut store = Store::open(&db, 5).unwrap();
    let mut world = store.load_or_seed(populated).unwrap();
    let mut samples = Vec::new();
    for window in 0..WINDOWS {
        let intents = intents_for(&world, window, brook);
        let started = Instant::now();
        world.step(&intents);
        store
            .commit(
                &world,
                &Record::Step {
                    intents: intents.clone(),
                },
            )
            .unwrap();
        samples.push(started.elapsed());
    }
    let _ = std::fs::remove_dir_all(dir);
    samples
}

fn populate(tuning: Tuning, count: usize, brook: bool) -> World {
    let mut world = World::valley(tuning, 1);
    for index in 0..count {
        let (x, y) = if brook {
            (0, 1)
        } else {
            ((index % 7) as i32, ((index / 7) % 5) as i32)
        };
        world
            .spawn_at(&format!("A{index}"), &format!("t{index}"), x, y)
            .unwrap();
    }
    world
}

fn intents_for(world: &World, window: u32, brook: bool) -> Vec<Intent> {
    let mut intents = Vec::new();
    let mut index = 0u32;
    while let Some((x, _y)) = world.actor_cell(index + 1) {
        let actor = index + 1;
        if brook {
            intents.push(Intent {
                command_id: format!("w{window}-a{actor}"),
                actor_id: actor,
                attempt: Attempt::Say("here".into()),
            });
        } else {
            let mut dir = if window % 2 == 0 {
                Dir::East
            } else {
                Dir::West
            };
            let dest = x + if dir == Dir::East { 1 } else { -1 };
            if dest < 0 || dest >= WIDTH {
                dir = if dir == Dir::East {
                    Dir::West
                } else {
                    Dir::East
                };
            }
            intents.push(Intent {
                command_id: format!("w{window}-a{actor}"),
                actor_id: actor,
                attempt: Attempt::Move(dir),
            });
        }
        index += 1;
        if index > 5000 {
            break;
        }
    }
    intents
}
