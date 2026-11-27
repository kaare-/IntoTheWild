use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use villmarks_server::{Record, Store};
use villmarks_sim::{Attempt, Dir, Intent, Material, Tuning, World};

fn scratch() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("villmarks-{nanos}"));
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn valley() -> World {
    let mut tuning = Tuning::evening();
    tuning.rain_starts_at = 40;
    World::valley(tuning, 11)
}

fn commit(store: &mut Store, world: &World, record: Record) {
    store.commit(world, &record).unwrap();
}

#[test]
fn restart_keeps_grips_and_does_not_repeat_a_command() {
    let dir = scratch();
    let db = dir.join("valley.sqlite");
    let mut live = valley();
    let mut store = Store::open(&db, 2).unwrap();
    let mut saved = store.load_or_seed(valley()).unwrap();

    let sera = live.join("Sera", "t-sera").unwrap();
    saved.join("Sera", "t-sera").unwrap();
    commit(
        &mut store,
        &saved,
        Record::Join {
            name: "Sera".into(),
            token: "t-sera".into(),
        },
    );
    let otto = live.join("Otto", "t-otto").unwrap();
    saved.join("Otto", "t-otto").unwrap();
    commit(
        &mut store,
        &saved,
        Record::Join {
            name: "Otto".into(),
            token: "t-otto".into(),
        },
    );

    let walk = vec![
        Intent {
            command_id: "sera-north".into(),
            actor_id: sera,
            attempt: Attempt::Move(Dir::North),
        },
        Intent {
            command_id: "otto-north".into(),
            actor_id: otto,
            attempt: Attempt::Move(Dir::North),
        },
    ];
    live.step(&walk);
    saved.step(&walk);
    commit(
        &mut store,
        &saved,
        Record::Step {
            intents: walk.clone(),
        },
    );

    let beside = vec![Intent {
        command_id: "sera-west".into(),
        actor_id: sera,
        attempt: Attempt::Move(Dir::West),
    }];
    live.step(&beside);
    saved.step(&beside);
    commit(&mut store, &saved, Record::Step { intents: beside });
    assert_eq!(live.log_cell(), Some((3, 2)));
    assert_eq!(saved.actor_cell(sera), Some((3, 2)));
    assert_eq!(saved.actor_cell(otto), Some((3, 2)));

    let grabs = vec![
        Intent {
            command_id: "grab-sera".into(),
            actor_id: sera,
            attempt: Attempt::Grab,
        },
        Intent {
            command_id: "grab-otto".into(),
            actor_id: otto,
            attempt: Attempt::Grab,
        },
    ];
    live.step(&grabs);
    saved.step(&grabs);
    commit(
        &mut store,
        &saved,
        Record::Step {
            intents: grabs.clone(),
        },
    );
    assert_eq!(
        saved.log_gripped_by(),
        vec!["Otto".to_string(), "Sera".to_string()]
    );
    assert_eq!(live, saved);

    drop(store);
    let mut reopened = Store::open(&db, 2).unwrap();
    let mut restored = reopened.load_or_seed(valley()).unwrap();
    assert_eq!(restored, saved);
    assert_eq!(restored.log_gripped_by().len(), 2);

    let again = vec![grabs[0].clone()];
    let before = restored.material_tally(Material::Log);
    restored.step(&again);
    saved.step(&again);
    assert_eq!(restored.material_tally(Material::Log), before);
    assert_eq!(restored.log_gripped_by().len(), 2);
    assert_eq!(restored, saved);
    assert!(
        restored
            .event_log()
            .iter()
            .filter(|event| event.text.contains("grips the log"))
            .count()
            == 2
    );

    let _ = std::fs::remove_dir_all(dir);
}
