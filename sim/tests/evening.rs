use villmarks_sim::{parse_attempt, Attempt, Dir, ExamineTarget, Intent, Material, Tuning, World};

fn world_at(rain_at: u64) -> World {
    let mut tuning = Tuning::evening();
    tuning.rain_starts_at = rain_at;
    World::valley(tuning, 7)
}

fn cmd(id: &str, actor: u32, attempt: Attempt) -> Intent {
    Intent {
        command_id: id.into(),
        actor_id: actor,
        attempt,
    }
}

fn tally_carry(world: &World, actor: u32) -> u32 {
    world
        .observe(actor)
        .unwrap()
        .self_info
        .carrying
        .iter()
        .map(|item| item.count)
        .sum()
}

#[test]
fn the_valley_is_already_there() {
    let world = World::valley(Tuning::evening(), 1);
    assert_eq!(world.log_cell(), Some((3, 2)));
    assert_eq!(world.material_tally(Material::Branch), 8);
    assert_eq!(world.material_tally(Material::Log), 1);
    assert!(!world.is_raining());
    assert_eq!(world.exposure_at(6, 1), Some(0));
    assert_eq!(world.exposure_at(5, 2), Some(1000));
}

#[test]
fn simultaneous_drags_ignore_packet_order_and_record_causes() {
    let mut first = world_at(100);
    let mut second = world_at(100);
    let sera = first.spawn_at("Sera", "sera", 3, 2).unwrap();
    let otto = first.spawn_at("Otto", "otto", 3, 2).unwrap();
    let sera_b = second.spawn_at("Sera", "sera", 3, 2).unwrap();
    let otto_b = second.spawn_at("Otto", "otto", 3, 2).unwrap();

    first.step(&[
        cmd("g-sera", sera, Attempt::Grab),
        cmd("g-otto", otto, Attempt::Grab),
    ]);
    second.step(&[
        cmd("g-otto", otto_b, Attempt::Grab),
        cmd("g-sera", sera_b, Attempt::Grab),
    ]);
    first.step(&[
        cmd("d-sera", sera, Attempt::Drag(Dir::East)),
        cmd("d-otto", otto, Attempt::Drag(Dir::East)),
    ]);
    second.step(&[
        cmd("d-otto", otto_b, Attempt::Drag(Dir::East)),
        cmd("d-sera", sera_b, Attempt::Drag(Dir::East)),
    ]);
    assert_eq!(first.log_cell(), Some((3, 2)));
    assert_eq!(first, second);

    first.step(&[]);
    second.step(&[]);
    assert_eq!(first.log_cell(), Some((4, 2)));
    assert_eq!(
        first.log_gripped_by(),
        vec!["Otto".to_string(), "Sera".to_string()]
    );
    assert_eq!(first, second);

    let shift = first
        .event_log()
        .iter()
        .find(|event| event.text.contains("shifts"))
        .unwrap();
    let causes: Vec<_> = shift
        .causes
        .iter()
        .filter_map(|id| first.event_log().iter().find(|event| event.id == *id))
        .map(|event| event.text.as_str())
        .collect();
    assert!(causes.iter().any(|text| text.contains("grips")));
    assert!(causes.iter().any(|text| text.contains("weight")));
    assert!(causes.len() >= 4);
}

#[test]
fn one_person_cannot_drag_the_log() {
    let mut world = world_at(100);
    let sera = world.spawn_at("Sera", "sera", 3, 2).unwrap();
    world.step(&[cmd("g", sera, Attempt::Grab)]);
    world.step(&[cmd("d", sera, Attempt::Drag(Dir::East))]);
    world.step(&[]);
    assert_eq!(world.log_cell(), Some((3, 2)));
    assert!(world
        .event_log()
        .iter()
        .any(|event| event.text.contains("alone")));
}

#[test]
fn one_branch_goes_to_one_person_regardless_of_order() {
    let mut left = world_at(100);
    let mut right = world_at(100);
    let sera = left.spawn_at("Sera", "s", 5, 2).unwrap();
    let otto = left.spawn_at("Otto", "o", 5, 2).unwrap();
    let sera_b = right.spawn_at("Sera", "s", 5, 2).unwrap();
    let otto_b = right.spawn_at("Otto", "o", 5, 2).unwrap();
    for index in 0..3 {
        let id = format!("thin-{index}");
        left.step(&[cmd(&id, sera, Attempt::Gather(Material::Branch))]);
        right.step(&[cmd(&id, sera_b, Attempt::Gather(Material::Branch))]);
    }
    let before = left.material_tally(Material::Branch);
    left.step(&[
        cmd("ga", sera, Attempt::Gather(Material::Branch)),
        cmd("gb", otto, Attempt::Gather(Material::Branch)),
    ]);
    right.step(&[
        cmd("gb", otto_b, Attempt::Gather(Material::Branch)),
        cmd("ga", sera_b, Attempt::Gather(Material::Branch)),
    ]);
    assert_eq!(left, right);
    assert_eq!(left.material_tally(Material::Branch), before);
    assert_eq!(tally_carry(&left, sera) + tally_carry(&left, otto), 4);
    assert!(left
        .event_log()
        .iter()
        .any(|event| event.text.contains("another hand") && event.draw.is_some()));
}

#[test]
fn duplicate_command_does_not_run_twice() {
    let mut world = world_at(100);
    let sera = world.spawn_at("Sera", "s", 5, 2).unwrap();
    world.step(&[cmd("c1", sera, Attempt::Gather(Material::Branch))]);
    let events = world.event_log().len();
    assert_eq!(tally_carry(&world, sera), 1);
    world.step(&[cmd("c1", sera, Attempt::Gather(Material::Branch))]);
    assert_eq!(tally_carry(&world, sera), 1);
    assert_eq!(world.event_log().len(), events);
    assert_eq!(world.material_tally(Material::Branch), 8);
}

#[test]
fn interrupt_cancels_a_drag_before_it_lands() {
    let mut world = world_at(100);
    let sera = world.spawn_at("Sera", "s", 3, 2).unwrap();
    let otto = world.spawn_at("Otto", "o", 3, 2).unwrap();
    world.step(&[
        cmd("g1", sera, Attempt::Grab),
        cmd("g2", otto, Attempt::Grab),
    ]);
    world.step(&[
        cmd("d1", sera, Attempt::Drag(Dir::East)),
        cmd("d2", otto, Attempt::Drag(Dir::East)),
    ]);
    assert!(world.observe(sera).unwrap().work.is_some());
    world.step(&[cmd("stop", sera, Attempt::Interrupt)]);
    assert!(world.observe(sera).unwrap().work.is_none());
    assert!(world.observe(otto).unwrap().work.is_none());
    assert_eq!(world.log_cell(), Some((3, 2)));
    assert!(world
        .event_log()
        .iter()
        .any(|event| event.text.contains("settles")));
}

#[test]
fn half_finished_frame_remains_and_does_not_stop_rain() {
    let mut world = world_at(80);
    let sera = world.spawn_at("Sera", "s", 5, 2).unwrap();
    for index in 0..3 {
        world.step(&[cmd(
            &format!("g{index}"),
            sera,
            Attempt::Gather(Material::Branch),
        )]);
    }
    world.step(&[cmd("raise", sera, Attempt::RaiseFrame)]);
    world.step(&[]);
    world.step(&[cmd("stop", sera, Attempt::Interrupt)]);
    assert!(world.observe(sera).unwrap().work.is_none());
    assert_eq!(world.exposure_at(5, 2), Some(1000));
    let text = world.examine(sera, ExamineTarget::Frame);
    assert!(text.contains("not supported at both ends"), "{text}");
    assert!(text.contains("Supports: 1"), "{text}");
    assert_eq!(world.material_tally(Material::Branch), 8);
}

#[test]
fn rain_with_nobody_present_soaks_open_wood_and_spares_the_overhang() {
    let mut world = World::valley(Tuning::evening(), 7);
    for _ in 0..24 {
        world.step(&[]);
    }
    assert!(world.is_raining());
    let near = world.spawn_at("Sera", "s", 5, 3).unwrap();
    let text = world.examine(near, ExamineTarget::Wood);
    assert!(text.contains("soaked"), "{text}");
    assert!(text.contains("dry"), "{text}");
    assert!(text.contains("in contact with the ground"), "{text}");
    assert!(text.contains("under the rock overhang"), "{text}");
    assert!(text.contains("Rain does not reach it."), "{text}");
    assert!(text.contains("Rain reaches it."), "{text}");

    let far = world.spawn_at("Linde", "l", 0, 4).unwrap();
    let seen = world.observe(far).unwrap();
    assert!(seen
        .visible_cells
        .iter()
        .all(|cell| !((cell.x == 5 && cell.y == 2) || (cell.x == 6 && cell.y == 1))));
    let far_wood = world.examine(far, ExamineTarget::Wood);
    assert_eq!(far_wood, "You see no wood from here.");
    assert!(!seen.map.contains('W') && !seen.map.contains('w'));
}

#[test]
fn bark_on_a_supported_frame_keeps_the_wood_beneath_drier() {
    let mut world = world_at(100);
    let sera = world.spawn_at("Sera", "s", 6, 1).unwrap();
    for index in 0..3 {
        world.step(&[cmd(
            &format!("pole-{index}"),
            sera,
            Attempt::Gather(Material::Branch),
        )]);
    }
    world.step(&[cmd("bark", sera, Attempt::Gather(Material::Bark))]);
    for (index, dir) in [Dir::South, Dir::South, Dir::South, Dir::West, Dir::West]
        .into_iter()
        .enumerate()
    {
        world.step(&[cmd(&format!("walk-{index}"), sera, Attempt::Move(dir))]);
    }
    assert_eq!(
        (
            world.observe(sera).unwrap().self_info.x,
            world.observe(sera).unwrap().self_info.y
        ),
        (4, 4)
    );
    world.step(&[cmd("raise", sera, Attempt::RaiseFrame)]);
    world.step(&[]);
    world.step(&[]);
    world.step(&[]);
    assert!(world.observe(sera).unwrap().work.is_none());
    let framed = world.examine(sera, ExamineTarget::Frame);
    assert!(framed.contains("supported at both ends"), "{framed}");
    world.step(&[cmd("lay", sera, Attempt::LayBark)]);
    world.step(&[]);
    assert_eq!(world.exposure_at(4, 4), Some(50));
    assert_eq!(world.exposure_at(5, 2), Some(1000));

    while world.sim_time() < 104 {
        world.step(&[]);
    }
    let frame = world.examine(sera, ExamineTarget::Frame);
    let wood = world.examine(sera, ExamineTarget::Wood);
    assert!(frame.contains("The wood in the frame is dry."), "{frame}");
    assert!(
        frame.contains("under a roof")
            || frame.contains("under a bark roof")
            || wood.contains("under a bark roof")
            || frame.contains("Bark lies over it"),
        "{frame}"
    );
    assert!(wood.contains("soaked"), "{wood}");
    assert_eq!(world.material_tally(Material::Branch), 8);
    assert_eq!(world.material_tally(Material::Bark), 4);
}

#[test]
fn snapshot_roundtrip_keeps_grips_schedule_and_future_rain() {
    let mut world = world_at(12);
    let sera = world.spawn_at("Sera", "s", 3, 2).unwrap();
    let otto = world.spawn_at("Otto", "o", 3, 2).unwrap();
    world.step(&[
        cmd("g1", sera, Attempt::Grab),
        cmd("g2", otto, Attempt::Grab),
    ]);
    world.step(&[
        cmd("d1", sera, Attempt::Drag(Dir::North)),
        cmd("d2", otto, Attempt::Drag(Dir::North)),
    ]);
    let saved = serde_json::to_string(&world).unwrap();
    let mut restored: World = serde_json::from_str(&saved).unwrap();
    assert_eq!(world, restored);
    assert_eq!(restored.log_gripped_by().len(), 2);
    restored.step(&[]);
    world.step(&[]);
    assert_eq!(world, restored);
    assert_eq!(world.log_cell(), Some((3, 1)));
}

#[test]
fn thirst_rises_without_a_command_and_the_brook_eases_it() {
    let mut world = world_at(1_000);
    let sera = world.spawn_at("Sera", "s", 0, 1).unwrap();
    let otto = world.spawn_at("Otto", "o", 4, 3).unwrap();
    for _ in 0..30 {
        world.step(&[]);
    }
    assert_eq!(world.observe(sera).unwrap().self_info.thirst, "thirsty");
    assert_eq!(world.observe(otto).unwrap().self_info.thirst, "thirsty");
    assert!(
        !world.observe(sera).unwrap().text.contains("thirst:"),
        "raw thirst stays off the observation"
    );

    world.step(&[cmd("away", otto, Attempt::Drink)]);
    let refused = world
        .observe(otto)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text.contains("brook is not here"));
    assert!(refused, "otto is not standing in the brook");
    assert_eq!(world.observe(otto).unwrap().self_info.thirst, "thirsty");

    world.step(&[cmd("sip", sera, Attempt::Drink)]);
    let eased = world.observe(sera).unwrap();
    assert_eq!(eased.self_info.thirst, "not thirsty");
    assert!(eased.events.iter().any(|event| event.text.contains("drinks from the brook")));
    assert!(
        world
            .observe(otto)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("drinks from the brook")),
        "a distant character does not see the drink"
    );

    let mut rained = world_at(0);
    let linde = rained.spawn_at("Linde", "l", 4, 4).unwrap();
    for _ in 0..30 {
        rained.step(&[]);
    }
    assert!(rained.is_raining());
    assert_eq!(rained.observe(linde).unwrap().self_info.thirst, "thirsty");
    assert_eq!(rained.observe(linde).unwrap().self_info.wetness, "soaked");
}

#[test]
fn parser_accepts_the_evening_commands() {
    assert!(matches!(parse_attempt("look").unwrap(), Attempt::Look));
    assert!(matches!(
        parse_attempt("drag east").unwrap(),
        Attempt::Drag(Dir::East)
    ));
    assert!(matches!(
        parse_attempt("gather branches").unwrap(),
        Attempt::Gather(Material::Branch)
    ));
    assert!(matches!(
        parse_attempt("raise frame").unwrap(),
        Attempt::RaiseFrame
    ));
    assert!(matches!(
        parse_attempt("lay bark").unwrap(),
        Attempt::LayBark
    ));
    assert!(matches!(parse_attempt("drink").unwrap(), Attempt::Drink));
    assert!(matches!(
        parse_attempt("Say The wood is still dry.").unwrap(),
        Attempt::Say(text) if text == "The wood is still dry."
    ));
    assert!(parse_attempt("fly").is_err());
}
