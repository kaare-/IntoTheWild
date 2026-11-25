use villmarks_sim::{
    contest_hash, parse_attempt, Attempt, Dir, ExamineTarget, Intent, Material, Tuning, World,
};

fn rock_berry_pile(world: &World, actor: u32) -> u32 {
    world
        .observe(actor)
        .unwrap()
        .visible_cells
        .iter()
        .find(|cell| cell.x == 6 && cell.y == 1)
        .map(|cell| cell.berry_pile)
        .unwrap_or(0)
}

fn world_at(rain_at: u64) -> World {
    let mut tuning = Tuning::evening();
    tuning.rain_starts_at = rain_at;
    World::valley(tuning, 7)
}

fn pair(
    world: &mut World,
    id: &str,
    near: u32,
    near_attempt: Attempt,
    west: u32,
    west_attempt: Attempt,
) -> villmarks_sim::StepReport {
    world.step(&[
        cmd(&format!("n-{id}"), near, near_attempt),
        cmd(&format!("w-{id}"), west, west_attempt),
    ])
}

fn relight_charcoal(world: &mut World, actor: u32) {
    let mut waits = 0;
    while !world
        .observe(actor)
        .unwrap()
        .text
        .contains("A lump of charcoal lies here")
    {
        world.step(&[]);
        waits += 1;
        assert!(waits < 20, "{}", world.observe(actor).unwrap().text);
    }
    let gathered = world.step(&[cmd(
        "charcoal-lump",
        actor,
        Attempt::Gather(Material::Charcoal),
    )]);
    assert!(
        gathered.results[0].success,
        "{}",
        gathered.results[0].message
    );
    let lit = world.step(&[cmd("charcoal-light", actor, Attempt::Light)]);
    assert!(lit.results[0].success, "{}", lit.results[0].message);
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
    assert_eq!(world.material_tally(Material::Berry), 4);
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
    let gathers = world
        .event_log()
        .iter()
        .filter(|event| event.text.contains("gathers a"))
        .count();
    assert_eq!(tally_carry(&world, sera), 1);
    world.step(&[cmd("c1", sera, Attempt::Gather(Material::Branch))]);
    assert_eq!(tally_carry(&world, sera), 1);
    assert_eq!(
        world
            .event_log()
            .iter()
            .filter(|event| event.text.contains("gathers a"))
            .count(),
        gathers
    );
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
    assert_eq!(world.material_tally(Material::Branch), 6);
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
    assert!(text.contains("dry"), "{text}");
    assert!(text.contains("in contact with the ground"), "{text}");
    assert!(text.contains("under the rock overhang"), "{text}");
    assert!(text.contains("Rain does not reach it."), "{text}");
    assert!(!text.contains("Rain reaches it."), "{text}");
    let log = world.examine(near, ExamineTarget::Log);
    assert!(log.contains("soaked"), "{log}");
    assert!(log.contains("Rain reaches it."), "{log}");

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
    // A second meal under the rock would hold the deer and change this count.
    world.tuning.berry_ticks = 200;
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
    let seen = world.observe(sera).unwrap();
    assert!(frame.contains("The wood in the frame is dry."), "{frame}");
    assert!(
        frame.contains("under a roof")
            || frame.contains("under a bark roof")
            || seen.text.contains("under a bark roof")
            || frame.contains("Bark lies over it"),
        "{frame}"
    );
    let open = seen
        .visible_cells
        .iter()
        .find(|cell| cell.x == 5 && cell.y == 2)
        .unwrap();
    assert_eq!(open.branch_pile, 0, "open branches rot away");
    assert_eq!(
        world.material_tally(Material::Branch),
        6,
        "three branches in the frame, and three of the dry pile under the rock"
    );
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

    let away = world.step(&[cmd("away", otto, Attempt::Drink)]);
    assert_eq!(away.results[0].message, "The brook is not here.");
    assert!(world
        .observe(otto)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != "The brook is not here."));
    assert_eq!(world.observe(otto).unwrap().self_info.thirst, "thirsty");

    let sip = world.step(&[cmd("sip", sera, Attempt::Drink)]);
    assert!(
        sip.results[0]
            .message
            .starts_with("You drink from the brook."),
        "{}",
        sip.results[0].message
    );
    let eased = world.observe(sera).unwrap();
    assert_eq!(eased.self_info.thirst, "not thirsty");
    assert!(eased
        .events
        .iter()
        .all(|event| !event.text.contains("Sera drinks from the brook")));
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
fn thirst_and_hunger_are_first_person_for_that_body() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
    let quiet = world.spawn_at("Quiet", "q", 0, 0).unwrap();
    for _ in 0..25 {
        world.step(&[]);
    }
    assert_eq!(world.observe(near).unwrap().self_info.thirst, "thirsty");
    assert_eq!(world.observe(near).unwrap().self_info.hunger, "hungry");
    let near_look = world.observe(near).unwrap();
    assert!(near_look
        .events
        .iter()
        .any(|event| event.text == "Thirst is in your mouth."));
    assert!(near_look
        .events
        .iter()
        .any(|event| event.text == "Hunger is in your stomach."));
    assert!(near_look.events.iter().all(|event| {
        !event.text.contains("Near's") && !event.text.contains("Thirst is in Near")
    }));
    assert!(near_look
        .events
        .iter()
        .any(|event| event.text == "Thirst is in Seen's mouth."));
    assert!(near_look
        .events
        .iter()
        .any(|event| event.text == "Hunger is in Seen's stomach."));
    let seen_look = world.observe(seen).unwrap();
    assert!(seen_look
        .events
        .iter()
        .any(|event| event.text == "Thirst is in your mouth."));
    assert!(seen_look
        .events
        .iter()
        .any(|event| event.text == "Thirst is in Near's mouth."));
    assert!(seen_look
        .events
        .iter()
        .all(|event| !event.text.contains("Seen's")));
    let quiet_look = world.observe(quiet).unwrap();
    assert!(quiet_look
        .events
        .iter()
        .any(|event| event.text == "Thirst is in your mouth."));
    assert!(quiet_look
        .events
        .iter()
        .any(|event| event.text == "Hunger is in your stomach."));
    assert!(quiet_look
        .events
        .iter()
        .all(|event| { !event.text.contains("Near") && !event.text.contains("Seen") }));
}

#[test]
fn tiredness_is_first_person_for_that_body() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
    let quiet = world.spawn_at("Quiet", "q", 0, 0).unwrap();
    let mut ticks = 0;
    while world.observe(near).unwrap().self_info.energy != "tiring" {
        world.step(&[]);
        ticks += 1;
        assert!(ticks < 40, "tiredness should arrive");
    }
    let near_look = world.observe(near).unwrap();
    assert!(near_look.text.contains("You are tiring."));
    assert!(near_look
        .events
        .iter()
        .any(|event| event.text == "You are tiring."));
    assert!(near_look
        .events
        .iter()
        .all(|event| event.text != "Near is tiring."));
    assert!(near_look
        .events
        .iter()
        .any(|event| event.text == "Seen is tiring."));
    let seen_look = world.observe(seen).unwrap();
    assert!(seen_look
        .events
        .iter()
        .any(|event| event.text == "You are tiring."));
    assert!(seen_look
        .events
        .iter()
        .any(|event| event.text == "Near is tiring."));
    assert!(seen_look
        .events
        .iter()
        .all(|event| event.text != "Seen is tiring."));
    let quiet_look = world.observe(quiet).unwrap();
    assert!(quiet_look
        .events
        .iter()
        .any(|event| event.text == "You are tiring."));
    assert!(quiet_look
        .events
        .iter()
        .all(|event| { event.text != "Near is tiring." && event.text != "Seen is tiring." }));

    while world.observe(near).unwrap().self_info.energy != "spent" {
        world.step(&[]);
        ticks += 1;
        assert!(ticks < 80, "spent should arrive");
    }
    let spent = world.observe(near).unwrap();
    assert!(spent.text.contains("You are spent."));
    assert!(spent
        .events
        .iter()
        .any(|event| event.text == "You are spent."));
    assert!(spent
        .events
        .iter()
        .all(|event| event.text != "Near is spent."));
    let seen_spent = world.observe(seen).unwrap();
    assert!(seen_spent
        .events
        .iter()
        .any(|event| event.text == "Near is spent."));
    assert!(seen_spent
        .events
        .iter()
        .any(|event| event.text == "You are spent."));
    assert!(seen_spent
        .events
        .iter()
        .all(|event| event.text != "Seen is spent."));
    assert!(world
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| { event.text != "Near is spent." && event.text != "Seen is spent." }));
    assert!(world
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "You are spent."));
}

#[test]
fn hunger_rises_without_a_command_and_a_fire_cooks_berries() {
    let mut world = world_at(1_000);
    let sera = world.spawn_at("Sera", "s", 6, 1).unwrap();
    let otto = world.spawn_at("Otto", "o", 2, 3).unwrap();
    let bark = world.material_tally(Material::Bark);
    let berries = world.material_tally(Material::Berry);
    assert_eq!(berries, 4);

    world.step(&[cmd("raw", sera, Attempt::Cook)]);
    assert!(world
        .observe(sera)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("no fire")));
    assert!(world
        .observe(otto)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("no fire")));
    assert_eq!(world.material_tally(Material::Berry), berries);
    assert_eq!(world.material_tally(Material::Bark), bark);

    world.step(&[cmd("pocket", sera, Attempt::Gather(Material::Berry))]);
    assert_eq!(world.material_tally(Material::Berry), berries);
    while world.sim_time() < 30 {
        world.step(&[]);
    }
    assert_eq!(world.observe(sera).unwrap().self_info.hunger, "hungry");
    assert_eq!(world.observe(otto).unwrap().self_info.hunger, "hungry");
    let prose = world.observe(sera).unwrap().text;
    assert!(
        !prose.contains("hunger:"),
        "raw hunger stays off the observation"
    );
    assert!(!prose.contains("0.45"));
    assert_eq!(
        world.material_tally(Material::Berry),
        2,
        "the deer eats the pile; the carried handful and the one it left remain"
    );

    world.step(&[cmd("none", otto, Attempt::Light)]);
    assert_eq!(world.material_tally(Material::Bark), bark);
    assert_eq!(world.observe(otto).unwrap().self_info.hunger, "hungry");

    assert!(world
        .observe(sera)
        .unwrap()
        .allowed_actions
        .iter()
        .any(|action| action == "light fire"));
    let branches_before_light = world.material_tally(Material::Branch);
    world.step(&[cmd("tinder", sera, Attempt::Light)]);
    let lit = world.observe(sera).unwrap();
    assert_eq!(lit.self_info.hunger, "hungry", "bark is tinder, not a meal");
    assert!(lit.map.contains('@') && lit.map.contains('*'));
    assert!(lit.text.contains("A small fire burns here."));
    assert!(lit.events.iter().all(|event| {
        !event
            .text
            .contains("You light a fire with bark and a dry branch from the pile at the feet")
            && !event.text.contains("lights a fire")
    }));
    assert_eq!(world.material_tally(Material::Bark), bark - 1);
    assert_eq!(
        world.material_tally(Material::Branch),
        branches_before_light - 1
    );
    assert_eq!(world.material_tally(Material::Berry), 2);
    let here = lit
        .visible_cells
        .iter()
        .find(|cell| cell.x == 6 && cell.y == 1)
        .unwrap();
    assert!(here.fire);
    assert_eq!(here.berry_pile, 0);

    assert!(lit.allowed_actions.iter().any(|action| action == "cook"));
    world.step(&[cmd("meal", sera, Attempt::Cook)]);
    let eased = world.observe(sera).unwrap();
    assert_eq!(eased.self_info.hunger, "not hungry");
    assert!(eased.events.iter().all(|event| {
        !event.text.contains("Sera cooks a handful of berries")
            && event.text != "You cook a handful of berries. You are not hungry."
            && !event.text.contains("You cook a handful of berries")
    }));
    assert_eq!(
        world.material_tally(Material::Berry),
        5,
        "the carried handful is cooked, the deer left one, and the patch grew back the same step"
    );
    assert_eq!(world.material_tally(Material::Bark), bark - 1);
    assert!(world
        .observe(otto)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("cooks a handful of berries")));

    let mut rained = world_at(0);
    let linde = rained.spawn_at("Linde", "l", 4, 4).unwrap();
    for _ in 0..30 {
        rained.step(&[]);
    }
    assert!(rained.is_raining());
    assert_eq!(rained.observe(linde).unwrap().self_info.hunger, "hungry");
    assert_eq!(rained.observe(linde).unwrap().self_info.wetness, "soaked");
}

#[test]
fn damp_bark_will_not_catch_and_open_rain_puts_a_fire_out() {
    let mut slope = world_at(3);
    let sera = slope.spawn_at("Sera", "s", 6, 1).unwrap();
    slope.step(&[cmd("sheet", sera, Attempt::Gather(Material::Bark))]);
    slope.step(&[cmd("sheet-2", sera, Attempt::Gather(Material::Bark))]);
    slope.step(&[cmd("out", sera, Attempt::Move(Dir::West))]);
    slope.step(&[]);
    slope.step(&[]);
    slope.step(&[cmd("back", sera, Attempt::Move(Dir::East))]);
    let bark = slope.material_tally(Material::Bark);
    let damp = slope.step(&[cmd("damp", sera, Attempt::Light)]);
    let moss_line = "You light a fire with moss and a dry branch from the pile at the feet.";
    assert_eq!(damp.results[0].message, moss_line);
    let seen: Vec<_> = slope
        .observe(sera)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .collect();
    assert!(
        seen.iter()
            .all(|text| !text.contains("too damp") && text != moss_line),
        "{seen:?}"
    );
    assert_eq!(slope.material_tally(Material::Bark), bark);
    assert!(slope
        .observe(sera)
        .unwrap()
        .text
        .contains("The shade here has no moss."));

    let mut open = world_at(4);
    let nils = open.spawn_at("Nils", "n", 6, 1).unwrap();
    open.step(&[cmd("branch", nils, Attempt::Gather(Material::Branch))]);
    open.step(&[cmd("bark", nils, Attempt::Gather(Material::Bark))]);
    open.step(&[cmd("west", nils, Attempt::Move(Dir::West))]);
    open.step(&[cmd("light", nils, Attempt::Light)]);
    assert!(!open.is_raining());
    assert!(open
        .observe(nils)
        .unwrap()
        .text
        .contains("A small fire burns here."));
    open.step(&[]);
    assert!(open.is_raining());
    assert!(open
        .observe(nils)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text.contains("Rain puts out the fire.")));
    assert!(!open
        .observe(nils)
        .unwrap()
        .text
        .contains("A small fire burns here."));

    let mut rock = world_at(1);
    let otto = rock.spawn_at("Otto", "o", 6, 1).unwrap();
    rock.step(&[cmd("light", otto, Attempt::Light)]);
    rock.step(&[]);
    assert!(rock.is_raining());
    assert!(rock
        .observe(otto)
        .unwrap()
        .text
        .contains("A small fire burns here."));
    assert!(rock
        .observe(otto)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("puts out")));

    let mut brief = world_at(1_000);
    let linde = brief.spawn_at("Linde", "l", 6, 1).unwrap();
    let life = brief.tuning.fire_ticks;
    brief.step(&[cmd("light", linde, Attempt::Light)]);
    for _ in 1..life {
        brief.step(&[]);
    }
    assert!(brief
        .observe(linde)
        .unwrap()
        .text
        .contains("A small fire burns here."));
    brief.step(&[]);
    assert!(brief
        .observe(linde)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text.contains("The fire burns down.")));
    assert!(!brief
        .observe(linde)
        .unwrap()
        .text
        .contains("A small fire burns here."));
}

#[test]
fn rain_names_the_fire_it_does_not_reach() {
    let mut world = world_at(16);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let far = world.spawn_at("Far", "f", 0, 0).unwrap();
    for (id, dir) in [
        ("east-1", Dir::East),
        ("east-2", Dir::East),
        ("north-1", Dir::North),
        ("north-2", Dir::North),
    ] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    assert_eq!(world.sim_time(), 4);
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    let lit = world.step(&[cmd("light", near, Attempt::Light)]);
    assert_eq!(
        lit.results[0].message,
        "You light a fire with bark and a dry branch from the pile at the feet."
    );
    assert_eq!(world.sim_time(), 5);
    let at_five = world.observe(near).unwrap();
    assert!(at_five
        .text
        .contains("A small fire burns here. It warms you."));
    assert!(at_five.map.contains("@*+"));
    assert!(!at_five.text.contains("The rain does not reach the fire."));
    assert!(at_five
        .events
        .iter()
        .all(|event| !event.text.contains("The rain does not reach the fire.")));
    while world.sim_time() < 13 {
        world.step(&[]);
    }
    let at_thirteen = world.observe(near).unwrap();
    assert!(!world.is_raining());
    assert!(at_thirteen
        .text
        .contains("A small fire burns here. It warms you."));
    assert!(at_thirteen.map.contains("@*+"));
    assert!(at_thirteen
        .events
        .iter()
        .all(|event| !event.text.contains("The fire burns down.")));
    assert!(!at_thirteen
        .text
        .contains("The rain does not reach the fire."));
    while world.sim_time() < 17 {
        world.step(&[]);
    }
    assert!(world.is_raining());
    let at_seventeen = world.observe(near).unwrap();
    let place = at_seventeen
        .text
        .lines()
        .find(|line| line.starts_with("You are on the"))
        .unwrap_or("");
    assert!(place.contains("Rock leans overhead, and the rain does not reach this ground."));
    assert!(place.contains("A small fire burns here. It warms you."));
    assert!(!place.contains("The rain does not reach the fire."));
    assert!(at_seventeen.map.contains("@*+"));
    let rain = at_seventeen
        .events
        .iter()
        .find(|event| event.text.contains("Rain reaches"))
        .map(|event| event.text.as_str())
        .unwrap_or("");
    assert_eq!(
        rain,
        "Rain reaches the valley and finds whatever was left in the open. The rain does not reach the fire."
    );
    assert!(at_seventeen
        .events
        .iter()
        .all(|event| !event.text.contains("The fire burns down.")));
    assert!(at_seventeen
        .events
        .iter()
        .all(|event| !event.text.contains("puts out")));
    let unseen = world.observe(far).unwrap();
    assert!(unseen
        .events
        .iter()
        .all(|event| !event.text.contains("The rain does not reach the fire.")));
    assert!(unseen
        .events
        .iter()
        .any(|event| event.text == "The rain wets your clothes."));
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
    assert!(matches!(parse_attempt("boil").unwrap(), Attempt::Boil));
    assert!(matches!(
        parse_attempt("gather water").unwrap(),
        Attempt::Gather(Material::Water)
    ));
    assert!(matches!(
        parse_attempt("gather stone").unwrap(),
        Attempt::Gather(Material::Stone)
    ));
    assert!(matches!(
        parse_attempt("drop stone").unwrap(),
        Attempt::Drop(Material::Stone)
    ));
    assert!(matches!(
        parse_attempt("examine stone").unwrap(),
        Attempt::Examine(ExamineTarget::Stone)
    ));
    assert!(matches!(
        parse_attempt("gather clay").unwrap(),
        Attempt::Gather(Material::Clay)
    ));
    assert!(matches!(
        parse_attempt("set clay").unwrap(),
        Attempt::SetClay
    ));
    assert!(matches!(
        parse_attempt("bind spear").unwrap(),
        Attempt::BindSpear
    ));
    assert!(matches!(parse_attempt("knap").unwrap(), Attempt::Knap));
    assert!(matches!(parse_attempt("haft").unwrap(), Attempt::Haft));
    assert!(matches!(
        parse_attempt("examine spear").unwrap(),
        Attempt::Examine(ExamineTarget::Spear)
    ));
    assert!(matches!(parse_attempt("hunt").unwrap(), Attempt::Hunt));
    assert!(matches!(parse_attempt("hunt deer").unwrap(), Attempt::Hunt));
    assert!(parse_attempt("kill deer").is_err());
    assert!(matches!(
        parse_attempt("examine bowl").unwrap(),
        Attempt::Examine(ExamineTarget::Bowl)
    ));
    assert!(matches!(
        parse_attempt("light fire").unwrap(),
        Attempt::Light
    ));
    assert!(matches!(
        parse_attempt("take brand").unwrap(),
        Attempt::TakeBrand
    ));
    assert!(matches!(
        parse_attempt("take ember").unwrap(),
        Attempt::TakeBrand
    ));
    assert!(matches!(
        parse_attempt("take a burning brand").unwrap(),
        Attempt::TakeBrand
    ));
    assert!(matches!(
        parse_attempt("examine brand").unwrap(),
        Attempt::Examine(ExamineTarget::Brand)
    ));
    assert!(matches!(
        parse_attempt("take bark").unwrap(),
        Attempt::Gather(Material::Bark)
    ));
    assert!(matches!(parse_attempt("cook").unwrap(), Attempt::Cook));
    assert!(matches!(
        parse_attempt("examine deer").unwrap(),
        Attempt::Examine(ExamineTarget::Deer)
    ));
    assert!(matches!(
        parse_attempt("examine tracks").unwrap(),
        Attempt::Examine(ExamineTarget::Tracks)
    ));
    assert!(matches!(
        parse_attempt("examine rock").unwrap(),
        Attempt::Examine(ExamineTarget::Rock)
    ));
    assert!(matches!(
        parse_attempt("examine overhang").unwrap(),
        Attempt::Examine(ExamineTarget::Rock)
    ));
    assert!(matches!(
        parse_attempt("examine rock overhang").unwrap(),
        Attempt::Examine(ExamineTarget::Rock)
    ));
    assert!(matches!(
        parse_attempt("examine slope").unwrap(),
        Attempt::Examine(ExamineTarget::Slope)
    ));
    assert!(matches!(
        parse_attempt("gather berries").unwrap(),
        Attempt::Gather(Material::Berry)
    ));
    assert!(matches!(
        parse_attempt("eat").unwrap(),
        Attempt::Eat(Material::Berry)
    ));
    assert!(matches!(
        parse_attempt("gather flint").unwrap(),
        Attempt::Gather(Material::Flint)
    ));
    assert!(matches!(
        parse_attempt("gather ochre").unwrap(),
        Attempt::Gather(Material::Ochre)
    ));
    assert!(matches!(
        parse_attempt("gather copper ore").unwrap(),
        Attempt::Gather(Material::CopperOre)
    ));
    assert!(matches!(
        parse_attempt("gather iron ore").unwrap(),
        Attempt::Gather(Material::IronOre)
    ));
    assert!(matches!(
        parse_attempt("examine iron").unwrap(),
        Attempt::Examine(ExamineTarget::IronOre)
    ));
    assert!(matches!(
        parse_attempt("gather bead").unwrap(),
        Attempt::Gather(Material::Copper)
    ));
    assert!(matches!(
        parse_attempt("gather copper").unwrap(),
        Attempt::Gather(Material::Copper)
    ));
    assert!(matches!(parse_attempt("smelt").unwrap(), Attempt::Smelt));
    assert!(matches!(
        parse_attempt("smelt copper ore").unwrap(),
        Attempt::Smelt
    ));
    assert!(matches!(
        parse_attempt("smelt iron").unwrap(),
        Attempt::SmeltIron
    ));
    assert!(matches!(parse_attempt("feed").unwrap(), Attempt::Feed));
    assert!(matches!(parse_attempt("feed fire").unwrap(), Attempt::Feed));
    assert!(matches!(
        parse_attempt("gather bloom of iron").unwrap(),
        Attempt::Gather(Material::Iron)
    ));
    assert!(matches!(
        parse_attempt("examine bloom").unwrap(),
        Attempt::Examine(ExamineTarget::Iron)
    ));
    assert!(matches!(
        parse_attempt("hammer bloom").unwrap(),
        Attempt::HammerIron
    ));
    assert!(matches!(
        parse_attempt("gather iron knife").unwrap(),
        Attempt::Gather(Material::Knife)
    ));
    assert!(matches!(
        parse_attempt("examine knife").unwrap(),
        Attempt::Examine(ExamineTarget::Knife)
    ));
    assert!(matches!(parse_attempt("hammer").unwrap(), Attempt::Hammer));
    assert!(matches!(
        parse_attempt("forge bead").unwrap(),
        Attempt::Hammer
    ));
    assert!(matches!(
        parse_attempt("pierce hide").unwrap(),
        Attempt::Pierce
    ));
    assert!(matches!(
        parse_attempt("scrape hide").unwrap(),
        Attempt::Scrape
    ));
    assert!(matches!(parse_attempt("scrape").unwrap(), Attempt::Scrape));
    assert!(matches!(parse_attempt("sew hide").unwrap(), Attempt::Sew));
    assert!(matches!(parse_attempt("sew").unwrap(), Attempt::Sew));
    assert!(matches!(parse_attempt("rest").unwrap(), Attempt::Rest));
    assert!(matches!(parse_attempt("sleep").unwrap(), Attempt::Rest));
    assert!(matches!(
        parse_attempt("smear ochre").unwrap(),
        Attempt::SmearOchre
    ));
    assert!(matches!(parse_attempt("mark").unwrap(), Attempt::Mark));
    assert!(matches!(
        parse_attempt("mark ground").unwrap(),
        Attempt::Mark
    ));
    assert!(matches!(
        parse_attempt("mark rock").unwrap(),
        Attempt::MarkRock
    ));
    assert!(matches!(parse_attempt("wear hide").unwrap(), Attempt::Wear));
    assert!(matches!(parse_attempt("remove").unwrap(), Attempt::Remove));
    assert!(matches!(
        parse_attempt("remove hide").unwrap(),
        Attempt::Remove
    ));
    assert!(matches!(
        parse_attempt("eat mushrooms").unwrap(),
        Attempt::Eat(Material::Mushroom)
    ));
    assert!(matches!(
        parse_attempt("eat carcass").unwrap(),
        Attempt::Eat(Material::Carcass)
    ));
    assert!(matches!(
        parse_attempt("eat meat").unwrap(),
        Attempt::Eat(Material::Meat)
    ));
    assert!(matches!(
        parse_attempt("eat bark").unwrap(),
        Attempt::Eat(Material::Bark)
    ));
    assert!(matches!(
        parse_attempt("eat sheets of bark").unwrap(),
        Attempt::Eat(Material::Bark)
    ));
    assert!(matches!(
        parse_attempt("Say The wood is still dry.").unwrap(),
        Attempt::Say(text) if text == "The wood is still dry."
    ));
    assert!(parse_attempt("fly").is_err());
}

fn illness_world(chance: u32, recovery: u32) -> World {
    let mut tuning = Tuning::evening();
    tuning.rain_starts_at = 10_000;
    tuning.illness_chance = chance;
    tuning.illness_hit = 600;
    tuning.ill_at = 250;
    tuning.illness_recovery = recovery;
    World::valley(tuning, 7)
}

#[test]
fn raw_brook_water_records_a_draw_and_time_clears_the_illness() {
    let hash = contest_hash(7, 0, "drink", "7:brook", 1);
    let roll = (hash % 1000) as u32;

    let mut safe = illness_world(roll, 0);
    let sera = safe.spawn_at("Sera", "s", 0, 1).unwrap();
    safe.step(&[cmd("sip", sera, Attempt::Drink)]);
    assert_eq!(safe.observe(sera).unwrap().self_info.illness, "well");
    let draw = safe
        .event_log()
        .iter()
        .find_map(|event| event.draw.clone())
        .unwrap();
    assert_eq!(draw.tag, "drink");
    assert_eq!(draw.value, hash);

    let mut sick = illness_world(roll + 1, 0);
    let sera = sick.spawn_at("Sera", "s", 0, 1).unwrap();
    sick.step(&[cmd("sip", sera, Attempt::Drink)]);
    assert_eq!(sick.observe(sera).unwrap().self_info.illness, "ill");

    let mut world = illness_world(1000, 100);
    let sera = world.spawn_at("Sera", "s", 0, 1).unwrap();
    let nils = world.spawn_at("Nils", "n", 6, 3).unwrap();
    world.step(&[cmd("sip", sera, Attempt::Drink)]);
    let ill = world.observe(sera).unwrap();
    assert_eq!(ill.self_info.illness, "ill");
    assert!(ill.text.contains("You are ill."));
    assert!(!ill.text.contains("illness:"));
    assert!(!ill.text.contains("0.60"));
    assert!(ill
        .events
        .iter()
        .any(|event| event.text.contains("stomach turns")));
    assert!(world
        .observe(nils)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("stomach")
            && !event.text.contains("drinks from the brook")));
    assert_eq!(
        world
            .event_log()
            .iter()
            .filter(|event| event.draw.as_ref().is_some_and(|draw| draw.tag == "drink"))
            .count(),
        1
    );

    world.step(&[cmd("sip", sera, Attempt::Drink)]);
    assert_eq!(
        world
            .event_log()
            .iter()
            .filter(|event| event.draw.as_ref().is_some_and(|draw| draw.tag == "drink"))
            .count(),
        1
    );

    let mut ticks = 0;
    while world.observe(sera).unwrap().self_info.illness == "ill" && ticks < 30 {
        world.step(&[]);
        ticks += 1;
    }
    assert!(ticks > 0 && ticks < 20, "illness faded after {ticks} ticks");
    assert_eq!(world.observe(sera).unwrap().self_info.illness, "well");
    assert!(world
        .observe(sera)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text.contains("stomach settles")));

    let mut ordered = illness_world(1000, 0);
    let mut swapped = illness_world(1000, 0);
    let left = ordered.spawn_at("Sera", "s", 0, 1).unwrap();
    let right = ordered.spawn_at("Otto", "o", 1, 1).unwrap();
    let left_b = swapped.spawn_at("Sera", "s", 0, 1).unwrap();
    let right_b = swapped.spawn_at("Otto", "o", 1, 1).unwrap();
    ordered.step(&[
        cmd("a", left, Attempt::Drink),
        cmd("b", right, Attempt::Drink),
    ]);
    swapped.step(&[
        cmd("b", right_b, Attempt::Drink),
        cmd("a", left_b, Attempt::Drink),
    ]);
    assert_eq!(ordered, swapped);
}

#[test]
fn boiled_water_eases_thirst_and_does_not_sicken() {
    let mut world = illness_world(1000, 0);
    let sera = world.spawn_at("Sera", "s", 0, 1).unwrap();
    let otto = world.spawn_at("Otto", "o", 1, 1).unwrap();
    for _ in 0..30 {
        world.step(&[]);
    }
    assert_eq!(world.observe(sera).unwrap().self_info.thirst, "thirsty");
    assert_eq!(world.observe(sera).unwrap().self_info.illness, "well");

    world.step(&[cmd("raw", otto, Attempt::Drink)]);
    assert_eq!(world.observe(otto).unwrap().self_info.illness, "ill");

    world.step(&[cmd("early", sera, Attempt::Boil)]);
    assert!(world
        .observe(sera)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text.contains("no fire")));
    assert_eq!(world.material_tally(Material::Water), 0);
    assert_eq!(world.observe(sera).unwrap().self_info.illness, "well");

    world.step(&[cmd("fill", sera, Attempt::Gather(Material::Water))]);
    assert_eq!(world.material_tally(Material::Water), 1);
    for (n, _) in (0..6).enumerate() {
        world.step(&[cmd(&format!("east-{n}"), sera, Attempt::Move(Dir::East))]);
    }
    assert_eq!(world.actor_cell(sera), Some((6, 1)));
    world.step(&[cmd("dry", sera, Attempt::Gather(Material::Water))]);
    assert_eq!(world.material_tally(Material::Water), 1);
    world.step(&[cmd("light", sera, Attempt::Light)]);
    world.step(&[cmd("nopot", sera, Attempt::Boil)]);
    assert!(world
        .observe(sera)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text.contains("no pot")));
    assert_eq!(world.material_tally(Material::Water), 1);
    assert_eq!(world.observe(sera).unwrap().self_info.illness, "well");
    world.step(&[cmd("getpot", sera, Attempt::Gather(Material::Pot))]);
    assert_eq!(world.material_tally(Material::Pot), 1);
    world.step(&[cmd("pot", sera, Attempt::Boil)]);
    let boiled = world.observe(sera).unwrap();
    assert_eq!(boiled.self_info.illness, "well");
    assert_eq!(boiled.self_info.thirst, "not thirsty");
    assert!(boiled.events.iter().all(|event| {
        !event.text.contains("You boil") && !event.text.contains("boils the drink")
    }));
    assert!(boiled
        .events
        .iter()
        .all(|event| !event.text.contains("stomach turns")));
    assert_eq!(world.material_tally(Material::Water), 0);
    assert_eq!(world.observe(otto).unwrap().self_info.illness, "ill");
    assert!(world
        .event_log()
        .iter()
        .filter(|event| event.text.contains("boils"))
        .all(|event| event.draw.is_none()));
}

#[test]
fn a_carried_pot_is_named_when_water_is_gathered_and_boiled() {
    let mut bare = world_at(10_000);
    let hands = bare.spawn_at("Near", "n", 1, 1).unwrap();
    let gathered = bare.step(&[cmd("bare", hands, Attempt::Gather(Material::Water))]);
    assert_eq!(
        gathered.results[0].message,
        "You gather a drink of water from the brook."
    );
    assert!(!gathered.results[0].message.contains("pot"));
    assert!(bare
        .observe(hands)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("drink of water")));
    let bare_arms = arms_sentence(&bare.observe(hands).unwrap().text);
    assert!(bare_arms.contains("a drink of water"), "{bare_arms}");
    assert!(!bare_arms.contains("1 water"), "{bare_arms}");
    assert!(!bare_arms.contains("pot"));

    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    world.step(&[cmd("pot", near, Attempt::Gather(Material::Pot))]);
    assert_eq!(
        arms_sentence(&world.observe(near).unwrap().text),
        "You have a dry pot."
    );
    for step in 0..5 {
        world.step(&[cmd(&format!("west-{step}"), near, Attempt::Move(Dir::West))]);
    }
    assert_eq!(world.actor_cell(near), Some((1, 1)));
    let filled = world.step(&[cmd("fill", near, Attempt::Gather(Material::Water))]);
    assert_eq!(
        filled.results[0].message,
        "You gather a drink of water from the brook into the pot."
    );
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("drink of water")));
    let filled_arms = arms_sentence(&world.observe(near).unwrap().text);
    assert_eq!(
        filled_arms,
        "You have a pot with a drink of raw water in it."
    );
    assert!(!filled_arms.contains("dry pot"));
    assert!(!filled_arms.contains("1 water"));
    for step in 0..5 {
        world.step(&[cmd(&format!("east-{step}"), near, Attempt::Move(Dir::East))]);
    }
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    world.step(&[cmd("light", near, Attempt::Light)]);
    let boiled = world.step(&[cmd("boil", near, Attempt::Boil)]);
    assert_eq!(
        boiled.results[0].message,
        "You boil the drink in the pot over the fire and drink it."
    );
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("boil")));
    assert!(world
        .material_items(Material::Pot)
        .iter()
        .any(|item| item.1 == Some(near)));
    assert!(world.material_items(Material::Water).is_empty());
    let body = world.observe(near).unwrap();
    assert_eq!(body.self_info.thirst, "not thirsty");
    assert_eq!(body.self_info.illness, "well");
    assert!(body.text.contains("A small fire burns here."));
    let empty_again = arms_sentence(&body.text);
    assert_eq!(empty_again, "You have a dry pot.");
    assert!(!empty_again.contains("water"));
    assert!(!empty_again.contains("raw"));
}

#[test]
fn boiling_a_drink_is_one_sentence_for_each_pair_of_eyes() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    let far = world.spawn_at("Far", "f", 0, 0).unwrap();
    world.step(&[cmd("pot", near, Attempt::Gather(Material::Pot))]);
    for step in 0..5 {
        world.step(&[cmd(&format!("west-{step}"), near, Attempt::Move(Dir::West))]);
    }
    world.step(&[cmd("fill", near, Attempt::Gather(Material::Water))]);
    for step in 0..5 {
        world.step(&[cmd(&format!("east-{step}"), near, Attempt::Move(Dir::East))]);
    }
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    world.step(&[cmd("light", near, Attempt::Light)]);
    let boiled = world.step(&[cmd("boil", near, Attempt::Boil)]);
    let sentence = "You boil the drink in the pot over the fire and drink it.";
    let witnessed = "Near boils the drink in the pot over the fire and drinks it.";
    assert_eq!(boiled.results[0].message, sentence);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("boil")));
    let seen: Vec<_> = world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("boil"))
        .collect();
    assert_eq!(seen, vec![witnessed.to_string()]);
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("boil")));
    assert!(world
        .event_log()
        .iter()
        .any(|event| event.text.starts_with(witnessed)));
}

#[test]
fn a_look_places_the_person_and_the_pot_that_are_still_under_the_rock() {
    let mut world = world_at(10_000);
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    let far = world.spawn_at("Far", "f", 0, 0).unwrap();
    let before = world.observe(west).unwrap().text;
    assert!(
        before.contains("A pot sits on the ground under the rock overhang."),
        "{before}"
    );
    assert!(!before.contains("Near stands"), "{before}");
    assert!(
        !world.observe(far).unwrap().text.contains("pot"),
        "Far cannot see the rock"
    );

    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let standing = world.observe(west).unwrap().text;
    assert!(
        standing.contains("Near stands on the valley floor under the rock overhang."),
        "{standing}"
    );
    assert!(
        standing.contains("A pot sits on the ground there."),
        "{standing}"
    );
    assert!(!standing.contains("1 person"), "{standing}");

    let report = world.step(&[cmd("pot", near, Attempt::Gather(Material::Pot))]);
    assert_eq!(
        report.results[0].message,
        "You gather a pot from the ground under the rock overhang."
    );
    let after = world.observe(west).unwrap().text;
    assert!(
        after.contains(
            "Near stands on the valley floor under the rock overhang, carrying a dry pot."
        ),
        "{after}"
    );
    assert!(
        !after.contains("sits on the ground"),
        "the pot is in the arms, not on the ground: {after}"
    );
    let arms = arms_sentence(&world.observe(near).unwrap().text);
    assert_eq!(arms, "You have a dry pot.");
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| { !event.text.contains("pot") && !event.text.contains("gather") }));
    assert!(world.observe(west).unwrap().events.iter().any(|event| {
        event.text == "Near gathers a pot from the ground under the rock overhang."
    }));
}

#[test]
fn lighting_a_fire_is_one_sentence_for_each_pair_of_eyes() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    let far = world.spawn_at("Far", "f", 0, 0).unwrap();
    let before = world.observe(west).unwrap().text;
    assert!(
        !before.contains("fire"),
        "no fire before it is lit: {before}"
    );
    let report = world.step(&[cmd("light", near, Attempt::Light)]);
    let sentence = "You light a fire with bark and a dry branch from the pile at the feet.";
    let witnessed = "Near lights a fire with bark and a dry branch from the pile at the feet.";
    assert_eq!(report.results[0].message, sentence);
    let yours: Vec<_> = world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("light"))
        .collect();
    assert!(yours.is_empty(), "{yours:?}");
    let seen: Vec<_> = world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("light"))
        .collect();
    assert_eq!(seen, vec![witnessed.to_string()]);
    let after = world.observe(west).unwrap().text;
    assert!(
        after.contains("A small fire burns there. It warms Near."),
        "{after}"
    );
    assert!(!after.contains("anyone standing"), "{after}");
    let own = world.observe(near).unwrap().text;
    assert!(
        own.contains("A small fire burns here. It warms you."),
        "{own}"
    );
    world.step(&[cmd("off", near, Attempt::Move(Dir::South))]);
    let empty = world.observe(west).unwrap().text;
    assert!(empty.contains("A small fire burns"), "{empty}");
    assert!(!empty.contains("warms"), "{empty}");
    assert!(
        !world.observe(far).unwrap().text.contains("A small fire"),
        "Far cannot see the fire"
    );
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("light")));
    assert!(world
        .event_log()
        .iter()
        .any(|event| event.text == witnessed));
}

#[test]
fn gathering_water_is_one_sentence_for_each_pair_of_eyes() {
    let mut bare = world_at(10_000);
    let near = bare.spawn_at("Near", "n", 0, 1).unwrap();
    let west = bare.spawn_at("West", "w", 2, 3).unwrap();
    let far = bare.spawn_at("Far", "f", 3, 3).unwrap();
    let gathered = bare.step(&[cmd("bare", near, Attempt::Gather(Material::Water))]);
    let sentence = "You gather a drink of water from the brook.";
    let witnessed = "Near gathers a drink of water from the brook.";
    assert_eq!(gathered.results[0].message, sentence);
    assert!(!sentence.contains("pot"));
    assert!(bare
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("drink of water")));
    let seen: Vec<_> = bare
        .observe(west)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("drink of water"))
        .collect();
    assert_eq!(seen, vec![witnessed.to_string()]);
    assert!(bare
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("drink of water")));

    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let west = world.spawn_at("West", "w", 2, 3).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    world.step(&[cmd("pot", near, Attempt::Gather(Material::Pot))]);
    for step in 0..6 {
        world.step(&[cmd(&format!("west-{step}"), near, Attempt::Move(Dir::West))]);
    }
    assert_eq!(world.actor_cell(near), Some((0, 1)));
    let filled = world.step(&[cmd("fill", near, Attempt::Gather(Material::Water))]);
    let sentence = "You gather a drink of water from the brook into the pot.";
    let witnessed = "Near gathers a drink of water from the brook into the pot.";
    assert_eq!(filled.results[0].message, sentence);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("drink of water")));
    let seen: Vec<_> = world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("drink of water"))
        .collect();
    assert_eq!(seen, vec![witnessed.to_string()]);
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("drink of water")));

    let mut pool = world_at(10_000);
    let near = pool.spawn_at("Near", "n", 2, 1).unwrap();
    let west = pool.spawn_at("West", "w", 2, 3).unwrap();
    let far = pool.spawn_at("Far", "f", 5, 3).unwrap();
    let pooled = pool.step(&[cmd("pool", near, Attempt::Gather(Material::Water))]);
    let sentence = "You gather a drink of water from the still water.";
    let witnessed = "Near gathers a drink of water from the still water.";
    assert_eq!(pooled.results[0].message, sentence);
    assert!(!sentence.contains("brook"));
    assert!(!witnessed.contains("brook"));
    assert!(pool
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("drink of water")));
    let seen: Vec<_> = pool
        .observe(west)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("drink of water"))
        .collect();
    assert_eq!(seen, vec![witnessed.to_string()]);
    assert!(pool
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("drink of water")));

    let mut held = world_at(10_000);
    let near = held.spawn_at("Near", "n", 6, 1).unwrap();
    held.step(&[cmd("pot", near, Attempt::Gather(Material::Pot))]);
    for step in 0..4 {
        held.step(&[cmd(
            &format!("pool-west-{step}"),
            near,
            Attempt::Move(Dir::West),
        )]);
    }
    assert_eq!(held.actor_cell(near), Some((2, 1)));
    held.step(&[cmd("pool-fill", near, Attempt::Gather(Material::Water))]);
    let still_arms = arms_sentence(&held.observe(near).unwrap().text);
    assert_eq!(still_arms, "You have a pot with a drink of water in it.");
    assert!(!still_arms.contains("raw"));
}

#[test]
fn gathering_a_pot_is_one_sentence_for_each_pair_of_eyes() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let west = world.spawn_at("West", "w", 4, 1).unwrap();
    let far = world.spawn_at("Far", "f", 0, 0).unwrap();
    let report = world.step(&[cmd("pot", near, Attempt::Gather(Material::Pot))]);
    assert_eq!(
        report.results[0].message,
        "You gather a pot from the ground under the rock overhang."
    );
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("pot") && !event.text.contains("gather")));
    let seen: Vec<_> = world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("pot") || text.contains("gather"))
        .collect();
    assert_eq!(
        seen,
        vec!["Near gathers a pot from the ground under the rock overhang.".to_string()]
    );
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| { !event.text.contains("pot") && !event.text.contains("gather") }));
    assert!(world.event_log().iter().any(|event| {
        event.text == "Near gathers a pot from the ground under the rock overhang."
    }));
}

fn arms_sentence(text: &str) -> String {
    let Some(start) = text.find("You have ") else {
        return String::new();
    };
    let rest = &text[start..];
    let end = rest.find('.').unwrap_or(rest.len() - 1);
    rest[..=end].to_string()
}

#[test]
fn a_deer_walks_without_a_command_eats_berries_and_leaves_hoofprints() {
    let mut near = world_at(10_000);
    let sera = near.spawn_at("Sera", "s", 4, 3).unwrap();
    let otto = near.spawn_at("Otto", "o", 0, 0).unwrap();
    assert_eq!(near.deer_cell(), Some((2, 3)));
    assert!(!near.hoofprints_at(2, 3));
    let before = near.observe(sera).unwrap();
    assert!(before.map.contains('d'));
    assert!(before
        .text
        .contains("A deer stands two cells west, in the open."));
    assert!(before
        .visible_cells
        .iter()
        .any(|cell| cell.x == 2 && cell.y == 3 && cell.deer));
    assert!(before
        .allowed_actions
        .iter()
        .any(|action| action == "examine deer"));
    assert!(near
        .examine(sera, ExamineTarget::Deer)
        .contains("A deer stands"));
    let distant = near.observe(otto).unwrap();
    assert!(!distant.map.contains('d'));
    assert!(distant.visible_cells.iter().all(|cell| !cell.deer));
    assert!(!distant.text.contains("deer"));
    assert!(near
        .examine(otto, ExamineTarget::Deer)
        .contains("cannot see a deer"));

    near.step(&[]);
    assert_eq!(near.deer_cell(), Some((3, 3)));
    assert!(near.hoofprints_at(2, 3));
    let moved = near.observe(sera).unwrap();
    assert!(moved
        .events
        .iter()
        .any(|event| event.text == "A deer steps east."));
    assert!(moved.text.contains("a deer"));
    assert!(!moved.text.contains("A deer stands one cell west"));
    assert!(moved
        .visible_cells
        .iter()
        .any(|cell| cell.x == 2 && cell.y == 3 && cell.tracks && !cell.deer));
    let missed = near.observe(otto).unwrap();
    assert!(missed
        .events
        .iter()
        .all(|event| !event.text.contains("deer")));
    assert!(missed
        .visible_cells
        .iter()
        .all(|cell| !cell.deer && !cell.tracks));
    assert!(!missed.map.contains('d'));
    assert!(!missed.map.contains('"'));

    let mut twin = world_at(10_000);
    twin.spawn_at("Sera", "s", 4, 3).unwrap();
    twin.spawn_at("Otto", "o", 0, 0).unwrap();
    twin.step(&[]);
    assert_eq!(near, twin);

    for _ in 0..5 {
        near.step(&[]);
    }
    assert_eq!(near.deer_cell(), Some((6, 1)));
    assert_eq!(near.material_tally(Material::Berry), 4);
    assert!(near
        .observe(sera)
        .unwrap()
        .visible_cells
        .iter()
        .any(|cell| cell.x == 6 && cell.y == 1 && cell.deer && cell.berry_pile == 4));
    near.step(&[]);
    assert_eq!(near.deer_cell(), Some((6, 1)));
    assert_eq!(near.material_tally(Material::Berry), 3);
    let ate = near.observe(sera).unwrap();
    assert!(ate
        .events
        .iter()
        .any(|event| event.text == "A deer eats a handful of berries."));
    assert!(near
        .observe(otto)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("deer")));
    for _ in 0..3 {
        near.step(&[]);
    }
    assert_eq!(near.material_tally(Material::Berry), 1);
    assert_eq!(near.deer_cell(), Some((6, 1)));
    let eats: Vec<_> = near
        .event_log()
        .iter()
        .filter(|event| event.text.contains("eats"))
        .map(|event| event.text.clone())
        .collect();
    assert_eq!(
        eats.iter()
            .filter(|text| *text == "A deer eats a handful of berries.")
            .count(),
        3
    );
    assert_eq!(
        eats.iter()
            .filter(|text| {
                *text == "A deer eats from the pile and leaves a handful of berries."
            })
            .count(),
        1
    );
    assert!(eats
        .iter()
        .all(|text| *text != "A deer eats the last handful of berries."));
    assert!(near.observe(sera).unwrap().events.iter().any(|event| {
        event.text == "A deer eats from the pile and leaves a handful of berries."
    }));
    assert!(near
        .observe(otto)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("handful")));
    let witness = near.spawn_at("Witness", "w", 6, 1).unwrap();
    let left = near.observe(witness).unwrap().text;
    assert!(
        left.contains(
            "A handful of berries lies here. It is good to eat raw. A deer ate the rest."
        ),
        "{left}"
    );
    assert!(!left.contains("are gone"), "{left}");
    assert!(!left.contains("ate the berries that were here"), "{left}");
    let body = near.examine(witness, ExamineTarget::Berries);
    assert!(
        body.contains("A handful of berries lies in contact with the ground")
            && body.contains("A deer ate the rest."),
        "{body}"
    );
    assert!(!body.contains("are gone"), "{body}");
    let got = near.step(&[cmd("spare", witness, Attempt::Gather(Material::Berry))]);
    assert_eq!(got.results[0].message, "You gather a handful of berries.");
    assert!(near
        .observe(witness)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != "You gather a handful of berries."));
    assert_eq!(near.deer_cell(), Some((5, 1)));

    let mut race = world_at(10_000);
    let nils = race.spawn_at("Nils", "n", 6, 1).unwrap();
    for _ in 0..6 {
        race.step(&[]);
    }
    assert_eq!(race.material_tally(Material::Berry), 4);
    race.step(&[cmd("hand", nils, Attempt::Gather(Material::Berry))]);
    assert_eq!(
        race.material_tally(Material::Berry),
        3,
        "the gather resolves first, then the deer eats one handful"
    );
    assert_eq!(race.deer_cell(), Some((6, 1)));

    let mut finder = world_at(10_000);
    let walker = finder.spawn_at("Otto", "o", 0, 0).unwrap();
    finder.step(&[]);
    assert!(finder.hoofprints_at(2, 3));
    assert_ne!(finder.deer_cell(), Some((2, 3)));
    for (n, dir) in [Dir::East, Dir::East, Dir::South, Dir::South, Dir::South]
        .into_iter()
        .enumerate()
    {
        finder.step(&[cmd(&format!("walk-{n}"), walker, Attempt::Move(dir))]);
    }
    assert_eq!(finder.actor_cell(walker), Some((2, 3)));
    assert_ne!(finder.deer_cell(), Some((2, 3)));
    let found = finder.observe(walker).unwrap();
    assert!(found.text.contains("Hoofprints mark the ground."));
    assert!(!found.text.contains("A deer stands here."));
    assert!(found
        .visible_cells
        .iter()
        .any(|cell| cell.x == 2 && cell.y == 3 && cell.tracks));
    assert!(finder
        .examine(walker, ExamineTarget::Tracks)
        .contains("Hoofprints mark the ground"));
    assert!(finder
        .examine(walker, ExamineTarget::Deer)
        .contains("cannot see a deer"));
}

#[test]
fn the_wet_season_makes_open_rain_heavier_and_stays_local() {
    let mut world = World::valley(Tuning::evening(), 1);
    let sera = world.spawn_at("Sera", "s", 4, 3).unwrap();
    let otto = world.spawn_at("Otto", "o", 0, 4).unwrap();
    while world.sim_time() < 18 {
        world.step(&[]);
    }
    assert!(world.is_raining());
    assert!(!world.wet_season);
    let before = world.examine(sera, ExamineTarget::Wood);
    assert!(before.contains("dry"), "{before}");
    assert!(before.contains("Rain does not reach it."), "{before}");
    assert!(!before.contains("heavier"));
    let before_log = world.examine(sera, ExamineTarget::Log);
    assert!(!before_log.contains("soaked"), "{before_log}");
    assert!(!before_log.contains("heavier"), "{before_log}");
    assert!(!world.observe(sera).unwrap().text.contains("heavier"));
    assert_eq!(
        world.examine(otto, ExamineTarget::Wood),
        "You see no wood from here."
    );

    world.step(&[]);
    assert!(world.wet_season);
    let after = world.examine(sera, ExamineTarget::Wood);
    assert!(after.contains("dry"), "{after}");
    assert!(after.contains("Rain does not reach it."), "{after}");
    assert!(!after.contains("heavier"), "{after}");
    let after_log = world.examine(sera, ExamineTarget::Log);
    assert!(after_log.contains("damp"), "{after_log}");
    assert!(after_log.contains("Rain reaches it."), "{after_log}");
    let seen = world.observe(sera).unwrap();
    assert!(seen
        .text
        .contains("The rain is heavier on the wood this season."));
    assert!(seen
        .events
        .iter()
        .any(|event| event.text == "The rain is heavier this season."));
    let far = world.observe(otto).unwrap();
    assert!(!far.text.contains("heavier"));
    assert!(far
        .events
        .iter()
        .all(|event| !event.text.contains("heavier")));
    assert_eq!(
        world.examine(otto, ExamineTarget::Wood),
        "You see no wood from here."
    );
    let announced = world
        .event_log()
        .iter()
        .filter(|event| event.text == "The rain is heavier this season.")
        .count();
    world.step(&[]);
    assert_eq!(
        world
            .event_log()
            .iter()
            .filter(|event| event.text == "The rain is heavier this season.")
            .count(),
        announced
    );

    let mut twin = World::valley(Tuning::evening(), 1);
    twin.spawn_at("Sera", "s", 4, 3).unwrap();
    twin.spawn_at("Otto", "o", 0, 4).unwrap();
    while twin.sim_time() < world.sim_time() {
        twin.step(&[]);
    }
    assert_eq!(world, twin);
}

#[test]
fn open_berries_soak_when_the_wet_season_arrives_and_the_rock_stays_dry() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    let rock = world.examine(near, ExamineTarget::Berries);
    assert!(rock.contains("The sweet berries are dry."), "{rock}");
    assert!(rock.contains("Rain does not reach it."), "{rock}");
    let open = world.examine(near, ExamineTarget::BitterBerries);
    assert!(open.contains("The dark berries are dry."), "{open}");

    while world.sim_time() < 18 {
        world.step(&[]);
    }
    assert!(!world.wet_season);
    let before = world.observe(near).unwrap();
    assert!(!before.text.contains("are soaked"), "{}", before.text);
    let before_open = world.examine(near, ExamineTarget::BitterBerries);
    assert!(before_open.contains("are damp."), "{before_open}");
    assert!(!before_open.contains("soaked"), "{before_open}");
    let before_rock = world.examine(near, ExamineTarget::Berries);
    assert!(!before_rock.contains("soaked"), "{before_rock}");
    assert!(
        before_rock.contains(
            "A handful of berries lies in contact with the ground to the north and east, under the rock overhang. It is dry. Rain does not reach it. They are good to eat raw. A deer ate the rest."
        ),
        "{before_rock}"
    );

    world.step(&[]);
    assert!(world.wet_season);
    assert_eq!(world.sim_time(), 19);
    let after_open = world.examine(near, ExamineTarget::BitterBerries);
    assert!(after_open.contains("are soaked."), "{after_open}");
    let after_rock = world.examine(near, ExamineTarget::Berries);
    assert!(!after_rock.contains("soaked"), "{after_rock}");
    let near_look = world.observe(near).unwrap().text;
    let far_look = world.observe(far).unwrap().text;
    let west_look = world.observe(west).unwrap().text;
    assert!(near_look.contains("The rain is heavier on the wood this season."));
    assert!(
        near_look.contains("The dark berries to the north are soaked."),
        "{near_look}"
    );
    assert!(
        far_look.contains("The dark berries to the north and east are soaked."),
        "{far_look}"
    );
    assert!(
        west_look.contains("The dark berries to the north and west are soaked."),
        "{west_look}"
    );
    assert!(!near_look.contains("The berries to the"));
    assert!(!far_look.contains("The berries under the rock are soaked."));
}

#[test]
fn open_branches_rot_on_the_clock_and_shelter_keeps_them() {
    let mut world = World::valley(Tuning::evening(), 1);
    let sera = world.spawn_at("Sera", "s", 4, 3).unwrap();
    let otto = world.spawn_at("Otto", "o", 0, 4).unwrap();
    let wood = world.examine(sera, ExamineTarget::Wood);
    let rock_line = wood
        .lines()
        .find(|line| line.contains("under the rock overhang"))
        .unwrap();
    let open_line = wood
        .lines()
        .find(|line| line.contains("in the open"))
        .unwrap();
    assert!(
        !rock_line.contains("rotting"),
        "shelter keeps this wood: {rock_line}"
    );
    assert!(
        open_line.contains("It is rotting in the open."),
        "{open_line}"
    );
    assert!(
        !open_line.contains("hoofprints"),
        "no prints on this wood yet: {open_line}"
    );
    assert!(
        !open_line.contains("deer"),
        "no deer on this wood yet: {open_line}"
    );
    assert!(world
        .observe(sera)
        .unwrap()
        .text
        .contains("Wood left in the open is rotting one cell north and one cell east."));
    assert!(!world.observe(sera).unwrap().text.contains("hoofprints"));
    assert!(!world.observe(otto).unwrap().text.contains("rotting"));
    assert_eq!(
        world.examine(otto, ExamineTarget::Wood),
        "You see no wood from here."
    );
    let rock = world
        .observe(sera)
        .unwrap()
        .visible_cells
        .into_iter()
        .find(|cell| cell.x == 6 && cell.y == 1)
        .unwrap();
    let open = world
        .observe(sera)
        .unwrap()
        .visible_cells
        .into_iter()
        .find(|cell| cell.x == 5 && cell.y == 2)
        .unwrap();
    assert_eq!(rock.branch_pile, 4);
    assert_eq!(open.branch_pile, 4);

    while world.sim_time() < 8 {
        world.step(&[]);
    }
    assert_eq!(world.material_tally(Material::Branch), 6);
    world.step(&[]);
    assert_eq!(world.material_tally(Material::Branch), 5);
    let seen = world.observe(sera).unwrap();
    let open = seen
        .visible_cells
        .iter()
        .find(|cell| cell.x == 5 && cell.y == 2)
        .unwrap();
    let rock = seen
        .visible_cells
        .iter()
        .find(|cell| cell.x == 6 && cell.y == 1)
        .unwrap();
    assert_eq!(open.branch_pile, 2);
    assert_eq!(rock.branch_pile, 3);
    assert!(seen.events.iter().any(|event| {
        event.text
            == "A branch rots away one cell north and one cell east, in the open. You hear branches breaking."
    }));
    let far = world.observe(otto).unwrap();
    assert!(far.events.iter().all(|event| !event.text.contains("rots")));
    assert!(!far.text.contains("rotting"));

    while world.sim_time() < 33 {
        world.step(&[]);
    }
    assert_eq!(world.material_tally(Material::Branch), 2);
    let gone = world.observe(sera).unwrap();
    assert_eq!(
        gone.visible_cells
            .iter()
            .find(|cell| cell.x == 5 && cell.y == 2)
            .unwrap()
            .branch_pile,
        0
    );
    assert_eq!(
        gone.visible_cells
            .iter()
            .find(|cell| cell.x == 6 && cell.y == 1)
            .unwrap()
            .branch_pile,
        2
    );
    assert!(!gone.text.contains("Wood left in the open is rotting"));
    assert!(world
        .observe(otto)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("rots")));

    let mut twin = World::valley(Tuning::evening(), 1);
    twin.spawn_at("Sera", "s", 4, 3).unwrap();
    twin.spawn_at("Otto", "o", 0, 4).unwrap();
    while twin.sim_time() < world.sim_time() {
        twin.step(&[]);
    }
    assert_eq!(world, twin);
}

#[test]
fn breaking_branches_are_heard_nearby_and_not_far_off() {
    let mut world = World::valley(Tuning::evening(), 1);
    let sera = world.spawn_at("Sera", "s", 4, 3).unwrap();
    let otto = world.spawn_at("Otto", "o", 0, 4).unwrap();
    let clock = world.sim_time();
    let _ = world.examine(sera, ExamineTarget::Wood);
    assert_eq!(world.sim_time(), clock, "examine does not step the valley");

    world.step(&[]);
    let first = world.observe(sera).unwrap();
    assert!(first
        .events
        .iter()
        .any(|event| event.text == "A deer steps east."));
    assert!(first
        .events
        .iter()
        .all(|event| !event.text.contains("branches breaking")));

    for _ in 0..3 {
        world.step(&[]);
    }
    assert_eq!(world.deer_cell(), Some((5, 2)));
    let brush = world.observe(sera).unwrap();
    assert!(brush.events.iter().any(|event| {
        event.text
            == "A deer steps east. A branch snapped under the deer one cell north and one cell east, in the open."
    }));
    assert_eq!(
        brush
            .visible_cells
            .iter()
            .find(|cell| cell.x == 5 && cell.y == 2)
            .unwrap()
            .branch_pile,
        3
    );
    assert!(world.observe(otto).unwrap().events.iter().all(|event| {
        !event.text.contains("branches breaking") && !event.text.contains("snapped")
    }));

    world.step(&[]);
    let left = world.observe(sera).unwrap();
    let tick = left.sim_time - 1;
    assert!(left
        .events
        .iter()
        .any(|event| event.time == tick && event.text == "A deer steps north."));
    assert!(left
        .events
        .iter()
        .filter(|event| event.time == tick)
        .all(|event| {
            !event.text.contains("branches breaking") && !event.text.contains("snapped")
        }));
    assert_eq!(
        left.visible_cells
            .iter()
            .find(|cell| cell.x == 5 && cell.y == 2)
            .unwrap()
            .branch_pile,
        3
    );

    while world.sim_time() < 8 {
        world.step(&[]);
    }
    world.step(&[]);
    let rotted = world.observe(sera).unwrap();
    assert!(rotted.events.iter().any(|event| {
        event.text
            == "A branch rots away one cell north and one cell east, in the open. You hear branches breaking."
    }));
    let under = world.spawn_at("Under", "u", 6, 1).unwrap();
    assert!(world.observe(under).unwrap().events.iter().any(|event| {
        event.text
            == "A branch rots away one cell south and one cell west, in the open. You hear branches breaking."
    }));
    let on_it = world.spawn_at("On", "on", 5, 2).unwrap();
    assert!(world.observe(on_it).unwrap().events.iter().any(|event| {
        event.text == "A branch rots away here, in the open. You hear branches breaking."
    }));
    assert_eq!(
        world
            .observe(on_it)
            .unwrap()
            .visible_cells
            .iter()
            .find(|cell| cell.x == 5 && cell.y == 2)
            .unwrap()
            .branch_pile,
        2
    );
    assert!(world
        .observe(otto)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("branches breaking")));
    let mid = world.spawn_at("Mid", "m", 3, 3).unwrap();
    let clock = world.sim_time();
    let from_mid = world.examine(mid, ExamineTarget::Wood);
    assert_eq!(world.sim_time(), clock);
    assert!(
        from_mid.contains(
            "A pile of two branches is in contact with the ground one cell north and two cells east, in the open, and hoofprints mark the ground there."
        ),
        "{from_mid}"
    );
    assert!(!from_mid.contains("to the north and east"), "{from_mid}");
    let from_rock = world.examine(under, ExamineTarget::Wood);
    assert!(
        from_rock.contains(
            "A pile of two branches is in contact with the ground one cell south and one cell west, in the open, and hoofprints mark the ground there."
        ),
        "{from_rock}"
    );
    assert!(!from_rock.contains("to the south and west"), "{from_rock}");
    assert_eq!(
        world.examine(otto, ExamineTarget::Wood),
        "You see no wood from here."
    );
    assert!(world
        .observe(mid)
        .unwrap()
        .text
        .contains("Wood left in the open is rotting one cell north and two cells east, and the deer left hoofprints on the rotting wood."));
    assert!(world
        .observe(under)
        .unwrap()
        .text
        .contains("Wood left in the open is rotting one cell south and one cell west, and hoofprints mark the ground there."));
    assert!(!world.observe(otto).unwrap().text.contains("rotting"));
    assert_eq!(
        world
            .observe(on_it)
            .unwrap()
            .visible_cells
            .iter()
            .find(|cell| cell.x == 5 && cell.y == 2)
            .unwrap()
            .branch_pile,
        2
    );

    let mut taken = world_at(10_000);
    let near = taken.spawn_at("Sera", "s", 4, 3).unwrap();
    let far = taken.spawn_at("Otto", "o", 0, 4).unwrap();
    let hand = taken.spawn_at("Nils", "n", 5, 2).unwrap();
    taken.step(&[cmd("snap", hand, Attempt::Gather(Material::Branch))]);
    assert!(taken
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| { event.text == "Nils gathers a branch." }));
    assert!(taken
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("branches breaking")));
}

#[test]
fn one_snap_points_each_listener_toward_the_wood() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
    let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
    let far = world.spawn_at("Far", "f", 0, 4).unwrap();
    for _ in 0..4 {
        world.step(&[]);
    }
    assert_eq!(world.deer_cell(), Some((5, 2)));
    let snap = |actor| {
        world
            .observe(actor)
            .unwrap()
            .events
            .into_iter()
            .find(|event| event.text.contains("A branch snapped under the deer"))
            .unwrap()
    };
    let near_snap = snap(near);
    let seen_snap = snap(seen);
    let quiet_snap = snap(quiet);
    assert_eq!(near_snap.id, seen_snap.id);
    assert_eq!(seen_snap.id, quiet_snap.id);
    assert_eq!(near_snap.time, 3);
    assert_eq!(
        near_snap.text,
        "A deer steps east. A branch snapped under the deer one cell south and one cell west, in the open."
    );
    assert_eq!(
        seen_snap.text,
        "A deer steps east. A branch snapped under the deer one cell north and two cells east, in the open."
    );
    assert_eq!(
        quiet_snap.text,
        "A deer steps east. A branch snapped under the deer one cell north, in the open."
    );
    assert!(world.observe(far).unwrap().events.iter().all(|event| {
        !event.text.contains("snapped") && !event.text.contains("branches breaking")
    }));

    while world.sim_time() < 9 {
        world.step(&[]);
    }
    let rot = |actor| {
        world
            .observe(actor)
            .unwrap()
            .events
            .into_iter()
            .find(|event| event.time == 8 && event.text.contains("rots away"))
            .unwrap()
    };
    let near_rot = rot(near);
    let seen_rot = rot(seen);
    let quiet_rot = rot(quiet);
    assert_eq!(near_rot.id, seen_rot.id);
    assert_eq!(seen_rot.id, quiet_rot.id);
    assert_eq!(
        near_rot.text,
        "A branch rots away one cell south and one cell west, in the open. You hear branches breaking."
    );
    assert_eq!(
        seen_rot.text,
        "A branch rots away one cell north and two cells east, in the open. You hear branches breaking."
    );
    assert_eq!(
        quiet_rot.text,
        "A branch rots away one cell north, in the open. You hear branches breaking."
    );
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("rots")));
}

#[test]
fn a_branch_snap_is_heard_one_cell_past_sight() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 5, 2).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let west = world.spawn_at("West", "w", 2, 3).unwrap();
    let otto = world.spawn_at("Otto", "o", 1, 4).unwrap();
    assert_eq!(world.tuning.hearing, world.tuning.sight + 1);
    let denied = world.step(&[cmd("no-branch", otto, Attempt::Gather(Material::Branch))]);
    assert_eq!(
        denied.results[0].message,
        "There is no branch here to gather."
    );
    assert!(world
        .observe(otto)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != "There is no branch here to gather."));
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("no branch")));
    let snapped = world.step(&[cmd("snap", near, Attempt::Gather(Material::Branch))]);
    assert_eq!(snapped.results[0].message, "You gather a branch.");
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != "You gather a branch."));
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("You hear")));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Near gathers a branch."));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("You hear")));
    let heard = world.observe(west).unwrap();
    assert!(heard
        .events
        .iter()
        .any(|event| event.text == "You hear branches breaking to the north and east."));
    assert!(heard
        .events
        .iter()
        .all(|event| !event.text.contains("gathers")));
    assert!(world
        .observe(otto)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("branches breaking")));
}

#[test]
fn a_lean_to_of_three_branches_breaks_the_wind() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 5, 2).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let west = world.spawn_at("West", "w", 0, 4).unwrap();
    assert!(matches!(
        parse_attempt("build shelter").unwrap(),
        Attempt::BuildLeanTo
    ));
    let denied = world.step(&[cmd(
        "no-wood",
        west,
        parse_attempt("build lean-to").unwrap(),
    )]);
    assert_eq!(
        denied.results[0].message,
        "You need three dry branches to build a lean-to."
    );
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("three dry branches")));
    let before = world.observe(near).unwrap().text;
    assert!(before.contains("You are in the wind."), "{before}");
    assert!(!before.contains("lean-to"), "{before}");
    let built = world.step(&[cmd("build", near, Attempt::BuildLeanTo)]);
    assert_eq!(
        built.results[0].message,
        "You build a lean-to of three branches."
    );
    let yours = "You build a lean-to of three branches.";
    let seen = "Near builds a lean-to of three branches.";
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == yours));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == seen));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("builds a lean-to")));
    let here = world.observe(near).unwrap().text;
    assert!(
        here.contains("A lean-to of branches stands here. It breaks the wind.")
            && here.contains("under a lean-to")
            && here.contains("You are in a little wind."),
        "{here}"
    );
    let branch_pile = world
        .observe(near)
        .unwrap()
        .visible_cells
        .iter()
        .find(|cell| cell.x == 5 && cell.y == 2)
        .unwrap()
        .branch_pile;
    assert_eq!(branch_pile, 1);
    let distant = world.observe(far).unwrap().text;
    assert!(
        distant.contains("A lean-to of branches stands on the valley floor to the north and east.")
            && distant.contains("You are in the wind."),
        "{distant}"
    );
    for _ in 0..7 {
        world.step(&[]);
    }
    let sheltered = world.observe(near).unwrap().self_info.cold;
    let open = world.observe(far).unwrap().self_info.cold;
    assert_eq!(sheltered, "warm");
    assert_eq!(open, "cold");
}

#[test]
fn a_lean_to_leaves_clothes_damp_while_open_rain_soaks() {
    let mut world = world_at(2);
    let near = world.spawn_at("Near", "n", 5, 2).unwrap();
    let open = world.spawn_at("Open", "o", 5, 3).unwrap();
    let far = world.spawn_at("Far", "f", 0, 4).unwrap();
    let built = world.step(&[cmd("build", near, Attempt::BuildLeanTo)]);
    assert!(built.results[0].success, "{}", built.results[0].message);
    world.step(&[]);
    world.step(&[]);
    assert!(world.is_raining());
    let sheltered = world.observe(near).unwrap();
    assert_eq!(sheltered.self_info.wetness, "damp");
    assert!(
        sheltered
            .text
            .contains("A little rain dampens your clothes.")
            && sheltered
                .text
                .contains("Only a little rain reaches you under it"),
        "{}",
        sheltered.text
    );
    assert!(!sheltered.text.contains("The rain wets your clothes."));
    assert!(!sheltered.text.contains("Your clothes are soaked."));
    let beside = world.observe(open).unwrap();
    assert_eq!(beside.self_info.wetness, "wet");
    assert!(beside.text.contains("The rain wets your clothes."));
    assert!(!beside.text.contains("A little rain dampens your clothes."));
    assert!(beside
        .text
        .contains("The rain does not wash anyone under it."));
    let distant = world.observe(far).unwrap().text;
    assert!(
        !distant.contains("A little rain dampens") && !distant.contains("lean-to"),
        "{distant}"
    );
    for _ in 0..12 {
        world.step(&[]);
    }
    let still = world.observe(near).unwrap();
    assert_eq!(still.self_info.wetness, "damp");
    assert!(still.text.contains("A little rain dampens your clothes."));
    assert!(!still.text.contains("Your clothes are soaked."));
    let soaked = world.observe(open).unwrap();
    assert_eq!(soaked.self_info.wetness, "soaked");
    assert!(soaked.text.contains("Your clothes are soaked."));
}

#[test]
fn rain_does_not_wash_a_smear_under_a_lean_to() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 5, 2).unwrap();
    let far = world.spawn_at("Far", "f", 1, 2).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    let built = world.step(&[cmd("build", near, Attempt::BuildLeanTo)]);
    assert_eq!(
        built.results[0].message,
        "You build a lean-to of three branches."
    );
    world.step(&[cmd("mud-far", far, Attempt::Gather(Material::Mud))]);
    world.step(&[cmd("smear-far", far, Attempt::Smear)]);
    assert_eq!(world.observe(far).unwrap().self_info.mud, "mud-smeared");
    for step in 0..2 {
        world.step(&[cmd(
            &format!("north-{step}"),
            near,
            Attempt::Move(Dir::North),
        )]);
    }
    for step in 0..2 {
        world.step(&[cmd(&format!("west-{step}"), near, Attempt::Move(Dir::West))]);
    }
    assert_eq!(world.actor_cell(near), Some((3, 0)));
    let gathered = world.step(&[cmd("ochre", near, Attempt::Gather(Material::Ochre))]);
    assert_eq!(gathered.results[0].message, "You gather a lump of ochre.");
    let smeared = world.step(&[cmd("smear-near", near, Attempt::SmearOchre)]);
    assert_eq!(
        smeared.results[0].message,
        "You smear red ochre on your skin."
    );
    assert_eq!(
        world.observe(near).unwrap().self_info.ochre,
        "red-ochre-smeared"
    );
    for step in 0..2 {
        world.step(&[cmd(&format!("east-{step}"), near, Attempt::Move(Dir::East))]);
    }
    for step in 0..2 {
        world.step(&[cmd(
            &format!("south-{step}"),
            near,
            Attempt::Move(Dir::South),
        )]);
    }
    assert_eq!(world.actor_cell(near), Some((5, 2)));
    let dry = world.observe(near).unwrap().text;
    assert!(
        dry.contains("A lean-to of branches stands here. It breaks the wind.")
            && !dry.contains("does not wash you"),
        "{dry}"
    );
    world.raining = true;
    world.step(&[]);
    let sheltered = world.observe(near).unwrap();
    assert_eq!(sheltered.self_info.ochre, "red-ochre-smeared");
    assert!(sheltered.events.iter().all(|event| {
        !event.text.contains("washes the red ochre") && !event.text.contains("washes the mud")
    }));
    assert!(
        sheltered
            .text
            .contains("Only a little rain reaches you under it, and it does not wash you."),
        "{sheltered:?}"
    );
    let open = world.observe(far).unwrap();
    assert_eq!(open.self_info.mud, "bare skin");
    assert!(open
        .events
        .iter()
        .any(|event| event.text == "The rain washes the mud off you."));
    let from_beside = world.observe(west).unwrap().text;
    assert!(
        from_beside.contains(
            "A lean-to of branches stands on the valley floor to the north. The rain does not wash anyone under it."
        ),
        "{from_beside}"
    );
}

#[test]
fn rain_puts_out_an_open_fire_and_the_lean_to_keeps_its_fire() {
    let mut world = world_at(18);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    let far = world.spawn_at("Far", "f", 0, 4).unwrap();
    world.step(&[cmd("stash", near, Attempt::Gather(Material::Branch))]);
    world.step(&[cmd("to-w", near, Attempt::Move(Dir::West))]);
    world.step(&[cmd("to-s", near, Attempt::Move(Dir::South))]);
    let dropped = world.step(&[cmd("drop-wood", near, Attempt::Drop(Material::Branch))]);
    assert!(dropped.results[0].success, "{}", dropped.results[0].message);
    let built = world.step(&[cmd("build", near, Attempt::BuildLeanTo)]);
    assert_eq!(
        built.results[0].message,
        "You build a lean-to of three branches."
    );
    world.step(&[cmd("back-n", near, Attempt::Move(Dir::North))]);
    world.step(&[cmd("back-e", near, Attempt::Move(Dir::East))]);
    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    world.step(&[cmd("out-w", near, Attempt::Move(Dir::West))]);
    let open = world.step(&[cmd("open-fire", near, Attempt::Light)]);
    assert!(open.results[0].success, "{}", open.results[0].message);
    assert_eq!(world.actor_cell(near), Some((5, 1)));
    world.step(&[cmd("more-e", near, Attempt::Move(Dir::East))]);
    world.step(&[cmd("bark-2", near, Attempt::Gather(Material::Bark))]);
    world.step(&[cmd("home-w", near, Attempt::Move(Dir::West))]);
    world.step(&[cmd("home-s", near, Attempt::Move(Dir::South))]);
    assert_eq!(world.actor_cell(near), Some((5, 2)));
    world.step(&[cmd("wood", near, Attempt::Gather(Material::Branch))]);
    let sheltered = world.step(&[cmd("shelter-fire", near, Attempt::Light)]);
    assert!(
        sheltered.results[0].success,
        "{}",
        sheltered.results[0].message
    );
    let dry = world.observe(near).unwrap().text;
    assert!(
        dry.contains("A small fire burns here. It warms you.")
            && !dry.contains("The lean-to keeps the fire."),
        "{dry}"
    );
    while !world.is_raining() {
        world.step(&[]);
        assert!(world.sim_time() < 20, "rain should arrive");
    }
    let here = world.observe(near).unwrap();
    assert!(
        here.text
            .contains("A small fire burns here. It warms you. The lean-to keeps the fire."),
        "{here:?}"
    );
    assert!(here
        .visible_cells
        .iter()
        .any(|cell| cell.x == 5 && cell.y == 2 && cell.fire));
    assert!(here
        .visible_cells
        .iter()
        .all(|cell| !(cell.x == 5 && cell.y == 1 && cell.fire)));
    assert!(here
        .events
        .iter()
        .any(|event| event.text.contains("Rain puts out the fire.")));
    let beside = world.observe(west).unwrap().text;
    assert!(beside.contains("The lean-to keeps the fire."), "{beside}");
    assert!(beside.contains("The rain does not wash anyone under it."));
    let distant = world.observe(far).unwrap();
    assert!(
        !distant.text.contains("The lean-to keeps the fire."),
        "{}",
        distant.text
    );
    assert!(distant
        .events
        .iter()
        .all(|event| !event.text.contains("Rain puts out the fire.")));
}

#[test]
fn still_water_sickens_more_often_than_the_brook() {
    let mut pool = World::valley(Tuning::evening(), 1);
    pool.tuning.still_chance = 1000;
    pool.tuning.illness_chance = 0;
    let sera = pool.spawn_at("Sera", "s", 2, 1).unwrap();
    let looked = pool.examine(sera, ExamineTarget::Water);
    assert_eq!(pool.sim_time(), 0);
    assert!(looked.contains("almost every time"));
    assert!(looked.contains("only occasionally"));
    assert!(!looked.contains("0.90") && !looked.contains("0.20"));
    let pooled = pool.step(&[cmd("pool", sera, Attempt::Drink)]);
    assert!(
        pooled.results[0]
            .message
            .starts_with("You drink the still water."),
        "{}",
        pooled.results[0].message
    );
    let ill = pool.observe(sera).unwrap();
    assert_eq!(ill.self_info.illness, "ill");
    assert_eq!(ill.self_info.thirst, "not thirsty");
    assert!(ill
        .events
        .iter()
        .all(|event| !event.text.contains("Sera drinks the still water")));
    let draw = pool
        .event_log()
        .iter()
        .find(|event| event.draw.as_ref().is_some_and(|draw| draw.tag == "drink"))
        .unwrap()
        .draw
        .clone()
        .unwrap();
    assert!(draw.subject.ends_with(":still"));
    assert_eq!(
        draw.value,
        contest_hash(pool.seed, 0, "drink", &draw.subject, sera)
    );
    assert!(ill.text.contains("ill"));
    assert!(!ill.text.contains("900"));

    let mut running = World::valley(Tuning::evening(), 1);
    running.tuning.still_chance = 1000;
    running.tuning.illness_chance = 0;
    let sera = running.spawn_at("Sera", "s", 0, 1).unwrap();
    let ran = running.step(&[cmd("run", sera, Attempt::Drink)]);
    assert!(
        ran.results[0]
            .message
            .starts_with("You drink from the brook."),
        "{}",
        ran.results[0].message
    );
    let well = running.observe(sera).unwrap();
    assert_eq!(well.self_info.illness, "well");
    assert!(well
        .events
        .iter()
        .all(|event| !event.text.contains("Sera drinks from the brook")));

    let mut swapped = World::valley(Tuning::evening(), 3);
    swapped.tuning.still_chance = 0;
    swapped.tuning.illness_chance = 1000;
    let at_pool = swapped.spawn_at("Sera", "s", 2, 1).unwrap();
    swapped.step(&[cmd("safe", at_pool, Attempt::Drink)]);
    assert_eq!(swapped.observe(at_pool).unwrap().self_info.illness, "well");
    let at_brook = swapped.spawn_at("Otto", "o", 0, 1).unwrap();
    swapped.step(&[cmd("bad", at_brook, Attempt::Drink)]);
    assert_eq!(swapped.observe(at_brook).unwrap().self_info.illness, "ill");
}

#[test]
fn seed_one_still_water_sickens_on_the_next_drink_and_the_brook_does_not() {
    let mut early = World::valley(Tuning::evening(), 1);
    let near = early.spawn_at("Near", "n", 2, 1).unwrap();
    for _ in 0..4 {
        early.step(&[]);
    }
    let arrived = early.step(&[cmd("arrive", near, Attempt::Drink)]);
    assert_eq!(
        arrived.results[0].message, "You drink the still water. You are not thirsty.",
        "seed 1, the drink on arrival at the pool, is the safe draw"
    );
    assert_eq!(early.observe(near).unwrap().self_info.illness, "well");

    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    pair(
        &mut world,
        "w",
        near,
        Attempt::Move(Dir::West),
        west,
        Attempt::Move(Dir::West),
    );
    pair(
        &mut world,
        "w2",
        near,
        Attempt::Move(Dir::West),
        west,
        Attempt::Move(Dir::West),
    );
    pair(
        &mut world,
        "nw",
        near,
        Attempt::Move(Dir::North),
        west,
        Attempt::Move(Dir::West),
    );
    pair(
        &mut world,
        "nn",
        near,
        Attempt::Move(Dir::North),
        west,
        Attempt::Move(Dir::West),
    );
    assert_eq!(world.actor_cell(near), Some((2, 1)));
    world.step(&[cmd("to-brook", west, Attempt::Move(Dir::North))]);
    assert_eq!(world.actor_cell(west), Some((1, 2)));
    assert_eq!(world.sim_time(), 5);
    let drunk = world.step(&[
        cmd("still", near, Attempt::Drink),
        cmd("brook", west, Attempt::Drink),
    ]);
    assert_eq!(
        drunk.results[0].message,
        "You drink the still water. You are not thirsty. Your stomach turns."
    );
    assert_eq!(
        drunk.results[1].message,
        "You drink from the brook. You are not thirsty."
    );
    let pool = world.observe(near).unwrap();
    let running = world.observe(west).unwrap();
    let witness = world.observe(far).unwrap();
    assert_eq!(pool.self_info.illness, "ill");
    assert!(pool.text.contains("You are ill."));
    assert_eq!(running.self_info.illness, "well");
    assert!(running.text.contains("You are well."));
    assert!(pool
        .events
        .iter()
        .any(|event| event.text == "Your stomach turns."));
    assert!(pool
        .events
        .iter()
        .all(|event| event.text != "Near's stomach turns."));
    assert!(running
        .events
        .iter()
        .any(|event| event.text == "Near's stomach turns."));
    assert!(running
        .events
        .iter()
        .all(|event| !event.text.contains("West's stomach")));
    assert!(witness
        .events
        .iter()
        .any(|event| event.text == "Near drinks the still water."));
    assert!(witness
        .events
        .iter()
        .any(|event| event.text == "West drinks from the brook."));
    assert!(witness
        .events
        .iter()
        .any(|event| event.text == "Near's stomach turns."));

    world.step(&[cmd("fill", west, Attempt::Gather(Material::Water))]);
    for id in ["e1", "e2", "e3", "e4", "e5"] {
        world.step(&[cmd(id, west, Attempt::Move(Dir::East))]);
    }
    world.step(&[cmd("up", west, Attempt::Move(Dir::North))]);
    assert_eq!(world.actor_cell(west), Some((6, 1)));
    world.step(&[cmd("pot", west, Attempt::Gather(Material::Pot))]);
    world.step(&[cmd("light", west, Attempt::Light)]);
    let boiled = world.step(&[cmd("boil", west, Attempt::Boil)]);
    assert_eq!(
        boiled.results[0].message,
        "You boil the drink in the pot over the fire and drink it."
    );
    let safe = world.observe(west).unwrap();
    assert_eq!(safe.self_info.illness, "well");
    assert!(safe.text.contains("You are well."));
    assert!(safe
        .events
        .iter()
        .all(|event| !event.text.contains("stomach")));
    assert_eq!(world.observe(near).unwrap().self_info.illness, "ill");
    assert_eq!(world.sim_time(), 16);
    assert!(!world.is_raining());
}

#[test]
fn illness_fades_and_the_actor_hears_it_settle() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    for (id, dir) in [
        ("w", Dir::West),
        ("w2", Dir::West),
        ("n", Dir::North),
        ("n2", Dir::North),
    ] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    assert_eq!(world.actor_cell(near), Some((2, 1)));
    world.step(&[]);
    let drunk = world.step(&[cmd("still", near, Attempt::Drink)]);
    assert_eq!(
        drunk.results[0].message,
        "You drink the still water. You are not thirsty. Your stomach turns."
    );
    assert_eq!(world.observe(near).unwrap().self_info.illness, "ill");
    assert_eq!(world.sim_time(), 6);
    let mut ticks = 0;
    while world.observe(near).unwrap().self_info.illness == "ill" {
        world.step(&[]);
        ticks += 1;
        assert!(ticks < 30, "illness should fade");
    }
    assert_eq!(ticks, 15);
    assert_eq!(world.sim_time(), 21);
    let settled = world.observe(near).unwrap();
    assert_eq!(settled.self_info.illness, "well");
    assert!(settled.text.contains("You are well."));
    assert_eq!(
        settled
            .events
            .iter()
            .filter(|event| event.text.contains("settles"))
            .map(|event| event.text.as_str())
            .collect::<Vec<_>>(),
        vec!["Your stomach settles."]
    );
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("settles")));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("settles")));
    assert_eq!(world.actor_cell(far), Some((3, 3)));
    assert_eq!(world.actor_cell(west), Some((5, 3)));
}

#[test]
fn poison_berry_illness_fades_and_rest_under_the_rock_hurries_it() {
    let mut open = World::valley(Tuning::evening(), 1);
    let near = open.spawn_at("Near", "n", 4, 1).unwrap();
    let far = open.spawn_at("Far", "f", 5, 3).unwrap();
    let eaten = open.step(&[cmd("eat", near, Attempt::Eat(Material::BitterBerry))]);
    assert_eq!(
        eaten.results[0].message,
        "You eat a handful of poison berries. They taste bitter. Your stomach turns."
    );
    let ill = open.observe(near).unwrap();
    assert_eq!(ill.self_info.illness, "ill");
    assert!(ill.text.contains("You are ill."));
    assert!(ill.text.contains("Your stomach has not settled."));
    assert!(ill
        .events
        .iter()
        .any(|event| event.text == "Your stomach turns."));
    assert!(ill.events.iter().all(|event| {
        !event.text.contains("eats a handful")
            && event.text != "You eat a handful of poison berries."
    }));
    assert!(open
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Near eats a handful of dark berries."));
    assert!(open
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("stomach")));
    let mut open_ticks = 0;
    while open.observe(near).unwrap().self_info.illness == "ill" {
        open.step(&[]);
        open_ticks += 1;
        assert!(open_ticks < 30, "illness should fade");
    }
    assert_eq!(open_ticks, 15);
    let well = open.observe(near).unwrap();
    assert_eq!(well.self_info.illness, "well");
    assert!(well.text.contains("You are well."));
    assert!(!well.text.contains("You are ill."));
    assert!(!well.text.contains("has not settled"));
    assert_eq!(
        well.events
            .iter()
            .filter(|event| event.text.contains("settles"))
            .map(|event| event.text.as_str())
            .collect::<Vec<_>>(),
        vec!["Your stomach settles."]
    );
    assert!(open
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("settles") && !event.text.contains("stomach")));

    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 1).unwrap();
    let far = world.spawn_at("Far", "f", 5, 3).unwrap();
    world.step(&[cmd("eat", near, Attempt::Eat(Material::BitterBerry))]);
    world.step(&[cmd("e1", near, Attempt::Move(Dir::East))]);
    world.step(&[cmd("e2", near, Attempt::Move(Dir::East))]);
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    assert_eq!(world.observe(near).unwrap().self_info.illness, "ill");
    let mut rests = 0;
    while world.observe(near).unwrap().self_info.illness == "ill" {
        let rested = world.step(&[cmd(&format!("r{rests}"), near, Attempt::Rest)]);
        assert!(
            rested.results[0]
                .message
                .starts_with("You rest under the rock."),
            "{}",
            rested.results[0].message
        );
        rests += 1;
        assert!(rests < 10, "rest should hurry the stomach");
    }
    assert!(
        rests < open_ticks,
        "rest took {rests}, waiting took {open_ticks}"
    );
    assert_eq!(rests, 3);
    let settled = world.observe(near).unwrap();
    assert_eq!(settled.self_info.illness, "well");
    assert!(settled.text.contains("You are well."));
    assert!(!settled.text.contains("has not settled"));
    assert!(settled
        .events
        .iter()
        .all(|event| !event.text.contains("Near rests")));
    assert_eq!(
        settled
            .events
            .iter()
            .filter(|event| event.text.contains("settles"))
            .map(|event| event.text.as_str())
            .collect::<Vec<_>>(),
        vec!["Your stomach settles."]
    );
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Near rests under the rock."));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("settles") && !event.text.contains("stomach")));
}

#[test]
fn the_still_pool_freezes_at_tick_twelve_and_the_refusal_stays_private() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    for (id, dir) in [
        ("w", Dir::West),
        ("w2", Dir::West),
        ("n", Dir::North),
        ("n2", Dir::North),
    ] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    assert_eq!(world.actor_cell(near), Some((2, 1)));
    assert_eq!(world.sim_time(), 4);
    let open = world.observe(near).unwrap();
    assert!(open
        .text
        .contains("Still water lies here. It is not the brook."));
    assert!(!open.text.contains("frozen"));
    while world.sim_time() < 12 {
        world.step(&[]);
    }
    assert_eq!(world.sim_time(), 12);
    assert!(!world.is_raining());
    let iced = world.observe(near).unwrap();
    assert!(iced.text.contains("The still water is frozen."));
    assert!(iced
        .visible_cells
        .iter()
        .any(|cell| cell.x == 2 && cell.y == 1 && cell.frozen));
    assert!(iced
        .visible_cells
        .iter()
        .any(|cell| cell.x == 1 && cell.y == 2 && !cell.frozen));
    let drink = world.step(&[cmd("drink", near, Attempt::Drink)]);
    assert_eq!(
        drink.results[0].message,
        "The still water is frozen. Ice covers it."
    );
    assert_eq!(world.observe(near).unwrap().self_info.illness, "well");
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != "The still water is frozen. Ice covers it."));
    let gather = world.step(&[cmd("gather", near, Attempt::Gather(Material::Water))]);
    assert_eq!(gather.results[0].message, "The still water is frozen.");
    assert_eq!(world.material_tally(Material::Water), 0);
    let actor = world.observe(near).unwrap();
    assert_eq!(
        actor
            .events
            .iter()
            .filter(|event| event.text == "The still water is frozen.")
            .count(),
        1
    );
    for witness in [far, west] {
        assert!(world
            .observe(witness)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "The still water is frozen."));
    }
    assert_eq!(world.actor_cell(far), Some((3, 3)));
    assert_eq!(world.actor_cell(west), Some((5, 3)));
    assert_eq!(world.actor_cell(near), Some((2, 1)));
}

#[test]
fn a_hearer_gets_the_crack_when_the_still_pool_freezes() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 2, 1).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
    let crack = "You hear ice forming to the north and west.";
    while world.sim_time() < 12 {
        world.step(&[]);
    }
    assert_eq!(world.sim_time(), 12);
    let under = "The still freezes under you.";
    let standing = world.observe(near).unwrap();
    assert!(standing
        .text
        .contains("The still water is frozen. Ice covers it."));
    assert!(standing.events.iter().all(|event| {
        event.text != "The still water is frozen. Ice covers it."
            && !event.text.contains("ice forming")
            && !event.text.contains("Ice forms")
    }));
    let feet: Vec<_> = standing
        .events
        .iter()
        .filter(|event| event.text == under)
        .collect();
    assert_eq!(feet.len(), 1);
    assert_eq!(feet[0].time, 12);
    assert!(
        standing.legend.contains("= frozen still"),
        "{}",
        standing.legend
    );
    assert!(!standing.legend.contains("= still water"));
    assert!(world
        .observe(far)
        .unwrap()
        .legend
        .contains("= frozen still"));
    assert!(world
        .observe(west)
        .unwrap()
        .legend
        .contains("= still water"));
    assert!(!world.observe(west).unwrap().legend.contains("frozen still"));
    let seen = world.observe(far).unwrap();
    assert!(seen
        .text
        .contains("The still water to the north and west is frozen. Ice covers it."));
    assert!(seen.events.iter().all(|event| {
        event.text != under
            && !event.text.contains("ice forming")
            && !event.text.contains("Ice forms")
    }));
    let heard = world.observe(west).unwrap();
    assert!(!heard.text.contains("frozen"));
    assert!(!heard.text.contains("Ice covers"));
    let cracks: Vec<_> = heard
        .events
        .iter()
        .filter(|event| event.text == crack)
        .collect();
    assert_eq!(cracks.len(), 1);
    assert_eq!(cracks[0].time, 12);
    assert!(heard
        .events
        .iter()
        .all(|event| { event.text != "Ice forms on the still water." && event.text != under }));
    let past = world.observe(quiet).unwrap();
    assert!(past
        .events
        .iter()
        .all(|event| { !event.text.contains("ice forming") && !event.text.contains("Ice forms") }));
    assert!(!past.text.contains("frozen"));
    world.step(&[]);
    assert_eq!(
        world
            .observe(west)
            .unwrap()
            .events
            .iter()
            .filter(|event| event.text == crack)
            .count(),
        1
    );
    let dawn = world.tuning.night_at + world.tuning.night_len;
    while world.sim_time() < dawn {
        world.step(&[]);
    }
    let thawed = world.observe(near).unwrap();
    assert!(thawed.text.contains("The day has thawed the still water."));
    assert!(thawed.legend.contains("= still water"), "{}", thawed.legend);
    assert!(!thawed.legend.contains("= frozen still"));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != crack && !event.text.contains("thaw")));
    let next_night = dawn + world.tuning.day_len;
    while world.sim_time() < next_night {
        world.step(&[]);
    }
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains("The still water is frozen. Ice covers it."));
    let again: Vec<_> = world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .filter(|event| event.text == crack)
        .map(|event| event.time)
        .collect();
    assert_eq!(again, vec![next_night]);
    let again_under: Vec<_> = world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .filter(|event| event.text == under)
        .map(|event| event.time)
        .collect();
    assert_eq!(again_under, vec![next_night]);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| { !event.text.contains("ice forming") && !event.text.contains("Ice forms") }));
    assert!(world.observe(far).unwrap().events.iter().all(|event| {
        event.text != under
            && !event.text.contains("ice forming")
            && !event.text.contains("Ice forms")
    }));
    assert_eq!(
        world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .filter(|event| event.text.contains("ice forming"))
            .count(),
        0
    );
}

#[test]
fn far_reads_the_ice_once_the_pool_in_sight_is_frozen() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    let ice = "The still water to the north and west is frozen.";
    assert!(!world.observe(far).unwrap().text.contains("frozen"));
    assert!(!world.observe(far).unwrap().text.contains("ice"));
    assert!(!world.observe(west).unwrap().text.contains("frozen"));
    for (id, dir) in [
        ("w", Dir::West),
        ("w2", Dir::West),
        ("n", Dir::North),
        ("n2", Dir::North),
    ] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    assert_eq!(world.actor_cell(near), Some((2, 1)));
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains("Still water lies here. It is not the brook."));
    assert!(!world.observe(far).unwrap().text.contains("frozen"));
    assert!(!world.observe(west).unwrap().text.contains("frozen"));
    while world.sim_time() < 12 {
        world.step(&[]);
    }
    let standing = world.observe(near).unwrap();
    assert!(standing.text.contains("The still water is frozen."));
    assert!(!standing.text.contains(ice));
    assert!(world.observe(far).unwrap().text.contains(ice));
    assert!(!world.observe(west).unwrap().text.contains("frozen"));
    let drink = world.step(&[cmd("drink", near, Attempt::Drink)]);
    assert_eq!(
        drink.results[0].message,
        "The still water is frozen. Ice covers it."
    );
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != "The still water is frozen. Ice covers it."));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != "The still water is frozen. Ice covers it."));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != "The still water is frozen. Ice covers it."));
}

#[test]
fn far_reads_the_fire_keeping_the_pool_open() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    for (id, dir) in [
        ("w", Dir::West),
        ("w2", Dir::West),
        ("w3", Dir::West),
        ("w4", Dir::West),
    ] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    assert_eq!(world.actor_cell(near), Some((2, 1)));
    while world.sim_time() < 12 {
        world.step(&[]);
    }
    let ice = "The still water to the north and west is frozen.";
    let open = "The fire is keeping the still water to the north and west from freezing.";
    assert!(world.observe(far).unwrap().text.contains(ice));
    assert!(!world
        .observe(far)
        .unwrap()
        .text
        .contains("keeping the still water"));
    assert!(!world.observe(west).unwrap().text.contains("frozen"));
    assert!(!world
        .observe(west)
        .unwrap()
        .text
        .contains("keeping the still water"));
    let lit = world.step(&[cmd("light", near, Attempt::Light)]);
    assert!(lit.results[0].success, "{}", lit.results[0].message);
    let near_look = world.observe(near).unwrap().text;
    let far_look = world.observe(far).unwrap().text;
    let west_look = world.observe(west).unwrap().text;
    assert!(near_look.contains("The fire is keeping the still water from freezing."));
    assert!(!near_look.contains("to the north and west from freezing"));
    assert!(far_look.contains(open), "{far_look}");
    assert!(!far_look.contains("is frozen"));
    assert!(!west_look.contains("frozen"));
    assert!(!west_look.contains("keeping the still water"));
}

#[test]
fn the_day_thaws_the_still_pool_and_night_freezes_it_again() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 2, 1).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    let open = world.observe(near).unwrap().text;
    assert!(
        open.contains("Still water lies here. It is not the brook."),
        "{open}"
    );
    assert!(!open.contains("Ice covers"), "{open}");
    assert!(!world.observe(far).unwrap().text.contains("frozen"));
    while world.sim_time() < 12 {
        world.step(&[]);
    }
    let iced = world.observe(near).unwrap().text;
    assert!(
        iced.contains("The still water is frozen. Ice covers it."),
        "{iced}"
    );
    assert!(world
        .observe(far)
        .unwrap()
        .text
        .contains("The still water to the north and west is frozen. Ice covers it."));
    assert!(!world.observe(west).unwrap().text.contains("frozen"));
    assert!(!world.observe(west).unwrap().text.contains("Ice covers"));
    let drink = world.step(&[cmd("ice", near, Attempt::Drink)]);
    assert_eq!(
        drink.results[0].message,
        "The still water is frozen. Ice covers it."
    );
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != "The still water is frozen. Ice covers it."));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != "The still water is frozen. Ice covers it."));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != "The still water is frozen. Ice covers it."));
    let off = world.step(&[cmd("off", near, Attempt::Move(Dir::East))]);
    assert_eq!(off.results[0].message, "You walk east.");
    let back = world.step(&[cmd("back", near, Attempt::Move(Dir::West))]);
    assert_eq!(back.results[0].message, "You walk west across the ice.");
    assert_eq!(world.actor_cell(near), Some((2, 1)));
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| { !event.text.contains("Near walks west across the ice.") }));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Near walks west across the ice."));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("across the ice")));
    let wade = world.step(&[cmd("brook", near, Attempt::Move(Dir::West))]);
    assert_eq!(wade.results[0].message, "You wade west.");
    let brook = world.observe(near).unwrap().text;
    assert!(brook.contains("The water here is raw."), "{brook}");
    assert!(!brook.contains("The still water is frozen."), "{brook}");
    assert!(
        brook.contains("The still water to the east is frozen. Ice covers it."),
        "{brook}"
    );
    let running = world.step(&[cmd("running", near, Attempt::Drink)]);
    assert!(
        running.results[0]
            .message
            .starts_with("You drink from the brook."),
        "{}",
        running.results[0].message
    );
    let onto = world.step(&[cmd("onto", near, Attempt::Move(Dir::East))]);
    assert_eq!(onto.results[0].message, "You walk east across the ice.");
    while world.sim_time() < 80 {
        world.step(&[]);
    }
    assert_eq!(world.sim_time(), 80);
    let thawed = world.observe(near).unwrap().text;
    assert!(
        thawed.contains("The day has thawed the still water."),
        "{thawed}"
    );
    assert!(!thawed.contains("Ice covers"), "{thawed}");
    assert!(world
        .observe(far)
        .unwrap()
        .text
        .contains("The still water to the north and west has thawed."));
    assert!(!world.observe(west).unwrap().text.contains("thawed"));
    let again = world.step(&[cmd("thaw", near, Attempt::Drink)]);
    assert!(
        again.results[0]
            .message
            .starts_with("You drink the still water."),
        "{}",
        again.results[0].message
    );
    while world.sim_time() < 96 {
        world.step(&[]);
    }
    assert_eq!(world.sim_time(), 96);
    let night = world.observe(near).unwrap().text;
    assert!(
        night.contains("The still water is frozen. Ice covers it."),
        "{night}"
    );
    let hidden = world.observe(far).unwrap().text;
    assert!(
        hidden.contains("The dark hides what lies farther off."),
        "{hidden}"
    );
    assert!(!hidden.contains("is frozen"), "{hidden}");
    assert!(!world.observe(west).unwrap().text.contains("frozen"));
}

#[test]
fn eat_bark_is_refused_and_the_sheet_still_lights() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let bark = world.material_tally(Material::Bark);
    let line = "Bark does not feed you.";
    let hunger = world.observe(near).unwrap().self_info.hunger.clone();
    let absent = world.spawn_at("Dry", "d", 4, 3).unwrap();
    let none = world.step(&[cmd("none", absent, Attempt::Eat(Material::Bark))]);
    assert_eq!(none.results[0].message, "There is no bark here to eat.");
    assert!(world
        .observe(absent)
        .unwrap()
        .events
        .iter()
        .any(|event| { event.text == "There is no bark here to eat." }));
    for witness in [near, west, far] {
        assert!(world
            .observe(witness)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "There is no bark here to eat."));
    }
    assert!(!world
        .observe(near)
        .unwrap()
        .text
        .contains("Bark does not feed you."));
    assert!(world
        .observe(near)
        .unwrap()
        .allowed_actions
        .iter()
        .all(|action| action != "eat bark"));

    let ground = world.step(&[cmd("eat-ground", near, Attempt::Eat(Material::Bark))]);
    assert!(!ground.results[0].success);
    assert_eq!(ground.results[0].message, line);
    assert_eq!(world.material_tally(Material::Bark), bark);
    assert_eq!(world.observe(near).unwrap().self_info.hunger, hunger);
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains("A pile of bark lies here. Bark does not feed you."));
    assert!(world
        .examine(near, ExamineTarget::Bark)
        .contains("Bark does not feed you."));
    assert!(!world
        .examine(west, ExamineTarget::Bark)
        .contains("does not feed"));
    assert!(!world.observe(west).unwrap().text.contains("does not feed"));
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != line));
    for witness in [west, far] {
        assert!(world
            .observe(witness)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != line && !event.text.contains("does not feed")));
    }

    let gathered = world.step(&[cmd("gather", near, Attempt::Gather(Material::Bark))]);
    assert!(
        gathered.results[0].success,
        "{}",
        gathered.results[0].message
    );
    assert_eq!(world.material_tally(Material::Bark), bark);
    assert!(world
        .observe(near)
        .unwrap()
        .self_info
        .carrying
        .iter()
        .any(|item| item.material == "bark" && item.count >= 1));

    let carried = world.step(&[cmd("eat-carried", near, Attempt::Eat(Material::Bark))]);
    assert!(!carried.results[0].success);
    assert_eq!(carried.results[0].message, line);
    assert_eq!(world.material_tally(Material::Bark), bark);
    assert!(world
        .observe(near)
        .unwrap()
        .self_info
        .carrying
        .iter()
        .any(|item| item.material == "bark" && item.count >= 1));
    assert_eq!(world.observe(near).unwrap().self_info.hunger, hunger);
    for witness in [west, far] {
        assert!(world
            .observe(witness)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != line && !event.text.contains("tinder")));
    }

    let lit = world.step(&[cmd("light", near, Attempt::Light)]);
    assert_eq!(
        lit.results[0].message,
        "You light a fire with the bark from the arms and a dry branch from the pile at the feet."
    );
    assert!(lit.results[0].success);
    assert_eq!(world.material_tally(Material::Bark), bark - 1);
    assert!(world
        .observe(near)
        .unwrap()
        .self_info
        .carrying
        .iter()
        .all(|item| item.material != "bark"));
    assert_eq!(world.observe(near).unwrap().self_info.hunger, hunger);
    assert!(!lit.results[0].message.contains("eases"));
    let witnessed = "Near lights a fire with the bark from the arms and a dry branch from the pile at the feet.";
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == witnessed));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("lights a fire")));
}

#[test]
fn sweet_berries_give_energy_and_the_other_food_must_be_cooked() {
    let mut world = world_at(10_000);
    let sera = world.spawn_at("Sera", "s", 6, 1).unwrap();
    let nils = world.spawn_at("Nils", "n", 3, 1).unwrap();
    let linde = world.spawn_at("Linde", "l", 4, 1).unwrap();
    assert!(world
        .examine(sera, ExamineTarget::Berries)
        .contains("They are good to eat raw."));
    let naive = world.examine(nils, ExamineTarget::Mushrooms);
    assert!(naive.contains("mushrooms"), "{naive}");
    assert!(!naive.contains("Raw mushrooms do not feed you."), "{naive}");
    assert!(!naive.contains("must be cooked"), "{naive}");
    assert!(!naive.contains("poison"), "{naive}");
    let naive_look = world.observe(nils).unwrap().text;
    let mushroom_line = naive_look
        .lines()
        .find(|line| line.contains("Mushrooms lie here"))
        .unwrap_or("");
    assert!(mushroom_line.contains("Mushrooms lie here"), "{naive_look}");
    assert!(!mushroom_line.contains("must be cooked"), "{mushroom_line}");
    assert!(!mushroom_line.contains("poison"), "{mushroom_line}");
    let bitter = world.examine(linde, ExamineTarget::BitterBerries);
    assert!(bitter.contains("They taste bitter."), "{bitter}");
    assert!(bitter.contains("They are poisonous."), "{bitter}");
    assert_eq!(world.sim_time(), 0);

    let refused = world.step(&[cmd("raw-mushroom", nils, Attempt::Eat(Material::Mushroom))]);
    assert_eq!(
        refused.results[0].message,
        "Raw mushrooms do not feed you. They must be cooked."
    );
    assert!(!refused.results[0].success);
    assert_eq!(world.material_tally(Material::Mushroom), 2);
    assert!(world.observe(nils).unwrap().events.iter().all(|event| {
        !event.text.contains("Raw mushrooms") && !event.text.contains("must be cooked")
    }));
    assert!(world.observe(linde).unwrap().events.iter().all(|event| {
        !event.text.contains("Raw mushrooms") && !event.text.contains("mushroom")
    }));
    let learned = world.examine(nils, ExamineTarget::Mushrooms);
    assert!(
        learned.contains("They must be cooked. Raw mushrooms do not feed you."),
        "{learned}"
    );
    assert!(!learned.contains("poison"), "{learned}");
    assert!(world
        .observe(nils)
        .unwrap()
        .text
        .contains("Mushrooms lie here, dry. They must be cooked. Raw mushrooms do not feed you."));
    let hidden = world.examine(linde, ExamineTarget::Mushrooms);
    assert!(hidden.contains("mushrooms"), "{hidden}");
    assert!(!hidden.contains("must be cooked"), "{hidden}");
    assert!(!hidden.contains("do not feed"), "{hidden}");
    assert!(!hidden.contains("poison"), "{hidden}");
    world.step(&[cmd(
        "raw-bitter",
        linde,
        Attempt::Eat(Material::BitterBerry),
    )]);
    assert_eq!(world.material_tally(Material::BitterBerry), 1);
    assert_eq!(world.observe(linde).unwrap().self_info.illness, "ill");
    assert_eq!(world.observe(linde).unwrap().self_info.hunger, "not hungry");

    world.step(&[cmd("pocket", sera, Attempt::Gather(Material::Berry))]);
    while world.observe(sera).unwrap().self_info.energy != "tiring" {
        world.step(&[]);
        assert!(world.sim_time() < 40, "energy should reach tiring");
    }
    let hunger = world.observe(sera).unwrap().self_info.hunger.clone();
    assert_eq!(hunger, "hungry");
    let eaten = world.step(&[cmd("eat", sera, Attempt::Eat(Material::Berry))]);
    assert!(
        eaten.results[0]
            .message
            .starts_with("You eat a handful of berries."),
        "{}",
        eaten.results[0].message
    );
    let ate = world.observe(sera).unwrap();
    assert_eq!(ate.self_info.energy, "rested");
    assert_eq!(ate.self_info.hunger, hunger);
    assert!(ate.events.iter().all(|event| {
        event.text != "You eat a handful of berries."
            && event.text != "Sera eats a handful of berries."
    }));
    assert!(!ate.text.contains("energy:"));

    world.step(&[cmd("pick", nils, Attempt::Gather(Material::Mushroom))]);
    for n in 0..3 {
        world.step(&[cmd(&format!("to-fire-{n}"), nils, Attempt::Move(Dir::East))]);
    }
    assert_eq!(world.actor_cell(nils), Some((6, 1)));
    world.step(&[cmd("light", nils, Attempt::Light)]);
    let hunger_before = world.observe(nils).unwrap().self_info.hunger.clone();
    world.step(&[cmd("meal", nils, Attempt::Cook)]);
    let cooked = world.observe(nils).unwrap();
    assert!(cooked.events.iter().all(|event| {
        event.text != "You cook the mushroom over the fire." && !event.text.contains("Nils cooks")
    }));
    assert_eq!(cooked.self_info.hunger, hunger_before);
    assert_eq!(world.material_tally(Material::Mushroom), 2);
    assert!(world
        .observe(nils)
        .unwrap()
        .text
        .contains("a cooked mushroom"));
    assert!(world
        .observe(sera)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Nils cooks the mushroom over the fire."));
    while world.observe(nils).unwrap().self_info.hunger != "hungry" {
        world.step(&[]);
        assert!(world.sim_time() < 80, "hunger should arrive");
    }
    let fed = world.step(&[cmd("eat-mushroom", nils, Attempt::Eat(Material::Mushroom))]);
    assert_eq!(
        fed.results[0].message,
        "You eat the cooked mushroom. The hunger eases."
    );
    assert_eq!(world.observe(nils).unwrap().self_info.hunger, "not hungry");
    assert!(world
        .observe(nils)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "The hunger eases."));
    assert!(world.observe(nils).unwrap().events.iter().all(|event| {
        event.text != "You eat the cooked mushroom." && !event.text.contains("Nils eats")
    }));
    assert_eq!(world.material_tally(Material::Mushroom), 1);
    assert!(world
        .observe(sera)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Nils eats the cooked mushroom."));
}

#[test]
fn pale_berries_do_not_feed_until_they_are_cooked() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 1, 4).unwrap();
    let west = world.spawn_at("West", "w", 0, 4).unwrap();
    let far = world.spawn_at("Far", "f", 5, 1).unwrap();
    let sweet_body = world.spawn_at("Sera", "s", 6, 1).unwrap();
    let sweet = world.examine(sweet_body, ExamineTarget::Berries);
    assert!(sweet.contains("They are good to eat raw."), "{sweet}");
    let fed = world.step(&[cmd("sweet", sweet_body, Attempt::Eat(Material::Berry))]);
    assert!(
        fed.results[0].message == "You eat a handful of berries. You were rested."
            || fed.results[0].message == "You eat a handful of berries. The tiredness eases.",
        "{}",
        fed.results[0].message
    );
    assert_eq!(world.observe(sweet_body).unwrap().self_info.illness, "well");
    let look = world.observe(near).unwrap().text;
    assert!(look.contains("Pale berries lie here, dry."), "{look}");
    assert!(!look.contains("must be cooked"), "{look}");
    assert!(!look.contains("poison"), "{look}");
    assert!(!look.contains("good to eat raw"), "{look}");
    let examined = world.examine(near, ExamineTarget::PaleBerries);
    assert!(examined.contains("pale berries"), "{examined}");
    assert!(!examined.contains("must be cooked"), "{examined}");
    assert!(!examined.contains("poison"), "{examined}");
    let hunger = world.observe(near).unwrap().self_info.hunger.clone();
    let energy = world.observe(near).unwrap().self_info.energy.clone();
    let eaten = world.step(&[cmd("raw", near, Attempt::Eat(Material::PaleBerry))]);
    assert_eq!(
        eaten.results[0].message,
        "You eat a handful of pale berries. They do not feed you. They must be cooked."
    );
    assert_eq!(world.observe(near).unwrap().self_info.hunger, hunger);
    assert_eq!(world.observe(near).unwrap().self_info.energy, energy);
    assert_eq!(world.material_tally(Material::PaleBerry), 1);
    let after = world.observe(near).unwrap();
    assert!(after
        .events
        .iter()
        .all(|event| event.text != "You eat a handful of pale berries."));
    assert!(after.events.iter().all(|event| {
        !event.text.contains("Near eats")
            && !event.text.contains("do not feed")
            && !event.text.contains("must be cooked")
    }));
    assert!(after
        .text
        .contains("Pale berries lie here, dry. They must be cooked."));
    let seen = world.observe(west).unwrap();
    assert!(seen
        .events
        .iter()
        .any(|event| event.text == "Near eats a handful of pale berries."));
    assert!(seen.events.iter().all(|event| {
        !event.text.contains("do not feed") && !event.text.contains("must be cooked")
    }));
    let hidden = world.examine(west, ExamineTarget::PaleBerries);
    assert!(hidden.contains("pale berries"), "{hidden}");
    assert!(!hidden.contains("must be cooked"), "{hidden}");
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("pale")));

    world.step(&[cmd("pick", near, Attempt::Gather(Material::PaleBerry))]);
    for step in 0..5 {
        world.step(&[cmd(&format!("east-{step}"), near, Attempt::Move(Dir::East))]);
    }
    for step in 0..3 {
        world.step(&[cmd(
            &format!("north-{step}"),
            near,
            Attempt::Move(Dir::North),
        )]);
    }
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    let lit = world.step(&[cmd("light", near, Attempt::Light)]);
    assert!(lit.results[0].success, "{}", lit.results[0].message);
    let cooked = world.step(&[cmd("cook", near, Attempt::Cook)]);
    assert_eq!(
        cooked.results[0].message,
        "You cook the pale berries over the fire."
    );
    let cooked_look = world.observe(near).unwrap();
    assert!(cooked_look.events.iter().all(|event| {
        event.text != "You cook the pale berries over the fire."
            && !event.text.contains("Near cooks")
    }));
    assert!(cooked_look
        .events
        .iter()
        .all(|event| !event.text.contains("Near cooks")));
    assert!(cooked_look
        .text
        .contains("a cooked handful of pale berries"));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Near cooks the pale berries over the fire."));
    while world.observe(near).unwrap().self_info.hunger != "hungry" {
        world.step(&[]);
        assert!(world.sim_time() < 80, "hunger should arrive");
    }
    let meal = world.step(&[cmd("meal", near, Attempt::Eat(Material::PaleBerry))]);
    assert_eq!(
        meal.results[0].message,
        "You eat the cooked pale berries. The hunger eases."
    );
    assert_eq!(world.observe(near).unwrap().self_info.hunger, "not hungry");
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "The hunger eases."));
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != "You eat the cooked pale berries."));
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("Near eats the cooked pale berries.")));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Near eats the cooked pale berries."));
}

#[test]
fn cooking_does_not_make_poison_berries_food() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 4, 1).unwrap();
    let west = world.spawn_at("West", "w", 5, 1).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let examined = world.examine(near, ExamineTarget::BitterBerries);
    assert!(examined.contains("dark berries"), "{examined}");
    assert!(examined.contains("They taste bitter."), "{examined}");
    assert!(examined.contains("They are poisonous."), "{examined}");
    assert!(
        !examined.contains("Cooking does not make them food."),
        "{examined}"
    );
    world.step(&[cmd("pick", near, Attempt::Gather(Material::BitterBerry))]);
    world.step(&[cmd("east", near, Attempt::Move(Dir::East))]);
    world.step(&[cmd("east-2", near, Attempt::Move(Dir::East))]);
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    let bark = world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    assert!(bark.results[0].success, "{}", bark.results[0].message);
    let branch = world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    assert!(branch.results[0].success, "{}", branch.results[0].message);
    world.step(&[cmd("back", near, Attempt::Move(Dir::West))]);
    world.step(&[cmd("back-2", near, Attempt::Move(Dir::West))]);
    assert_eq!(world.actor_cell(near), Some((4, 1)));
    let lit = world.step(&[cmd("light", near, Attempt::Light)]);
    assert!(lit.results[0].success, "{}", lit.results[0].message);
    let before = world.observe(near).unwrap().self_info;
    assert_eq!(before.illness, "well");
    let cooked = world.step(&[cmd("cook", near, Attempt::Cook)]);
    assert!(!cooked.results[0].success);
    assert_eq!(
        cooked.results[0].message,
        "Cooking does not make dark berries food."
    );
    assert_eq!(world.material_tally(Material::BitterBerry), 2);
    let after = world.observe(near).unwrap().self_info;
    assert_eq!(after.illness, "well");
    assert_eq!(after.hunger, before.hunger);
    let refusal = "Cooking does not make dark berries food.";
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == refusal));
    for witness in [west, far] {
        assert!(world
            .observe(witness)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != refusal && !event.text.contains("cooks") }));
    }
    let eaten = world.step(&[cmd("eat", near, Attempt::Eat(Material::BitterBerry))]);
    assert_eq!(
        eaten.results[0].message,
        "You eat a handful of poison berries. They taste bitter. Your stomach turns."
    );
    let after_eat = world.observe(near).unwrap();
    assert!(after_eat.text.contains("Poison berries lie here"));
    assert!(after_eat.text.contains("They taste bitter."));
    assert!(after_eat.text.contains("They are poisonous."));
    assert!(after_eat
        .events
        .iter()
        .any(|event| event.text == "Your stomach turns."));
    assert!(after_eat
        .events
        .iter()
        .all(|event| !event.text.contains("Near's stomach")));
    let known = world.examine(near, ExamineTarget::BitterBerries);
    assert!(known.contains("They taste bitter."), "{known}");
    assert!(known.contains("They are poisonous."), "{known}");
    let hidden = world.examine(west, ExamineTarget::BitterBerries);
    assert!(hidden.contains("dark berries"), "{hidden}");
    assert!(hidden.contains("They taste bitter."), "{hidden}");
    assert!(hidden.contains("They are poisonous."), "{hidden}");
    assert_eq!(world.observe(near).unwrap().self_info.illness, "ill");
    assert_eq!(world.observe(near).unwrap().self_info.hunger, before.hunger);
    assert_eq!(world.material_tally(Material::BitterBerry), 1);
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Near eats a handful of dark berries."));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Near eats a handful of dark berries."));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| { !event.text.contains("stomach") }));
}

#[test]
fn open_sky_chills_and_burns_while_shelter_and_mud_hold_them_off() {
    let mut world = world_at(10_000);
    let open = world.spawn_at("Sera", "s", 4, 3).unwrap();
    let sheltered = world.spawn_at("Otto", "o", 6, 1).unwrap();
    let clock = world.sim_time();
    let _ = world.examine(open, ExamineTarget::SelfBody);
    assert_eq!(world.sim_time(), clock);
    for _ in 0..8 {
        world.step(&[]);
    }
    let outside = world.observe(open).unwrap();
    let under = world.observe(sheltered).unwrap();
    assert_eq!(outside.self_info.cold, "cold");
    assert_eq!(outside.self_info.wind, "in the wind");
    assert_eq!(outside.self_info.sunburn, "pink");
    assert_eq!(under.self_info.cold, "warm");
    assert_eq!(under.self_info.wind, "out of the wind");
    assert_eq!(under.self_info.sunburn, "unburned");
    assert!(!outside.text.contains("0.04"));
    assert!(outside
        .events
        .iter()
        .any(|event| event.text == "You are chilly. The wind is on you."));
    assert!(outside
        .events
        .iter()
        .any(|event| event.text == "You are cold. The wind is on you."));
    assert!(outside.events.iter().all(|event| {
        !event.text.contains("Sera is chilly")
            && !event.text.contains("Sera is cold")
            && !event.text.contains("on them")
    }));
    assert!(under
        .events
        .iter()
        .any(|event| event.text == "Sera is chilly. The wind is on them."));
    assert!(under
        .events
        .iter()
        .any(|event| event.text == "Sera is cold. The wind is on them."));
    assert!(under.events.iter().all(|event| {
        !event.text.contains("You are chilly") && !event.text.contains("You are cold")
    }));

    let mut mud = world_at(10_000);
    let bare = mud.spawn_at("Sera", "s", 0, 2).unwrap();
    let covered = mud.spawn_at("Nils", "n", 1, 2).unwrap();
    mud.step(&[cmd("scoop", covered, Attempt::Gather(Material::Mud))]);
    mud.step(&[cmd("smear", covered, Attempt::Smear)]);
    assert_eq!(mud.observe(covered).unwrap().self_info.mud, "mud-smeared");
    for _ in 0..6 {
        mud.step(&[]);
    }
    assert_eq!(mud.observe(bare).unwrap().self_info.sunburn, "pink");
    assert_eq!(mud.observe(covered).unwrap().self_info.sunburn, "unburned");
    assert_ne!(mud.observe(covered).unwrap().self_info.mud, "bare skin");
}

#[test]
fn red_ochre_holds_the_sun_off_until_the_rain_washes_it() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let seen = world.spawn_at("Seen", "s", 6, 2).unwrap();
    let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
    let bare = world.spawn_at("Bare", "b", 4, 3).unwrap();
    let none = world.step(&[cmd("none", bare, Attempt::SmearOchre)]);
    assert_eq!(none.results[0].message, "You have no ochre to smear.");
    assert!(world
        .observe(bare)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "You have no ochre to smear."));
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| { event.text != "You have no ochre to smear." }));
    assert!(world
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("ochre")));
    let got = world.step(&[cmd("ochre", near, Attempt::Gather(Material::Ochre))]);
    assert!(got.results[0].success, "{}", got.results[0].message);
    let smeared = world.step(&[cmd("smear", near, Attempt::SmearOchre)]);
    assert_eq!(
        smeared.results[0].message,
        "You smear red ochre on your skin."
    );
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("smear")));
    assert!(world
        .observe(seen)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Near smears red ochre on their skin."));
    assert!(world
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("smear")));
    let covered = world.observe(near).unwrap();
    assert!(covered.text.contains("Your skin is red-ochre-smeared."));
    assert!(covered.text.contains("The red ochre holds the sun off."));
    assert!(world
        .examine(seen, ExamineTarget::Person("Near".into()))
        .contains("The red ochre holds the sun off."));
    world.step(&[cmd("south", near, Attempt::Move(Dir::South))]);
    assert_eq!(world.actor_cell(near), Some((6, 2)));
    while world.observe(bare).unwrap().self_info.sunburn != "burned" {
        assert!(!world.is_raining());
        world.step(&[]);
        assert!(world.sim_time() < 16);
    }
    let still = world.observe(near).unwrap();
    assert_eq!(still.self_info.ochre, "red-ochre-smeared");
    assert_ne!(still.self_info.sunburn, "burned");
    assert!(still.text.contains("The red ochre holds the sun off."));
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("burned") && !event.text.contains("The sun burns")));
    assert_eq!(world.observe(bare).unwrap().self_info.sunburn, "burned");
    while !world.is_raining() {
        world.step(&[]);
    }
    world.step(&[]);
    let washed = world.observe(near).unwrap();
    assert_eq!(washed.self_info.ochre, "bare");
    assert!(!washed.text.contains("The red ochre holds the sun off."));
    assert!(!washed.text.contains("red-ochre-smeared"));
    assert!(washed
        .events
        .iter()
        .any(|event| event.text == "The rain washes the red ochre off you."));
    assert!(world
        .observe(seen)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "The rain washes the red ochre off Near."));
    assert!(world
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("ochre")));
}

#[test]
fn mud_on_the_skin_keeps_the_same_sun_from_burning() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    for (id, dir) in [
        ("w", Dir::West),
        ("w2", Dir::West),
        ("n", Dir::North),
        ("n2", Dir::North),
    ] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    assert_eq!(world.actor_cell(near), Some((2, 1)));
    let scooped = world.step(&[cmd("mud", near, Attempt::Gather(Material::Mud))]);
    assert_eq!(scooped.results[0].message, "You gather a handful of mud.");
    let smeared = world.step(&[cmd("smear", near, Attempt::Smear)]);
    assert_eq!(smeared.results[0].message, "You smear mud on your skin.");
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != "You smear mud on your skin."));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Near smears mud on their skin."));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("smear") && !event.text.contains("mud")));
    while world.observe(west).unwrap().self_info.sunburn != "burned" {
        assert!(
            !world.is_raining(),
            "the contrast has to show before the rain"
        );
        world.step(&[]);
        assert!(
            world.sim_time() < 16,
            "sun should burn the bare skin before the rain"
        );
    }
    let covered = world.observe(near).unwrap().self_info;
    let bare = world.observe(west).unwrap().self_info;
    assert_eq!(covered.mud, "mud-smeared");
    assert_eq!(bare.mud, "bare skin");
    assert_ne!(covered.sunburn, "burned");
    assert_eq!(bare.sunburn, "burned");
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains("Your skin is mud-smeared."));
    assert!(world
        .observe(west)
        .unwrap()
        .text
        .contains("Your skin is bare skin."));
    assert!(world
        .observe(west)
        .unwrap()
        .text
        .contains("Your skin is burned."));
}

#[test]
fn clear_day_sun_hurts_bare_skin_and_stays_on_that_body() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    let shade = world.spawn_at("Shade", "s", 6, 1).unwrap();
    let open = world.observe(west).unwrap();
    assert!(open.text.contains("The sky is clear."), "{}", open.text);
    assert!(open.text.contains("Your skin is unburned."));
    assert!(open.text.contains("Your skin is bare skin."));
    assert!(!open.text.contains("The sun is hot"));
    for (id, dir) in [
        ("w", Dir::West),
        ("w2", Dir::West),
        ("n", Dir::North),
        ("n2", Dir::North),
    ] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    world.step(&[cmd("mud", near, Attempt::Gather(Material::Mud))]);
    let smeared = world.step(&[cmd("smear", near, Attempt::Smear)]);
    assert_eq!(smeared.results[0].message, "You smear mud on your skin.");
    let pink = "Your skin is pink. The sun is hot on your skin.";
    let burned = "Your skin is burned. The sun burns. It hurts.";
    while world.observe(west).unwrap().self_info.sunburn != "burned" {
        assert!(!world.is_raining());
        world.step(&[]);
        assert!(world.sim_time() < 16);
    }
    let bare = world.observe(west).unwrap();
    assert_eq!(bare.self_info.sunburn, "burned");
    assert_eq!(bare.self_info.mud, "bare skin");
    assert!(bare.text.contains("The sky is clear."), "{}", bare.text);
    assert!(bare.text.contains("Evening is coming."), "{}", bare.text);
    assert!(bare.text.contains("Your skin is burned."));
    assert!(!bare.text.contains("The sun burns"), "{}", bare.text);
    assert!(!bare.text.contains("The sun is hot"), "{}", bare.text);
    assert!(bare.events.iter().any(|event| event.text == pink));
    assert!(bare.events.iter().any(|event| event.text == burned));
    assert!(bare
        .events
        .iter()
        .all(|event| !event.text.contains("West's")));
    let covered = world.observe(near).unwrap();
    assert_eq!(covered.self_info.mud, "mud-smeared");
    assert_ne!(covered.self_info.sunburn, "burned");
    assert!(covered.text.contains("Your skin is mud-smeared."));
    let witness = world.observe(shade).unwrap();
    assert_eq!(witness.self_info.sunburn, "unburned");
    assert!(witness.events.iter().all(|event| {
        event.text != pink && event.text != burned && !event.text.contains("skin is")
    }));
    let seen = world.examine(shade, ExamineTarget::Person("West".into()));
    assert!(seen.contains("Their skin is bare skin."), "{seen}");
    assert!(!seen.contains("burned"), "{seen}");
    assert!(!seen.contains("pink"), "{seen}");
    assert!(!seen.contains("The sun burns"), "{seen}");
    while world.sim_time() < 16 {
        world.step(&[]);
    }
    world.step(&[]);
    assert!(world.is_raining());
    let soaked = world.observe(west).unwrap();
    assert!(
        soaked.text.contains("Your skin is burned.") || soaked.text.contains("Your skin is pink."),
        "{}",
        soaked.text
    );
    assert!(
        !soaked.text.contains("The sun is hot on your skin."),
        "{}",
        soaked.text
    );
    assert!(!soaked.text.contains("The sun burns"), "{}", soaked.text);
    assert!(
        soaked
            .text
            .contains("The rain takes the sun off your skin."),
        "{}",
        soaked.text
    );
    assert!(soaked
        .events
        .iter()
        .any(|event| event.text == "The rain takes the sun off your skin."));
    let sheltered = world.observe(shade).unwrap();
    assert!(!sheltered.text.contains("The rain takes the sun off"));
    assert!(sheltered
        .events
        .iter()
        .all(|event| event.text != "The rain takes the sun off your skin."));
    let washed = world.observe(near).unwrap();
    assert!(washed
        .events
        .iter()
        .any(|event| event.text == "The rain washes the mud off you."));
    assert_eq!(washed.self_info.mud, "bare skin");
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "The rain washes the mud off Near."));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("washes the mud")));

    let mut night = World::valley(Tuning::evening(), 1);
    let shade = night.spawn_at("Shade", "s", 6, 1).unwrap();
    while !night
        .observe(shade)
        .unwrap()
        .text
        .contains("Night has fallen. The stars are out")
    {
        night.step(&[]);
        assert!(night.sim_time() < 120);
    }
    assert_eq!(night.observe(shade).unwrap().self_info.sunburn, "unburned");
    let out = night.step(&[cmd("out", shade, Attempt::Move(Dir::West))]);
    assert!(out.results[0].success, "{}", out.results[0].message);
    assert_eq!(night.actor_cell(shade), Some((5, 1)));
    for _ in 0..6 {
        night.step(&[]);
    }
    let dark = night.observe(shade).unwrap();
    assert_eq!(dark.self_info.sunburn, "unburned");
    assert!(dark.text.contains("Night has fallen. The stars are out"));
    assert!(!dark.text.contains("The sky is clear."));
    assert!(!dark.text.contains("The sun is hot"));
    assert!(!dark.text.contains("The sun burns"));
}

#[test]
fn shade_and_rest_ease_a_sunburn_and_the_open_sky_does_not() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let seen = world.spawn_at("Seen", "s", 5, 3).unwrap();
    let shade = world.spawn_at("Shade", "h", 6, 1).unwrap();
    while world.observe(near).unwrap().self_info.sunburn != "burned" {
        assert!(!world.is_raining());
        world.step(&[]);
        assert!(
            world.sim_time() < 16,
            "the burn should arrive before the rain"
        );
    }
    assert_eq!(world.observe(seen).unwrap().self_info.sunburn, "burned");
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains("Your skin is burned."));
    assert!(!world.observe(near).unwrap().text.contains("The sun burns"));
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains("Evening is coming."));
    for (id, dir) in [
        ("n", Dir::North),
        ("n2", Dir::North),
        ("e", Dir::East),
        ("e2", Dir::East),
    ] {
        let walked = world.step(&[cmd(id, near, Attempt::Move(dir))]);
        assert!(walked.results[0].success, "{}", walked.results[0].message);
    }
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    let mut rests = 0;
    while world.observe(near).unwrap().self_info.sunburn != "unburned" {
        let rested = world.step(&[cmd(&format!("r{rests}"), near, Attempt::Rest)]);
        assert!(
            rested.results[0]
                .message
                .starts_with("You rest under the rock."),
            "{}",
            rested.results[0].message
        );
        rests += 1;
        assert!(rests < 8, "shade rest should ease the burn");
    }
    assert!(rests < 6, "rests {rests}");
    let eased = world.observe(near).unwrap();
    assert_eq!(eased.self_info.sunburn, "unburned");
    assert!(eased.text.contains("Your skin is unburned."));
    assert!(!eased.text.contains("The sun burns"));
    assert!(!eased.text.contains("The sun is hot"));
    assert!(eased
        .events
        .iter()
        .any(|event| event.text == "The burn settles."));
    assert!(eased
        .events
        .iter()
        .any(|event| event.text == "The pink fades."));
    assert!(eased
        .events
        .iter()
        .all(|event| !event.text.contains("Near's skin")));
    assert_ne!(world.observe(seen).unwrap().self_info.sunburn, "unburned");
    assert_eq!(world.observe(shade).unwrap().self_info.sunburn, "unburned");
    assert!(world
        .observe(shade)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != "The burn settles." && event.text != "The pink fades."));
    assert!(world
        .observe(shade)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Near rests under the rock."));
}

#[test]
fn shade_does_not_call_the_sun_hot_and_does_not_burn() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
    let quiet = world.spawn_at("Quiet", "q", 6, 1).unwrap();
    let hot = "The sun is hot on your skin.";
    while world.observe(near).unwrap().self_info.sunburn != "pink" {
        world.step(&[]);
        assert!(!world.is_raining());
        assert!(world.sim_time() < 16);
    }
    let open = world.observe(near).unwrap();
    assert!(open.text.contains("The sky is clear."), "{}", open.text);
    assert!(open.text.contains("Evening is coming."), "{}", open.text);
    assert!(open.text.contains("Your skin is pink."));
    assert!(!open.text.contains(hot), "{}", open.text);
    assert!(!world.observe(seen).unwrap().text.contains(hot));
    let said = open
        .events
        .iter()
        .filter(|event| event.text.contains(hot))
        .count();
    assert_eq!(said, 1);
    assert!(world
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains(hot)));
    for (id, dir) in [
        ("n", Dir::North),
        ("n2", Dir::North),
        ("e", Dir::East),
        ("e2", Dir::East),
    ] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    assert!(!world.is_raining());
    let under = world.observe(near).unwrap();
    assert!(
        under
            .text
            .contains("You are on the valley floor, under the rock overhang."),
        "{}",
        under.text
    );
    assert!(!under.text.contains(hot), "{}", under.text);
    assert!(!under.text.contains("The sun burns"), "{}", under.text);
    assert_eq!(under.self_info.sunburn, "pink");
    for _ in 0..3 {
        world.step(&[]);
    }
    let stayed = world.observe(near).unwrap();
    assert!(!world.is_raining());
    assert!(!stayed.text.contains(hot), "{}", stayed.text);
    assert!(!stayed.text.contains("The sun burns"), "{}", stayed.text);
    assert_ne!(stayed.self_info.sunburn, "burned");
    assert_eq!(
        stayed
            .events
            .iter()
            .filter(|event| event.text.contains(hot))
            .count(),
        said
    );
    assert!(world
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains(hot) && !event.text.contains("The sun burns")));
    let still_open = world.observe(seen).unwrap();
    assert!(
        still_open.text.contains("Your skin is pink.")
            || still_open.text.contains("Your skin is burned."),
        "{}",
        still_open.text
    );
    assert!(!still_open.text.contains(hot), "{}", still_open.text);
    assert!(
        !still_open.text.contains("The sun burns"),
        "{}",
        still_open.text
    );
}

#[test]
fn evening_and_the_ice_do_not_say_the_sun_burns_and_morning_does() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 2, 1).unwrap();
    while world.sim_time() < 12 {
        world.step(&[]);
    }
    let iced = world.observe(near).unwrap();
    assert!(iced.text.contains("Evening is coming."), "{}", iced.text);
    assert!(iced.text.contains("You are cold."), "{}", iced.text);
    assert_eq!(iced.self_info.sunburn, "burned");
    assert!(iced.text.contains("Your skin is burned."));
    assert!(!iced.text.contains("The sun burns"), "{}", iced.text);
    assert!(!iced.text.contains("The sun is hot"), "{}", iced.text);
    let dawn = world.tuning.night_at + world.tuning.night_len;
    while world.sim_time() < dawn || world.is_raining() {
        world.step(&[]);
        assert!(world.sim_time() < dawn + 4);
    }
    let morning = world.observe(near).unwrap();
    assert!(
        morning.text.contains("Morning is on the valley."),
        "{}",
        morning.text
    );
    assert!(!morning.text.contains("Evening is coming."));
    assert!(
        morning.text.contains("The sun is hot on your skin.")
            || morning.text.contains("The sun burns. It hurts."),
        "{}",
        morning.text
    );
}

#[test]
fn a_night_in_the_open_settles_a_burn_and_nobody_else_hears_it() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let shade = world.spawn_at("Shade", "h", 6, 1).unwrap();
    while world.sim_time() < 72 {
        world.step(&[]);
    }
    assert_eq!(world.observe(near).unwrap().self_info.sunburn, "burned");
    assert_eq!(world.observe(shade).unwrap().self_info.sunburn, "unburned");
    while world.sim_time() < 80 {
        world.step(&[]);
    }
    let eased = world.observe(near).unwrap();
    assert_eq!(eased.self_info.sunburn, "pink");
    assert!(eased.text.contains("Your skin is pink."));
    assert!(world.is_raining());
    assert!(
        !eased.text.contains("The sun is hot on your skin."),
        "{}",
        eased.text
    );
    assert!(!eased.text.contains("The sun burns"));
    assert!(
        eased.text.contains("The rain takes the sun off your skin."),
        "{}",
        eased.text
    );
    let settled: Vec<_> = eased
        .events
        .iter()
        .filter(|event| event.text == "The burn settles.")
        .collect();
    assert_eq!(settled.len(), 1);
    assert!(settled[0].time >= 72 && settled[0].time < 80);
    assert!(eased
        .events
        .iter()
        .all(|event| event.text != "The pink fades."));
    assert_eq!(world.observe(shade).unwrap().self_info.sunburn, "unburned");
    assert!(world
        .observe(shade)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != "The burn settles." && event.text != "The pink fades."));
}

#[test]
fn rain_washes_open_mud_and_leaves_the_rock() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    pair(
        &mut world,
        "w",
        near,
        Attempt::Move(Dir::West),
        west,
        Attempt::Move(Dir::West),
    );
    pair(
        &mut world,
        "w2",
        near,
        Attempt::Move(Dir::West),
        west,
        Attempt::Move(Dir::West),
    );
    pair(
        &mut world,
        "nw",
        near,
        Attempt::Move(Dir::North),
        west,
        Attempt::Move(Dir::West),
    );
    pair(
        &mut world,
        "nn",
        near,
        Attempt::Move(Dir::North),
        west,
        Attempt::Move(Dir::North),
    );
    assert_eq!(world.actor_cell(near), Some((2, 1)));
    pair(
        &mut world,
        "pool",
        near,
        Attempt::Gather(Material::Mud),
        west,
        Attempt::Move(Dir::North),
    );
    assert_eq!(world.actor_cell(west), Some((2, 1)));
    pair(
        &mut world,
        "smear-gather",
        near,
        Attempt::Smear,
        west,
        Attempt::Gather(Material::Mud),
    );
    let apart = pair(
        &mut world,
        "walk-smear",
        near,
        Attempt::Move(Dir::East),
        west,
        Attempt::Smear,
    );
    assert_eq!(apart.results[1].message, "You smear mud on your skin.");
    for id in ["e2", "e3", "e4"] {
        world.step(&[cmd(id, near, Attempt::Move(Dir::East))]);
    }
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    assert_eq!(world.actor_cell(west), Some((2, 1)));
    assert_eq!(world.actor_cell(far), Some((3, 3)));
    while world.sim_time() < 16 {
        world.step(&[]);
    }
    assert!(!world.is_raining());
    assert_eq!(world.observe(near).unwrap().self_info.mud, "mud-smeared");
    assert_eq!(world.observe(west).unwrap().self_info.mud, "mud-smeared");
    world.step(&[]);
    assert_eq!(world.sim_time(), 17);
    assert!(world.is_raining());
    let sheltered = world.observe(near).unwrap();
    let open = world.observe(west).unwrap();
    let witness = world.observe(far).unwrap();
    assert_eq!(sheltered.self_info.mud, "mud-smeared");
    assert_eq!(open.self_info.mud, "bare skin");
    assert!(sheltered.text.contains("Your skin is mud-smeared."));
    assert!(open.text.contains("Your skin is bare skin."));
    assert!(open
        .events
        .iter()
        .any(|event| event.text == "The rain washes the mud off you."));
    assert!(witness
        .events
        .iter()
        .any(|event| event.text == "The rain washes the mud off West."));
    assert!(sheltered
        .events
        .iter()
        .all(|event| !event.text.contains("washes the mud")));
    assert!(witness
        .events
        .iter()
        .all(|event| !event.text.contains("washes the mud off Near")));
    assert!(open
        .events
        .iter()
        .all(|event| !event.text.contains("washes the mud off Near")));
}

#[test]
fn the_rock_takes_the_cold_off() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let step = world.spawn_at("Step", "s", 5, 1).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let plain = world.step(&[cmd("plain", far, Attempt::Move(Dir::East))]);
    assert_eq!(plain.results[0].message, "You walk east.");
    while world.observe(step).unwrap().self_info.cold != "cold" {
        world.step(&[]);
        assert!(world.sim_time() < 20, "open sky should make Step cold");
    }
    assert_eq!(world.observe(step).unwrap().self_info.wind, "in the wind");
    assert_eq!(world.observe(near).unwrap().self_info.cold, "warm");
    assert_eq!(
        world.observe(near).unwrap().self_info.wind,
        "out of the wind"
    );
    let arrived = world.step(&[cmd("in", step, Attempt::Move(Dir::East))]);
    assert_eq!(
        arrived.results[0].message,
        "You walk east, under the rock overhang, out of the wind."
    );
    assert_eq!(world.actor_cell(step), Some((6, 1)));
    let look = world.observe(step).unwrap();
    assert_eq!(look.self_info.cold, "warm");
    assert_eq!(look.self_info.wind, "out of the wind");
    assert!(look.text.contains("You are warm."));
    assert!(look.text.contains("You are out of the wind."));
    let place = look
        .text
        .lines()
        .find(|line| line.starts_with("You are on the"))
        .unwrap_or("");
    assert!(
        place.starts_with("You are on the valley floor, under the rock overhang."),
        "{place}"
    );
    assert!(
        place.contains(
            "Rock leans overhead. Near is here. Wood left here stays out of the weather."
        ),
        "{place}"
    );
    assert!(!place.ends_with("Near is here."), "{place}");
    assert!(place.contains("A pot sits here."), "{place}");
    assert!(place.contains("bark"), "{place}");
    assert!(place.contains("branch"), "{place}");
    assert!(!place.contains("1 person"), "{place}");
    assert!(!place.contains("cell"), "{place}");
}

#[test]
fn rain_reaches_an_open_body_and_the_outside_floor_stays() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let step = world.spawn_at("Step", "s", 4, 1).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    while world.sim_time() < 15 {
        world.step(&[]);
        assert!(world.sim_time() <= 15, "the evening should reach 15");
    }
    let before = world.observe(near).unwrap();
    let west_before = before
        .text
        .lines()
        .find(|line| line.starts_with("West:"))
        .unwrap_or("");
    assert_eq!(
        west_before,
        "West: hoofprints mark the outside floor. The rotting wood is the next square south of that floor. West of that floor, Step stands on the dark berries."
    );
    let plain = world.step(&[cmd("out", step, Attempt::Move(Dir::East))]);
    assert_eq!(plain.results[0].message, "You walk east.");
    assert_eq!(world.sim_time(), 16);
    let at_sixteen = world.observe(step).unwrap();
    let south_sixteen = at_sixteen
        .text
        .lines()
        .find(|line| line.starts_with("South:"))
        .unwrap_or("");
    assert_eq!(
        south_sixteen,
        "South: the last branch. The deer left hoofprints on the ground there."
    );
    assert!(
        !south_sixteen.contains("Hoofprints mark the ground"),
        "{south_sixteen}"
    );
    let place_sixteen = at_sixteen
        .text
        .lines()
        .find(|line| line.starts_with("You stand on the outside floor."))
        .unwrap_or("");
    assert_eq!(
        place_sixteen,
        "You stand on the outside floor. Hoofprints mark the ground. The rock overhang is the next square east. The last branch is the next square south. Worms are in the dry ground here. Carve an antler into an antler pick to dig them."
    );
    let opening = at_sixteen
        .text
        .lines()
        .find(|line| line.starts_with("The last branch is rotting"))
        .unwrap_or("");
    assert_eq!(opening, "The last branch is rotting to the south.");
    assert!(!opening.contains("deer"), "{opening}");
    assert!(
        !at_sixteen.text.contains("under that branch"),
        "{}",
        at_sixteen.text
    );
    assert!(!at_sixteen.text.contains("under it"), "{}", at_sixteen.text);
    assert!(
        !at_sixteen.text.contains("that ground"),
        "{}",
        at_sixteen.text
    );
    assert!(
        !at_sixteen.text.contains("Hoofprints mark that branch"),
        "{}",
        at_sixteen.text
    );
    assert!(
        !at_sixteen.text.contains("on that branch"),
        "{}",
        at_sixteen.text
    );
    assert!(
        !at_sixteen.text.contains("Wood left in the open"),
        "{}",
        at_sixteen.text
    );
    assert!(
        !at_sixteen.text.contains("the rotting wood"),
        "{}",
        at_sixteen.text
    );
    assert_eq!(
        at_sixteen
            .text
            .matches("Hoofprints mark the ground")
            .count(),
        1
    );
    assert_eq!(
        at_sixteen
            .visible_cells
            .iter()
            .find(|cell| cell.x == 5 && cell.y == 2)
            .unwrap()
            .branch_pile,
        1
    );
    let near_sixteen = world.observe(near).unwrap();
    let west_sixteen = near_sixteen
        .text
        .lines()
        .find(|line| line.starts_with("West:"))
        .unwrap_or("");
    assert_eq!(
        west_sixteen,
        "West: hoofprints mark the outside floor. The last branch is the next square south of that floor."
    );
    let arrived = world.step(&[cmd("in", step, Attempt::Move(Dir::East))]);
    assert_eq!(
        arrived.results[0].message,
        "You walk east, under the rock overhang, out of the wind and the rain."
    );
    assert!(world.is_raining());
    assert_eq!(world.sim_time(), 17);
    assert_eq!(world.actor_cell(step), Some((6, 1)));
    let step_look = world.observe(step).unwrap();
    assert_eq!(step_look.self_info.cold, "warm");
    assert_eq!(step_look.self_info.wind, "out of the wind");
    assert_eq!(step_look.self_info.wetness, "dry");
    assert_eq!(step_look.self_info.sunburn, "burned");
    let step_place = step_look
        .text
        .lines()
        .find(|line| line.starts_with("You are on the"))
        .unwrap_or("");
    assert_eq!(
        step_place,
        "You are on the valley floor, under the rock overhang. Rock leans overhead, and the rain does not reach this ground. Near is here. The branches here stay out of the weather. A handful of berries lies here. It is good to eat raw. A deer ate the rest. A pot sits here. A pile of branches lies here. A pile of bark lies here. A lump of ochre lies on the rock face. A clear stone of quartz lies on the rock face. It is a clear stone. It can spark. Dry moss grows in the shade here. A bone lies here. Hoofprints mark the ground."
    );
    let step_west = step_look
        .text
        .lines()
        .find(|line| line.starts_with("West:"))
        .unwrap_or("");
    assert_eq!(
        step_west,
        "West: hoofprints mark the outside floor. The last branch is gone from the next square south. Hoofprints mark that ground."
    );
    assert_eq!(
        step_look
            .visible_cells
            .iter()
            .find(|cell| cell.x == 5 && cell.y == 2)
            .unwrap()
            .branch_pile,
        0
    );
    assert!(step_look
        .text
        .contains("The rain does not wet your clothes."));
    assert!(!step_look.text.contains("Your clothes are dry."));
    assert!(!step_west.contains("poison berries"), "{step_west}");
    assert!(
        !step_west.contains("valley floor, hoofprints"),
        "{step_west}"
    );
    let near_look = world.observe(near).unwrap();
    assert_eq!(near_look.self_info.cold, "warm");
    assert_eq!(near_look.self_info.wind, "out of the wind");
    assert_eq!(near_look.self_info.wetness, "dry");
    assert_eq!(near_look.self_info.sunburn, "unburned");
    assert!(near_look
        .text
        .contains("The rain does not wet your clothes."));
    assert!(!near_look.text.contains("Your clothes are dry."));
    let near_place = near_look
        .text
        .lines()
        .find(|line| line.starts_with("You are on the"))
        .unwrap_or("");
    assert_eq!(
        near_place,
        "You are on the valley floor, under the rock overhang. Rock leans overhead, and the rain does not reach this ground. Step is here. The branches here stay out of the weather. A handful of berries lies here. It is good to eat raw. A deer ate the rest. A pot sits here. A pile of branches lies here. A pile of bark lies here. A lump of ochre lies on the rock face. A clear stone of quartz lies on the rock face. It is a clear stone. It can spark. Dry moss grows in the shade here. A bone lies here. Hoofprints mark the ground."
    );
    let far_look = world.observe(far).unwrap();
    assert_eq!(far_look.self_info.cold, "cold");
    assert_eq!(far_look.self_info.wind, "in the wind");
    assert_eq!(far_look.self_info.wetness, "wet");
    assert!(far_look.text.contains("The rain wets your clothes."));
    assert!(!far_look.text.contains("Your clothes are wet."));
    assert!(!far_look.text.contains("Your clothes are dry."));
    assert!(far_look.text.contains("You are cold."));
    assert!(far_look.text.contains("You are in the wind."));
    assert!(far_look
        .text
        .contains("You are on the valley floor, in the rain."));
    assert!(!far_look.text.contains("Near"));
    assert!(!far_look.text.contains("Step"));
    assert!(!far_look.text.contains("rock overhang"));
    let snap = far_look
        .events
        .iter()
        .find(|event| event.time == 3 && event.text.contains("snapped"))
        .map(|event| event.text.as_str())
        .unwrap_or("");
    assert_eq!(
        snap,
        "A deer steps east. A branch snapped under the deer one cell north and two cells east, in the open."
    );
    assert!(!snap.contains("in the rain"), "{snap}");
    let rot = far_look
        .events
        .iter()
        .find(|event| event.time == 8 && event.text.contains("rots"))
        .map(|event| event.text.as_str())
        .unwrap_or("");
    assert_eq!(
        rot,
        "A branch rots away one cell north and two cells east, in the open. You hear branches breaking."
    );
    assert!(!rot.contains("in the rain"), "{rot}");
    let rain = far_look
        .events
        .iter()
        .find(|event| event.text.contains("rain") || event.text.contains("Rain"))
        .map(|event| event.text.as_str())
        .unwrap_or("");
    assert_eq!(rain, "The rain wets your clothes.");
    let step_rot = step_look
        .events
        .iter()
        .find(|event| event.time == 16 && event.text.contains("rots"))
        .map(|event| event.text.as_str())
        .unwrap_or("");
    assert_eq!(
        step_rot,
        "The last branch rots away one cell south and one cell west, in the rain. You hear a branch break."
    );
    assert!(!step_rot.contains("branches breaking"), "{step_rot}");
    assert!(!step_rot.contains("A branch rots away"), "{step_rot}");
}

#[test]
fn poison_berries_are_not_food_and_still_water_freezes() {
    let mut cook = world_at(10_000);
    let sera = cook.spawn_at("Sera", "s", 6, 1).unwrap();
    cook.step(&[cmd("bark", sera, Attempt::Gather(Material::Bark))]);
    cook.step(&[cmd("branch", sera, Attempt::Gather(Material::Branch))]);
    cook.step(&[cmd("west", sera, Attempt::Move(Dir::West))]);
    cook.step(&[cmd("west-2", sera, Attempt::Move(Dir::West))]);
    assert_eq!(cook.actor_cell(sera), Some((4, 1)));
    assert_eq!(cook.material_tally(Material::BitterBerry), 2);
    cook.step(&[cmd("light", sera, Attempt::Light)]);
    cook.step(&[cmd("meal", sera, Attempt::Cook)]);
    let refused = cook.observe(sera).unwrap();
    assert!(refused.events.iter().any(|event| event
        .text
        .contains("Cooking does not make dark berries food.")));
    assert_eq!(cook.material_tally(Material::BitterBerry), 2);
    assert!(matches!(
        parse_attempt("eat bark").unwrap(),
        Attempt::Eat(Material::Bark)
    ));

    let mut pool = World::valley(Tuning::evening(), 1);
    pool.tuning.chill_at = 0;
    pool.tuning.rain_starts_at = 10_000;
    let sera = pool.spawn_at("Sera", "s", 2, 1).unwrap();
    let refused = pool.step(&[cmd("ice", sera, Attempt::Drink)]);
    assert_eq!(
        refused.results[0].message,
        "The still water is frozen. Ice covers it."
    );
    let frozen = pool.observe(sera).unwrap();
    assert!(frozen.events.iter().all(|event| {
        event.text != "The still water is frozen."
            && event.text != "The still water is frozen. Ice covers it."
    }));
    assert_eq!(frozen.self_info.illness, "well");
    assert!(frozen.visible_cells.iter().any(|cell| cell.frozen));
    let otto = pool.spawn_at("Otto", "o", 0, 1).unwrap();
    let ran = pool.step(&[cmd("run", otto, Attempt::Drink)]);
    assert!(
        ran.results[0]
            .message
            .starts_with("You drink from the brook."),
        "{}",
        ran.results[0].message
    );
    assert!(pool
        .observe(otto)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("Otto drinks from the brook")));

    let mut thaw = world_at(10_000);
    thaw.tuning.chill_at = 0;
    let sera = thaw.spawn_at("Sera", "s", 6, 1).unwrap();
    thaw.step(&[cmd("bark", sera, Attempt::Gather(Material::Bark))]);
    thaw.step(&[cmd("branch", sera, Attempt::Gather(Material::Branch))]);
    for n in 0..4 {
        thaw.step(&[cmd(&format!("west-{n}"), sera, Attempt::Move(Dir::West))]);
    }
    assert_eq!(thaw.actor_cell(sera), Some((2, 1)));
    let iced = thaw.step(&[cmd("ice", sera, Attempt::Drink)]);
    assert_eq!(
        iced.results[0].message,
        "The still water is frozen. Ice covers it."
    );
    assert!(thaw.observe(sera).unwrap().events.iter().all(|event| {
        event.text != "The still water is frozen."
            && event.text != "The still water is frozen. Ice covers it."
    }));
    assert!(thaw
        .observe(sera)
        .unwrap()
        .text
        .contains("The still water is frozen."));
    thaw.step(&[cmd("light", sera, Attempt::Light)]);
    let thawed = thaw.step(&[cmd("thaw", sera, Attempt::Drink)]);
    assert!(
        thawed.results[0]
            .message
            .starts_with("You drink the still water."),
        "{}",
        thawed.results[0].message
    );
    let open = thaw.observe(sera).unwrap();
    assert!(open
        .events
        .iter()
        .all(|event| !event.text.contains("Sera drinks the still water")));
    assert!(open.text.contains("keeping the still water from freezing"));
}

#[test]
fn a_fire_warms_the_body_standing_on_it_and_not_the_next_one() {
    let mut world = world_at(10_000);
    let sera = world.spawn_at("Sera", "s", 6, 1).unwrap();
    let otto = world.spawn_at("Otto", "o", 4, 3).unwrap();
    let nils = world.spawn_at("Nils", "n", 5, 2).unwrap();
    world.step(&[cmd("bark", sera, Attempt::Gather(Material::Bark))]);
    world.step(&[cmd("branch", sera, Attempt::Gather(Material::Branch))]);
    world.step(&[cmd("west", sera, Attempt::Move(Dir::West))]);
    assert_eq!(world.actor_cell(sera), Some((5, 1)));
    while world.observe(sera).unwrap().self_info.cold != "cold" {
        world.step(&[]);
        assert!(world.sim_time() < 20, "open sky should make her cold");
    }
    let sun_at_light = world.observe(sera).unwrap().self_info.sunburn.clone();
    world.step(&[cmd("light", sera, Attempt::Light)]);
    let lit = world.observe(sera).unwrap();
    assert!(lit.text.contains("It warms you."));
    assert_eq!(lit.self_info.wind, "in the wind");
    assert!(lit
        .events
        .iter()
        .any(|event| event.text.contains("warming at the fire")));
    assert_ne!(lit.self_info.cold, "cold");
    for _ in 0..3 {
        world.step(&[]);
    }
    let warmed = world.observe(sera).unwrap();
    assert_eq!(warmed.self_info.cold, "warm");
    assert!(warmed
        .events
        .iter()
        .any(|event| event.text.contains("is warm at the fire")));
    assert!(!warmed.text.contains("does not reach you"));
    assert_eq!(world.observe(otto).unwrap().self_info.cold, "cold");
    assert!(!world
        .observe(otto)
        .unwrap()
        .text
        .contains("does not reach you"));
    let beside = world.observe(nils).unwrap();
    assert_eq!(beside.self_info.cold, "cold");
    assert!(beside
        .text
        .contains("The heat of the fire does not reach you."));
    assert!(!beside.text.contains("still on you"));
    let clock = world.sim_time();
    let examined = world.examine(nils, ExamineTarget::Fire);
    assert_eq!(world.sim_time(), clock);
    assert!(examined.contains("does not reach you from here"));
    assert_ne!(
        warmed.self_info.sunburn, "unburned",
        "the fire does not fade a sunburn"
    );
    assert!(sun_at_light == "pink" || sun_at_light == "burned" || sun_at_light == "unburned");
    world.step(&[cmd("off", sera, Attempt::Move(Dir::West))]);
    let stepped = world.observe(sera).unwrap();
    assert_eq!(world.actor_cell(sera), Some((4, 1)));
    assert_eq!(stepped.self_info.cold, "warm");
    assert_eq!(stepped.self_info.wind, "in the wind");
    assert!(stepped
        .text
        .contains("The warmth of the fire is still on you."));
    assert!(stepped
        .text
        .contains("The heat of the fire does not reach you."));
    assert!(stepped
        .events
        .iter()
        .any(|event| event.text.contains("still has the warmth of the fire")));
    assert_eq!(world.observe(nils).unwrap().self_info.cold, "cold");
    world.step(&[cmd("back", sera, Attempt::Move(Dir::East))]);
    assert_eq!(world.actor_cell(sera), Some((5, 1)));
    let mut ticks = 0;
    while world
        .observe(sera)
        .unwrap()
        .visible_cells
        .iter()
        .any(|cell| cell.x == 5 && cell.y == 1 && cell.fire)
        && ticks < world.tuning.fire_ticks as usize + 2
    {
        world.step(&[]);
        ticks += 1;
    }
    assert!(
        world
            .observe(sera)
            .unwrap()
            .visible_cells
            .iter()
            .all(|cell| !(cell.x == 5 && cell.y == 1 && cell.fire)),
        "the fire goes out"
    );
    let lingered = world.observe(sera).unwrap();
    assert_eq!(lingered.self_info.cold, "warm");
    assert_eq!(lingered.self_info.wind, "in the wind");
    assert!(lingered
        .text
        .contains("The warmth of the fire is still on you."));
    assert!(lingered
        .events
        .iter()
        .any(|event| event.text.contains("still has the warmth of the fire")));
    let mut ticks = 0;
    while world.observe(sera).unwrap().self_info.cold == "warm" {
        let body = world.observe(sera).unwrap();
        assert!(
            body.text
                .contains("The warmth of the fire is still on you."),
            "the leftover sentence stays while the word is warm from the fire"
        );
        world.step(&[]);
        ticks += 1;
        assert!(ticks < 12, "open wind should cool her");
    }
    let cooled = world.observe(sera).unwrap();
    assert_eq!(cooled.self_info.cold, "chilly");
    assert!(!cooled.text.contains("still on you"));
    assert_eq!(world.observe(otto).unwrap().self_info.cold, "cold");
    assert_eq!(world.observe(nils).unwrap().self_info.cold, "cold");
}

#[test]
fn rain_still_puts_out_an_open_fire_and_the_warmth_lingers() {
    let mut world = world_at(8);
    let sera = world.spawn_at("Sera", "s", 6, 1).unwrap();
    let beside = world.spawn_at("Nils", "n", 5, 2).unwrap();
    world.step(&[cmd("bark", sera, Attempt::Gather(Material::Bark))]);
    world.step(&[cmd("branch", sera, Attempt::Gather(Material::Branch))]);
    world.step(&[cmd("west", sera, Attempt::Move(Dir::West))]);
    world.step(&[cmd("light", sera, Attempt::Light)]);
    assert!(world.observe(sera).unwrap().text.contains("It warms you."));
    assert!(world
        .observe(beside)
        .unwrap()
        .text
        .contains("The heat of the fire does not reach you."));
    while !world.is_raining() {
        world.step(&[]);
        assert!(world.sim_time() < 20, "rain should arrive");
    }
    let doused = world.observe(sera).unwrap();
    assert!(doused
        .events
        .iter()
        .any(|event| event.text.contains("Rain puts out the fire.")));
    assert!(doused
        .visible_cells
        .iter()
        .all(|cell| !(cell.x == 5 && cell.y == 1 && cell.fire)));
    assert_eq!(doused.self_info.cold, "warm");
    assert!(doused
        .text
        .contains("The warmth of the fire is still on you."));
    let mut ticks = 0;
    while world.observe(sera).unwrap().self_info.cold == "warm" {
        assert!(world
            .observe(sera)
            .unwrap()
            .text
            .contains("The warmth of the fire is still on you."));
        world.step(&[]);
        ticks += 1;
        assert!(
            ticks < 12,
            "open wind should cool her after the leftover warmth"
        );
    }
    assert!(
        ticks > 4,
        "the leftover warmth holds the warm word for a few ticks"
    );
    let cooled = world.observe(sera).unwrap();
    assert_eq!(cooled.self_info.cold, "chilly");
    assert!(!cooled.text.contains("still on you"));
    assert_ne!(world.observe(beside).unwrap().self_info.cold, "warm");
    assert!(!world.observe(beside).unwrap().text.contains("still on you"));
}

#[test]
fn under_the_rock_the_leftover_sentence_goes_once_the_counter_is_spent() {
    let mut world = world_at(10_000);
    let sera = world.spawn_at("Sera", "s", 6, 1).unwrap();
    world.step(&[cmd("bark", sera, Attempt::Gather(Material::Bark))]);
    world.step(&[cmd("branch", sera, Attempt::Gather(Material::Branch))]);
    world.step(&[cmd("west", sera, Attempt::Move(Dir::West))]);
    world.step(&[cmd("light", sera, Attempt::Light)]);
    while world.observe(sera).unwrap().self_info.cold != "warm" {
        world.step(&[]);
        assert!(
            world.sim_time() < 16,
            "the fire should warm her before the rain"
        );
    }
    assert!(!world.observe(sera).unwrap().text.contains("still on you"));
    world.step(&[cmd("shelter", sera, Attempt::Move(Dir::East))]);
    assert_eq!(world.actor_cell(sera), Some((6, 1)));
    let mut saw = false;
    for _ in 0..6 {
        let body = world.observe(sera).unwrap();
        assert_eq!(body.self_info.cold, "warm");
        assert_eq!(body.self_info.wind, "out of the wind");
        if body.text.contains("still on you") {
            saw = true;
            world.step(&[]);
        } else {
            break;
        }
    }
    assert!(
        saw,
        "the leftover sentence shows under the rock while it is still on her"
    );
    let sheltered = world.observe(sera).unwrap();
    assert_eq!(sheltered.self_info.cold, "warm");
    assert_eq!(sheltered.self_info.wind, "out of the wind");
    assert!(!sheltered.text.contains("still on you"));
}

#[test]
fn a_stone_on_the_slope_can_be_carried_and_stays_unseen_from_far() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Sera", "s", 2, 0).unwrap();
    let far = world.spawn_at("Otto", "o", 6, 4).unwrap();
    let clock = world.sim_time();
    let here = world.observe(near).unwrap();
    assert!(here.text.contains("A loose stone lies here."));
    assert!(here.text.contains("slope"));
    assert!(here
        .visible_cells
        .iter()
        .any(|cell| cell.x == 2 && cell.y == 0 && cell.stone_pile == 1));
    assert!(here.map.contains('s'));
    assert!(here
        .allowed_actions
        .iter()
        .any(|action| action == "gather stone"));
    let examined = world.examine(near, ExamineTarget::Stone);
    assert_eq!(world.sim_time(), clock);
    assert_eq!(
        examined,
        "A stone lies in contact with the ground here, in the open. The stone is dry. The sky is open above it. It is a found stone."
    );
    assert!(!examined.contains("pile"));
    assert!(!examined.contains("1 loose"));
    let distant = world.observe(far).unwrap();
    assert!(!distant.text.contains("stone"));
    assert!(distant
        .visible_cells
        .iter()
        .all(|cell| cell.stone_pile == 0 && cell.loose_stone == 0));
    assert_eq!(
        world.examine(far, ExamineTarget::Stone),
        "You see no stone from here."
    );

    world.step(&[cmd("get", near, Attempt::Gather(Material::Stone))]);
    let carried = world.observe(near).unwrap();
    assert!(carried
        .self_info
        .carrying
        .iter()
        .any(|item| item.material == "stone" && item.count == 1));
    assert!(carried.text.contains("You have a dry stone."));
    assert!(!carried.text.contains("1 stone"));
    let held = world.examine(near, ExamineTarget::Stone);
    assert_eq!(
        held,
        "A dry stone is carried by you, in the open, and it is a found stone."
    );
    assert!(!held.contains("1 stone"));
    assert!(!carried.text.contains("A loose stone lies here."));
    assert!(carried
        .visible_cells
        .iter()
        .all(|cell| cell.stone_pile == 0 && cell.loose_stone == 0));
    assert_eq!(world.material_tally(Material::Stone), 1);
    let id = world.material_items(Material::Stone)[0].0;
    assert_eq!(world.material_items(Material::Stone)[0].1, Some(near));

    world.step(&[cmd("east", near, Attempt::Move(Dir::East))]);
    world.step(&[cmd("drop", near, Attempt::Drop(Material::Stone))]);
    let dropped = world.material_items(Material::Stone);
    assert_eq!(dropped.len(), 1);
    assert_eq!(dropped[0].0, id);
    assert_eq!(dropped[0].1, None);
    assert_eq!((dropped[0].2, dropped[0].3), (3, 0));
    assert_eq!(world.actor_cell(near), Some((3, 0)));
    let back = world.observe(near).unwrap();
    assert!(back.text.contains("A loose stone lies here."));
    let dropped_text = world.examine(near, ExamineTarget::Stone);
    assert_eq!(
        dropped_text,
        "A stone lies in contact with the ground here, in the open. The stone is dry. The sky is open above it. It is a found stone."
    );
    assert!(!dropped_text.contains("1 loose"));
    assert!(!dropped_text.contains("pile"));
    assert!(back.visible_cells.iter().any(|cell| {
        cell.x == 3 && cell.y == 0 && cell.loose_stone == 1 && cell.stone_pile == 0
    }));
    assert!(world
        .observe(far)
        .unwrap()
        .visible_cells
        .iter()
        .all(|cell| cell.stone_pile == 0 && cell.loose_stone == 0));

    world.step(&[cmd("again", near, Attempt::Gather(Material::Stone))]);
    let again = world.material_items(Material::Stone);
    assert_eq!(again, vec![(id, Some(near), 3, 0)]);
    assert_eq!(world.material_tally(Material::Stone), 1);
    assert!(!world
        .observe(near)
        .unwrap()
        .text
        .contains("A loose stone lies here."));
}

#[test]
fn clay_on_the_brook_bank_is_named_and_gathered() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let far = world.spawn_at("Far", "f", 3, 2).unwrap();
    let west = world.spawn_at("West", "w", 6, 4).unwrap();
    assert_eq!(world.material_tally(Material::Clay), 2);
    let from_near = world.observe(near).unwrap().text;
    assert!(
        from_near.contains("Clay lies on the brook bank to the north and west."),
        "{from_near}"
    );
    let from_far = world.observe(far).unwrap().text;
    assert!(
        from_far.contains("Clay lies on the brook bank to the west."),
        "{from_far}"
    );
    assert!(!world.observe(west).unwrap().text.contains("brook bank"));
    for (id, dir) in [("w1", Dir::West), ("w2", Dir::West), ("n", Dir::North)] {
        let step = world.step(&[cmd(id, near, Attempt::Move(dir))]);
        assert!(step.results[0].success, "{}", step.results[0].message);
    }
    assert_eq!(world.actor_cell(near), Some((2, 2)));
    let here = world.observe(near).unwrap().text;
    assert!(here.contains("Clay lies on the bank here."), "{here}");
    assert!(!here.contains("brook bank to the"), "{here}");
    let yours = "You gather a lump of clay.";
    let seen = "Near gathers a lump of clay.";
    let gathered = world.step(&[cmd("clay", near, Attempt::Gather(Material::Clay))]);
    assert_eq!(gathered.results[0].message, yours);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != yours));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == seen));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("gather")));
    assert!(arms_sentence(&world.observe(near).unwrap().text).contains("a lump of clay"));
    assert_eq!(world.material_tally(Material::Clay), 2);
    assert_eq!(world.material_items(Material::Clay).len(), 1);
    assert_eq!(world.material_items(Material::Clay)[0].1, Some(near));
    let after = world.observe(far).unwrap().text;
    assert!(
        after.contains("A lump of clay lies on the brook bank to the west."),
        "{after}"
    );
}

#[test]
fn an_unfired_pot_is_shaped_from_clay() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let far = world.spawn_at("Far", "f", 3, 2).unwrap();
    let west = world.spawn_at("West", "w", 6, 4).unwrap();
    assert!(matches!(
        parse_attempt("shape pot").unwrap(),
        Attempt::Shape
    ));
    let denied = world.step(&[cmd("no-clay", west, parse_attempt("shape clay").unwrap())]);
    assert_eq!(denied.results[0].message, "You have no clay to shape.");
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("no clay")));
    for (id, dir) in [("w1", Dir::West), ("w2", Dir::West), ("n", Dir::North)] {
        let step = world.step(&[cmd(id, near, Attempt::Move(dir))]);
        assert!(step.results[0].success, "{}", step.results[0].message);
    }
    assert_eq!(world.actor_cell(near), Some((2, 2)));
    let yours = "You shape the clay into an unfired pot.";
    let seen = "Near shapes the clay into an unfired pot.";
    let shaped = world.step(&[cmd("shape", near, Attempt::Shape)]);
    assert_eq!(shaped.results[0].message, yours);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != yours && !event.text.contains("shapes the clay")));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == seen));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("shapes the clay")));
    let here = world.observe(near).unwrap().text;
    assert!(
        here.contains("An unfired clay pot lies here. It is not hard yet.")
            && here.contains("A lump of clay lies here. It is unfired."),
        "{here}"
    );
    let gathered = world.step(&[cmd("clay", near, Attempt::Gather(Material::Clay))]);
    assert_eq!(gathered.results[0].message, "You gather a lump of clay.");
    let carried = world.step(&[cmd("shape-carried", near, Attempt::Shape)]);
    assert_eq!(carried.results[0].message, yours);
    let arms = arms_sentence(&world.observe(near).unwrap().text);
    assert!(arms.contains("an unfired clay pot"), "{arms}");
    assert!(!arms.contains("lump of clay"), "{arms}");
    let examined = world.examine(near, ExamineTarget::Clay);
    assert!(
        examined.contains("An unfired clay pot is carried by you")
            && examined.contains("It is not hard yet."),
        "{examined}"
    );
    let again = world.step(&[cmd("again", near, Attempt::Shape)]);
    assert_eq!(again.results[0].message, "The clay is already a pot.");
}

#[test]
fn an_unfired_pot_hardens_in_the_fire_and_boils() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 2, 2).unwrap();
    let far = world.spawn_at("Far", "f", 5, 2).unwrap();
    let west = world.spawn_at("West", "w", 6, 4).unwrap();
    assert!(matches!(
        parse_attempt("fire pot").unwrap(),
        Attempt::FirePot
    ));
    assert!(matches!(
        parse_attempt("bake pot").unwrap(),
        Attempt::FirePot
    ));
    let denied = world.step(&[cmd("no-pot", west, parse_attempt("fire pot").unwrap())]);
    assert_eq!(
        denied.results[0].message,
        "You have no unfired pot to fire."
    );
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("unfired pot")));
    world.step(&[cmd("clay", near, Attempt::Gather(Material::Clay))]);
    world.step(&[cmd("shape", near, Attempt::Shape)]);
    let before = world.observe(near).unwrap().text;
    let before_pot = world.examine(near, ExamineTarget::Pot);
    assert!(
        before.contains("an unfired clay pot") && !before.contains("not done yet"),
        "{before}"
    );
    assert!(
        before_pot.contains("An unfired clay pot is carried by you")
            && before_pot.contains("It is not hard yet.")
            && !before_pot.contains("not done yet"),
        "{before_pot}"
    );
    let cold = world.step(&[cmd("cold", near, Attempt::FirePot)]);
    assert_eq!(cold.results[0].message, "There is no fire burning here.");
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("no fire")));
    let north = world.step(&[cmd("north", near, Attempt::Move(Dir::North))]);
    assert!(north.results[0].success, "{}", north.results[0].message);
    let water = world.step(&[cmd("water", near, Attempt::Gather(Material::Water))]);
    assert!(water.results[0].success, "{}", water.results[0].message);
    for step in 0..4 {
        let moved = world.step(&[cmd(&format!("east-{step}"), near, Attempt::Move(Dir::East))]);
        assert!(moved.results[0].success, "{}", moved.results[0].message);
    }
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    world.step(&[cmd("seed-pot", near, Attempt::Gather(Material::Pot))]);
    world.step(&[cmd("aside", near, Attempt::Move(Dir::South))]);
    world.step(&[cmd("drop-seed", near, Attempt::Drop(Material::Pot))]);
    world.step(&[cmd("return", near, Attempt::Move(Dir::North))]);
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    let lit = world.step(&[cmd("light", near, Attempt::Light)]);
    assert!(lit.results[0].success, "{}", lit.results[0].message);
    let set = world.step(&[cmd("bake", near, parse_attempt("bake pot").unwrap())]);
    assert_eq!(
        set.results[0].message,
        "You set the unfired pot by the fire. It leaves your arms. It is not hard yet."
    );
    let seen = "Near sets the unfired pot by the fire. It leaves their arms.";
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("unfired pot")));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == seen));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("sets the unfired pot")));
    let waiting = world.observe(near).unwrap().text;
    let waiting_pot = world.examine(near, ExamineTarget::Pot);
    assert!(
        waiting.contains("An unfired clay pot is hardening in the fire. It is not done yet.")
            && waiting.contains("A small fire burns here."),
        "{waiting}"
    );
    assert!(
        waiting_pot.contains("An unfired clay pot is hardening in the fire. It is not done yet."),
        "{waiting_pot}"
    );
    let far_wait = world.observe(far).unwrap().text;
    assert!(far_wait.contains("It is not done yet."), "{far_wait}");
    assert_eq!(world.material_tally(Material::Bowl), 0);
    let after_set = world.sim_time();
    for _ in 0..3 {
        world.step(&[]);
        assert_eq!(world.material_tally(Material::Pot), 1);
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("An unfired clay pot is hardening in the fire."));
    }
    world.step(&[]);
    assert_eq!(world.sim_time(), after_set + 4);
    assert_eq!(world.material_tally(Material::Pot), 2);
    assert_eq!(world.material_tally(Material::Bowl), 0);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "The clay pot hardens. It can hold water."));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "The clay pot hardens. It can hold water."));
    let hard = world.observe(near).unwrap().text;
    assert!(
        hard.contains("A pot sits here.") && hard.contains("A small fire burns here."),
        "{hard}"
    );
    assert!(!hard.contains("hardening in the fire."), "{hard}");
    assert!(!hard.contains("not done yet"), "{hard}");
    let examined = world.examine(near, ExamineTarget::Pot);
    assert!(
        examined.contains("It can boil water over a fire.") && !examined.contains("not done yet"),
        "{examined}"
    );
    world.step(&[cmd("take-pot", near, Attempt::Gather(Material::Pot))]);
    let boiled = world.step(&[cmd("boil", near, Attempt::Boil)]);
    assert!(
        boiled.results[0]
            .message
            .contains("You boil the drink in the pot over the fire and drink it."),
        "{}",
        boiled.results[0].message
    );
    assert!(world.material_items(Material::Water).is_empty());
    assert_eq!(world.material_tally(Material::Pot), 2);
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains("A small fire burns here."));
}

#[test]
fn clay_on_the_bank_hardens_by_the_fire_into_a_bowl() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 2, 2).unwrap();
    let far = world.spawn_at("Far", "f", 6, 4).unwrap();
    assert_eq!(world.material_tally(Material::Clay), 2);
    assert_eq!(world.material_tally(Material::Pot), 1);
    assert_eq!(world.actor_cell(near), Some((2, 2)));
    let seen = world.observe(near).unwrap();
    assert!(seen.text.contains("Clay lies on the bank here."));
    assert!(seen
        .visible_cells
        .iter()
        .any(|cell| { cell.x == 2 && cell.y == 2 && cell.clay_pile == 2 && cell.bowl == 0 }));
    assert!(seen
        .allowed_actions
        .iter()
        .any(|action| action == "gather clay"));
    let clock = seen.sim_time;
    let examined = world.examine(near, ExamineTarget::Clay);
    assert!(examined.contains("A pile of 2 lumps of clay"));
    assert!(examined.contains("It is unfired. It cannot boil water."));
    assert!(!examined.to_lowercase().contains("kiln"));
    assert!(!examined.to_lowercase().contains("wheel"));
    assert_eq!(world.observe(near).unwrap().sim_time, clock);
    let distant = world.observe(far).unwrap();
    assert!(distant
        .visible_cells
        .iter()
        .all(|cell| { cell.clay_pile == 0 && cell.loose_clay == 0 && cell.bowl == 0 }));
    assert_eq!(
        world.examine(far, ExamineTarget::Clay),
        "You see no clay from here."
    );

    world.step(&[cmd("gather", near, Attempt::Gather(Material::Clay))]);
    assert_eq!(world.material_tally(Material::Clay), 2);
    let carried = world
        .material_items(Material::Clay)
        .into_iter()
        .find(|item| item.1 == Some(near))
        .expect("the gathered lump");
    let id = carried.0;
    world.step(&[cmd("north", near, Attempt::Move(Dir::North))]);
    world.step(&[cmd("water", near, Attempt::Gather(Material::Water))]);
    for step in 0..4 {
        world.step(&[cmd(&format!("east-{step}"), near, Attempt::Move(Dir::East))]);
    }
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    world.step(&[cmd("light", near, Attempt::Light)]);
    let boiled = world.step(&[cmd("boil", near, Attempt::Boil)]);
    assert!(
        boiled.results[0]
            .message
            .contains("Unfired clay cannot boil water."),
        "{}",
        boiled.results[0].message
    );
    assert!(world
        .material_items(Material::Clay)
        .iter()
        .any(|item| item.0 == id && item.1 == Some(near)));
    assert!(world
        .material_items(Material::Water)
        .iter()
        .any(|item| item.1 == Some(near)));
    assert_eq!(world.material_tally(Material::Pot), 1);
    assert_eq!(world.material_tally(Material::Bowl), 0);

    let set = world.step(&[cmd("set", near, Attempt::SetClay)]);
    assert!(
        set.results[0].message.contains("It is not hard yet."),
        "{}",
        set.results[0].message
    );
    let after_set = world.sim_time();
    assert!(world
        .material_items(Material::Clay)
        .iter()
        .any(|item| item.0 == id && item.1.is_none() && (item.2, item.3) == (6, 1)));
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains("A lump of clay is hardening in the fire."));
    assert_eq!(
        world.examine(near, ExamineTarget::Clay),
        "A lump of clay is hardening in the fire."
    );
    for _ in 0..3 {
        world.step(&[]);
        assert_eq!(world.material_tally(Material::Bowl), 0);
        let waiting = world.observe(near).unwrap().text;
        assert!(waiting.contains("A lump of clay is hardening in the fire."));
        assert!(!waiting.contains("A fired bowl sits here."));
    }
    world.step(&[]);
    assert_eq!(world.sim_time(), after_set + 4);
    assert_eq!(world.material_tally(Material::Bowl), 1);
    let bowl = world.material_items(Material::Bowl);
    assert_eq!(bowl, vec![(id, None, 6, 1)]);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| { event.text.contains("The clay hardens into a bowl.") }));
    let fired = world.observe(near).unwrap().text;
    assert!(fired.contains("A fired bowl sits here."));
    assert!(!fired.contains("is hardening in the fire."));
    let bowl_text = world.examine(near, ExamineTarget::Bowl);
    assert!(bowl_text.contains("A fired bowl lies in contact with the ground"));
    assert!(bowl_text.contains("It can boil water over a fire."));
    assert!(world
        .observe(far)
        .unwrap()
        .visible_cells
        .iter()
        .all(|cell| { cell.clay_pile == 0 && cell.loose_clay == 0 && cell.bowl == 0 }));

    world.step(&[cmd("take", near, Attempt::Gather(Material::Bowl))]);
    assert_eq!(
        world.material_items(Material::Bowl),
        vec![(id, Some(near), 6, 1)]
    );
    let drank = world.step(&[cmd("boil-bowl", near, Attempt::Boil)]);
    assert!(
        drank.results[0].message.contains("boil a drink of water"),
        "{}",
        drank.results[0].message
    );
    assert_eq!(
        world.material_items(Material::Bowl),
        vec![(id, Some(near), 6, 1)]
    );
    assert!(world.material_items(Material::Water).is_empty());
    assert_eq!(world.material_tally(Material::Pot), 1);
    assert_eq!(world.observe(near).unwrap().self_info.thirst, "not thirsty");
}

#[test]
fn clay_stays_soft_when_the_fire_goes_out() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    for step in 0..4 {
        world.step(&[cmd(&format!("west-{step}"), near, Attempt::Move(Dir::West))]);
    }
    world.step(&[cmd("south", near, Attempt::Move(Dir::South))]);
    assert_eq!(world.actor_cell(near), Some((2, 2)));
    world.step(&[cmd("clay", near, Attempt::Gather(Material::Clay))]);
    for step in 0..4 {
        world.step(&[cmd(&format!("back-{step}"), near, Attempt::Move(Dir::East))]);
    }
    world.step(&[cmd("north", near, Attempt::Move(Dir::North))]);
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    world.step(&[cmd("light", near, Attempt::Light)]);
    let goes_out = world.sim_time() - 1 + world.tuning.fire_ticks;
    let set_at = goes_out - world.tuning.clay_ticks + 1;
    while world.sim_time() < set_at {
        world.step(&[]);
    }
    let set = world.step(&[cmd("set", near, Attempt::SetClay)]);
    assert!(set.results[0].success, "{}", set.results[0].message);
    let mut softened = false;
    for _ in 0..8 {
        world.step(&[]);
        if world.observe(near).unwrap().events.iter().any(|event| {
            event
                .text
                .contains("The clay is still soft. The fire is out.")
        }) {
            softened = true;
            break;
        }
    }
    assert!(softened, "the lump should stay soft once the fire is out");
    assert_eq!(world.material_tally(Material::Bowl), 0);
    assert!(world
        .material_items(Material::Clay)
        .iter()
        .any(|item| item.1.is_none()));
}

fn pile_at(world: &World, actor: u32, x: i32, y: i32) -> (u32, u32, u32) {
    let cell = world
        .observe(actor)
        .unwrap()
        .visible_cells
        .into_iter()
        .find(|cell| cell.x == x && cell.y == y)
        .unwrap();
    (cell.bark_pile, cell.branch_pile, cell.pot_pile)
}

fn carrying(world: &World, actor: u32) -> Vec<String> {
    world
        .observe(actor)
        .unwrap()
        .self_info
        .carrying
        .into_iter()
        .map(|item| item.material)
        .collect()
}

#[test]
fn a_fire_spends_carried_fuel_before_the_pile_at_the_feet() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 2, 2).unwrap();
    world.step(&[cmd("clay", near, Attempt::Gather(Material::Clay))]);
    world.step(&[cmd("north", near, Attempt::Move(Dir::North))]);
    world.step(&[cmd("water", near, Attempt::Gather(Material::Water))]);
    for step in 0..4 {
        world.step(&[cmd(&format!("east-{step}"), near, Attempt::Move(Dir::East))]);
    }
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    let (bark_pile, branch_pile, pot_pile) = pile_at(&world, near, 6, 1);
    assert_eq!((bark_pile, branch_pile, pot_pile), (1, 2, 1));
    assert_eq!(carrying(&world, near).len(), 4);

    let lit = world.step(&[cmd("light", near, Attempt::Light)]);
    assert_eq!(
        lit.results[0].message,
        "You light a fire with the bark and the branch from the arms."
    );
    let seen = world.observe(near).unwrap();
    assert!(seen.events.iter().all(|event| {
        event.text != "You light a fire with the bark and the branch from the arms."
            && !event.text.contains("lights a fire")
    }));
    assert!(seen.text.contains("A small fire burns here."));
    assert_eq!(pile_at(&world, near, 6, 1), (1, 2, 1));
    assert_eq!(
        carrying(&world, near),
        vec!["clay".to_string(), "water".to_string()]
    );
    assert!(seen
        .allowed_actions
        .iter()
        .any(|action| action == "gather pot"));

    let took = world.step(&[cmd("pot", near, Attempt::Gather(Material::Pot))]);
    assert!(took.results[0].success, "{}", took.results[0].message);
    assert_eq!(pile_at(&world, near, 6, 1).2, 0);
    assert!(carrying(&world, near).contains(&"pot".to_string()));
    assert!(!carrying(&world, near)
        .iter()
        .any(|item| item == "bark" || item == "branch"));

    let mut piled = world_at(10_000);
    let feet = piled.spawn_at("Feet", "f", 6, 1).unwrap();
    let from_pile = piled.step(&[cmd("light", feet, Attempt::Light)]);
    assert_eq!(
        from_pile.results[0].message,
        "You light a fire with bark and a dry branch from the pile at the feet."
    );
    assert_eq!(pile_at(&piled, feet, 6, 1), (1, 3, 1));
    assert!(carrying(&piled, feet).is_empty());

    let mut mixed = world_at(10_000);
    let half = mixed.spawn_at("Half", "h", 6, 1).unwrap();
    mixed.step(&[cmd("bark", half, Attempt::Gather(Material::Bark))]);
    let mixed_light = mixed.step(&[cmd("light", half, Attempt::Light)]);
    assert_eq!(
        mixed_light.results[0].message,
        "You light a fire with the bark from the arms and a dry branch from the pile at the feet."
    );
    assert_eq!(pile_at(&mixed, half, 6, 1), (1, 3, 1));
    assert!(carrying(&mixed, half).is_empty());

    let mut damp = world_at(3);
    let sera = damp.spawn_at("Sera", "s", 6, 1).unwrap();
    damp.step(&[cmd("bark", sera, Attempt::Gather(Material::Bark))]);
    damp.step(&[cmd("branch", sera, Attempt::Gather(Material::Branch))]);
    damp.step(&[cmd("west", sera, Attempt::Move(Dir::West))]);
    damp.step(&[]);
    damp.step(&[]);
    damp.step(&[cmd("back", sera, Attempt::Move(Dir::East))]);
    let wet = damp.step(&[cmd("light", sera, Attempt::Light)]);
    assert_eq!(
        wet.results[0].message,
        "You light a fire with bark and a dry branch from the pile at the feet."
    );
    assert!(carrying(&damp, sera).contains(&"bark".to_string()));
    assert!(carrying(&damp, sera).contains(&"branch".to_string()));
    assert_eq!(pile_at(&damp, sera, 6, 1), (0, 1, 1));
}

#[test]
fn a_carried_sheet_and_a_carried_pot_are_one_sentence() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let berries = world.examine(near, ExamineTarget::Berries);
    assert!(
        berries.contains("A pile of 4 handfuls of berries"),
        "{berries}"
    );
    let wood = world.examine(near, ExamineTarget::Wood);
    assert!(
        wood.contains("A pile of four branches") || wood.contains("A pile of 4"),
        "{wood}"
    );
    let clock = world.sim_time();
    let ground = world.examine(near, ExamineTarget::Pot);
    assert_eq!(
        ground,
        "A pot lies in contact with the ground here, under the rock overhang. The pot is dry. Rain does not reach it. It can boil water over a fire."
    );
    assert!(!ground.contains("1 pot"));
    assert!(!ground.contains("pile"));
    assert_eq!(world.sim_time(), clock);

    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    world.step(&[cmd("pot", near, Attempt::Gather(Material::Pot))]);
    let held = world.observe(near).unwrap();
    assert!(held.text.contains("a dry sheet of bark"));
    assert!(held.text.contains("a dry pot"));
    assert!(!held.text.contains("1 pot"));
    assert!(!held.text.contains("1 bark"));
    for step in 0..3 {
        world.step(&[cmd(&format!("west-{step}"), near, Attempt::Move(Dir::West))]);
    }
    world.step(&[cmd("south-a", near, Attempt::Move(Dir::South))]);
    world.step(&[cmd("south-b", near, Attempt::Move(Dir::South))]);
    assert_eq!(world.actor_cell(near), Some((3, 3)));
    let away = world.sim_time();
    let bark = world.examine(near, ExamineTarget::Bark);
    let pot = world.examine(near, ExamineTarget::Pot);
    assert_eq!(world.sim_time(), away);
    assert_eq!(bark, "A dry sheet of bark is carried by you, in the open.");
    assert!(!bark.contains("1 sheet"));
    assert!(!bark.contains(". dry"));
    assert_eq!(
        pot,
        "A dry pot is carried by you, in the open, and it can boil water over a fire."
    );
    assert!(!pot.contains("1 pot"));
    assert!(!pot.contains(". dry"));

    world.step(&[cmd("drop", near, Attempt::Drop(Material::Pot))]);
    let dropped = world.examine(near, ExamineTarget::Pot);
    assert_eq!(
        dropped,
        "A pot lies in contact with the ground here, in the open. The pot is dry. The sky is open above it. It can boil water over a fire."
    );
    assert!(!dropped.contains("1 pot"));
    assert!(!dropped.contains("pile"));
    let still = world.examine(near, ExamineTarget::Bark);
    assert_eq!(still, "A dry sheet of bark is carried by you, in the open.");
}

#[test]
fn one_thing_on_the_ground_is_named_and_is_not_a_pile() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let piled = world.examine(near, ExamineTarget::Bark);
    assert!(piled.contains("A pile of 2 sheets of bark"), "{piled}");
    assert!(!piled.contains("pile of 1"));
    let before = world.observe(near).unwrap();
    assert!(!before.text.contains("A sheet of bark lies here."));
    assert!(before.text.contains("A pot sits here."));
    let berries = world.examine(near, ExamineTarget::Berries);
    let wood = world.examine(near, ExamineTarget::Wood);
    assert!(
        berries.contains("A pile of 4 handfuls of berries"),
        "{berries}"
    );
    assert!(wood.contains("A pile of four branches"), "{wood}");

    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    let one = world.observe(near).unwrap();
    assert!(one.text.contains("A sheet of bark lies here."));
    let clock = one.sim_time;
    let sheet = world.examine(near, ExamineTarget::Bark);
    assert_eq!(world.sim_time(), clock);
    assert!(sheet.contains(
        "A sheet of bark lies in contact with the ground here, under the rock overhang. The bark is dry. Rain does not reach it."
    ));
    assert!(sheet.contains("A dry sheet of bark is carried by you, under the rock overhang."));
    assert!(!sheet.contains("pile"), "{sheet}");

    world.step(&[cmd("pot", near, Attempt::Gather(Material::Pot))]);
    for step in 0..3 {
        world.step(&[cmd(&format!("west-{step}"), near, Attempt::Move(Dir::West))]);
    }
    for step in 0..3 {
        world.step(&[cmd(
            &format!("south-{step}"),
            near,
            Attempt::Move(Dir::South),
        )]);
    }
    assert_eq!(world.actor_cell(near), Some((3, 4)));
    world.step(&[cmd("drop-pot", near, Attempt::Drop(Material::Pot))]);
    let pot_look = world.observe(near).unwrap();
    assert!(pot_look.text.contains("A pot sits here."));
    assert!(!pot_look.text.contains("A sheet of bark lies here."));
    let pot = world.examine(near, ExamineTarget::Pot);
    assert_eq!(
        pot,
        "A pot lies in contact with the ground here, in the open. The pot is dry. The sky is open above it. It can boil water over a fire."
    );
    assert!(!pot.contains("pile") && !pot.contains("1 pot"));

    world.step(&[cmd("drop-bark", near, Attempt::Drop(Material::Bark))]);
    let both = world.observe(near).unwrap();
    assert!(both.text.contains("A pot sits here."));
    assert!(both.text.contains("A sheet of bark lies here."));
    let dropped_sheet = world.examine(near, ExamineTarget::Bark);
    assert_eq!(
        dropped_sheet,
        "A sheet of bark lies in contact with the ground here, in the open. The bark is dry. The sky is open above it."
    );
    assert!(!dropped_sheet.contains("pile"));

    world.step(&[cmd("north", near, Attempt::Move(Dir::North))]);
    assert_eq!(world.actor_cell(near), Some((3, 3)));
    let beside = world.observe(near).unwrap();
    let south = beside
        .text
        .split("South:")
        .nth(1)
        .unwrap_or("")
        .split("West:")
        .next()
        .unwrap_or("");
    assert!(south.contains("a pot"), "{south}");
    assert!(south.contains("a sheet of bark"), "{south}");
}

fn map_cell(map: &str, x: i32, y: i32) -> String {
    let row = map.lines().nth((y + 1) as usize).unwrap_or("");
    let start = 2 + (x as usize) * 3;
    row.get(start..start + 3).unwrap_or("").to_string()
}

#[test]
fn look_names_every_kind_and_the_map_marks_what_the_letter_hides() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let rock = world.observe(near).unwrap();
    assert!(
        rock.text.contains("A pile of bark lies here."),
        "{}",
        rock.text
    );
    assert!(rock.text.contains("A pile of branches lies here."));
    assert!(rock.text.contains("Berries lie here"));
    assert!(rock.text.contains("A pot sits here."));
    assert!(!rock.text.contains("A sheet of bark lies here."));
    assert_eq!(map_cell(&rock.map, 6, 1), "@w+");
    assert!(rock.legend.contains("+ more than the letter shows"));
    let berries = world.examine(near, ExamineTarget::Berries);
    assert!(
        berries.contains("A pile of 4 handfuls of berries"),
        "{berries}"
    );

    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    world.step(&[cmd("pot", near, Attempt::Gather(Material::Pot))]);
    for step in 0..3 {
        world.step(&[cmd(&format!("west-{step}"), near, Attempt::Move(Dir::West))]);
    }
    for step in 0..3 {
        world.step(&[cmd(
            &format!("south-{step}"),
            near,
            Attempt::Move(Dir::South),
        )]);
    }
    assert_eq!(world.actor_cell(near), Some((3, 4)));
    world.step(&[cmd("drop-pot", near, Attempt::Drop(Material::Pot))]);
    let pot_only = world.observe(near).unwrap();
    assert!(pot_only.text.contains("A pot sits here."));
    assert_eq!(map_cell(&pot_only.map, 3, 4), "@k ");

    world.step(&[cmd("drop-bark", near, Attempt::Drop(Material::Bark))]);
    let both = world.observe(near).unwrap();
    assert!(both.text.contains("A pot sits here."));
    assert!(both.text.contains("A sheet of bark lies here."));
    assert_eq!(map_cell(&both.map, 3, 4), "@b+");

    world.step(&[cmd("north", near, Attempt::Move(Dir::North))]);
    let beside = world.observe(near).unwrap();
    assert_eq!(map_cell(&beside.map, 3, 4), ".b+");
    assert!(beside.text.contains("South:"));
    let south = beside
        .text
        .split("South:")
        .nth(1)
        .unwrap_or("")
        .split("West:")
        .next()
        .unwrap_or("");
    assert!(south.contains("a pot"), "{south}");
    assert!(south.contains("a sheet of bark"), "{south}");
}

#[test]
fn a_stone_and_a_branch_bind_into_a_spear_whose_point_hardens() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 2, 0).unwrap();
    let beside = world.spawn_at("Beside", "b", 5, 1).unwrap();
    assert!(world.deer_cell().is_some());
    world.step(&[cmd("stone", near, Attempt::Gather(Material::Stone))]);
    let stone_id = world.material_items(Material::Stone)[0].0;
    for step in 0..4 {
        world.step(&[cmd(&format!("east-{step}"), near, Attempt::Move(Dir::East))]);
    }
    world.step(&[cmd("south", near, Attempt::Move(Dir::South))]);
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    let branch_id = world
        .material_items(Material::Branch)
        .into_iter()
        .find(|item| item.1 == Some(near))
        .unwrap()
        .0;
    let branches_before = world.material_tally(Material::Branch);
    let bound = world.step(&[cmd("bind", near, Attempt::BindSpear)]);
    assert_eq!(
        bound.results[0].message,
        "You bind a stone and a branch into a spear."
    );
    assert!(world.material_items(Material::Stone).is_empty());
    assert!(world
        .material_items(Material::Branch)
        .iter()
        .all(|item| item.0 != branch_id));
    assert_eq!(world.material_tally(Material::Branch), branches_before - 1);
    assert_eq!(world.material_tally(Material::Spear), 1);
    let spear = world.spears();
    assert_eq!(spear.len(), 1);
    assert_ne!(spear[0].0, stone_id);
    assert_ne!(spear[0].0, branch_id);
    let spear_id = spear[0].0;
    assert_eq!(spear[0].1, Some(near));
    assert!(!spear[0].2);
    let soft = world.examine(beside, ExamineTarget::Spear);
    assert_eq!(
        soft,
        "A spear with an unhardened point is carried by Near, under the rock overhang."
    );
    assert!(world
        .examine(beside, ExamineTarget::Person("Near".into()))
        .contains("They carry a spear with an unhardened point."));
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains("a spear with an unhardened point"));
    assert!(!world.observe(near).unwrap().text.contains("hardening"));

    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    world.step(&[cmd("tinder", near, Attempt::Gather(Material::Branch))]);
    let lit = world.step(&[cmd("light", near, Attempt::Light)]);
    assert!(lit.results[0].success, "{}", lit.results[0].message);
    let started = world.sim_time();
    assert!(!world.spears()[0].2);
    let waiting = world.observe(near).unwrap().text;
    assert!(waiting.contains("The point of your spear is hardening in the fire."));
    assert!(waiting.contains("a spear with an unhardened point"));
    assert!(world
        .observe(beside)
        .unwrap()
        .text
        .contains("a spear hardening in the fire"));
    assert_eq!(
        world.examine(beside, ExamineTarget::Spear),
        "A spear with an unhardened point is carried by Near, under the rock overhang. The point is hardening in the fire."
    );
    for _ in 0..3 {
        world.step(&[]);
        assert!(!world.spears()[0].2, "the point is not hardened yet");
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The point of your spear is hardening in the fire."));
    }
    world.step(&[]);
    assert_eq!(world.sim_time(), started + 4);
    assert_eq!(world.spears(), vec![(spear_id, Some(near), true)]);
    let hard = world.examine(beside, ExamineTarget::Spear);
    assert_eq!(
        hard,
        "A spear with a hardened point is carried by Near, under the rock overhang."
    );
    assert_ne!(soft, hard);
    assert!(world
        .examine(beside, ExamineTarget::Person("Near".into()))
        .contains("They carry a spear with a hardened point."));
    assert!(world
        .observe(beside)
        .unwrap()
        .events
        .iter()
        .any(|event| { event.text.contains("The point of Near's spear hardens.") }));
    let held = world.observe(near).unwrap().text;
    assert!(held.contains("a spear with a hardened point"));
    assert!(!held.contains("is hardening in the fire."));
    assert!(!world
        .observe(beside)
        .unwrap()
        .text
        .contains("a spear hardening in the fire"));
    assert!(world.deer_cell().is_some());
    assert_eq!(world.material_tally(Material::Spear), 1);
}

fn step_words(from: (i32, i32), to: (i32, i32)) -> String {
    let ns = match (to.1 - from.1).signum() {
        -1 => "north",
        1 => "south",
        _ => "",
    };
    let ew = match (to.0 - from.0).signum() {
        1 => "east",
        -1 => "west",
        _ => "",
    };
    match (ns, ew) {
        ("", ew) => ew.to_string(),
        (ns, "") => ns.to_string(),
        (ns, ew) => format!("{ns} and {ew}"),
    }
}

fn xy_dist(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

fn spear_bound_at_the_rock() -> (World, u32, u32) {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 2, 0).unwrap();
    let far = world.spawn_at("Far", "f", 0, 0).unwrap();
    world.step(&[cmd("stone", near, Attempt::Gather(Material::Stone))]);
    for step in 0..4 {
        world.step(&[cmd(&format!("east-{step}"), near, Attempt::Move(Dir::East))]);
    }
    world.step(&[cmd("south", near, Attempt::Move(Dir::South))]);
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    world.step(&[cmd("bind", near, Attempt::BindSpear)]);
    (world, near, far)
}

fn harden_the_spear(world: &mut World, near: u32) {
    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    world.step(&[cmd("tinder", near, Attempt::Gather(Material::Branch))]);
    let lit = world.step(&[cmd("light", near, Attempt::Light)]);
    assert!(lit.results[0].success, "{}", lit.results[0].message);
    world.step(&[cmd("bark-later", near, Attempt::Gather(Material::Bark))]);
    world.step(&[cmd("branch-later", near, Attempt::Gather(Material::Branch))]);
    for _ in 0..4 {
        world.step(&[]);
    }
    assert!(world.spears()[0].2);
}

#[test]
fn a_hardened_spear_hunts_the_deer_and_the_carcass_cooks() {
    let (mut world, near, _far) = spear_bound_at_the_rock();
    let (mut quiet, _, _) = spear_bound_at_the_rock();
    let refused = world.step(&[cmd("soft", near, Attempt::Hunt)]);
    quiet.step(&[]);
    assert_eq!(
        refused.results[0].message,
        "The spear's point is not hardened."
    );
    assert!(!refused.results[0].success);
    assert_eq!(world.deer_cell(), quiet.deer_cell());
    assert!(world.deer_cell().is_some());
    assert_eq!(world.material_tally(Material::Carcass), 0);

    let (mut world, near, _far) = spear_bound_at_the_rock();
    harden_the_spear(&mut world, near);
    let mut steps = 0;
    while world
        .deer_cell()
        .is_some_and(|deer| xy_dist(world.actor_cell(near).unwrap(), deer) != 1)
        && steps < 40
    {
        world.step(&[]);
        steps += 1;
    }
    let deer_at = world.deer_cell().unwrap();
    assert_eq!(xy_dist(world.actor_cell(near).unwrap(), deer_at), 1);
    let time = world.sim_time();
    let hash = contest_hash(
        world.seed,
        time,
        "hunt",
        &format!("{},{}:deer", deer_at.0, deer_at.1),
        near,
    );
    let drawn = world.step(&[cmd("draw", near, Attempt::Hunt)]);
    let recorded = world
        .event_log()
        .iter()
        .filter_map(|event| event.draw.clone())
        .find(|draw| draw.tag == "hunt")
        .unwrap();
    assert_eq!(recorded.value, hash);
    assert_eq!(
        recorded.subject,
        format!("{},{}:deer", deer_at.0, deer_at.1)
    );
    match hash % 3 {
        0 => {
            let way = step_words(world.actor_cell(near).unwrap(), deer_at);
            assert_eq!(
                drawn.results[0].message,
                format!("You bring the deer down. A carcass lies to the {way}.")
            );
            assert!(world.deer_cell().is_none());
            assert_eq!(world.material_tally(Material::Carcass), 1);
        }
        1 => {
            assert_eq!(drawn.results[0].message, "You miss the deer. It moves off.");
            assert_ne!(world.deer_cell(), Some(deer_at));
            assert!(world.deer_cell().is_some());
            assert_eq!(world.material_tally(Material::Carcass), 0);
        }
        _ => {
            assert_eq!(
                drawn.results[0].message,
                "The deer bolts. Your spear does not land."
            );
            assert_ne!(world.deer_cell(), Some(deer_at));
            assert!(world.deer_cell().is_some());
            assert_eq!(world.material_tally(Material::Carcass), 0);
        }
    }
    assert!(world
        .observe(_far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("deer")));

    let (mut again, again_near, _) = spear_bound_at_the_rock();
    harden_the_spear(&mut again, again_near);
    for _ in 0..steps {
        again.step(&[]);
    }
    let repeated = again.step(&[cmd("draw", again_near, Attempt::Hunt)]);
    assert_eq!(repeated.results[0].message, drawn.results[0].message);
    assert_eq!(again.deer_cell(), world.deer_cell());
    let again_draw = again
        .event_log()
        .iter()
        .filter_map(|event| event.draw.clone())
        .find(|draw| draw.tag == "hunt")
        .unwrap();
    assert_eq!(again_draw.value, hash);

    let (mut kill, hunter, far) = spear_bound_at_the_rock();
    harden_the_spear(&mut kill, hunter);
    let mut waited = 0;
    while kill.deer_cell() != Some((6, 1)) && waited < 40 {
        kill.step(&[]);
        waited += 1;
    }
    assert_eq!(kill.deer_cell(), Some((6, 1)));
    assert_eq!(kill.actor_cell(hunter), Some((6, 1)));
    let brought = kill.step(&[cmd("down", hunter, Attempt::Hunt)]);
    assert_eq!(
        brought.results[0].message,
        "You bring the deer down. A carcass lies where it stood."
    );
    assert!(kill.deer_cell().is_none());
    assert_eq!(kill.material_tally(Material::Carcass), 1);
    assert_eq!(
        (
            kill.material_items(Material::Carcass)[0].2,
            kill.material_items(Material::Carcass)[0].3
        ),
        (6, 1)
    );
    assert!(kill
        .event_log()
        .iter()
        .all(|event| event.draw.as_ref().map(|draw| draw.tag.as_str()) != Some("hunt")));
    assert!(kill
        .observe(hunter)
        .unwrap()
        .text
        .contains("A deer carcass lies here."));
    let far_view = kill.observe(far).unwrap();
    assert!(far_view
        .events
        .iter()
        .all(|event| !event.text.contains("brings the deer down")));
    assert_eq!(
        kill.examine(far, ExamineTarget::Deer),
        "You cannot see a deer from here."
    );
    assert_eq!(kill.spears().len(), 1);
    assert!(kill.spears()[0].2);

    kill.step(&[cmd("lift", hunter, Attempt::Gather(Material::Carcass))]);
    let hunger_before = kill.observe(hunter).unwrap().self_info.hunger;
    let raw = kill.step(&[cmd("raw", hunter, Attempt::Eat(Material::Carcass))]);
    assert!(!raw.results[0].success);
    assert_eq!(raw.results[0].message, "The carcass must be cooked.");
    assert_eq!(kill.material_tally(Material::Carcass), 1);
    assert_eq!(kill.material_items(Material::Carcass)[0].1, Some(hunter));
    let hunger_after = kill.observe(hunter).unwrap().self_info.hunger;
    assert_ne!(
        (hunger_before.as_str(), hunger_after.as_str()),
        ("hungry", "not hungry")
    );
    assert_ne!(
        (hunger_before.as_str(), hunger_after.as_str()),
        ("starving", "not hungry")
    );
    if !burning(&kill, hunter) {
        kill.step(&[cmd("bark2", hunter, Attempt::Gather(Material::Bark))]);
        kill.step(&[cmd("branch2", hunter, Attempt::Gather(Material::Branch))]);
        let relit = kill.step(&[cmd("relight", hunter, Attempt::Light)]);
        assert!(relit.results[0].success, "{}", relit.results[0].message);
    }
    let energy_before = kill.observe(hunter).unwrap().self_info.energy;
    let cooked = kill.step(&[cmd("cook", hunter, Attempt::Cook)]);
    let after = kill.observe(hunter).unwrap().self_info;
    let mut expected = format!("You cook the carcass. You are {}.", after.hunger);
    if energy_before != after.energy {
        expected.push_str(&format!(" You are {}.", after.energy));
    }
    assert_eq!(cooked.results[0].message, expected);
    assert_eq!(kill.material_tally(Material::Carcass), 0);
    assert!(kill.deer_cell().is_none());
}

fn burning(world: &World, actor: u32) -> bool {
    let view = world.observe(actor).unwrap();
    let x = view.self_info.x;
    let y = view.self_info.y;
    view.visible_cells
        .iter()
        .any(|cell| cell.x == x && cell.y == y && cell.fire)
}

#[test]
fn cook_names_an_energy_change_and_stays_quiet_when_the_word_stays() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    world.step(&[cmd("b1", near, Attempt::Gather(Material::Berry))]);
    world.step(&[cmd("b2", near, Attempt::Gather(Material::Berry))]);
    world.step(&[cmd("b3", near, Attempt::Gather(Material::Berry))]);
    assert_eq!(world.actor_cell(near), Some((6, 1)));

    let mut ticks = 0;
    while world.observe(near).unwrap().self_info.energy != "tiring" {
        world.step(&[]);
        ticks += 1;
        assert!(ticks < 80, "energy never reached tiring");
    }
    assert_eq!(world.observe(near).unwrap().self_info.hunger, "hungry");
    if !burning(&world, near) {
        let lit = world.step(&[cmd("light", near, Attempt::Light)]);
        assert!(lit.results[0].success, "{}", lit.results[0].message);
    }
    let before = world.observe(near).unwrap().self_info;
    assert_eq!(before.hunger, "hungry");
    assert_eq!(before.energy, "tiring");
    let both = world.step(&[cmd("cook-both", near, Attempt::Cook)]);
    assert_eq!(
        both.results[0].message,
        "You cook a handful of berries. You are not hungry. You are rested."
    );
    let eased = world.observe(near).unwrap().self_info;
    assert_eq!(eased.hunger, "not hungry");
    assert_eq!(eased.energy, "rested");
    let spare_bark = world.step(&[cmd("bark-later", near, Attempt::Gather(Material::Bark))]);
    assert!(
        spare_bark.results[0].success,
        "{}",
        spare_bark.results[0].message
    );
    let spare_branch = world.step(&[cmd("branch-later", near, Attempt::Gather(Material::Branch))]);
    assert!(
        spare_branch.results[0].success,
        "{}",
        spare_branch.results[0].message
    );

    ticks = 0;
    while world.observe(near).unwrap().self_info.energy != "tiring" {
        world.step(&[]);
        ticks += 1;
        assert!(ticks < 80, "energy never tired again");
    }
    assert_eq!(world.observe(near).unwrap().self_info.hunger, "hungry");
    let eaten = world.step(&[cmd("eat", near, Attempt::Eat(Material::Berry))]);
    assert!(eaten.results[0].success, "{}", eaten.results[0].message);
    assert_eq!(world.observe(near).unwrap().self_info.energy, "rested");
    assert_eq!(world.observe(near).unwrap().self_info.hunger, "hungry");
    if !burning(&world, near) {
        let relit = world.step(&[cmd("relight", near, Attempt::Light)]);
        assert!(relit.results[0].success, "{}", relit.results[0].message);
    }
    let only = world.observe(near).unwrap().self_info;
    assert_eq!(only.hunger, "hungry");
    assert_eq!(only.energy, "rested");
    let cooked = world.step(&[cmd("cook-hunger", near, Attempt::Cook)]);
    assert_eq!(
        cooked.results[0].message,
        "You cook a handful of berries. You are not hungry."
    );
    assert!(!cooked.results[0].message.contains("rested"));
    assert!(!cooked.results[0].message.contains("tiring"));
    assert!(!cooked.results[0].message.contains("spent"));
    let done = world.observe(near).unwrap().self_info;
    assert_eq!(done.hunger, "not hungry");
    assert_eq!(done.energy, "rested");
}

#[test]
fn mushrooms_and_berries_in_the_arms_are_words() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 3, 1).unwrap();
    let first = world.step(&[cmd("m1", near, Attempt::Gather(Material::Mushroom))]);
    assert!(first.results[0].success, "{}", first.results[0].message);
    let one = world.observe(near).unwrap();
    assert!(
        one.text.contains("You have a dry mushroom."),
        "{}",
        one.text
    );
    assert!(!one.text.contains("1 mushroom"), "{}", one.text);
    assert!(!one.text.contains("carried by you"), "{}", one.text);
    let second = world.step(&[cmd("m2", near, Attempt::Gather(Material::Mushroom))]);
    assert!(second.results[0].success, "{}", second.results[0].message);
    let two = world.observe(near).unwrap();
    assert!(
        two.text.contains("You have two dry mushrooms."),
        "{}",
        two.text
    );
    assert!(!two.text.contains("2 mushroom"), "{}", two.text);
    for i in 0..3 {
        world.step(&[cmd(&format!("e{i}"), near, Attempt::Move(Dir::East))]);
    }
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    let berries = world.step(&[cmd("berry", near, Attempt::Gather(Material::Berry))]);
    assert!(berries.results[0].success, "{}", berries.results[0].message);
    let held = world.observe(near).unwrap();
    assert!(
        held.text
            .contains("You have two dry mushrooms, a dry handful of berries."),
        "{}",
        held.text
    );
    assert!(!held.text.contains("1 berry"), "{}", held.text);
    assert!(!held.text.contains("(dry)"), "{}", held.text);
    let pot = world.step(&[cmd("pot", near, Attempt::Gather(Material::Pot))]);
    assert!(pot.results[0].success, "{}", pot.results[0].message);
    let with_pot = world.observe(near).unwrap();
    assert!(
        with_pot
            .text
            .contains("two dry mushrooms, a dry handful of berries, a dry pot"),
        "{}",
        with_pot.text
    );
    assert!(!with_pot.text.contains("1 pot"), "{}", with_pot.text);
    let clock = world.sim_time();
    let _ = world.observe(near).unwrap();
    assert_eq!(world.sim_time(), clock);
}

#[test]
fn carried_food_examine_uses_the_arms_words() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 3, 1).unwrap();
    world.step(&[cmd("m1", near, Attempt::Gather(Material::Mushroom))]);
    world.step(&[cmd("m2", near, Attempt::Gather(Material::Mushroom))]);
    for i in 0..3 {
        world.step(&[cmd(&format!("e{i}"), near, Attempt::Move(Dir::East))]);
    }
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    world.step(&[cmd("berry", near, Attempt::Gather(Material::Berry))]);
    world.step(&[cmd("pot", near, Attempt::Gather(Material::Pot))]);
    let clock = world.sim_time();
    let mushrooms = world.examine(near, ExamineTarget::Mushrooms);
    let berries = world.examine(near, ExamineTarget::Berries);
    let pot = world.examine(near, ExamineTarget::Pot);
    assert_eq!(world.sim_time(), clock);
    assert_eq!(
        mushrooms,
        "Two dry mushrooms are carried by you, under the rock overhang."
    );
    assert!(!mushrooms.contains("must be cooked"), "{mushrooms}");
    assert!(!mushrooms.contains("poison"), "{mushrooms}");
    let west = world.spawn_at("West", "w", 5, 1).unwrap();
    let refused = world.step(&[cmd("raw", near, Attempt::Eat(Material::Mushroom))]);
    assert_eq!(
        refused.results[0].message,
        "Raw mushrooms do not feed you. They must be cooked."
    );
    assert!(world.observe(near).unwrap().events.iter().all(|event| {
        !event.text.contains("Raw mushrooms") && !event.text.contains("must be cooked")
    }));
    let after = world.examine(near, ExamineTarget::Mushrooms);
    assert!(
        after.contains(
            "Two dry mushrooms are carried by you, under the rock overhang. They must be cooked. Raw mushrooms do not feed you."
        ),
        "{after}"
    );
    let west_mushrooms = world.examine(west, ExamineTarget::Mushrooms);
    assert!(
        west_mushrooms.contains("carried by Near"),
        "{west_mushrooms}"
    );
    assert!(
        !west_mushrooms.contains("must be cooked"),
        "{west_mushrooms}"
    );
    assert!(!west_mushrooms.contains("do not feed"), "{west_mushrooms}");
    assert!(!west_mushrooms.contains("poison"), "{west_mushrooms}");
    assert!(!mushrooms.contains("2 mushrooms"), "{mushrooms}");
    assert!(!mushrooms.contains(". dry"), "{mushrooms}");
    let carried_berries = berries
        .lines()
        .find(|line| line.contains("carried by you"))
        .unwrap_or("");
    assert_eq!(
        carried_berries,
        "A dry handful of berries is carried by you, under the rock overhang. They are good to eat raw."
    );
    assert!(!carried_berries.contains("1 handful"), "{berries}");
    assert!(!carried_berries.contains(". dry"), "{berries}");
    assert_eq!(
        pot,
        "A dry pot is carried by you, under the rock overhang, and it can boil water over a fire."
    );
}

#[test]
fn examine_rock_names_the_shelter_you_can_see() {
    let mut world = world_at(10_000);
    let under = world.spawn_at("Near", "n", 6, 1).unwrap();
    let clock = world.sim_time();
    let here = world.examine(under, ExamineTarget::Rock);
    assert_eq!(world.sim_time(), clock);
    assert_eq!(
        here,
        "Rock leans overhead. You are out of the wind. Rain does not reach you."
    );
    assert!(world
        .observe(under)
        .unwrap()
        .text
        .contains("Rock leans overhead."));
    assert!(world
        .observe(under)
        .unwrap()
        .text
        .contains("You are out of the wind."));
    assert!(world
        .observe(under)
        .unwrap()
        .allowed_actions
        .iter()
        .any(|action| action == "examine rock"));

    let beside = world.spawn_at("Beside", "b", 5, 1).unwrap();
    assert_eq!(
        world.examine(beside, ExamineTarget::Rock),
        "Rock leans overhead to the east. It is out of the wind. Rain does not reach it."
    );

    let far = world.spawn_at("Far", "f", 3, 1).unwrap();
    assert_eq!(
        world.examine(far, ExamineTarget::Rock),
        "You cannot see a rock from here."
    );
    assert!(!world
        .observe(far)
        .unwrap()
        .allowed_actions
        .iter()
        .any(|action| action == "examine rock"));
}

#[test]
fn examine_slope_names_the_open_ground_you_can_see() {
    let mut world = world_at(10_000);
    let under = world.spawn_at("Near", "n", 6, 1).unwrap();
    let clock = world.sim_time();
    let looked = world.observe(under).unwrap();
    assert!(
        looked.text.contains("North: slope, a bees' nest."),
        "{}",
        looked.text
    );
    let slope = world.examine(under, ExamineTarget::Slope);
    assert_eq!(world.sim_time(), clock);
    assert_eq!(
        slope,
        "The slope is open ground. It runs north from where you stand."
    );
    assert!(looked
        .allowed_actions
        .iter()
        .any(|action| action == "examine slope"));

    let on_it = world.spawn_at("On", "o", 2, 0).unwrap();
    assert_eq!(
        world.examine(on_it, ExamineTarget::Slope),
        "The slope is open ground. It runs east and west from where you stand."
    );

    let far = world.spawn_at("Far", "f", 4, 3).unwrap();
    assert_eq!(
        world.examine(far, ExamineTarget::Slope),
        "You cannot see a slope from here."
    );
    assert!(!world
        .observe(far)
        .unwrap()
        .allowed_actions
        .iter()
        .any(|action| action == "examine slope"));
}

#[test]
fn examine_tracks_names_each_visible_mark_once() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let far = world.spawn_at("Far", "f", 0, 0).unwrap();
    for (i, dir) in [Dir::East, Dir::East, Dir::North, Dir::North]
        .into_iter()
        .enumerate()
    {
        world.step(&[cmd(&format!("w{i}"), near, Attempt::Move(dir))]);
    }
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    let clock = world.sim_time();
    let text = world.examine(near, ExamineTarget::Tracks);
    assert_eq!(world.sim_time(), clock);
    assert_eq!(
        text,
        "Hoofprints mark the ground one cell south and two cells west, in the open.\nHoofprints mark the ground two cells south and two cells west, in the open."
    );
    let mut lines: Vec<_> = text.lines().collect();
    let count = lines.len();
    lines.sort();
    lines.dedup();
    assert_eq!(lines.len(), count, "{text}");
    assert_eq!(
        world.examine(far, ExamineTarget::Tracks),
        "You see no hoofprints from here."
    );
}

#[test]
fn deer_stands_on_the_trail_already_there() {
    let mut world = world_at(10_000);
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    while world.sim_time() < 10 {
        world.step(&[]);
    }
    assert_eq!(world.sim_time(), 10);
    assert_eq!(world.deer_cell(), Some((6, 1)));
    assert!(world.hoofprints_at(3, 3));
    assert_eq!(
        world.examine(far, ExamineTarget::Here),
        "You are on the valley floor, in the open. A deer trail passes here. Hoofprints mark the ground."
    );
    let at_ten = world.observe(far).unwrap();
    let east_ten = at_ten
        .text
        .lines()
        .find(|line| line.starts_with("East:"))
        .unwrap_or("");
    assert_eq!(east_ten, "East: valley floor, hoofprints.");
    assert!(at_ten.text.contains(
        "Wood left in the open is rotting east of the west floor, and the deer left hoofprints on the rotting wood."
    ));
    assert!(!at_ten.text.contains("one cell"), "{}", at_ten.text);
    assert!(!at_ten.text.contains("two cells"), "{}", at_ten.text);
    world.step(&[]);
    assert_eq!(world.sim_time(), 11);
    assert_eq!(world.deer_cell(), Some((5, 1)));
    let at_eleven = world.observe(far).unwrap();
    assert!(at_eleven
        .text
        .contains("A deer stands on the outside floor, in the open."));
    assert!(
        !at_eleven
            .text
            .contains("A deer stands two cells north and two cells east, in the open."),
        "{}",
        at_eleven.text
    );
    assert!(!at_eleven.text.contains("one cell"), "{}", at_eleven.text);
    assert!(!at_eleven.text.contains("two cells"), "{}", at_eleven.text);
    assert_eq!(
        at_eleven
            .text
            .lines()
            .find(|line| line.starts_with("East:"))
            .unwrap_or(""),
        "East: valley floor, hoofprints."
    );
    while world.sim_time() < 12 {
        world.step(&[]);
    }
    let on_wood = world.observe(far).unwrap();
    assert!(on_wood.text.contains(
        "A deer stands on the rotting wood east of the west floor, in the open, and hoofprints mark the rotting wood under the deer."
    ));
    assert!(!on_wood.text.contains("one cell"), "{}", on_wood.text);
    assert!(!on_wood.text.contains("two cells"), "{}", on_wood.text);
    assert_eq!(
        on_wood
            .text
            .lines()
            .find(|line| line.starts_with("East:"))
            .unwrap_or(""),
        "East: valley floor, hoofprints."
    );
    world.step(&[]);
    assert_eq!(world.sim_time(), 13);
    assert_eq!(world.deer_cell(), Some((4, 2)));
    let at_thirteen = world.observe(far).unwrap();
    assert!(at_thirteen
        .text
        .contains("A deer stands on the west floor, in the open."));
    assert!(
        !at_thirteen
            .text
            .contains("A deer stands one cell north and one cell east, in the open."),
        "{}",
        at_thirteen.text
    );
    assert!(
        !at_thirteen.text.contains("one cell"),
        "{}",
        at_thirteen.text
    );
    assert!(
        !at_thirteen.text.contains("two cells"),
        "{}",
        at_thirteen.text
    );
    assert_eq!(
        at_thirteen
            .text
            .lines()
            .find(|line| line.starts_with("East:"))
            .unwrap_or(""),
        "East: valley floor, hoofprints."
    );
    while world.sim_time() < 14 {
        world.step(&[]);
    }
    assert_eq!(world.deer_cell(), Some((4, 3)));
    assert!(world.hoofprints_at(3, 3));
    let step = world.spawn_at("Step", "s", 4, 1).unwrap();
    assert_eq!(
        world.examine(far, ExamineTarget::Here),
        "You are on the valley floor, in the open. A deer trail passes here. Hoofprints mark the ground."
    );
    let at_fourteen = world.observe(far).unwrap();
    assert!(at_fourteen
        .text
        .contains("Step stands in the open, on the dark berries north of the west floor."));
    assert!(!at_fourteen.text.contains("Near"), "{}", at_fourteen.text);
    assert!(
        !at_fourteen.text.contains("rock overhang"),
        "{}",
        at_fourteen.text
    );
    let east_fourteen = at_fourteen
        .text
        .lines()
        .find(|line| line.starts_with("East:"))
        .unwrap_or("");
    assert_eq!(
        east_fourteen,
        "East: the south floor. The deer stands on that floor. Hoofprints mark that floor. North of that floor is the west floor. Hoofprints mark that floor. East of that floor is the rotting wood. North of that wood is the outside floor. Hoofprints mark that floor."
    );
    assert!(!east_fourteen.contains("valley floor"), "{east_fourteen}");
    assert!(!east_fourteen.contains("Step"), "{east_fourteen}");
    assert!(!east_fourteen.contains("rock"), "{east_fourteen}");
    assert!(!east_fourteen.contains("Near"), "{east_fourteen}");
    assert_eq!(
        at_fourteen
            .text
            .lines()
            .find(|line| line.starts_with("North:"))
            .unwrap_or(""),
        "North: valley floor, the heavy log."
    );
    assert_eq!(
        at_fourteen
            .text
            .lines()
            .find(|line| line.starts_with("South:"))
            .unwrap_or(""),
        "South: valley floor."
    );
    assert_eq!(
        at_fourteen
            .text
            .lines()
            .find(|line| line.starts_with("West:"))
            .unwrap_or(""),
        "West: valley floor, hoofprints."
    );
    assert!(at_fourteen.text.contains(
        "Wood left in the open is rotting east of the west floor, and the deer left hoofprints on the rotting wood."
    ));
    assert!(
        !at_fourteen.text.contains("one cell"),
        "{}",
        at_fourteen.text
    );
    assert!(
        !at_fourteen.text.contains("two cells"),
        "{}",
        at_fourteen.text
    );
    let from_step = world.observe(step).unwrap();
    assert_eq!(world.sim_time(), 14);
    assert!(from_step
        .text
        .contains("Far stands on the deer trail west of the south floor."));
    assert_eq!(from_step.text.matches("A deer stands").count(), 0);
    assert!(from_step.text.contains(
        "South: the west floor. Hoofprints mark that floor. South of that floor is the south floor, still in sight. The deer stands on the south floor."
    ));
    world.step(&[]);
    assert_eq!(world.sim_time(), 15);
    assert_eq!(world.deer_cell(), Some((3, 3)));
    assert!(world.hoofprints_at(3, 3));
    let here = world.examine(far, ExamineTarget::Here);
    assert_eq!(
        here,
        "You are on the valley floor, in the open. A deer trail passes here. A deer stands on that trail. Hoofprints mark the ground."
    );
    assert!(!here.contains("fresh"), "{here}");
    assert!(!here.contains("new hoofprints"), "{here}");
    assert!(!here.contains("A deer stands here"), "{here}");
    assert!(!here.contains("the south floor"), "{here}");
    assert!(!here.contains("the west floor"), "{here}");
    assert!(!here.contains("the outside floor"), "{here}");
    assert!(!here.contains("ate the berries"), "{here}");
    let far_look = world.observe(far).unwrap();
    let far_place = far_look
        .text
        .lines()
        .find(|line| line.starts_with("You are on the"))
        .unwrap_or("");
    assert_eq!(
        far_place,
        "You are on the valley floor, in the open. A deer trail passes here. A deer stands on that trail. Hoofprints mark the ground."
    );
    assert!(!far_place.contains("the outside floor"), "{far_place}");
    assert!(!far_place.contains("the south floor"), "{far_place}");
    assert!(!far_place.contains("the west floor"), "{far_place}");
    let far_east = far_look
        .text
        .lines()
        .find(|line| line.starts_with("East:"))
        .unwrap_or("");
    assert_eq!(
        far_east,
        "East: the south floor. Hoofprints mark that floor. North of that floor is the west floor. Hoofprints mark that floor. East of that floor is the rotting wood. North of that wood is the outside floor. Hoofprints mark that floor."
    );
    assert!(!far_east.contains("deer"), "{far_east}");
    assert!(!far_east.contains("valley floor"), "{far_east}");
    assert!(!far_east.contains("Step"), "{far_east}");
    assert!(!far_east.contains("rock"), "{far_east}");
    assert!(!far_east.contains("Near"), "{far_east}");
    assert!(!far_place.contains("Step"), "{far_place}");
    assert!(far_look
        .text
        .contains("Step stands in the open, on the dark berries north of the west floor."));
    let from_step_later = world.observe(step).unwrap();
    assert!(from_step_later.text.contains(
        "A deer stands in the open, on the deer trail west of the south floor. Far stands on that trail."
    ));
    assert!(from_step_later.text.contains(
        "South: the west floor. Hoofprints mark that floor. South of that floor is the south floor, still in sight. Hoofprints mark the south floor."
    ));
    assert_eq!(
        far_look
            .text
            .lines()
            .find(|line| line.starts_with("North:"))
            .unwrap_or(""),
        "North: valley floor, the heavy log."
    );
    assert_eq!(
        far_look
            .text
            .lines()
            .find(|line| line.starts_with("South:"))
            .unwrap_or(""),
        "South: valley floor."
    );
    assert_eq!(
        far_look
            .text
            .lines()
            .find(|line| line.starts_with("West:"))
            .unwrap_or(""),
        "West: valley floor, hoofprints."
    );
    assert!(far_look.text.contains(
        "Wood left in the open is rotting east of the west floor, and the deer left hoofprints on the rotting wood."
    ));
    assert!(!far_look.text.contains("one cell"), "{}", far_look.text);
    assert!(!far_look.text.contains("two cells"), "{}", far_look.text);
}

#[test]
fn examine_deer_counts_the_cells_the_way_the_tracks_do() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let far = world.spawn_at("Far", "f", 0, 0).unwrap();
    for (i, dir) in [Dir::East, Dir::East, Dir::North, Dir::North]
        .into_iter()
        .enumerate()
    {
        world.step(&[cmd(&format!("w{i}"), near, Attempt::Move(dir))]);
    }
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    assert_eq!(world.deer_cell(), Some((5, 2)));
    let clock = world.sim_time();
    let text = world.examine(near, ExamineTarget::Deer);
    assert_eq!(world.sim_time(), clock);
    assert_eq!(
        text,
        "A deer stands on the rotting wood one cell south and one cell west, in the open."
    );
    assert!(!text.contains("to the south and west"), "{text}");
    let look = world.observe(near).unwrap();
    assert_eq!(world.sim_time(), clock);
    assert!(look.map.contains('d'));
    assert!(look.text.contains(
        "A deer stands on the rotting wood one cell south and one cell west, in the open."
    ));
    assert!(!look.text.contains("Wood left in the open is rotting"));
    let wood_while = world.examine(near, ExamineTarget::Wood);
    assert_eq!(world.sim_time(), clock);
    assert!(wood_while.contains(
        "A pile of three branches is in contact with the ground one cell south and one cell west, in the open."
    ));
    assert!(wood_while.contains("A deer stands on the rotting branches."));
    assert!(
        !wood_while.contains("handful"),
        "no handful went on the open pile: {wood_while}"
    );
    let rock_while = wood_while
        .lines()
        .find(|line| line.contains("under the rock overhang"))
        .unwrap();
    assert!(
        !rock_while.contains("deer"),
        "the deer is on the open pile: {rock_while}"
    );
    assert!(
        !wood_while.contains("hoofprints"),
        "the deer has not left this wood: {wood_while}"
    );
    assert_eq!(
        world.examine(far, ExamineTarget::Wood),
        "You see no wood from here."
    );
    assert!(!look
        .text
        .contains("A deer stands one cell south and one cell west, in the open."));
    assert!(!world.observe(far).unwrap().text.contains("deer"));
    assert_eq!(
        world.examine(far, ExamineTarget::Deer),
        "You cannot see a deer from here."
    );
    world.step(&[]);
    assert_eq!(world.deer_cell(), Some((5, 1)));
    let apart = world.observe(near).unwrap();
    assert!(apart.text.contains(
        "Wood left in the open is rotting one cell south and one cell west, and hoofprints mark the ground there."
    ));
    assert!(apart.text.contains("a deer"));
    assert!(!apart.text.contains("on the rotting wood"));
    let wood_after = world.examine(near, ExamineTarget::Wood);
    assert!(wood_after.contains(
        "A pile of three branches is in contact with the ground one cell south and one cell west, in the open, and hoofprints mark the ground there."
    ));
    assert!(
        !wood_after.contains("A deer"),
        "the deer has left the wood: {wood_after}"
    );
    assert_eq!(
        world.examine(far, ExamineTarget::Wood),
        "You see no wood from here."
    );
    let left = world.examine(near, ExamineTarget::Deer);
    assert_eq!(left, "A deer stands one cell west, in the open.");
    assert!(!left.contains("rotting"), "{left}");
    assert_eq!(
        world.examine(far, ExamineTarget::Deer),
        "You cannot see a deer from here."
    );
    world.step(&[]);
    assert_eq!(world.deer_cell(), Some((6, 1)));
    assert_eq!(world.material_tally(Material::Berry), 4);
    let snapped_clock = world.sim_time();
    let snapped = world.examine(near, ExamineTarget::Wood);
    assert_eq!(world.sim_time(), snapped_clock);
    assert!(snapped.contains(
        "A pile of three branches is in contact with the ground here, under the rock overhang. The wood is dry. Rain does not reach it. A deer stands on these branches."
    ));
    assert!(!snapped.contains("eating"), "{snapped}");
    assert!(
        !snapped.contains("handful"),
        "no handful went on the snap: {snapped}"
    );
    let snap_look = world.observe(near).unwrap();
    assert_eq!(world.sim_time(), snapped_clock);
    assert!(
        snap_look.text.contains("A deer stands on these branches."),
        "{}",
        snap_look.text
    );
    assert!(
        !snap_look.text.contains("eating the berries"),
        "{}",
        snap_look.text
    );
    assert!(
        !snap_look.text.contains("handful is gone"),
        "{}",
        snap_look.text
    );
    assert!(!world.observe(far).unwrap().text.contains("these branches"));
    let open_on_snap = snapped
        .lines()
        .find(|line| line.contains("in the open"))
        .unwrap();
    assert!(
        !open_on_snap.contains("A deer"),
        "the deer has left the open pile: {open_on_snap}"
    );
    world.step(&[]);
    assert_eq!(world.deer_cell(), Some((6, 1)));
    assert_eq!(world.material_tally(Material::Berry), 3);
    let eating_clock = world.sim_time();
    let eating = world.examine(near, ExamineTarget::Wood);
    assert_eq!(world.sim_time(), eating_clock);
    assert!(eating.contains(
        "A pile of three branches is in contact with the ground here, under the rock overhang. The wood is dry. Rain does not reach it. A deer is eating the berries here. A handful is gone."
    ));
    assert!(!eating.contains("last handful"), "{eating}");
    assert!(!eating.contains("these branches"), "{eating}");
    let eat_look = world.observe(near).unwrap();
    assert!(
        eat_look
            .text
            .contains("A deer is eating the berries here. A handful is gone. More are left."),
        "{}",
        eat_look.text
    );
    assert!(
        !eat_look.text.contains("Another handful"),
        "{}",
        eat_look.text
    );
    assert!(
        !eat_look.text.contains("This handful is the last."),
        "{}",
        eat_look.text
    );
    assert!(
        !eat_look.text.contains("The last handful"),
        "{}",
        eat_look.text
    );
    assert!(!world.observe(far).unwrap().text.contains("handful is gone"));
    assert!(!world.observe(far).unwrap().text.contains("More are left"));
    assert!(
        !eat_look.text.contains("these branches"),
        "{}",
        eat_look.text
    );
    world.step(&[]);
    assert_eq!(world.material_tally(Material::Berry), 2);
    let two = world.observe(near).unwrap();
    assert!(
        two.text
            .contains("A deer is eating the berries here. Another handful is gone. More are left."),
        "{}",
        two.text
    );
    assert!(
        !two.text.contains("A handful is gone. More are left."),
        "{}",
        two.text
    );
    assert!(
        !two.text.contains("This handful is the last."),
        "{}",
        two.text
    );
    assert!(!world.observe(far).unwrap().text.contains("Another handful"));
    world.step(&[]);
    assert_eq!(world.material_tally(Material::Berry), 1);
    let one_left = world.observe(near).unwrap();
    assert!(
        one_left
            .text
            .contains("A deer is eating the berries here. This handful is the last."),
        "{}",
        one_left.text
    );
    assert!(
        !one_left.text.contains("More are left."),
        "{}",
        one_left.text
    );
    assert!(
        !one_left.text.contains("A handful is gone."),
        "{}",
        one_left.text
    );
    assert!(!world
        .observe(far)
        .unwrap()
        .text
        .contains("This handful is the last"));
    world.step(&[]);
    assert_eq!(world.deer_cell(), Some((6, 1)));
    assert_eq!(world.material_tally(Material::Berry), 1);
    assert_eq!(world.sim_time(), 10);
    assert!(world.hoofprints_at(3, 3));
    let last = world.examine(near, ExamineTarget::Wood);
    assert!(
        last.contains("A deer ate from the pile. A handful of berries is left."),
        "{last}"
    );
    assert!(!last.contains("The last handful is gone."), "{last}");
    assert!(!last.contains("these branches"), "{last}");
    let last_look = world.observe(near).unwrap();
    assert!(
        !last_look.text.contains("The last handful is gone."),
        "{}",
        last_look.text
    );
    let deer_sentence =
        "A handful of berries lies here. It is good to eat raw. A deer ate the rest.";
    let stand_at = last_look.text.find(deer_sentence).expect(&last_look.text);
    let pot_at = last_look
        .text
        .find("A pot sits here.")
        .expect(&last_look.text);
    let branches_at = last_look
        .text
        .find("A pile of branches lies here.")
        .expect(&last_look.text);
    let bark_at = last_look
        .text
        .find("A pile of bark lies here.")
        .expect(&last_look.text);
    assert!(
        stand_at < pot_at && pot_at < branches_at && branches_at < bark_at,
        "{}",
        last_look.text
    );
    assert_eq!(last_look.text.matches(deer_sentence).count(), 1);
    assert!(!last_look.text.contains("is eating"), "{}", last_look.text);
    assert!(
        !last_look
            .text
            .contains("The deer standing here ate the berries that were here."),
        "{}",
        last_look.text
    );
    assert!(
        !last_look.text.contains("A deer stands here."),
        "{}",
        last_look.text
    );
    let off = world.spawn_at("Ash", "h", 5, 3).unwrap();
    let off_look = world.observe(off).unwrap().text;
    assert_eq!(world.sim_time(), 10);
    assert!(
        off_look.contains(
            "A deer stands two cells north and one cell east, under the rock overhang, after eating the berries there."
        ),
        "{off_look}"
    );
    assert!(
        !off_look.contains("The last handful is gone."),
        "{off_look}"
    );
    assert!(!off_look.contains("A deer stands here."), "{off_look}");
    assert!(
        !last_look.text.contains("The deer here ate them"),
        "{}",
        last_look.text
    );
    assert!(
        !last_look
            .text
            .contains("The berries that were here are gone"),
        "{}",
        last_look.text
    );
    assert!(
        last_look
            .text
            .contains("Wood left in the open is rotting to the south and west."),
        "{}",
        last_look.text
    );
    assert!(
        last_look.text.contains(
            "South: valley floor, nettles. The corner beyond, to the west, has that wood."
        ),
        "{}",
        last_look.text
    );
    assert!(
        !last_look.text.contains("Past that wood"),
        "{}",
        last_look.text
    );
    assert!(
        !last_look.text.contains("the south floor"),
        "{}",
        last_look.text
    );
    assert!(
        last_look.text.contains(
            "West: valley floor, hoofprints. That way, to the south, reaches the same wood."
        ),
        "{}",
        last_look.text
    );
    assert!(
        !last_look.text.contains("has the rotting wood and the deer"),
        "{}",
        last_look.text
    );
    assert!(
        !last_look.text.contains("on that wood"),
        "{}",
        last_look.text
    );
    assert!(
        !last_look.text.contains("one cell south and one cell west"),
        "{}",
        last_look.text
    );
    assert!(
        !last_look.text.contains("hoofprints mark the ground there"),
        "{}",
        last_look.text
    );
    assert!(!world
        .observe(far)
        .unwrap()
        .text
        .contains("A deer stands here."));
    world.step(&[]);
    assert_eq!(world.deer_cell(), Some((5, 1)));
    assert_eq!(world.material_tally(Material::Berry), 1);
    let left_clock = world.sim_time();
    let left_look = world.observe(near).unwrap();
    assert_eq!(world.sim_time(), left_clock);
    assert!(
        left_look.text.contains(
            "A handful of berries lies here. It is good to eat raw. A deer ate the rest."
        ),
        "{}",
        left_look.text
    );
    assert!(
        !left_look
            .text
            .contains("The berries that were here are gone."),
        "{}",
        left_look.text
    );
    assert!(
        !left_look.text.contains("one cell west"),
        "{}",
        left_look.text
    );
    assert!(
        !left_look.text.contains("stands there"),
        "{}",
        left_look.text
    );
    assert!(left_look.text.contains(
        "West: valley floor, a deer, hoofprints. That way, to the south, reaches the same wood."
    ));
    assert!(
        !left_look.text.contains("ate the berries that were here"),
        "{}",
        left_look.text
    );
    assert!(
        left_look
            .text
            .contains("Wood left in the open is rotting to the south and west."),
        "{}",
        left_look.text
    );
    assert!(
        left_look.text.contains(
            "South: valley floor, nettles. The corner beyond, to the west, has that wood."
        ),
        "{}",
        left_look.text
    );
    assert!(
        !left_look.text.contains("Past that wood"),
        "{}",
        left_look.text
    );
    assert!(
        !left_look.text.contains("the south floor"),
        "{}",
        left_look.text
    );
    assert!(
        left_look.text.contains(
            "West: valley floor, a deer, hoofprints. That way, to the south, reaches the same wood."
        ),
        "{}",
        left_look.text
    );
    assert!(
        !left_look.text.contains("one cell south and one cell west"),
        "{}",
        left_look.text
    );
    assert!(
        !left_look.text.contains("hoofprints mark the ground there"),
        "{}",
        left_look.text
    );
    assert!(
        !left_look.text.contains("on that wood"),
        "{}",
        left_look.text
    );
    assert!(
        !left_look.text.contains("A deer stands here."),
        "{}",
        left_look.text
    );
    assert!(!world.observe(far).unwrap().text.contains("stands there"));
    world.step(&[]);
    assert_eq!(world.deer_cell(), Some((5, 2)));
    let wood_clock = world.sim_time();
    let wood_look = world.observe(near).unwrap();
    assert!(
        wood_look.text.contains(
            "A handful of berries lies here. It is good to eat raw. A deer ate the rest."
        ),
        "{}",
        wood_look.text
    );
    assert!(
        !wood_look
            .text
            .contains("The berries that were here are gone."),
        "{}",
        wood_look.text
    );
    assert!(
        !wood_look.text.contains("one cell south and one cell west"),
        "{}",
        wood_look.text
    );
    assert!(
        !wood_look.text.contains("Wood left in the open is rotting"),
        "{}",
        wood_look.text
    );
    assert!(
        !wood_look
            .text
            .contains("The deer stands to the south and west."),
        "{}",
        wood_look.text
    );
    assert!(wood_look
        .text
        .contains("The corner beyond, to the west, has the rotting wood and the deer"));
    assert!(
        !wood_look
            .text
            .contains("The deer on that wood ate the berries."),
        "{}",
        wood_look.text
    );
    assert!(!wood_look.text.contains("ate them"), "{}", wood_look.text);
    assert!(
        !wood_look.text.contains("Past that wood"),
        "{}",
        wood_look.text
    );
    assert!(
        !wood_look.text.contains("the south floor"),
        "{}",
        wood_look.text
    );
    assert!(
        !wood_look.text.contains("stands there"),
        "{}",
        wood_look.text
    );
    assert!(
        !wood_look.text.contains("that rotting wood"),
        "{}",
        wood_look.text
    );
    assert!(
        !wood_look
            .text
            .contains("The deer one cell south and one cell west stands there."),
        "{}",
        wood_look.text
    );
    assert!(
        wood_look.text.contains(
            "South: valley floor, nettles. The corner beyond, to the west, has the rotting wood and the deer, in the open, and hoofprints mark the wood under the deer."
        ),
        "{}",
        wood_look.text
    );
    assert!(
        wood_look.text.contains(
            "West: valley floor, hoofprints. That way, to the south, reaches the same wood."
        ),
        "{}",
        wood_look.text
    );
    let west_line = wood_look
        .text
        .lines()
        .find(|line| line.starts_with("West:"))
        .unwrap();
    assert!(!west_line.contains("corner"), "{west_line}");
    assert!(
        !wood_look.text.contains("That corner"),
        "{}",
        wood_look.text
    );
    assert!(
        !wood_look.text.contains("A deer stands on the rotting wood"),
        "{}",
        wood_look.text
    );
    assert!(
        !wood_look.text.contains("hoofprints mark the ground there"),
        "{}",
        wood_look.text
    );
    assert!(
        !wood_look.text.contains("on the rotting wood there"),
        "{}",
        wood_look.text
    );
    assert!(
        !wood_look.text.contains("on the rotting wood one cell"),
        "{}",
        wood_look.text
    );
    let corner = world.spawn_at("Corner", "c", 3, 3).unwrap();
    let corner_look = world.observe(corner).unwrap();
    assert!(
        corner_look.text.contains(
            "A deer stands on the rotting wood east of the west floor, in the open, and hoofprints mark the rotting wood under the deer."
        ),
        "{}",
        corner_look.text
    );
    assert!(
        !corner_look
            .text
            .contains("hoofprints mark the ground there"),
        "{}",
        corner_look.text
    );
    assert!(
        !corner_look.text.contains("on the rotting wood there"),
        "{}",
        corner_look.text
    );
    assert!(
        !corner_look.text.contains("berries that were here are gone"),
        "{}",
        corner_look.text
    );
    assert!(!world
        .observe(far)
        .unwrap()
        .text
        .contains("The deer stands to the south and west"));
    assert!(
        !wood_look.text.contains("A deer stands here."),
        "{}",
        wood_look.text
    );
    assert!(
        !wood_look.text.contains("A deer ate them."),
        "{}",
        wood_look.text
    );
    let prints_before_return = world.hoofprints_at(4, 2);
    world.step(&[]);
    assert_eq!(world.sim_time(), wood_clock + 1);
    assert_eq!(world.deer_cell(), Some((4, 2)));
    assert!(
        world.hoofprints_at(4, 2),
        "prints before the deer returned: {prints_before_return}"
    );
    assert!(
        prints_before_return,
        "the floor west of the wood had no hoofprints until the deer stepped back on"
    );
    let on_left = world.observe(near).unwrap();
    assert_eq!(world.sim_time(), wood_clock + 1);
    assert!(
        on_left.text.contains(
            "A handful of berries lies here. It is good to eat raw. A deer ate the rest."
        ),
        "{}",
        on_left.text
    );
    assert!(!on_left.text.contains("are gone"), "{}", on_left.text);
    assert_eq!(on_left.text.matches("The deer ate them.").count(), 0);
    assert_eq!(
        on_left
            .text
            .matches("one cell south and two cells west")
            .count(),
        0,
        "{}",
        on_left.text
    );
    assert_eq!(on_left.text.matches("The deer stands there.").count(), 0);
    assert!(
        !on_left
            .text
            .contains("A deer stands one cell south and two cells west"),
        "{}",
        on_left.text
    );
    let south = on_left
        .text
        .lines()
        .find(|line| line.starts_with("South:"))
        .unwrap_or("");
    assert_eq!(
        south,
        "South: valley floor, nettles. The corner beyond, to the west, has that wood. Past that wood is the west floor. Hoofprints mark that floor. The deer stands on the west floor."
    );
    assert!(south.contains("Hoofprints mark that floor."), "{south}");
    assert!(!south.contains("to its west"), "{south}");
    assert!(
        !south.contains("the valley floor on the wood's west side"),
        "{south}"
    );
    assert!(!on_left.text.contains("open ground"), "{}", on_left.text);
    assert!(
        !on_left.text.contains("That deer stands"),
        "{}",
        on_left.text
    );
    assert!(
        !on_left.text.contains("the deer stands in the open"),
        "{}",
        on_left.text
    );
    assert_eq!(
        on_left
            .text
            .matches("the valley floor on the wood's west side")
            .count(),
        0,
        "{}",
        on_left.text
    );
    assert!(!south.contains("west of the rotting wood"), "{south}");
    let examined = world.examine(near, ExamineTarget::Deer);
    assert_eq!(world.sim_time(), wood_clock + 1);
    assert_eq!(
        examined,
        "A deer stands on the west floor. Hoofprints mark that floor."
    );
    assert!(!examined.contains("west of the rotting wood"), "{examined}");
    assert!(
        examined.contains("Hoofprints mark that floor."),
        "{examined}"
    );
    assert!(!examined.contains("open ground"), "{examined}");
    assert!(!examined.contains("that wood"), "{examined}");
    assert!(!examined.contains("Past that wood"), "{examined}");
    assert!(!examined.contains("in the open"), "{examined}");
    assert!(!examined.contains("past that wood"), "{examined}");
    assert!(
        !examined.contains("one cell south and two cells west"),
        "{examined}"
    );
    assert!(!examined.contains("ate"), "{examined}");
    let west = on_left
        .text
        .lines()
        .find(|line| line.starts_with("West:"))
        .unwrap_or("");
    assert_eq!(
        west,
        "West: hoofprints mark the outside floor. The rotting wood is the next square south of that floor. Past that wood, the deer stands on the west floor, and hoofprints mark that floor."
    );
    assert!(west.starts_with(
        "West: hoofprints mark the outside floor. The rotting wood is the next square south of that floor."
    ));
    assert!(!west.contains("this square"), "{west}");
    assert!(!west.contains("that square"), "{west}");
    assert!(!west.contains("the near floor"), "{west}");
    assert!(!west.contains("the north floor"), "{west}");
    assert!(west.contains(
        "Past that wood, the deer stands on the west floor, and hoofprints mark that floor."
    ));
    assert!(!west.contains("the next square south."), "{west}");
    assert!(
        !west.contains("the square north of the rotting wood"),
        "{west}"
    );
    assert!(!west.contains("to the south"), "{west}");
    assert!(!west.contains("them"), "{west}");
    assert!(!west.contains("the near floor"), "{west}");
    assert!(!west.contains("one step west"), "{west}");
    assert!(!west.contains("the floor north of that wood"), "{west}");
    assert!(!west.contains("valley floor"), "{west}");
    assert!(!west.contains("a valley floor"), "{west}");
    assert!(!west.contains("is valley floor"), "{west}");
    assert!(!west.contains("the next step west"), "{west}");
    assert_eq!(west.matches("the west floor").count(), 1, "{west}");
    assert!(
        !west.contains("the valley floor on the wood's west side"),
        "{west}"
    );
    assert!(!west.contains("valley floor, hoofprints"), "{west}");
    assert!(
        !west.contains("one step west is valley floor, hoofprints"),
        "{west}"
    );
    assert!(!west.contains("this way"), "{west}");
    assert!(!west.contains("that ground"), "{west}");
    assert!(!west.contains("a deer"));
    assert!(
        !west.contains("the deer stands on the hoofprints"),
        "{west}"
    );
    let deer_clause = west.split("Past that wood, ").nth(1).unwrap_or("");
    let square = west.split(". Past that wood").next().unwrap_or("");
    assert!(!square.contains("the deer stands"), "{square}");
    assert!(
        deer_clause.contains("hoofprints mark that floor"),
        "{deer_clause}"
    );
    assert!(
        !deer_clause.contains("with hoofprints on it"),
        "{deer_clause}"
    );
    assert!(!west.contains("open ground"), "{west}");
    assert!(!west.contains("in the open"), "{west}");
    assert!(
        !west.contains("one cell south and two cells west"),
        "{west}"
    );
    let ending = on_left.text.trim_end().lines().last().unwrap_or("");
    assert_eq!(ending, west);
    assert_eq!(
        on_left
            .text
            .matches("the deer stands on the west floor")
            .count(),
        1,
        "{}",
        on_left.text
    );
    assert!(
        on_left
            .text
            .contains("Wood left in the open is rotting to the south and west."),
        "{}",
        on_left.text
    );
    assert!(
        !on_left.text.contains("the south floor"),
        "{}",
        on_left.text
    );
    let on_square = world.spawn_at("Step", "step", 5, 1).unwrap();
    let square_look = world.observe(on_square).unwrap();
    assert_eq!(world.sim_time(), wood_clock + 1);
    assert_eq!(world.deer_cell(), Some((4, 2)));
    let place = square_look
        .text
        .lines()
        .find(|line| line.starts_with("You stand on the outside floor."))
        .unwrap_or("");
    assert_eq!(
        place,
        "You stand on the outside floor. Hoofprints mark the ground. The rock overhang is the next square east. The rotting wood is the next square south. Worms are in the dry ground here. Carve an antler into an antler pick to dig them."
    );
    let square_east = square_look
        .text
        .lines()
        .find(|line| line.starts_with("East:"))
        .unwrap_or("");
    assert_eq!(
        square_east,
        "East: the rock overhang. Near stands on the valley floor under that rock. A pile of branches lies there. A pot sits there. A pile of bark lies there. Hoofprints mark that floor."
    );
    assert!(!square_east.contains("1 person"), "{square_east}");
    assert!(
        !square_east.contains("Hoofprints mark the ground"),
        "{square_east}"
    );
    assert!(
        !square_east.contains("valley floor, rock overhang"),
        "{square_east}"
    );
    assert!(!place.contains("this square"), "{place}");
    assert!(!place.contains("that square"), "{place}");
    assert!(
        !place.contains("Hoofprints mark the ground underfoot"),
        "{place}"
    );
    assert!(
        !place.contains("You are on the square north of the rotting wood"),
        "{place}"
    );
    assert!(!place.contains("the near floor"), "{place}");
    assert!(!place.contains("the north floor"), "{place}");
    assert!(!place.contains("the west floor"), "{place}");
    let square_south = square_look
        .text
        .lines()
        .find(|line| line.starts_with("South:"))
        .unwrap_or("");
    assert_eq!(
        square_south,
        "South: the rotting wood. Hoofprints mark that wood."
    );
    assert!(
        !square_look
            .text
            .contains("South: valley floor, a pile of branches"),
        "{}",
        square_look.text
    );
    assert!(square_look.text.contains(
        "Wood left in the open is rotting to the south, and the deer left hoofprints on the rotting wood."
    ));
    assert!(square_look
        .text
        .lines()
        .any(|line| line == "A deer stands on the west floor."));
    assert!(
        !square_look
            .text
            .contains("on the floor west of the rotting wood"),
        "{}",
        square_look.text
    );
    assert!(
        !square_look.text.contains("one cell south"),
        "{}",
        square_look.text
    );
    assert!(
        !square_look
            .text
            .contains("one cell south and one cell west"),
        "{}",
        square_look.text
    );
    assert!(
        !place.contains("You are on the square north of the rotting wood"),
        "{place}"
    );
    assert!(!place.contains("valley floor"), "{place}");
    assert!(!place.contains("the near floor"), "{place}");
    let under_again = world.observe(near).unwrap();
    assert!(under_again.text.contains(
        "West: hoofprints mark the outside floor. Step stands on that floor. The rotting wood is the next square south of that floor. Past that wood, the deer stands on the west floor, and hoofprints mark that floor."
    ));
    assert!(under_again.text.contains(
        "South: valley floor, nettles. The corner beyond, to the west, has that wood. Past that wood is the west floor. Hoofprints mark that floor. The deer stands on the west floor."
    ));
    assert!(
        !under_again.text.contains("1 person"),
        "{}",
        under_again.text
    );
    world.step(&[]);
    assert_eq!(world.sim_time(), wood_clock + 2);
    assert_eq!(world.deer_cell(), Some((4, 3)));
    assert!(world.hoofprints_at(3, 3));
    let on_south = world.observe(near).unwrap();
    assert_eq!(world.sim_time(), wood_clock + 2);
    assert!(
        on_south.text.contains(
            "South: valley floor, nettles. The corner beyond, to the west, has that wood. Past that wood is the west floor. The deer has left it. Hoofprints mark that ground. South of the west floor is the south floor, still in sight. The deer stands on the south floor."
        ),
        "{}",
        on_south.text
    );
    assert!(
        !on_south.text.contains("Hoofprints mark the south floor"),
        "{}",
        on_south.text
    );
    assert_eq!(
        on_south
            .text
            .matches("The deer stands on the south floor")
            .count(),
        1
    );
    assert!(
        !on_south
            .text
            .contains("The deer two cells south and two cells west stands there"),
        "{}",
        on_south.text
    );
    assert!(
        !on_south
            .text
            .contains("A deer stands two cells south and two cells west, in the open."),
        "{}",
        on_south.text
    );
    let west_later = on_south
        .text
        .lines()
        .find(|line| line.starts_with("West:"))
        .unwrap_or("");
    assert_eq!(
        west_later,
        "West: hoofprints mark the outside floor. The rotting wood is the next square south of that floor."
    );
    let aside = world.spawn_at("Aside", "a", 4, 1).unwrap();
    let from_west = world.observe(aside).unwrap();
    assert_eq!(world.sim_time(), wood_clock + 2);
    assert_eq!(world.actor_cell(aside), Some((4, 1)));
    let aside_place = from_west
        .text
        .lines()
        .find(|line| line.starts_with("You are on the"))
        .unwrap_or("");
    assert_eq!(
        aside_place,
        "You are on the valley floor, in the open. The outside floor is the next square east. Dark berries lie here, dry. They taste bitter. They are poisonous. Worms are in the dry ground here. Carve an antler into an antler pick to dig them."
    );
    assert!(aside_place.starts_with("You are on the valley floor, in the open."));
    assert!(!aside_place.contains("rock"), "{aside_place}");
    assert!(
        !aside_place.starts_with("You stand on the outside floor"),
        "{aside_place}"
    );
    assert!(aside_place.contains("poisonous"), "{aside_place}");
    let aside_east = from_west
        .text
        .lines()
        .find(|line| line.starts_with("East:"))
        .unwrap_or("");
    assert_eq!(
        aside_east,
        "East: the outside floor. Hoofprints mark that floor. The rock overhang is the next square east of that floor. Near stands on the valley floor under that rock."
    );
    assert!(!aside_east.contains("person"), "{aside_east}");
    assert!(
        aside_east.contains("on the valley floor under that rock"),
        "{aside_east}"
    );
    assert!(!aside_east.contains("East: valley floor"), "{aside_east}");
    assert!(!aside_east.contains("a deer"), "{aside_east}");
    assert!(!aside_east.contains("berries"), "{aside_east}");
    let aside_north = from_west
        .text
        .lines()
        .find(|line| line.starts_with("North:"))
        .unwrap_or("");
    let aside_south = from_west
        .text
        .lines()
        .find(|line| line.starts_with("South:"))
        .unwrap_or("");
    let aside_west = from_west
        .text
        .lines()
        .find(|line| line.starts_with("West:"))
        .unwrap_or("");
    assert_eq!(aside_north, "North: slope, a flint nodule.");
    assert_eq!(
        aside_south,
        "South: the west floor. Hoofprints mark that floor. South of that floor is the south floor, still in sight. The deer stands on the south floor."
    );
    assert!(!aside_south.contains("valley floor"), "{aside_south}");
    assert!(!aside_south.contains("the outside floor"), "{aside_south}");
    assert_eq!(from_west.text.matches("A deer stands").count(), 0);
    assert!(from_west
        .text
        .contains("Corner stands on the deer trail west of the south floor."));
    assert!(from_west.text.contains(
        "Wood left in the open is rotting south of the outside floor, and the deer left hoofprints on the rotting wood."
    ));
    assert!(!from_west.text.contains("one cell"), "{}", from_west.text);
    assert!(!from_west.text.contains("two cells"), "{}", from_west.text);
    assert_eq!(aside_west, "West: valley floor, mushrooms.");
    assert!(!west_later.contains("this square"), "{west_later}");
    assert!(!west_later.contains("that square"), "{west_later}");
    assert!(!west_later.contains("the deer"), "{west_later}");
    assert!(
        !west_later.contains("the next square south."),
        "{west_later}"
    );
    assert!(
        !west_later.contains("the square north of the rotting wood"),
        "{west_later}"
    );
    assert!(!west_later.contains("the deer"), "{west_later}");
    assert!(!west_later.contains("them"), "{west_later}");
    assert!(!west_later.contains("the near floor"), "{west_later}");
    assert!(!west_later.contains("one step west"), "{west_later}");
    assert!(!west_later.contains("valley floor"), "{west_later}");
    assert!(
        !west_later.contains("the floor north of that wood"),
        "{west_later}"
    );
    assert!(!west_later.contains("the deer stands"), "{west_later}");
    assert!(
        !on_south.text.contains("Hoofprints mark that floor"),
        "{}",
        on_south.text
    );
    assert!(
        !on_south.text.contains("stands on the west floor"),
        "{}",
        on_south.text
    );
    let look_lines: Vec<_> = on_south.text.lines().collect();
    let south_at = look_lines
        .iter()
        .position(|line| line.starts_with("South:"))
        .expect(&on_south.text);
    assert!(
        look_lines[south_at + 1].starts_with("West:"),
        "{}",
        on_south.text
    );
    assert_ne!(
        look_lines[south_at + 2],
        "The deer ate the berries that were here.",
        "{}",
        on_south.text
    );
    assert!(on_south
        .text
        .contains("A handful of berries lies here. It is good to eat raw. A deer ate the rest."));
    let around = on_south.text.find("Around you:").expect(&on_south.text);
    assert!(!on_south.text.contains("That deer"), "{}", on_south.text);
    assert!(
        !on_south.text.contains("ate the berries that were here"),
        "{}",
        on_south.text
    );
    let named = on_south
        .text
        .find("is the south floor, still in sight. The deer stands on the south floor.")
        .expect(&on_south.text);
    let onto = on_south
        .text
        .find("Hoofprints on that wood mark the deer's walk south off the west floor, onto the south floor, still in sight.")
        .expect(&on_south.text);
    assert!(around < named && named < onto, "{}", on_south.text);
    assert!(
        !on_south.text.contains("where the deer is now")
            && !on_south.text.contains("which is still in sight")
            && !on_south.text.contains("no longer in sight")
            && !on_south.text.contains("as far as")
            && !on_south.text.contains("the last you can see")
            && !on_south.text.contains("passes on out of view")
            && !on_south.text.contains("pass on out of view")
            && !on_south.text.contains("A line of hoofprints")
            && !on_south.text.contains("The row of hoofprints")
            && !on_south.text.contains("no more hoofprints")
            && !on_south.text.contains("The deer is not")
            && !on_south.text.contains("went on out of sight")
            && !on_south.text.contains("you lose sight")
            && !on_south.text.contains("across the south floor")
            && !on_south.text.contains("and on")
            && !on_south.text.contains("out of sight")
            && !on_south.text.contains("the ground beyond")
            && !on_south.text.contains("then west")
            && !on_south.text.contains("The walk turns")
            && !on_south.text.contains("turning west there")
            && !on_south.text.contains("turning west out of sight")
            && !on_south.text.contains("turns west out of sight")
            && !on_south.text.contains("as it goes")
            && !on_south.text.contains("the walk goes")
            && !on_south.text.contains("walk going")
            && !on_south.text.contains("and west out of sight")
            && !on_south.text.contains("which goes")
            && !on_south.text.contains("which, from")
            && !on_south.text.contains("which turns")
            && !on_south.text.contains("you cannot see where the deer went")
            && !on_south.text.contains("west of that floor")
            && !on_south.text.contains("the valley floor to its west")
            && !on_south.text.contains("another valley floor")
            && !on_south.text.contains("the open valley floor")
            && !on_south.text.contains("the valley floor, in the open")
            && !on_south.text.contains("The deer has left the south floor")
            && !on_south.text.contains("no longer stands")
            && !on_south.text.contains("continue that walk")
            && !on_south.text.contains("the same walk")
            && !on_south.text.contains("and by the south floor")
            && !on_south.text.contains("and on the south floor")
            && !on_south.text.contains("hoofprints on the south floor")
            && !on_south.text.contains(" to the south floor")
            && !on_south.text.contains("the line of them"),
        "{}",
        on_south.text
    );
    assert!(
        !on_south.text.contains("on the south floor ate"),
        "{}",
        on_south.text
    );
    assert!(
        !on_south
            .text
            .contains("The berries that were here are gone"),
        "{}",
        on_south.text
    );
    assert!(
        !on_south
            .text
            .contains("The deer left hoofprints on that wood, and walked"),
        "{}",
        on_south.text
    );
    assert!(
        !on_south.text.contains("walked to the south floor"),
        "{}",
        on_south.text
    );
    assert!(
        !on_south
            .text
            .contains("after the deer ate the berries here"),
        "{}",
        on_south.text
    );
    assert!(
        !on_south
            .text
            .contains("The deer two cells south and two cells west ate them"),
        "{}",
        on_south.text
    );
    world.step(&[]);
    assert_eq!(world.deer_cell(), Some((3, 3)));
    let gone_clock = world.sim_time();
    assert_eq!(gone_clock, 15);
    assert!(world.hoofprints_at(3, 3));
    let unseen = world.observe(near).unwrap();
    assert_eq!(world.sim_time(), gone_clock);
    assert!(
        !unseen
            .text
            .contains("A deer ate the berries that were here."),
        "{}",
        unseen.text
    );
    assert!(unseen.text.contains(
        "Hoofprints on that wood mark the deer's walk, and on the south floor they turn west out of sight."
    ));
    assert!(unseen
        .text
        .contains("A handful of berries lies here. It is good to eat raw. A deer ate the rest."));
    assert!(
        !unseen
            .text
            .contains("The deer ate the berries that were here."),
        "{}",
        unseen.text
    );
    assert!(
        !unseen.text.contains("The deer is no longer in sight"),
        "{}",
        unseen.text
    );
    assert!(!unseen.text.contains("The deer is not."), "{}", unseen.text);
    assert!(
        !unseen.text.contains("The deer is not in sight"),
        "{}",
        unseen.text
    );
    assert!(
        !unseen.text.contains("That floor is still in sight"),
        "{}",
        unseen.text
    );
    assert_eq!(
        unseen
            .text
            .matches("One step west of the south floor, you lose sight of the deer")
            .count(),
        0
    );
    assert!(!unseen.text.contains("you lose sight"), "{}", unseen.text);
    assert!(
        !unseen.text.contains("onto the south floor"),
        "{}",
        unseen.text
    );
    assert_eq!(unseen.text.matches("the deer out of sight").count(), 0);
    assert_eq!(
        unseen
            .text
            .matches("you cannot see where the deer went")
            .count(),
        0
    );
    assert_eq!(unseen.text.matches("west out of sight").count(), 1);
    assert_eq!(
        unseen
            .text
            .matches("and on the south floor they turn west out of sight")
            .count(),
        1
    );
    assert_eq!(
        unseen
            .text
            .matches("which turns west off the south floor")
            .count(),
        0
    );
    assert!(!unseen.text.contains("you lose sight"), "{}", unseen.text);
    let last_line = unseen
        .text
        .trim_end()
        .rsplit_once('\n')
        .map(|(_, last)| last)
        .unwrap_or(unseen.text.trim_end());
    assert!(
        last_line.contains("and on the south floor they turn west out of sight")
            && last_line.matches("the south floor").count() == 1
            && last_line.matches("Hoofprints").count() == 1
            && last_line.matches("they").count() == 1
            && last_line.matches("the prints").count() == 0
            && !last_line.contains("which")
            && !last_line.contains("turns")
            && !last_line.contains("still in sight")
            && !last_line.contains(", out of sight"),
        "{last_line}"
    );
    assert!(
        !unseen.text.contains("to the south floor"),
        "{}",
        unseen.text
    );
    assert!(
        !unseen.text.contains("The deer has left the south floor"),
        "{}",
        unseen.text
    );
    assert_eq!(
        unseen
            .text
            .matches("where the deer no longer stands")
            .count(),
        0
    );
    assert!(
        !unseen.text.contains("continue that walk"),
        "{}",
        unseen.text
    );
    assert!(
        !unseen.text.contains("The hoofprints on the south floor"),
        "{}",
        unseen.text
    );
    assert!(!unseen.text.contains("shows"), "{}", unseen.text);
    assert!(!unseen.text.contains("those"), "{}", unseen.text);
    assert!(!unseen.text.contains("the same walk"), "{}", unseen.text);
    assert_eq!(unseen.text.matches("and by the south floor").count(), 0);
    assert_eq!(unseen.text.matches("and on the south floor").count(), 1);
    assert_eq!(
        unseen.text.matches("hoofprints on the south floor").count(),
        0
    );
    assert_eq!(
        unseen
            .text
            .matches("Hoofprints mark the south floor")
            .count(),
        1
    );
    let prints_line = unseen.text.trim_end().rsplit_once('\n').unwrap().1;
    assert_eq!(unseen.text.matches("The walk turns west").count(), 0);
    assert_eq!(
        prints_line.matches("Hoofprints").count(),
        1,
        "{prints_line}"
    );
    assert_eq!(
        prints_line.matches("hoofprints").count(),
        0,
        "{prints_line}"
    );
    assert_eq!(
        prints_line.matches("valley floor").count(),
        0,
        "{prints_line}"
    );
    assert_eq!(
        prints_line.matches("south floor").count(),
        1,
        "{prints_line}"
    );
    assert!(
        prints_line.ends_with("turn west out of sight."),
        "{prints_line}"
    );
    assert_eq!(
        prints_line.matches("turning west there").count(),
        0,
        "{prints_line}"
    );
    assert!(!prints_line.contains("The walk turns"), "{prints_line}");
    assert!(!prints_line.contains("another"), "{prints_line}");
    assert!(
        !prints_line.contains("west of the south floor"),
        "{prints_line}"
    );
    assert!(!prints_line.contains("its"), "{prints_line}");
    assert!(!prints_line.contains("west of that floor"), "{prints_line}");
    assert!(
        !prints_line.contains("onto the south floor"),
        "{prints_line}"
    );
    assert_eq!(
        prints_line.matches("to its west").count(),
        0,
        "{prints_line}"
    );
    assert_eq!(prints_line.matches("another").count(), 0, "{prints_line}");
    assert_eq!(
        prints_line.matches("that floor").count(),
        0,
        "{prints_line}"
    );
    assert!(!prints_line.contains("beyond"), "{prints_line}");
    assert!(!prints_line.contains("the ground"), "{prints_line}");
    assert!(!prints_line.contains("one step"), "{prints_line}");
    assert!(
        prints_line.contains(", and on the south floor they turn west out of sight")
            && !prints_line.contains("which")
            && !prints_line.contains("turns")
            && !prints_line.contains("goes")
            && !prints_line.contains("from the valley floor")
            && prints_line.matches("the south floor").count() == 1
            && prints_line.matches("the prints").count() == 0
            && prints_line.matches("they").count() == 1
            && prints_line.matches("Hoofprints").count() == 1
            && !prints_line.contains("off the south floor")
            && !prints_line.contains("still in sight")
            && !prints_line.contains(", out of sight")
            && prints_line.matches("and").count() == 1
            && prints_line.matches("the deer's walk").count() == 1
            && prints_line.matches("the walk").count() == 0
            && !prints_line.contains("going")
            && !prints_line.contains("crossing")
            && !prints_line.contains("turning")
            && !prints_line.contains(" it")
            && !prints_line.contains("there")
            && !prints_line.contains("where"),
        "{prints_line}"
    );
    assert!(
        !prints_line.contains("in the open") && !prints_line.contains("west onto"),
        "{prints_line}"
    );
    assert!(
        prints_line.starts_with("Hoofprints on that wood"),
        "{prints_line}"
    );
    assert!(
        !prints_line.contains("The walk turns west"),
        "{prints_line}"
    );
    assert!(
        !unseen.text.contains("The deer went on from"),
        "{}",
        unseen.text
    );
    assert!(
        !unseen.text.to_lowercase().contains("a line of hoofprints"),
        "{}",
        unseen.text
    );
    assert_eq!(unseen.text.matches("the deer went on").count(), 0);
    assert!(!unseen.text.contains("From there"), "{}", unseen.text);
    assert!(
        !unseen.text.to_lowercase().contains("past the south floor"),
        "{}",
        unseen.text
    );
    assert!(
        !unseen.text.contains("went on out of sight"),
        "{}",
        unseen.text
    );
    assert!(
        !unseen.text.contains("from the south floor"),
        "{}",
        unseen.text
    );
    assert!(
        !unseen
            .text
            .contains("A deer trail passes on that valley floor"),
        "{}",
        unseen.text
    );
    assert!(
        !unseen.text.contains("The berries that were here are gone"),
        "{}",
        unseen.text
    );
    assert!(
        unseen
            .text
            .contains("Wood left in the open is rotting to the south and west."),
        "{}",
        unseen.text
    );
    assert!(
        unseen.text.contains(
            "South: valley floor, nettles. The corner beyond, to the west, has that wood. Past that wood is the west floor. The deer has left it. Hoofprints mark that ground. South of the west floor is the south floor, still in sight. Hoofprints mark the south floor."
        ),
        "{}",
        unseen.text
    );
    assert!(
        unseen
            .text
            .contains("West: hoofprints mark the outside floor. The rotting wood is the next square south of that floor. West of that floor, Aside stands on the dark berries."),
        "{}",
        unseen.text
    );
    let from_west_later = world.observe(aside).unwrap();
    assert_eq!(world.actor_cell(aside), Some((4, 1)));
    assert_eq!(world.sim_time(), 15);
    let aside_place_later = from_west_later
        .text
        .lines()
        .find(|line| line.starts_with("You are on the"))
        .unwrap_or("");
    assert_eq!(
        aside_place_later,
        "You are on the valley floor, in the open. The outside floor is the next square east. Dark berries lie here, dry. They taste bitter. They are poisonous. Worms are in the dry ground here. Carve an antler into an antler pick to dig them."
    );
    assert!(
        aside_place_later.starts_with("You are on the valley floor, in the open."),
        "{aside_place_later}"
    );
    assert!(
        aside_place_later.contains("poisonous"),
        "{aside_place_later}"
    );
    assert!(!aside_place_later.contains("rock"), "{aside_place_later}");
    let aside_east_later = from_west_later
        .text
        .lines()
        .find(|line| line.starts_with("East:"))
        .unwrap_or("");
    assert_eq!(
        aside_east_later,
        "East: the outside floor. Hoofprints mark that floor. The rock overhang is the next square east of that floor. Near stands on the valley floor under that rock."
    );
    assert!(!aside_east_later.contains("person"), "{aside_east_later}");
    let aside_south_later = from_west_later
        .text
        .lines()
        .find(|line| line.starts_with("South:"))
        .unwrap_or("");
    assert_eq!(
        aside_south_later,
        "South: the west floor. Hoofprints mark that floor. South of that floor is the south floor, still in sight. Hoofprints mark the south floor."
    );
    assert!(
        !aside_south_later.contains("The deer stands"),
        "{aside_south_later}"
    );
    assert!(from_west_later.text.contains(
        "A deer stands in the open, on the deer trail west of the south floor. Corner stands on that trail."
    ));
    assert!(
        !from_west_later
            .text
            .contains("A deer stands two cells south and one cell west, in the open."),
        "{}",
        from_west_later.text
    );
    assert!(from_west_later.text.contains(
        "Wood left in the open is rotting south of the outside floor, and the deer left hoofprints on the rotting wood."
    ));
    assert!(
        !from_west_later.text.contains("one cell"),
        "{}",
        from_west_later.text
    );
    assert!(
        !from_west_later.text.contains("two cells"),
        "{}",
        from_west_later.text
    );
    assert!(
        !from_west_later.text.contains("out of sight"),
        "{}",
        from_west_later.text
    );
    assert!(
        !from_west_later
            .text
            .contains("East: valley floor, hoofprints."),
        "{}",
        from_west_later.text
    );
    let south_gone = unseen
        .text
        .find("The corner beyond, to the west, has that wood")
        .expect(&unseen.text);
    let prints_gone = unseen
        .text
        .find("Hoofprints on that wood mark the deer's walk, and on the south floor they turn west out of sight.")
        .expect(&unseen.text);
    assert!(south_gone < prints_gone, "{}", unseen.text);
    assert!(
        !unseen.text.contains("one cell south and one cell west"),
        "{}",
        unseen.text
    );
    assert!(
        !unseen.text.contains("hoofprints mark the ground there"),
        "{}",
        unseen.text
    );
    assert!(
        !unseen.text.contains("Hoofprints mark the ground here."),
        "{}",
        unseen.text
    );
    assert_eq!(unseen.text.matches("on the ground here").count(), 1);
    let ground_prints = unseen
        .text
        .find("The deer left hoofprints on the ground here.")
        .expect(&unseen.text);
    let bark_at = unseen
        .text
        .find("A pile of bark lies here.")
        .expect(&unseen.text);
    let around_at = unseen.text.find("Around you:").expect(&unseen.text);
    assert!(
        bark_at < ground_prints && ground_prints < around_at && around_at < prints_gone,
        "{}",
        unseen.text
    );
    let west_gone = unseen
        .text
        .find("West: hoofprints mark the outside floor. The rotting wood is the next square south of that floor. West of that floor, Aside stands on the dark berries.")
        .expect(&unseen.text);
    assert!(west_gone < prints_gone, "{}", unseen.text);
    assert!(!unseen.text.contains("as well as here"), "{}", unseen.text);
    assert!(!unseen.text.contains("rotting wood too"), "{}", unseen.text);
    assert!(!unseen.text.contains("walked off west"), "{}", unseen.text);
    assert!(
        !unseen.text.contains("walked off to the west"),
        "{}",
        unseen.text
    );
    assert!(
        !unseen.text.contains("as well as under this rock"),
        "{}",
        unseen.text
    );
    assert!(
        !unseen.text.contains("south of that west"),
        "{}",
        unseen.text
    );
    let after_compass = unseen.text.split("Around you:").nth(1).unwrap_or("");
    let after_without_the_meal = after_compass
        .replace("there", "")
        .replace("ate the berries here", "")
        .replace("the berries that were here", "")
        .replace("where the deer is now", "")
        .replace("where the deer no longer stands", "")
        .replace("where the deer went", "");
    assert!(!after_without_the_meal.contains("here"), "{}", unseen.text);
    assert!(!unseen.text.contains("West just showed"), "{}", unseen.text);
    assert!(
        !unseen.text.contains("on the valley floor to the west"),
        "{}",
        unseen.text
    );
    assert!(
        !unseen.text.contains("south of the prints"),
        "{}",
        unseen.text
    );
    assert!(
        !unseen.text.contains("prints on that floor"),
        "{}",
        unseen.text
    );
    assert!(
        !unseen.text.contains("one step farther west"),
        "{}",
        unseen.text
    );
    assert_eq!(unseen.text.matches("out of sight").count(), 1);
    assert_eq!(unseen.text.matches("Hoofprints on that wood").count(), 1);
    assert_eq!(unseen.text.matches("the ground beyond").count(), 0);
    assert_eq!(unseen.text.matches("then west").count(), 0);
    assert_eq!(unseen.text.matches("west of that floor").count(), 0);
    assert_eq!(
        unseen.text.matches("the valley floor to its west").count(),
        0
    );
    assert_eq!(
        unseen
            .text
            .matches("another valley floor west of the south floor")
            .count(),
        0
    );
    assert_eq!(unseen.text.matches("the open valley floor").count(), 0);
    assert_eq!(
        unseen.text.matches("the valley floor, in the open").count(),
        0
    );
    assert_eq!(unseen.text.matches("turning west there").count(), 0);
    assert_eq!(unseen.text.matches("west onto").count(), 0);
    assert_eq!(unseen.text.matches("and on.").count(), 0);
    assert_eq!(unseen.text.matches("still in sight").count(), 1);
    assert_eq!(
        unseen
            .text
            .matches("The deer stands on the south floor")
            .count(),
        0
    );
    assert_eq!(
        unseen
            .text
            .matches("Hoofprints mark the south floor")
            .count(),
        1
    );
    assert!(
        !unseen.text.contains("that is still in sight"),
        "{}",
        unseen.text
    );
    assert_eq!(
        unseen.text.matches("Hoofprints mark that ground").count(),
        1
    );
    assert!(
        !unseen.text.contains("the deer is not there"),
        "{}",
        unseen.text
    );
    assert!(
        !unseen.text.contains("the deer is there"),
        "{}",
        unseen.text
    );
    let after_south = unseen
        .text
        .split("Hoofprints mark that ground.")
        .nth(1)
        .unwrap_or("");
    assert!(!after_south.contains("that ground"), "{}", unseen.text);
    assert!(
        after_south.contains("South of the west floor is the south floor"),
        "{}",
        unseen.text
    );
    assert!(!after_south.contains("far floor"), "{}", unseen.text);
    assert!(
        after_south.matches("the valley floor, in the open").count() == 0
            && after_south.matches("turning west out of sight").count() == 0
            && after_south.matches("and west out of sight").count() == 0
            && after_south.matches("and turns west out of sight").count() == 0
            && after_south
                .matches("turns west off the south floor")
                .count()
                == 0
            && after_south.matches("which turns").count() == 0
            && after_south
                .matches("and on the south floor they turn west out of sight")
                .count()
                == 1
            && after_south.matches("turning west there").count() == 0
            && after_south.matches("The walk turns west").count() == 0,
        "{}",
        unseen.text
    );
    assert_eq!(after_south.matches("crossing the south floor").count(), 0);
    assert_eq!(after_south.matches("across the south floor").count(), 0);
    assert_eq!(
        after_south
            .replace("across the south floor", "")
            .matches("cross the south floor")
            .count(),
        0
    );
    assert_eq!(after_south.matches("passes on out of view").count(), 0);
    assert_eq!(after_south.matches("pass on out of view").count(), 0);
    assert_eq!(
        after_south
            .matches("past that floor no more hoofprints show")
            .count(),
        0
    );
    assert!(!after_south.contains(" row"), "{}", unseen.text);
    assert!(!after_south.contains("line"), "{}", unseen.text);
    assert_eq!(after_south.matches("the line of them").count(), 0);
    assert!(!after_south.contains(" them"), "{}", unseen.text);
    assert_eq!(
        after_south
            .matches("the last you can see are on the south floor")
            .count(),
        0
    );
    assert!(!after_south.contains("you can see"), "{}", unseen.text);
    assert_eq!(after_south.matches("as far as the south floor").count(), 0);
    assert!(
        !after_south.contains("into the open west of that floor"),
        "{}",
        unseen.text
    );
    assert!(
        !after_south.contains("where the deer is now"),
        "{}",
        unseen.text
    );
    assert_eq!(
        after_south
            .matches("and the deer is no longer in sight")
            .count(),
        0
    );
    assert_eq!(
        after_south
            .matches("The deer is no longer in sight.")
            .count(),
        0
    );
    assert!(
        !after_south.contains("not the valley floor one step"),
        "{}",
        unseen.text
    );
    assert!(
        !after_south.contains("hoofprints mark it"),
        "{}",
        unseen.text
    );
    assert!(!after_south.contains("stands on it"), "{}", unseen.text);
    assert!(
        !after_south.contains("from that valley floor"),
        "{}",
        unseen.text
    );
    assert!(
        !after_south.contains("past the south floor"),
        "{}",
        unseen.text
    );
    assert!(!after_south.contains("past the wood"), "{}", unseen.text);
    assert!(
        !after_south.contains("walked south from the valley floor past the wood"),
        "{}",
        unseen.text
    );
    assert!(
        !after_south.contains("walked south from the valley floor to the wood's west"),
        "{}",
        unseen.text
    );
    assert!(
        !after_south.contains("onto the valley floor to the south floor's west"),
        "{}",
        unseen.text
    );
    assert_eq!(
        after_south
            .matches("on the south floor's west side")
            .count(),
        0
    );
    assert_eq!(after_south.matches("on the wood's west side").count(), 0);
    assert_eq!(after_south.matches("that valley floor").count(), 0);
    assert_eq!(after_south.matches(", in the open,").count(), 0);
    assert_eq!(after_south.matches("on that floor").count(), 0);
    assert!(!after_south.contains("the ground"), "{}", unseen.text);
    assert!(!after_south.contains("the near floor"), "{}", unseen.text);
    assert!(!after_south.contains("one step west"), "{}", unseen.text);
    assert!(!after_south.contains("beyond"), "{}", unseen.text);
    assert!(
        !after_south.contains("the floor west of the south floor"),
        "{}",
        unseen.text
    );
    assert!(
        !after_south.contains("the deer is on the floor"),
        "{}",
        unseen.text
    );
    assert_eq!(
        after_south
            .matches("left hoofprints on a trail there")
            .count(),
        0
    );
    assert!(!after_south.contains("a trail there"), "{}", unseen.text);
    assert_eq!(
        after_south
            .matches("A deer trail passes on that valley floor")
            .count(),
        0
    );
    assert!(
        !after_south.contains("onto valley floor"),
        "{}",
        unseen.text
    );
    assert!(
        !unseen.text.contains("onto valley floor, then"),
        "{}",
        unseen.text
    );
    assert!(!unseen.text.contains("the cell"), "{}", unseen.text);
    assert!(!unseen.text.contains("open cell west"), "{}", unseen.text);
    assert!(!unseen.text.contains("underfoot"), "{}", unseen.text);
    assert!(!unseen.text.contains("first west"), "{}", unseen.text);
    assert!(
        !unseen.text.contains("you see to the west"),
        "{}",
        unseen.text
    );
    assert!(
        !unseen.text.contains("away from the hoofprints"),
        "{}",
        unseen.text
    );
    assert!(
        !unseen.text.contains("to the north of it"),
        "{}",
        unseen.text
    );
    assert!(
        !unseen.text.contains("has the rotting wood and the deer"),
        "{}",
        unseen.text
    );
    assert!(
        !unseen.text.contains("The deer on that wood"),
        "{}",
        unseen.text
    );
    assert!(
        !unseen.text.contains("The deer one cell"),
        "{}",
        unseen.text
    );
    assert!(!unseen.text.contains("stands there"), "{}", unseen.text);
    assert!(!unseen.text.contains("stands here"), "{}", unseen.text);
    assert!(!unseen.text.contains("eating"), "{}", unseen.text);
    assert!(!world
        .observe(far)
        .unwrap()
        .text
        .contains("berries that were here are gone"));
    assert!(!world.observe(far).unwrap().text.contains("last handful"));
    assert_eq!(
        world.examine(far, ExamineTarget::Wood),
        "You see no wood from here."
    );
}

#[test]
fn examine_berries_after_the_pile_is_gone_says_they_are_gone() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let far = world.spawn_at("Far", "f", 0, 0).unwrap();
    let before = world.examine(near, ExamineTarget::Berries);
    assert!(
        before.contains("A pile of 4 handfuls of berries"),
        "{before}"
    );
    assert!(!before.contains("are gone"), "{before}");
    let early = world.observe(near).unwrap().text;
    assert!(early.contains("Berries lie here"), "{early}");
    assert!(!early.contains("eating the berries"), "{early}");
    let mut saw_eating = false;
    let mut later_handful = false;
    for _ in 0..12 {
        if rock_berry_pile(&world, near) == 0 {
            break;
        }
        let before = rock_berry_pile(&world, near);
        world.step(&[]);
        let after = rock_berry_pile(&world, near);
        if world.deer_cell() == Some((6, 1)) && after > 0 {
            let text = world.observe(near).unwrap().text;
            if after < before {
                if after == 1 {
                    assert!(
                        text.contains(
                            "A deer is eating the berries here. This handful is the last."
                        ),
                        "{text}"
                    );
                    assert!(!text.contains("More are left."), "{text}");
                    assert!(!text.contains("A handful is gone."), "{text}");
                } else if later_handful {
                    assert!(
                        text.contains(
                            "A deer is eating the berries here. Another handful is gone. More are left."
                        ),
                        "{text}"
                    );
                    assert!(
                        !text.contains("A handful is gone. More are left."),
                        "{text}"
                    );
                    assert!(!text.contains("This handful is the last."), "{text}");
                } else {
                    assert!(
                        text.contains(
                            "A deer is eating the berries here. A handful is gone. More are left."
                        ),
                        "{text}"
                    );
                    assert!(!text.contains("Another handful"), "{text}");
                    assert!(!text.contains("This handful is the last."), "{text}");
                    later_handful = true;
                }
                assert!(!text.contains("The last handful"), "{text}");
                assert!(!text.contains("these branches"), "{text}");
            } else {
                assert!(text.contains("A deer stands on these branches."), "{text}");
                assert!(!text.contains("eating the berries"), "{text}");
            }
            assert!(!text.contains("Berries lie here"), "{text}");
            assert!(!text.contains("A deer stands here."), "{text}");
            let far_text = world.observe(far).unwrap().text;
            assert!(!far_text.contains("eating the berries"), "{far_text}");
            assert!(!far_text.contains("these branches"), "{far_text}");
            let count = after;
            let berries = world.examine(near, ExamineTarget::Berries);
            assert!(berries.contains("A deer is eating them."), "{berries}");
            if count == 1 {
                assert!(
                    berries.starts_with("A handful of berries is in contact with the ground here"),
                    "{berries}"
                );
                assert!(!berries.contains("pile"), "{berries}");
            } else {
                assert!(
                    berries.contains(&format!("A pile of {count} handfuls of berries")),
                    "{berries}"
                );
            }
            assert_eq!(
                world.examine(far, ExamineTarget::Berries),
                "You see no berries from here."
            );
            saw_eating = true;
        }
    }
    assert!(saw_eating);
    assert_eq!(rock_berry_pile(&world, near), 0);
    assert_eq!(world.material_tally(Material::Berry), 1);
    assert_eq!(world.deer_cell(), Some((6, 1)));
    let clock = world.sim_time();
    assert_eq!(
        world.examine(near, ExamineTarget::Berries),
        "A handful of berries lies in contact with the ground here, under the rock overhang. It is dry. Rain does not reach it. They are good to eat raw. A deer ate the rest."
    );
    assert_eq!(world.sim_time(), clock);
    assert_eq!(world.material_tally(Material::Berry), 1);
    let gone = world.observe(near).unwrap().text;
    assert!(!gone.contains("Berries lie here"), "{gone}");
    assert!(!gone.contains("The last handful is gone."), "{gone}");
    assert!(!gone.contains("is eating"), "{gone}");
    assert!(
        gone.contains(
            "A handful of berries lies here. It is good to eat raw. A deer ate the rest."
        ),
        "{gone}"
    );
    assert!(!gone.contains("after eating the berries"), "{gone}");
    assert!(
        !gone.contains("The deer standing here ate the berries that were here."),
        "{gone}"
    );
    assert!(!gone.contains("A deer stands here."), "{gone}");
    assert!(
        !gone.contains("The berries that were here are gone"),
        "{gone}"
    );
    assert!(!gone.contains("The deer here ate them"), "{gone}");
    assert!(!world.observe(far).unwrap().text.contains("last handful"));
    assert_eq!(
        world.examine(far, ExamineTarget::Berries),
        "You see no berries from here."
    );
    assert_eq!(
        world.examine(near, ExamineTarget::Deer),
        "A deer stands here, under the rock overhang. It ate the berries here."
    );
    let beside = world.spawn_at("Beside", "b", 5, 1).unwrap();
    assert_eq!(
        world.examine(beside, ExamineTarget::Deer),
        "A deer stands one cell east, under the rock overhang. It ate the berries there."
    );
    assert_eq!(
        world.examine(far, ExamineTarget::Deer),
        "You cannot see a deer from here."
    );
    assert_eq!(world.sim_time(), clock);
    world.step(&[]);
    assert_ne!(world.deer_cell(), Some((6, 1)));
    let clock = world.sim_time();
    assert_eq!(
        world.examine(near, ExamineTarget::Berries),
        "A handful of berries lies in contact with the ground here, under the rock overhang. It is dry. Rain does not reach it. They are good to eat raw. A deer ate the rest."
    );
    let deer = world.examine(near, ExamineTarget::Deer);
    assert_eq!(deer, "A deer stands one cell west, in the open.");
    assert!(!deer.contains("ate"), "{deer}");
    assert_eq!(world.sim_time(), clock);
    assert_eq!(
        world.examine(far, ExamineTarget::Berries),
        "You see no berries from here."
    );
}

#[test]
fn examine_berries_does_not_blame_a_deer_that_did_not_eat_them() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    for i in 0..3 {
        world.step(&[cmd(
            &format!("g{i}"),
            near,
            Attempt::Gather(Material::Berry),
        )]);
    }
    let one = world.examine(near, ExamineTarget::Berries);
    assert!(
        one.starts_with("A handful of berries is in contact with the ground here"),
        "{one}"
    );
    assert!(!one.contains("pile"), "{one}");
    assert!(!one.contains("deer"), "{one}");
    world.step(&[cmd("g3", near, Attempt::Gather(Material::Berry))]);
    for (i, dir) in [Dir::West, Dir::West, Dir::West].into_iter().enumerate() {
        world.step(&[cmd(&format!("w{i}"), near, Attempt::Move(dir))]);
    }
    assert_eq!(world.actor_cell(near), Some((3, 1)));
    assert!(world
        .event_log()
        .iter()
        .all(|event| !event.text.contains("eats")));
    let witness = world.spawn_at("Witness", "w", 6, 1).unwrap();
    let clock = world.sim_time();
    assert_eq!(
        world.examine(witness, ExamineTarget::Berries),
        "The berries that were here are gone."
    );
    assert_eq!(world.sim_time(), clock);
    for _ in 0..20 {
        if world.deer_cell() == Some((6, 1)) {
            break;
        }
        world.step(&[]);
    }
    assert_eq!(world.deer_cell(), Some((6, 1)));
    let clock = world.sim_time();
    let text = world.examine(witness, ExamineTarget::Berries);
    assert_eq!(text, "The berries that were here are gone.");
    assert!(!text.contains("deer"), "{text}");
    let deer = world.examine(witness, ExamineTarget::Deer);
    assert_eq!(deer, "A deer stands here, under the rock overhang.");
    assert!(!deer.contains("ate"), "{deer}");
    assert_eq!(world.sim_time(), clock);
    let far = world.spawn_at("Far", "f", 0, 0).unwrap();
    assert_eq!(
        world.examine(far, ExamineTarget::Berries),
        "You see no berries from here."
    );
}

#[test]
fn a_flint_nodule_on_the_slope_can_be_carried() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 4, 0).unwrap();
    let west = world.spawn_at("West", "w", 4, 1).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let here = world.observe(near).unwrap().text;
    assert!(here.contains("A flint nodule lies here."), "{here}");
    assert_eq!(
        world.examine(near, ExamineTarget::Flint),
        "A flint nodule lies here, on the slope. It is a sharp stone. It can be knapped."
    );
    let beside = world.observe(west).unwrap().text;
    assert!(
        beside.contains("A flint nodule lies on the slope to the north."),
        "{beside}"
    );
    assert!(beside.contains("North: slope, a flint nodule"), "{beside}");
    assert_eq!(
        world.examine(west, ExamineTarget::Flint),
        "A flint nodule lies on the slope to the north. It is a sharp stone. It can be knapped."
    );
    assert_eq!(
        world.examine(far, ExamineTarget::Flint),
        "You see no flint from here."
    );
    assert!(beside.contains("a flint nodule"), "{beside}");
    assert!(
        !world.observe(far).unwrap().text.contains("flint"),
        "{}",
        world.observe(far).unwrap().text
    );
    let report = world.step(&[cmd("flint", near, Attempt::Gather(Material::Flint))]);
    assert_eq!(report.results[0].message, "You gather a flint nodule.");
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("flint")));
    let seen: Vec<_> = world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("flint"))
        .collect();
    assert_eq!(seen, vec!["Near gathers a flint nodule.".to_string()]);
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("flint")));
    let after = world.observe(near).unwrap().text;
    assert!(!after.contains("A flint nodule lies here."), "{after}");
    assert_eq!(arms_sentence(&after), "You have a dry flint nodule.");
    let beside = world.observe(west).unwrap().text;
    assert!(
        !beside.contains("a flint nodule"),
        "the nodule is in the arms, not still on the slope: {beside}"
    );
}

#[test]
fn a_carried_flint_nodule_knaps_into_one_sharp_flake() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 4, 0).unwrap();
    let west = world.spawn_at("West", "w", 4, 1).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let bare = world.step(&[cmd("early", near, Attempt::Knap)]);
    let failure = "You are not carrying a flint nodule.";
    assert_eq!(bare.results[0].message, failure);
    let near_fail: Vec<_> = world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text == failure || text.to_lowercase().contains("knap"))
        .collect();
    assert_eq!(near_fail, vec![failure.to_string()]);
    for witness in [west, far] {
        assert!(
            world
                .observe(witness)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != failure && !event.text.to_lowercase().contains("knap")),
            "{:?}",
            world
                .observe(witness)
                .unwrap()
                .events
                .iter()
                .map(|event| event.text.clone())
                .collect::<Vec<_>>()
        );
    }
    world.step(&[cmd("flint", near, Attempt::Gather(Material::Flint))]);
    let before = world.observe(near).unwrap().text;
    assert_eq!(arms_sentence(&before), "You have a dry flint nodule.");
    assert_eq!(
        world.examine(near, ExamineTarget::Flint),
        "A flint nodule is carried by you, in the open. It is a sharp stone. It can be knapped."
    );
    assert_eq!(
        world.examine(west, ExamineTarget::Flint),
        "A flint nodule is carried by Near, in the open. It is a sharp stone. It can be knapped."
    );
    let report = world.step(&[cmd("knap", near, Attempt::Knap)]);
    let sentence = "You knap the flint nodule into a sharp flake.";
    let witnessed = "Near knaps the flint nodule into a sharp flake.";
    assert_eq!(report.results[0].message, sentence);
    let yours: Vec<_> = world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("knap") || text.contains("flake"))
        .collect();
    assert!(
        yours.is_empty(),
        "the knapper does not re-read the knap: {yours:?}"
    );
    let seen: Vec<_> = world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("knap"))
        .collect();
    assert_eq!(seen, vec![witnessed.to_string()]);
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("knap") && !event.text.contains("flake")));
    let after = world.observe(near).unwrap().text;
    assert_eq!(
        arms_sentence(&after),
        "You have a sharp flint flake that is sharp enough to cut or haft later."
    );
    assert!(
        !after.contains("It is sharp enough to cut or haft later."),
        "{after}"
    );
    assert!(!after.contains("nodule"), "{after}");
    let examined = world.examine(near, ExamineTarget::Flake);
    assert!(
        examined.contains("It is sharp enough to cut or haft later."),
        "{examined}"
    );
    assert!(!examined.contains("nodule"), "{examined}");
    let from_west = world.examine(west, ExamineTarget::Flint);
    assert!(
        from_west.contains("It is sharp enough to cut or haft later."),
        "{from_west}"
    );
    assert!(!examined.contains("knapped"), "{examined}");
    assert!(!from_west.contains("knapped"), "{from_west}");
    let again = world.step(&[cmd("again", near, Attempt::Knap)]);
    assert_eq!(
        again.results[0].message,
        "You are not carrying a flint nodule."
    );
}

#[test]
fn a_brand_lights_a_second_fire_then_burns_out() {
    let mut world = world_at(40);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    let far = world.spawn_at("Far", "f", 0, 4).unwrap();
    let early = world.step(&[cmd("early", near, Attempt::TakeBrand)]);
    assert_eq!(early.results[0].message, "There is no fire burning here.");
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("brand")));
    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    let light = world.step(&[cmd("light", near, Attempt::Light)]);
    assert!(light.results[0].success, "{}", light.results[0].message);
    world.step(&[cmd("bark-2", near, Attempt::Gather(Material::Bark))]);
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    let take = world.step(&[cmd("brand", near, Attempt::TakeBrand)]);
    let yours = "You take a burning brand from the fire. It stays lit.";
    let seen = "Near takes a burning brand from the fire. It stays lit.";
    assert_eq!(take.results[0].message, yours);
    assert!(
        world
            .observe(near)
            .unwrap()
            .text
            .contains("A small fire burns here."),
        "{}",
        world.observe(near).unwrap().text
    );
    let again = world.step(&[cmd("again", near, Attempt::TakeBrand)]);
    assert_eq!(
        again.results[0].message,
        "You are already carrying a burning brand."
    );
    let examined = world.examine(near, ExamineTarget::Brand);
    assert!(
        examined.contains(
            "A burning brand is carried by you, under the rock overhang. It will not burn long."
        ),
        "{examined}"
    );
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == yours));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == seen));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("brand")));
    world.step(&[cmd("south", near, Attempt::Move(Dir::South))]);
    assert_eq!(world.actor_cell(near), Some((6, 2)));
    let planted = world.step(&[cmd("plant", near, Attempt::Light)]);
    let planted_line =
        "You light a fire with the burning brand and the bark and the branch from the arms.";
    assert_eq!(planted.results[0].message, planted_line);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != planted_line && !event.text.contains("lights a fire")));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Near lights a fire with the burning brand and the bark and the branch from the arms."));
    world.step(&[cmd("back", near, Attempt::Move(Dir::North))]);
    assert!(
        world
            .observe(near)
            .unwrap()
            .text
            .contains("A small fire burns here."),
        "source fire"
    );
    world.step(&[cmd("see-new", near, Attempt::Move(Dir::South))]);
    assert!(
        world
            .observe(near)
            .unwrap()
            .text
            .contains("A small fire burns here."),
        "new fire"
    );
    world.step(&[cmd("home", near, Attempt::Move(Dir::North))]);
    assert!(
        world
            .observe(near)
            .unwrap()
            .text
            .contains("A small fire burns here."),
        "the fire they took the brand from is still burning"
    );
    let second = world.step(&[cmd("second", near, Attempt::TakeBrand)]);
    assert_eq!(second.results[0].message, yours);
    for _ in 0..4 {
        world.step(&[]);
    }
    assert!(
        world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Your brand burns out."),
        "{:?}",
        world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .map(|event| event.text.clone())
            .collect::<Vec<_>>()
    );
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Near's brand burns out."));
}

#[test]
fn rain_in_the_open_puts_a_carried_brand_out() {
    let mut world = world_at(4);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    let light = world.step(&[cmd("light", near, Attempt::Light)]);
    assert!(light.results[0].success, "{}", light.results[0].message);
    let take = world.step(&[cmd("brand", near, Attempt::TakeBrand)]);
    assert_eq!(
        take.results[0].message,
        "You take a burning brand from the fire. It stays lit."
    );
    world.step(&[cmd("south", near, Attempt::Move(Dir::South))]);
    assert_eq!(world.actor_cell(near), Some((6, 2)));
    world.step(&[]);
    assert!(
        world.observe(near).unwrap().raining,
        "rain should have reached the valley"
    );
    assert!(
        world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "The rain puts your brand out."),
        "{:?}",
        world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .map(|event| event.text.clone())
            .collect::<Vec<_>>()
    );
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "The rain puts Near's brand out."));
    world.step(&[cmd("back", near, Attempt::Move(Dir::North))]);
    assert!(
        world
            .observe(near)
            .unwrap()
            .text
            .contains("A small fire burns here."),
        "rain on the brand does not put the sheltered fire out"
    );
}

#[test]
fn smoke_shows_past_the_fire_and_stops_one_cell_farther() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let past = world.spawn_at("Past", "p", 0, 4).unwrap();
    assert!(!world.observe(west).unwrap().text.contains("Smoke"));
    assert!(!world.observe(far).unwrap().text.contains("Smoke"));
    assert!(!world.observe(past).unwrap().text.contains("Smoke"));
    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    let light = world.step(&[cmd("light", near, Attempt::Light)]);
    assert!(light.results[0].success, "{}", light.results[0].message);
    let here = world.observe(near).unwrap().text;
    assert!(
        here.contains("A small fire burns here. It warms you.") && !here.contains("Smoke rises"),
        "{here}"
    );
    let mid = world.observe(west).unwrap().text;
    assert!(
        mid.contains("A small fire burns") && !mid.contains("Smoke rises"),
        "{mid}"
    );
    let smoke = world.observe(far).unwrap().text;
    assert!(
        smoke.contains("Smoke rises to the north and east.")
            && !smoke.contains("small fire")
            && !smoke.contains("A fire burns"),
        "{smoke}"
    );
    let silent = world.observe(past).unwrap().text;
    assert!(
        !silent.contains("Smoke") && !silent.contains("small fire"),
        "{silent}"
    );
    let near_key = world.observe(near).unwrap();
    assert!(near_key.legend.contains("* fire"), "{}", near_key.legend);
    assert!(near_key.map.contains('@') && near_key.map.contains('*'));
    let west_key = world.observe(west).unwrap();
    assert!(west_key.legend.contains("* fire"), "{}", west_key.legend);
    assert!(west_key.map.contains('*'), "{}", west_key.map);
    let far_key = world.observe(far).unwrap();
    assert!(
        far_key.text.contains("Smoke rises to the north and east.")
            && !far_key.legend.contains("* fire")
            && !far_key.map.contains('*'),
        "legend:\n{}\nmap:\n{}",
        far_key.legend,
        far_key.map
    );
    let past_key = world.observe(past).unwrap();
    assert!(
        !past_key.legend.contains("* fire") && !past_key.map.contains('*'),
        "legend:\n{}\nmap:\n{}",
        past_key.legend,
        past_key.map
    );
}

#[test]
fn a_brook_stretch_is_quiet_after_a_fish_and_another_stretch_is_not() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 4, 0).unwrap();
    let west = world.spawn_at("West", "w", 3, 3).unwrap();
    let far = world.spawn_at("Far", "f", 6, 4).unwrap();
    world.step(&[cmd("flint", near, Attempt::Gather(Material::Flint))]);
    world.step(&[cmd("knap", near, Attempt::Knap)]);
    for (id, dir) in [("e1", Dir::East), ("e2", Dir::East), ("s", Dir::South)] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    let light = world.step(&[cmd("light", near, Attempt::Light)]);
    assert!(light.results[0].success, "{}", light.results[0].message);
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    let haft = world.step(&[cmd("haft", near, Attempt::Haft)]);
    assert!(haft.results[0].success, "{}", haft.results[0].message);
    for _ in 0..4 {
        world.step(&[]);
    }
    for (id, dir) in [
        ("w1", Dir::West),
        ("w2", Dir::West),
        ("w3", Dir::West),
        ("w4", Dir::West),
        ("bank-s", Dir::South),
    ] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    assert_eq!(world.actor_cell(near), Some((2, 2)));
    let yours = "You spear a fish from the brook.";
    let first = world.step(&[cmd("fish", near, Attempt::Fish)]);
    assert_eq!(first.results[0].message, yours);
    let quiet = "The fish are gone from this stretch of the brook.";
    let again = world.step(&[cmd("again", near, Attempt::Fish)]);
    assert_eq!(again.results[0].message, quiet);
    assert_eq!(world.material_tally(Material::Fish), 1);
    let here = world.observe(near).unwrap().text;
    assert!(
        here.contains("The brook is quiet. The fish are gone."),
        "{here}"
    );
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == quiet));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("fish are gone")));
    let seen = world.observe(west).unwrap().text;
    assert!(
        seen.contains("The brook to the north and west is quiet."),
        "{seen}"
    );
    assert!(
        !world.observe(far).unwrap().text.contains("is quiet"),
        "{}",
        world.observe(far).unwrap().text
    );
    world.step(&[cmd("north", near, Attempt::Move(Dir::North))]);
    assert_eq!(world.actor_cell(near), Some((2, 1)));
    let from_the_pool = world.observe(near).unwrap().text;
    assert!(
        from_the_pool.contains("The brook to the south and west is quiet."),
        "{from_the_pool}"
    );
    let other = world.step(&[cmd("other", near, Attempt::Fish)]);
    assert_eq!(other.results[0].message, yours);
    assert_eq!(world.material_tally(Material::Fish), 2);
    world.step(&[cmd("south", near, Attempt::Move(Dir::South))]);
    assert_eq!(world.actor_cell(near), Some((2, 2)));
    let back = world.step(&[cmd("back", near, Attempt::Fish)]);
    assert_eq!(back.results[0].message, yours);
    assert_eq!(world.material_tally(Material::Fish), 3);
    let once = world.observe(west).unwrap().text;
    let line = "The brook to the north and west is quiet.";
    assert_eq!(once.matches(line).count(), 1, "{once}");
}

#[test]
fn a_hardened_flint_spear_takes_a_fish_from_the_brook() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 4, 0).unwrap();
    let west = world.spawn_at("West", "w", 3, 3).unwrap();
    let far = world.spawn_at("Far", "f", 6, 4).unwrap();
    world.step(&[cmd("flint", near, Attempt::Gather(Material::Flint))]);
    world.step(&[cmd("knap", near, Attempt::Knap)]);
    for (id, dir) in [("e1", Dir::East), ("e2", Dir::East), ("s", Dir::South)] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    let light = world.step(&[cmd("light", near, Attempt::Light)]);
    assert!(light.results[0].success, "{}", light.results[0].message);
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    let haft = world.step(&[cmd("haft", near, Attempt::Haft)]);
    assert!(haft.results[0].success, "{}", haft.results[0].message);
    for _ in 0..4 {
        world.step(&[]);
    }
    assert!(
        world
            .observe(near)
            .unwrap()
            .text
            .contains("fire-hardened flint tip"),
        "{}",
        world.observe(near).unwrap().text
    );
    world.step(&[cmd("spare", near, Attempt::Gather(Material::Branch))]);
    world.step(&[cmd("tinder", near, Attempt::Gather(Material::Bark))]);
    let away = world.step(&[cmd("away", near, Attempt::Fish)]);
    assert_eq!(away.results[0].message, "The brook is not here.");
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("brook is not here")));
    for (id, dir) in [
        ("w1", Dir::West),
        ("w2", Dir::West),
        ("w3", Dir::West),
        ("w4", Dir::West),
        ("bank-s", Dir::South),
    ] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    assert_eq!(world.actor_cell(near), Some((2, 2)));
    world.step(&[cmd("drop-spear", near, Attempt::Drop(Material::Spear))]);
    let bare = world.step(&[cmd("bare", near, Attempt::Fish)]);
    assert_eq!(bare.results[0].message, "You are not carrying a spear.");
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("not carrying a spear")));
    world.step(&[cmd("spear", near, Attempt::Gather(Material::Spear))]);
    let yours = "You spear a fish from the brook.";
    let seen = "Near spears a fish from the brook.";
    let cast = world.step(&[cmd("fish", near, Attempt::Fish)]);
    assert_eq!(cast.results[0].message, yours);
    assert_eq!(world.material_tally(Material::Fish), 1);
    assert_eq!(world.material_tally(Material::Spear), 1);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == yours));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == seen));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("fish")));
    let examined = world.examine(near, ExamineTarget::Fish);
    assert!(
        examined.contains(
            "A raw fish is carried by you, in the open. It must be cooked before it feeds you."
        ),
        "{examined}"
    );
    let raw = world.step(&[cmd("raw", near, Attempt::Eat(Material::Fish))]);
    assert_eq!(raw.results[0].message, "Raw fish is not food.");
    world.step(&[cmd("into", near, Attempt::Move(Dir::West))]);
    assert_eq!(world.actor_cell(near), Some((1, 2)));
    world.step(&[]);
    let again = world.step(&[cmd(
        "spear-fish",
        near,
        parse_attempt("spear fish").unwrap(),
    )]);
    assert_eq!(again.results[0].message, yours);
    assert_eq!(world.material_tally(Material::Fish), 2);
    world.step(&[cmd("out", near, Attempt::Move(Dir::East))]);
    for (id, dir) in [
        ("back-e1", Dir::East),
        ("back-e2", Dir::East),
        ("back-e3", Dir::East),
        ("back-e4", Dir::East),
        ("back-n", Dir::North),
    ] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    let relight = world.step(&[cmd("relight", near, Attempt::Light)]);
    assert!(relight.results[0].success, "{}", relight.results[0].message);
    let cook = world.step(&[cmd("cook", near, Attempt::Cook)]);
    assert_eq!(cook.results[0].message, "You cook the fish over the fire.");
    let cooked = world.examine(near, ExamineTarget::Fish);
    assert!(
        cooked.contains(
            "A cooked fish is carried by you, under the rock overhang. It is good to eat."
        ),
        "{cooked}"
    );
    let eat = world.step(&[cmd("eat", near, Attempt::Eat(Material::Fish))]);
    assert!(
        eat.results[0]
            .message
            .starts_with("You eat the cooked fish."),
        "{}",
        eat.results[0].message
    );
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains("a spear with a fire-hardened flint tip"),);
}

#[test]
fn a_soaked_branch_dries_by_the_fire_and_then_hafts() {
    let mut world = world_at(4);
    let near = world.spawn_at("Near", "n", 4, 0).unwrap();
    let west = world.spawn_at("West", "w", 4, 1).unwrap();
    let far = world.spawn_at("Far", "f", 0, 4).unwrap();
    world.step(&[cmd("flint", near, Attempt::Gather(Material::Flint))]);
    world.step(&[cmd("knap", near, Attempt::Knap)]);
    for (id, dir) in [("e1", Dir::East), ("e2", Dir::East), ("s", Dir::South)] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    let light = world.step(&[cmd("light", near, Attempt::Light)]);
    assert!(light.results[0].success, "{}", light.results[0].message);
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    let already = world.step(&[cmd("early", near, Attempt::DryBranch)]);
    assert_eq!(already.results[0].message, "The branch is already dry.");
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("branch by the fire")));
    world.step(&[cmd("out", near, Attempt::Move(Dir::South))]);
    let no_fire = world.step(&[cmd("no-fire", near, Attempt::DryBranch)]);
    assert_eq!(no_fire.results[0].message, "There is no fire burning here.");
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("no fire")));
    let mut waited = 0;
    while !world.observe(near).unwrap().text.contains("soaked branch") && waited < 16 {
        world.step(&[]);
        waited += 1;
    }
    let soaked = world.observe(near).unwrap().text;
    assert!(soaked.contains("a soaked branch"), "{soaked}");
    let wet_haft = world.step(&[cmd("wet-haft", near, Attempt::Haft)]);
    assert_eq!(
        wet_haft.results[0].message,
        "The branch is too damp to haft."
    );
    world.step(&[cmd("back", near, Attempt::Move(Dir::North))]);
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    let mut cooled = 0;
    while world.observe(near).unwrap().text.contains("fire burns") && cooled < 20 {
        world.step(&[]);
        cooled += 1;
    }
    world.step(&[cmd("bark-2", near, Attempt::Gather(Material::Bark))]);
    let relight = world.step(&[cmd("relight", near, Attempt::Light)]);
    assert!(relight.results[0].success, "{}", relight.results[0].message);
    let drying = world.step(&[cmd("dry", near, Attempt::DryBranch)]);
    let yours = "You hold the soaked branch by the fire. It is not dry yet.";
    let seen = "Near holds the soaked branch by the fire. It is not dry yet.";
    assert_eq!(drying.results[0].message, yours);
    let looked = world.observe(near).unwrap().text;
    assert!(
        looked.contains("The soaked branch in your arms is drying by the fire. It is not dry yet."),
        "{looked}"
    );
    let examined = world.examine(near, ExamineTarget::Wood);
    assert!(
        examined.contains(
            "A soaked branch is carried by you, under the rock overhang. It is drying by the fire. It is not dry yet."
        ),
        "{examined}"
    );
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == yours));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == seen));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("branch by the fire")));
    for _ in 0..4 {
        world.step(&[]);
    }
    let done = "You dry the branch at the fire.";
    let done_seen = "Near dries the branch at the fire.";
    assert!(
        world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == done),
        "{:?}",
        world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .map(|event| event.text.clone())
            .collect::<Vec<_>>()
    );
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == done_seen));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("dries the branch")));
    let after = world.examine(near, ExamineTarget::Wood);
    assert!(
        after.contains("A dry branch is carried by you, under the rock overhang."),
        "{after}"
    );
    assert!(!after.contains("soaked"), "{after}");
    let arms = arms_sentence(&world.observe(near).unwrap().text);
    assert!(arms.contains("a dry branch"), "{arms}");
    let haft = world.step(&[cmd("haft", near, Attempt::Haft)]);
    assert_eq!(
        haft.results[0].message,
        "You haft the flint flake and a dry branch into a spear."
    );
}

#[test]
fn hide_strips_bind_the_flint_tip_when_the_spear_is_hafted() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 5, 2).unwrap();
    let west = world.spawn_at("West", "w", 4, 1).unwrap();
    let far = world.spawn_at("Far", "f", 0, 4).unwrap();
    world.step(&[cmd("stash", near, Attempt::Gather(Material::Branch))]);
    world.step(&[cmd("bind-branch", near, Attempt::Gather(Material::Branch))]);
    world.step(&[cmd("stash-e", near, Attempt::Move(Dir::East))]);
    world.step(&[cmd("stash-n", near, Attempt::Move(Dir::North))]);
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    world.step(&[cmd("drop-stash", near, Attempt::Drop(Material::Branch))]);
    for (id, dir) in [
        ("w1", Dir::West),
        ("w2", Dir::West),
        ("w3", Dir::West),
        ("w4", Dir::West),
        ("n", Dir::North),
    ] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    assert_eq!(world.actor_cell(near), Some((2, 0)));
    world.step(&[cmd("stone", near, Attempt::Gather(Material::Stone))]);
    for (id, dir) in [
        ("e1", Dir::East),
        ("e2", Dir::East),
        ("e3", Dir::East),
        ("e4", Dir::East),
        ("s", Dir::South),
    ] {
        let step = world.step(&[cmd(id, near, Attempt::Move(dir))]);
        assert!(step.results[0].success, "{}", step.results[0].message);
    }
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    let bound_stone = world.step(&[cmd("bind", near, Attempt::BindSpear)]);
    assert!(
        bound_stone.results[0].success,
        "{}",
        bound_stone.results[0].message
    );
    for (id, dir) in [("ore-n", Dir::North), ("ore-w", Dir::West)] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    assert_eq!(world.actor_cell(near), Some((5, 0)));
    world.step(&[cmd("ore", near, Attempt::Gather(Material::IronOre))]);
    world.step(&[cmd("back-e", near, Attempt::Move(Dir::East))]);
    world.step(&[cmd("back-s", near, Attempt::Move(Dir::South))]);
    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    let light = world.step(&[cmd("light", near, Attempt::Light)]);
    assert!(light.results[0].success, "{}", light.results[0].message);
    let feed = world.step(&[cmd("feed", near, Attempt::Feed)]);
    assert!(feed.results[0].success, "{}", feed.results[0].message);
    relight_charcoal(&mut world, near);
    let smelt = world.step(&[cmd("smelt", near, Attempt::SmeltIron)]);
    assert!(smelt.results[0].success, "{}", smelt.results[0].message);
    for _ in 0..4 {
        world.step(&[]);
    }
    assert!(
        world
            .observe(near)
            .unwrap()
            .text
            .contains("a hardened point"),
        "{}",
        world.observe(near).unwrap().text
    );
    world.step(&[cmd("bloom", near, Attempt::Gather(Material::Iron))]);
    let knife = world.step(&[cmd("hammer", near, Attempt::HammerIron)]);
    assert!(knife.results[0].success, "{}", knife.results[0].message);
    let mut waited = 0;
    while world.deer_cell() != Some((6, 1)) && waited < 40 {
        world.step(&[]);
        waited += 1;
    }
    assert_eq!(world.deer_cell(), Some((6, 1)));
    let hunt = world.step(&[cmd("hunt", near, Attempt::Hunt)]);
    assert!(hunt.results[0].success, "{}", hunt.results[0].message);
    let butcher = world.step(&[cmd("butcher", near, Attempt::Butcher)]);
    assert!(butcher.results[0].success, "{}", butcher.results[0].message);
    world.step(&[cmd("hide", near, Attempt::Gather(Material::Hide))]);
    let cut = world.step(&[cmd("cut", near, Attempt::CutHide)]);
    assert_eq!(
        cut.results[0].message,
        "You cut the deer hide into strips with the iron knife."
    );
    assert_eq!(world.material_tally(Material::Cord), 1);
    let stashed = world.step(&[cmd("stash-back", near, Attempt::Gather(Material::Branch))]);
    assert!(stashed.results[0].success, "{}", stashed.results[0].message);
    world.step(&[cmd("drop-knife", near, Attempt::Drop(Material::Knife))]);
    for (id, dir) in [
        ("to-flint-n", Dir::North),
        ("to-flint-w1", Dir::West),
        ("to-flint-w2", Dir::West),
    ] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    assert_eq!(world.actor_cell(near), Some((4, 0)));
    world.step(&[cmd("flint", near, Attempt::Gather(Material::Flint))]);
    world.step(&[cmd("knap", near, Attempt::Knap)]);
    let yours =
        "You haft the flint flake and a dry branch into a spear. The tip is bound with hide.";
    let seen =
        "Near hafts the flint flake and a dry branch into a spear. The tip is bound with hide.";
    let haft = world.step(&[cmd("haft", near, Attempt::Haft)]);
    assert_eq!(haft.results[0].message, yours);
    assert_eq!(world.material_tally(Material::Cord), 0);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != yours));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == seen));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("bound with hide")));
    let text = world.observe(near).unwrap().text;
    assert!(
        text.contains("a spear with a flint tip bound with hide, not yet fire-hardened"),
        "{text}"
    );
    let examined = world.examine(near, ExamineTarget::Spear);
    assert!(
        examined.contains(
            "A spear with a flint tip bound with hide, not yet fire-hardened, is carried by you"
        ),
        "{examined}"
    );
}

#[test]
fn a_flint_flake_and_a_dry_branch_haft_into_a_spear() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 4, 0).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    world.step(&[cmd("flint", near, Attempt::Gather(Material::Flint))]);
    world.step(&[cmd("knap", near, Attempt::Knap)]);
    let early = world.step(&[cmd("early", near, Attempt::Haft)]);
    assert_eq!(early.results[0].message, "You are not carrying a branch.");
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("haft")));
    for (id, dir) in [
        ("east", Dir::East),
        ("east-2", Dir::East),
        ("south", Dir::South),
    ] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    let before = world.observe(near).unwrap().text;
    assert_eq!(
        arms_sentence(&before),
        "You have a sharp flint flake that is sharp enough to cut or haft later, a dry branch."
    );
    let report = world.step(&[cmd("haft", near, Attempt::Haft)]);
    let sentence = "You haft the flint flake and a dry branch into a spear.";
    let witnessed = "Near hafts the flint flake and a dry branch into a spear.";
    assert_eq!(report.results[0].message, sentence);
    let yours: Vec<_> = world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("haft"))
        .collect();
    assert!(yours.is_empty(), "{yours:?}");
    let seen: Vec<_> = world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("haft"))
        .collect();
    assert_eq!(seen, vec![witnessed.to_string()]);
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("haft")));
    let after = world.observe(near).unwrap().text;
    let arms = arms_sentence(&after);
    assert_eq!(
        arms,
        "You have a spear with a flint tip, not yet fire-hardened."
    );
    assert!(!arms.contains("flake"), "{arms}");
    assert!(!arms.contains("branch"), "{arms}");
    assert!(
        after.contains("a spear with a flint tip, not yet fire-hardened"),
        "{after}"
    );
    let examined = world.examine(near, ExamineTarget::Spear);
    assert!(
        examined.contains("A spear with a flint tip, not yet fire-hardened,"),
        "{examined}"
    );
    let from_west = world.examine(west, ExamineTarget::Spear);
    assert!(
        from_west.contains("A spear with a flint tip, not yet fire-hardened,"),
        "{from_west}"
    );
}

#[test]
fn a_flint_tipped_spear_hardens_in_the_rock_fire() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 4, 0).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    world.step(&[cmd("flint", near, Attempt::Gather(Material::Flint))]);
    world.step(&[cmd("knap", near, Attempt::Knap)]);
    for (id, dir) in [
        ("east", Dir::East),
        ("east-2", Dir::East),
        ("south", Dir::South),
    ] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    world.step(&[cmd("haft", near, Attempt::Haft)]);
    let lit = world.step(&[cmd("light", near, Attempt::Light)]);
    assert!(lit.results[0].success, "{}", lit.results[0].message);
    let started = world.sim_time();
    let waiting = world.observe(near).unwrap().text;
    assert_eq!(
        arms_sentence(&waiting),
        "You have a spear with a flint tip, not yet fire-hardened."
    );
    assert!(
        waiting.contains("The flint tip of your spear is hardening in the fire."),
        "{waiting}"
    );
    assert!(
        !waiting.contains("The point of your spear is hardening"),
        "{waiting}"
    );
    let early = world.step(&[cmd("hunt", near, Attempt::Hunt)]);
    let waiting_tip = "The flint tip is still hardening in the fire. A little longer.";
    assert_eq!(early.results[0].message, waiting_tip);
    assert!(!early.results[0].success);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == waiting_tip));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != waiting_tip));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != waiting_tip));
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains("The flint tip of your spear is hardening in the fire."));
    for _ in 0..2 {
        world.step(&[]);
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The flint tip of your spear is hardening in the fire."));
    }
    let late = world.step(&[cmd("late", near, Attempt::Hunt)]);
    let almost = "The flint tip is still hardening in the fire. It is almost ready.";
    assert_eq!(late.results[0].message, almost);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == almost));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != almost));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != almost));
    assert_eq!(world.sim_time(), started + 4);
    let sentence = "You harden the flint tip of the spear in the fire.";
    let witnessed = "Near hardens the flint tip of the spear in the fire.";
    let yours: Vec<_> = world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("harden the flint"))
        .collect();
    assert_eq!(yours, vec![sentence.to_string()]);
    let seen: Vec<_> = world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("hardens the flint"))
        .collect();
    assert_eq!(seen, vec![witnessed.to_string()]);
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("flint tip")));
    let after = world.observe(near).unwrap().text;
    assert_eq!(
        arms_sentence(&after),
        "You have a spear with a fire-hardened flint tip."
    );
    assert!(!after.contains("not yet fire-hardened"), "{after}");
    assert!(!after.contains("hardening in the fire"), "{after}");
    let examined = world.examine(near, ExamineTarget::Spear);
    assert_eq!(
        examined,
        "A spear with a fire-hardened flint tip is carried by you, under the rock overhang."
    );
    let from_west = world.examine(west, ExamineTarget::Spear);
    assert!(
        from_west.contains("A spear with a fire-hardened flint tip is carried by Near"),
        "{from_west}"
    );
}

#[test]
fn a_fire_hardened_flint_spear_brings_the_deer_down_on_its_own_cell() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 4, 0).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    world.step(&[cmd("flint", near, Attempt::Gather(Material::Flint))]);
    world.step(&[cmd("knap", near, Attempt::Knap)]);
    for (id, dir) in [
        ("east", Dir::East),
        ("east-2", Dir::East),
        ("south", Dir::South),
    ] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    world.step(&[cmd("haft", near, Attempt::Haft)]);
    let lit = world.step(&[cmd("light", near, Attempt::Light)]);
    assert!(lit.results[0].success, "{}", lit.results[0].message);
    for _ in 0..4 {
        world.step(&[]);
    }
    assert_eq!(
        arms_sentence(&world.observe(near).unwrap().text),
        "You have a spear with a fire-hardened flint tip."
    );
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    let spare = world.step(&[cmd("spare", near, Attempt::Gather(Material::Branch))]);
    assert!(spare.results[0].success, "{}", spare.results[0].message);
    let mut waited = 0;
    while world.deer_cell() != Some((6, 1)) && waited < 40 {
        world.step(&[]);
        waited += 1;
    }
    assert_eq!(
        world.deer_cell(),
        Some((6, 1)),
        "the deer returns under the rock"
    );
    let report = world.step(&[cmd("hunt", near, Attempt::Hunt)]);
    let sentence = "You bring the deer down with the flint spear.";
    let witnessed = "Near brings the deer down with the flint spear.";
    assert_eq!(report.results[0].message, sentence);
    let yours: Vec<_> = world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("bring"))
        .collect();
    assert!(yours.is_empty(), "{yours:?}");
    let seen: Vec<_> = world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("brings"))
        .collect();
    assert_eq!(seen, vec![witnessed.to_string()]);
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("bring")));
    assert!(world.deer_cell().is_none());
    let after = world.observe(near).unwrap().text;
    assert!(after.contains("A deer carcass lies here."), "{after}");
    assert!(
        after.contains("Deer tracks mark the ground here."),
        "{after}"
    );
    assert!(!after.contains("A deer stands here."), "{after}");
    assert!(!world
        .observe(far)
        .unwrap()
        .text
        .contains("Deer tracks mark the ground here."));
    assert_eq!(world.material_tally(Material::Carcass), 1);

    let cut = world.step(&[cmd("butcher", near, Attempt::Butcher)]);
    let sentence =
        "You butcher the carcass into three portions of raw meat, a hide, a bone, and an antler.";
    let witnessed =
        "Near butchers the carcass into three portions of raw meat, a hide, a bone, and an antler.";
    assert_eq!(cut.results[0].message, sentence);
    let yours: Vec<_> = world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("butcher"))
        .collect();
    assert!(yours.is_empty(), "{yours:?}");
    let seen: Vec<_> = world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("butchers"))
        .collect();
    assert_eq!(seen, vec![witnessed.to_string()]);
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("butcher")));
    assert_eq!(world.material_tally(Material::Carcass), 0);
    assert_eq!(world.material_tally(Material::Meat), 3);
    let meat = world.material_items(Material::Meat);
    assert!(meat.iter().all(|item| item.1.is_none()));
    assert!(meat.iter().all(|item| (item.2, item.3) == (6, 1)));
    let ground = world.observe(near).unwrap().text;
    assert!(
        ground.contains("Three portions of raw meat lie here."),
        "{ground}"
    );
    assert!(ground.contains("A deer hide lies here."), "{ground}");
    assert!(ground.contains("Two bones lie here."), "{ground}");
    assert_eq!(world.material_tally(Material::Bone), 2);
    assert_eq!(world.material_tally(Material::Hide), 1);
    assert!(!ground.contains("A deer carcass lies here."), "{ground}");
    assert!(
        arms_sentence(&ground).contains("a spear with a fire-hardened flint tip"),
        "{ground}"
    );
    assert_eq!(
        world.examine(near, ExamineTarget::Meat),
        "Three portions of raw meat lie here, under the rock overhang."
    );
    let lifted = world.step(&[cmd("hide", near, Attempt::Gather(Material::Hide))]);
    assert_eq!(lifted.results[0].message, "You gather a deer hide.");
    let hide_seen: Vec<_> = world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("gathers a deer hide"))
        .collect();
    assert_eq!(hide_seen, vec!["Near gathers a deer hide.".to_string()]);
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("hide")));
    let carried = world.observe(near).unwrap().text;
    assert!(arms_sentence(&carried).contains("a deer hide"), "{carried}");
    assert!(!carried.contains("A deer hide lies here."), "{carried}");
    assert_eq!(world.material_tally(Material::Hide), 1);
    assert_eq!(world.material_tally(Material::Meat), 3);

    let mut bare = world.clone();
    bare.step(&[cmd("south-bare", near, Attempt::Move(Dir::South))]);
    let mut open_ticks = 0;
    while bare.observe(near).unwrap().self_info.cold != "cold" && open_ticks < 40 {
        bare.step(&[]);
        open_ticks += 1;
    }
    let bare_body = bare.observe(near).unwrap().self_info;
    assert_eq!(bare_body.cold, "cold");
    assert_eq!(bare_body.wind, "in the wind");

    let mut covered = world.clone();
    let wore = covered.step(&[cmd("wear", near, Attempt::Wear)]);
    assert_eq!(wore.results[0].message, "You wear the deer hide.");
    let worn_line: Vec<_> = covered
        .observe(near)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("wear the deer hide"))
        .collect();
    assert_eq!(worn_line, vec!["You wear the deer hide.".to_string()]);
    let west_wore: Vec<_> = covered
        .observe(west)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("wears the deer hide"))
        .collect();
    assert_eq!(west_wore, vec!["Near wears the deer hide.".to_string()]);
    assert!(covered
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("wears")));
    covered.step(&[cmd("south-hide", near, Attempt::Move(Dir::South))]);
    for _ in 0..open_ticks {
        covered.step(&[]);
    }
    let covered_body = covered.observe(near).unwrap().self_info;
    assert_ne!(covered_body.cold, "cold");
    assert_eq!(covered_body.wind, "in the wind");
    assert!(covered
        .observe(near)
        .unwrap()
        .text
        .contains("The deer hide is on you."));

    let raw = world.step(&[cmd("raw-meat", near, Attempt::Eat(Material::Meat))]);
    assert!(!raw.results[0].success);
    assert_eq!(raw.results[0].message, "Raw meat is not food.");
    assert_eq!(world.material_tally(Material::Meat), 3);
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains("Three portions of raw meat lie here."));

    let mut cooked = world.step(&[cmd("cook", near, Attempt::Cook)]);
    if cooked.results[0].message == "There is no fire burning here." {
        let relit = world.step(&[cmd("relight", near, Attempt::Light)]);
        assert!(relit.results[0].success, "{}", relit.results[0].message);
        cooked = world.step(&[cmd("cook-meat", near, Attempt::Cook)]);
    }
    let sentence = "You cook the raw meat over the fire.";
    let witnessed = "Near cooks the raw meat over the fire.";
    assert_eq!(cooked.results[0].message, sentence);
    assert!(!cooked.results[0].message.contains("hungry"));
    let yours: Vec<_> = world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("cook the raw meat"))
        .collect();
    assert!(yours.is_empty(), "{yours:?}");
    let seen: Vec<_> = world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("cooks the raw meat"))
        .collect();
    assert_eq!(seen, vec![witnessed.to_string()]);
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("raw meat")));
    let after = world.observe(near).unwrap().text;
    assert!(
        after.contains("A portion of cooked meat lies here."),
        "{after}"
    );
    assert!(
        after.contains("Two portions of raw meat lie here."),
        "{after}"
    );
    assert_eq!(
        world.examine(near, ExamineTarget::Meat),
        "Two portions of raw meat lie here, under the rock overhang.\nA portion of cooked meat lies here, under the rock overhang."
    );
    assert_eq!(world.material_tally(Material::Meat), 3);
    let before = world.observe(near).unwrap().self_info.hunger;
    let eaten = world.step(&[cmd("eat-meat", near, Attempt::Eat(Material::Meat))]);
    let after_word = world.observe(near).unwrap().self_info.hunger;
    let tail = if before != "not hungry" && after_word != before {
        " The hunger eases.".to_string()
    } else {
        format!(" You are {after_word}.")
    };
    let sentence = format!("You eat the cooked meat.{tail}");
    assert_eq!(eaten.results[0].message, sentence);
    let yours: Vec<_> = world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("eat the cooked meat") || text == "The hunger eases.")
        .collect();
    assert!(
        yours.iter().all(|text| text != "You eat the cooked meat."),
        "{yours:?}"
    );
    if tail.contains("eases") {
        assert!(yours.iter().any(|text| text == "The hunger eases."));
        assert_ne!(before, after_word);
    }
    let seen: Vec<_> = world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("eats the cooked meat"))
        .collect();
    assert_eq!(seen, vec!["Near eats the cooked meat.".to_string()]);
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("cooked meat")));
    assert_eq!(world.material_tally(Material::Meat), 2);
    let left = world.observe(near).unwrap().text;
    assert!(
        left.contains("Two portions of raw meat lie here."),
        "{left}"
    );
    assert!(!left.contains("cooked meat"), "{left}");
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains(&format!("You are {after_word}.")));
}

#[test]
fn a_fire_hardened_flint_spear_misses_the_deer_on_the_next_cell() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    for (id, dir) in [("n1", Dir::North), ("n2", Dir::North), ("n3", Dir::North)] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    world.step(&[cmd("flint", near, Attempt::Gather(Material::Flint))]);
    world.step(&[cmd("knap", near, Attempt::Knap)]);
    for (id, dir) in [("e1", Dir::East), ("e2", Dir::East), ("s1", Dir::South)] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    world.step(&[cmd("haft", near, Attempt::Haft)]);
    let lit = world.step(&[cmd("light", near, Attempt::Light)]);
    assert!(lit.results[0].success, "{}", lit.results[0].message);
    for _ in 0..4 {
        world.step(&[]);
    }
    assert_eq!(
        arms_sentence(&world.observe(near).unwrap().text),
        "You have a spear with a fire-hardened flint tip."
    );
    assert_eq!(world.sim_time(), 15);
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    for (id, dir) in [
        ("w1", Dir::West),
        ("w2", Dir::West),
        ("w3", Dir::West),
        ("w4", Dir::West),
        ("s2", Dir::South),
    ] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    assert_eq!(world.sim_time(), 20);
    assert_eq!(world.actor_cell(near), Some((2, 2)));
    for _ in 0..8 {
        world.step(&[]);
    }
    assert_eq!(world.sim_time(), 28);
    assert_eq!(world.deer_cell(), Some((2, 3)));
    let hash = contest_hash(world.seed, world.sim_time(), "hunt", "2,3:deer", near);
    assert_eq!(hash % 3, 1);
    let yours = "You miss the deer with the flint spear. It moves off.";
    let witnessed = "Near misses the deer with the flint spear. It moves off.";
    let report = world.step(&[cmd("hunt", near, Attempt::Hunt)]);
    assert!(!report.results[0].success);
    assert_eq!(report.results[0].message, yours);
    assert_eq!(world.sim_time(), 29);
    assert_eq!(world.deer_cell(), Some((3, 3)));
    assert_eq!(world.material_tally(Material::Carcass), 0);
    assert_eq!(world.material_tally(Material::Spear), 1);
    let near_lines: Vec<_> = world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("miss"))
        .collect();
    assert_eq!(near_lines, vec![yours.to_string()]);
    let far_lines: Vec<_> = world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("miss"))
        .collect();
    assert_eq!(far_lines, vec![witnessed.to_string()]);
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("miss")));
}

#[test]
fn a_fire_hardened_flint_spear_bolts_the_deer_on_the_next_cell() {
    let mut world = World::valley(Tuning::evening(), 1);
    // The bolt is pinned to the deer at (2, 3). A second meal would stall it.
    world.tuning.berry_ticks = 200;
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    for (id, dir) in [("n1", Dir::North), ("n2", Dir::North), ("n3", Dir::North)] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    world.step(&[cmd("flint", near, Attempt::Gather(Material::Flint))]);
    world.step(&[cmd("knap", near, Attempt::Knap)]);
    for (id, dir) in [("e1", Dir::East), ("e2", Dir::East), ("s1", Dir::South)] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    world.step(&[cmd("haft", near, Attempt::Haft)]);
    let lit = world.step(&[cmd("light", near, Attempt::Light)]);
    assert!(lit.results[0].success, "{}", lit.results[0].message);
    for _ in 0..4 {
        world.step(&[]);
    }
    for (id, dir) in [
        ("w1", Dir::West),
        ("w2", Dir::West),
        ("w3", Dir::West),
        ("w4", Dir::West),
        ("s2", Dir::South),
    ] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    assert_eq!(world.sim_time(), 20);
    assert_eq!(world.actor_cell(near), Some((2, 2)));
    while world.sim_time() < 64 {
        world.step(&[]);
    }
    assert_eq!(world.deer_cell(), Some((2, 3)));
    let hash = contest_hash(world.seed, world.sim_time(), "hunt", "2,3:deer", near);
    assert_eq!(hash % 3, 2);
    let yours = "The deer bolts. Your flint spear does not land.";
    let witnessed = "The deer bolts. Near's flint spear does not land.";
    let report = world.step(&[cmd("hunt", near, Attempt::Hunt)]);
    assert!(!report.results[0].success);
    assert_eq!(report.results[0].message, yours);
    assert_eq!(world.sim_time(), 65);
    assert_eq!(world.deer_cell(), Some((3, 3)));
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains("Deer tracks mark the ground to the south."));
    assert!(!world
        .observe(west)
        .unwrap()
        .text
        .contains("Deer tracks mark the ground to the south."));
    assert_eq!(world.material_tally(Material::Carcass), 0);
    assert_eq!(world.material_tally(Material::Spear), 1);
    let near_lines: Vec<_> = world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("bolt"))
        .collect();
    assert_eq!(near_lines, vec![yours.to_string()]);
    let far_lines: Vec<_> = world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("bolt"))
        .collect();
    assert_eq!(far_lines, vec![witnessed.to_string()]);
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("bolt")));
}

#[test]
fn removing_a_worn_hide_returns_it_to_the_arms() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    for (id, dir) in [("n1", Dir::North), ("n2", Dir::North), ("n3", Dir::North)] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    world.step(&[cmd("flint", near, Attempt::Gather(Material::Flint))]);
    world.step(&[cmd("knap", near, Attempt::Knap)]);
    for (id, dir) in [("e1", Dir::East), ("e2", Dir::East), ("s1", Dir::South)] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    world.step(&[cmd("haft", near, Attempt::Haft)]);
    let lit = world.step(&[cmd("light", near, Attempt::Light)]);
    assert!(lit.results[0].success, "{}", lit.results[0].message);
    for _ in 0..4 {
        world.step(&[]);
    }
    assert_eq!(world.sim_time(), 15);
    while world.sim_time() < 22 {
        world.step(&[]);
    }
    assert_eq!(world.deer_cell(), Some((6, 1)));
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    let brought = world.step(&[cmd("down", near, Attempt::Hunt)]);
    assert!(brought.results[0].success, "{}", brought.results[0].message);
    world.step(&[cmd("butcher", near, Attempt::Butcher)]);
    world.step(&[cmd("hide", near, Attempt::Gather(Material::Hide))]);
    let mut bare = world.clone();
    let refused = bare.step(&[cmd("bare", near, Attempt::Remove)]);
    assert!(!refused.results[0].success);
    assert_eq!(
        refused.results[0].message,
        "You are not wearing a deer hide."
    );
    let private: Vec<_> = bare
        .observe(near)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("wearing"))
        .collect();
    assert_eq!(
        private,
        vec!["You are not wearing a deer hide.".to_string()]
    );
    for id in [far, west] {
        assert!(bare
            .observe(id)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("wearing")));
    }
    let wore = world.step(&[cmd("wear", near, Attempt::Wear)]);
    assert_eq!(wore.results[0].message, "You wear the deer hide.");
    let mut sheltered = world.clone();
    let under = sheltered.step(&[cmd("off-rock", near, Attempt::Remove)]);
    assert_eq!(under.results[0].message, "You remove the deer hide.");
    assert!(!under.results[0].message.contains("wind"));
    world.step(&[cmd("south", near, Attempt::Move(Dir::South))]);
    assert_eq!(world.actor_cell(near), Some((6, 2)));
    let mut kept = world.clone();
    let before = world.observe(near).unwrap();
    assert_eq!(before.self_info.cold, "warm");
    assert!(
        before.text.contains("The deer hide is on you."),
        "{}",
        before.text
    );
    assert!(
        arms_sentence(&before.text).contains("a deer hide on you"),
        "{}",
        before.text
    );
    let yours = "You remove the deer hide. The wind reaches you again.";
    let witnessed = "Near removes the deer hide.";
    let report = world.step(&[cmd("remove", near, Attempt::Remove)]);
    assert!(report.results[0].success);
    assert_eq!(report.results[0].message, yours);
    let after = world.observe(near).unwrap();
    assert!(
        !after.text.contains("The deer hide is on you."),
        "{}",
        after.text
    );
    assert!(
        arms_sentence(&after.text).contains("a deer hide"),
        "{}",
        after.text
    );
    assert!(
        !arms_sentence(&after.text).contains("on you"),
        "{}",
        after.text
    );
    assert_eq!(world.material_tally(Material::Hide), 1);
    assert!(world.material_items(Material::Hide)[0].1 == Some(near));
    let near_lines: Vec<_> = after
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("remove"))
        .collect();
    assert_eq!(near_lines, vec![yours.to_string()]);
    let west_lines: Vec<_> = world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("remove"))
        .collect();
    assert_eq!(west_lines, vec![witnessed.to_string()]);
    assert!(west_lines.iter().all(|line| !line.contains("wind")));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("remove")));
    let mut ticks = 0;
    let mut removed_cold = after.self_info.cold.clone();
    let mut kept_cold = kept.observe(near).unwrap().self_info.cold.clone();
    while ticks < 40 && !(removed_cold == "cold" && kept_cold != "cold") {
        world.step(&[]);
        kept.step(&[]);
        ticks += 1;
        removed_cold = world.observe(near).unwrap().self_info.cold.clone();
        kept_cold = kept.observe(near).unwrap().self_info.cold.clone();
    }
    assert!(ticks < 40, "bare skin did not go cold ahead of the hide");
    assert_eq!(removed_cold, "cold");
    assert_eq!(kept_cold, "chilly");
    let again = world.step(&[cmd("again", near, Attempt::Remove)]);
    assert!(!again.results[0].success);
    assert_eq!(again.results[0].message, "You are not wearing a deer hide.");
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("wearing")));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("wearing")));
}

#[test]
fn a_lump_of_ochre_on_the_rock_face_can_be_carried() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let seen = world.spawn_at("Seen", "s", 6, 2).unwrap();
    let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
    let here = world.observe(near).unwrap().text;
    assert!(
        here.contains("A lump of ochre lies on the rock face."),
        "{here}"
    );
    assert!(!here.contains("The slope has no ochre."), "{here}");
    assert_eq!(
        world.examine(near, ExamineTarget::Ochre),
        "A lump of ochre lies here, on the rock face."
    );
    let from_seen = world.observe(seen).unwrap().text;
    assert!(
        from_seen.contains("a lump of ochre on the rock face"),
        "{from_seen}"
    );
    let yours = "You gather a lump of ochre.";
    let witnessed = "Near gathers a lump of ochre.";
    let report = world.step(&[cmd("ochre", near, Attempt::Gather(Material::Ochre))]);
    assert_eq!(report.results[0].message, yours);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != yours));
    assert!(world
        .observe(seen)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == witnessed));
    assert!(world
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("ochre")));
    let after = world.observe(near).unwrap().text;
    assert!(
        !after.contains("A lump of ochre lies on the rock face."),
        "{after}"
    );
    assert!(
        after.contains(
            "A clear stone of quartz lies on the rock face. It is a clear stone. It can spark."
        ),
        "{after}"
    );
    assert!(arms_sentence(&after).contains("a lump of ochre"), "{after}");
    assert_eq!(world.material_tally(Material::Ochre), 2);
    let smeared = world.step(&[cmd("smear", near, Attempt::SmearOchre)]);
    assert_eq!(
        smeared.results[0].message,
        "You smear red ochre on your skin."
    );
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("smear")));
    assert!(world
        .observe(seen)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Near smears red ochre on their skin."));
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains("Your skin is red-ochre-smeared."));
    let miss = world.step(&[cmd("again", near, Attempt::Gather(Material::Ochre))]);
    assert_eq!(miss.results[0].message, "There is no ochre here to gather.");
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != "There is no ochre here to gather."));
    assert!(world
        .observe(seen)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != "There is no ochre here to gather."));
    assert!(world
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != "There is no ochre here to gather."));
    assert_eq!(world.material_tally(Material::Ochre), 1);
}

#[test]
fn a_lump_of_ochre_on_the_slope_can_be_carried() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    let rock_face = "A lump of ochre lies on the rock face to the north and east.";
    assert!(
        world.observe(near).unwrap().text.contains(rock_face),
        "{}",
        world.observe(near).unwrap().text
    );
    assert!(
        !world.observe(far).unwrap().text.contains("ochre"),
        "{}",
        world.observe(far).unwrap().text
    );
    assert!(
        world.observe(west).unwrap().text.contains(rock_face),
        "{}",
        world.observe(west).unwrap().text
    );
    world.step(&[
        cmd("n-near", near, Attempt::Move(Dir::North)),
        cmd("n-far", far, Attempt::Move(Dir::North)),
    ]);
    world.step(&[
        cmd("n2-near", near, Attempt::Move(Dir::North)),
        cmd("n2-far", far, Attempt::Move(Dir::North)),
    ]);
    assert_eq!(world.actor_cell(near), Some((4, 1)));
    assert_eq!(world.actor_cell(far), Some((3, 1)));
    let from_near = world.observe(near).unwrap().text;
    assert!(
        from_near.contains("A lump of ochre lies on the slope to the north and west."),
        "{from_near}"
    );
    assert_eq!(
        world.examine(near, ExamineTarget::Ochre),
        "A lump of ochre lies on the slope to the north and west.\nA lump of ochre lies on the rock face to the east."
    );
    assert_eq!(
        world.examine(far, ExamineTarget::Ochre),
        "A lump of ochre lies on the slope to the north."
    );
    assert_eq!(
        world.examine(west, ExamineTarget::Ochre),
        "A lump of ochre lies on the rock face to the north and east."
    );
    let from_far = world.observe(far).unwrap().text;
    assert!(
        from_far.contains("North: slope, a lump of ochre."),
        "{from_far}"
    );
    assert!(world
        .observe(west)
        .unwrap()
        .text
        .contains("A lump of ochre lies on the rock face to the north and east."));
    world.step(&[cmd("north", near, Attempt::Move(Dir::North))]);
    world.step(&[cmd("west", near, Attempt::Move(Dir::West))]);
    assert_eq!(world.actor_cell(near), Some((3, 0)));
    let here = world.observe(near).unwrap().text;
    assert!(here.contains("A lump of ochre lies here."), "{here}");
    assert!(
        !here.contains("A lump of ochre lies here, on the slope."),
        "{here}"
    );
    assert_eq!(
        world.examine(near, ExamineTarget::Ochre),
        "A lump of ochre lies here, on the slope."
    );
    let yours = "You gather a lump of ochre.";
    let witnessed = "Near gathers a lump of ochre.";
    let report = world.step(&[cmd("ochre", near, Attempt::Gather(Material::Ochre))]);
    assert_eq!(report.results[0].message, yours);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("ochre")));
    let far_lines: Vec<_> = world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("ochre"))
        .collect();
    assert_eq!(far_lines, vec![witnessed.to_string()]);
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("ochre")));
    let after = world.observe(near).unwrap().text;
    assert!(!after.contains("A lump of ochre lies here."), "{after}");
    assert!(arms_sentence(&after).contains("a lump of ochre"), "{after}");
    assert_eq!(world.material_tally(Material::Ochre), 2);
    assert!(world
        .material_items(Material::Ochre)
        .iter()
        .any(|(_, carrier, _, _)| *carrier == Some(near)));
    let yours = "You smear red ochre on your skin.";
    let witnessed = "Near smears red ochre on their skin.";
    let smeared = world.step(&[cmd("smear", near, Attempt::SmearOchre)]);
    assert_eq!(smeared.results[0].message, yours);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("smear")));
    let far_smear: Vec<_> = world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("smear"))
        .collect();
    assert_eq!(far_smear, vec![witnessed.to_string()]);
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("smear") && !event.text.contains("ochre")));
    let marked = world.observe(near).unwrap();
    assert!(marked.text.contains("Your skin is red-ochre-smeared."));
    assert!(!marked.text.contains("bare skin"), "{marked:?}");
    assert!(!marked.text.contains("mud-smeared"));
    assert_eq!(marked.self_info.ochre, "red-ochre-smeared");
    assert_eq!(marked.text.matches("Your skin is ").count(), 2);
    assert!(marked.text.contains("Your arms are empty."), "{marked:?}");
    assert_eq!(world.material_tally(Material::Ochre), 1);
    let seen = world.examine(far, ExamineTarget::Person("Near".into()));
    assert!(seen.contains("Their skin is red-ochre-smeared."), "{seen}");
    assert!(!seen.contains("bare skin"), "{seen}");
    while world.observe(west).unwrap().self_info.sunburn != "burned" {
        assert!(
            !world.is_raining(),
            "the contrast has to show before the rain"
        );
        world.step(&[]);
        assert!(
            world.sim_time() < 16,
            "sun should burn the bare skin before the rain"
        );
    }
    assert_eq!(
        world.observe(near).unwrap().self_info.ochre,
        "red-ochre-smeared"
    );
    assert_ne!(world.observe(near).unwrap().self_info.sunburn, "burned");
    while world.sim_time() < 16 {
        world.step(&[]);
    }
    world.step(&[]);
    assert!(world.is_raining());
    let washed = world.observe(near).unwrap();
    assert!(washed
        .events
        .iter()
        .any(|event| event.text == "The rain washes the red ochre off you."));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "The rain washes the red ochre off Near."));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("ochre")));
    assert!(!washed.text.contains("red-ochre-smeared"));
    assert!(
        washed.text.contains("Your skin is bare skin."),
        "{washed:?}"
    );
    assert_eq!(washed.self_info.ochre, "bare");
}

#[test]
fn a_carried_lump_of_ochre_marks_the_ground() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    world.step(&[
        cmd("n-near", near, Attempt::Move(Dir::North)),
        cmd("n-far", far, Attempt::Move(Dir::North)),
    ]);
    world.step(&[
        cmd("n2-near", near, Attempt::Move(Dir::North)),
        cmd("n2-far", far, Attempt::Move(Dir::North)),
    ]);
    world.step(&[cmd("north", near, Attempt::Move(Dir::North))]);
    world.step(&[cmd("west", near, Attempt::Move(Dir::West))]);
    assert_eq!(world.actor_cell(near), Some((3, 0)));
    world.step(&[cmd("gather", near, Attempt::Gather(Material::Ochre))]);
    let off = world.step(&[cmd("rock", near, Attempt::MarkRock)]);
    assert!(!off.results[0].success);
    assert_eq!(off.results[0].message, "There is no rock here to mark.");
    assert_eq!(world.material_tally(Material::Ochre), 2);
    let yours = "You mark the ground with red ochre.";
    let witnessed = "Near marks the ground with red ochre.";
    let marked = world.step(&[cmd("mark", near, Attempt::Mark)]);
    assert_eq!(marked.results[0].message, yours);
    let near_text = world.observe(near).unwrap();
    assert!(near_text.events.iter().all(|event| event.text != yours));
    assert!(near_text
        .events
        .iter()
        .any(|event| event.text == "There is no rock here to mark."));
    let far_lines: Vec<_> = world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .filter(|text| text.contains("mark"))
        .collect();
    assert_eq!(far_lines, vec![witnessed.to_string()]);
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("mark")));
    let here = world.observe(near).unwrap().text;
    assert!(
        here.contains("A red ochre mark is on the ground here."),
        "{here}"
    );
    assert!(!here.contains("on the ground on the slope"), "{here}");
    let beside = world.observe(far).unwrap().text;
    assert!(
        beside.contains("A red ochre mark is on the ground on the slope to the north."),
        "{beside}"
    );
    assert!(
        beside.contains("North: slope, a red ochre mark, 1 person."),
        "{beside}"
    );
    assert!(!world.observe(west).unwrap().text.contains("ochre mark"));
    assert_eq!(world.material_tally(Material::Ochre), 1);
    let again = world.step(&[cmd("again", near, Attempt::Mark)]);
    assert!(!again.results[0].success);
    assert_eq!(again.results[0].message, "A red mark is already here.");
    world.step(&[cmd("east", near, Attempt::Move(Dir::East))]);
    world.step(&[cmd("south", near, Attempt::Move(Dir::South))]);
    assert_eq!(world.actor_cell(near), Some((4, 1)));
    let afar = world.observe(near).unwrap().text;
    assert!(
        afar.contains("A red ochre mark is on the ground on the slope to the north and west."),
        "{afar}"
    );
}

#[test]
fn rain_washes_an_open_ochre_mark_and_leaves_one_under_the_rock() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    world.step(&[
        cmd("n-near", near, Attempt::Move(Dir::North)),
        cmd("n-far", far, Attempt::Move(Dir::North)),
    ]);
    world.step(&[
        cmd("n2-near", near, Attempt::Move(Dir::North)),
        cmd("n2-far", far, Attempt::Move(Dir::North)),
    ]);
    world.step(&[cmd("north", near, Attempt::Move(Dir::North))]);
    world.step(&[cmd("west", near, Attempt::Move(Dir::West))]);
    world.step(&[cmd("gather", near, Attempt::Gather(Material::Ochre))]);
    world.step(&[cmd("mark", near, Attempt::Mark)]);
    assert_eq!(world.actor_cell(near), Some((3, 0)));
    assert_eq!(world.actor_cell(far), Some((3, 1)));
    while world.sim_time() < 16 {
        world.step(&[]);
    }
    assert!(!world.is_raining());
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains("A red ochre mark is on the ground here."));
    world.step(&[]);
    assert_eq!(world.sim_time(), 17);
    assert!(world.is_raining());
    let washed = "The rain washes the red ochre mark off the ground on the slope.";
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == washed));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == washed));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("ochre mark")));
    assert!(!world.observe(near).unwrap().text.contains("ochre mark"));
    assert!(!world.observe(far).unwrap().text.contains("ochre mark"));

    let mut sheltered = World::valley(Tuning::evening(), 1);
    let near = sheltered.spawn_at("Near", "n", 4, 3).unwrap();
    let far = sheltered.spawn_at("Far", "f", 3, 3).unwrap();
    let west = sheltered.spawn_at("West", "w", 5, 3).unwrap();
    for (id, dir) in [
        ("n", Dir::North),
        ("n2", Dir::North),
        ("n3", Dir::North),
        ("w", Dir::West),
    ] {
        sheltered.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    sheltered.step(&[cmd("gather", near, Attempt::Gather(Material::Ochre))]);
    for (id, dir) in [
        ("e", Dir::East),
        ("e2", Dir::East),
        ("e3", Dir::East),
        ("s", Dir::South),
    ] {
        sheltered.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    assert_eq!(sheltered.actor_cell(near), Some((6, 1)));
    let marked = sheltered.step(&[cmd("mark", near, Attempt::Mark)]);
    assert_eq!(
        marked.results[0].message,
        "You mark the rock with red ochre."
    );
    while sheltered.sim_time() < 16 {
        sheltered.step(&[]);
    }
    sheltered.step(&[]);
    assert!(sheltered.is_raining());
    assert!(sheltered
        .observe(near)
        .unwrap()
        .text
        .contains("A red ochre mark is on the rock here."));
    assert!(sheltered
        .observe(west)
        .unwrap()
        .text
        .contains("A red ochre mark is on the rock to the north and east."));
    for id in [near, far, west] {
        assert!(sheltered
            .observe(id)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("ochre mark")));
    }
    assert!(!sheltered.observe(far).unwrap().text.contains("ochre mark"));
}

#[test]
fn a_lump_of_copper_ore_on_the_brook_bank_can_be_carried() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 3, 3).unwrap();
    let far = world.spawn_at("Far", "f", 3, 2).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    assert_eq!(world.material_tally(Material::CopperOre), 1);

    let from_near = world.observe(near).unwrap().text;
    assert!(
        from_near.contains("A lump of copper ore lies on the brook to the north and west."),
        "{from_near}"
    );
    let from_far = world.observe(far).unwrap().text;
    assert!(
        from_far.contains("A lump of copper ore lies on the brook to the west."),
        "{from_far}"
    );
    assert!(
        !from_far.contains("West: brook, a lump of copper ore."),
        "{from_far}"
    );
    assert!(!world.observe(west).unwrap().text.contains("copper"));
    assert_eq!(
        world.examine(near, ExamineTarget::CopperOre),
        "A lump of copper ore lies on the brook to the north and west."
    );
    assert_eq!(
        world.examine(west, ExamineTarget::CopperOre),
        "You see no copper ore from here."
    );

    world.step(&[cmd("beside", far, Attempt::Move(Dir::West))]);
    assert_eq!(world.actor_cell(far), Some((2, 2)));
    let beside = world.observe(far).unwrap().text;
    assert!(
        beside.contains("West: brook, a lump of copper ore."),
        "{beside}"
    );
    assert!(
        beside.contains("A lump of copper ore lies on the brook to the west."),
        "{beside}"
    );

    world.step(&[cmd("w1", near, Attempt::Move(Dir::West))]);
    world.step(&[cmd("w2", near, Attempt::Move(Dir::West))]);
    world.step(&[cmd("n", near, Attempt::Move(Dir::North))]);
    assert_eq!(world.actor_cell(near), Some((1, 2)));
    let here = world.observe(near).unwrap().text;
    assert!(here.contains("You are on the brook,"), "{here}");
    assert!(here.contains("A lump of copper ore lies here."), "{here}");
    assert_eq!(
        world.examine(near, ExamineTarget::CopperOre),
        "A lump of copper ore lies here, on the brook."
    );

    let yours = "You gather a lump of copper ore.";
    let witnessed = "Near gathers a lump of copper ore.";
    let report = world.step(&[cmd("ore", near, Attempt::Gather(Material::CopperOre))]);
    assert_eq!(report.results[0].message, yours);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != yours));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == witnessed));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("copper")));
    let after = world.observe(near).unwrap().text;
    assert!(
        !after.contains("A lump of copper ore lies here."),
        "{after}"
    );
    assert!(
        arms_sentence(&after).contains("a lump of copper ore"),
        "{after}"
    );
    assert_eq!(world.material_tally(Material::CopperOre), 1);
    assert_eq!(world.material_items(Material::CopperOre)[0].1, Some(near));
}

#[test]
fn carried_copper_ore_smelts_into_a_bead_at_the_fire() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 1, 2).unwrap();
    let far = world.spawn_at("Far", "f", 4, 1).unwrap();
    let west = world.spawn_at("West", "w", 4, 4).unwrap();
    world.step(&[cmd("ore", near, Attempt::Gather(Material::CopperOre))]);
    let cold = world.step(&[cmd("early", near, Attempt::Smelt)]);
    assert_eq!(cold.results[0].message, "There is no fire burning here.");
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("fire burning")));
    for step in 0..5 {
        world.step(&[cmd(&format!("east-{step}"), near, Attempt::Move(Dir::East))]);
    }
    world.step(&[cmd("north", near, Attempt::Move(Dir::North))]);
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    world.step(&[cmd("light", near, Attempt::Light)]);
    let yours = "You set a lump of copper ore by the fire. It leaves your arms.";
    let witnessed = "Near sets a lump of copper ore by the fire. It leaves their arms.";
    let set = world.step(&[cmd("smelt", near, Attempt::Smelt)]);
    assert_eq!(
        set.results[0].message,
        "You set a lump of copper ore by the fire. It leaves your arms. It is not metal yet."
    );
    let after_set = world.sim_time();
    assert!(world.observe(near).unwrap().events.iter().all(|event| {
        event.text != yours && event.text != witnessed && !event.text.contains("not metal yet")
    }));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == witnessed));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("copper") && !event.text.contains("smelt")));
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains("A lump of copper ore is smelting in the fire."));
    assert!(!arms_sentence(&world.observe(near).unwrap().text).contains("copper"));
    for _ in 0..3 {
        world.step(&[]);
        assert_eq!(world.material_tally(Material::Copper), 0);
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("A lump of copper ore is smelting in the fire."));
    }
    world.step(&[]);
    assert_eq!(world.sim_time(), after_set + 4);
    assert_eq!(world.material_tally(Material::CopperOre), 0);
    assert_eq!(world.material_tally(Material::Copper), 1);
    assert_eq!(world.material_items(Material::Copper)[0].1, None);
    let bead = "The copper ore smelts into a bead.";
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == bead));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == bead));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != bead));
    let here = world.observe(near).unwrap().text;
    assert!(
        here.contains("A copper bead lies here. It is metal from the fire."),
        "{here}"
    );
    assert_eq!(
        world.examine(near, ExamineTarget::Copper),
        "A copper bead lies here. It is metal from the fire."
    );
    assert!(!here.contains("is smelting in the fire."), "{here}");
    assert!(!arms_sentence(&here).contains("bead"), "{here}");
    let gathered = "You gather a copper bead.";
    let seen = "Near gathers a copper bead.";
    let took = world.step(&[cmd("bead", near, parse_attempt("gather bead").unwrap())]);
    assert_eq!(took.results[0].message, gathered);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != gathered));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == seen));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("bead")));
    let arms = world.observe(near).unwrap().text;
    assert!(!arms.contains("A copper bead lies here."), "{arms}");
    assert_eq!(
        arms_sentence(&arms),
        "You have a copper bead, metal from the fire."
    );
    assert_eq!(
        world.examine(near, ExamineTarget::Copper),
        "A copper bead is carried by you, under the rock overhang. It is metal from the fire."
    );
    assert_eq!(
        world.examine(far, ExamineTarget::Copper),
        "A copper bead is carried by Near, under the rock overhang. It is metal from the fire."
    );
    assert_eq!(
        world.examine(west, ExamineTarget::Copper),
        "You see no copper bead from here."
    );
    assert_eq!(world.material_items(Material::Copper)[0].1, Some(near));
    world.step(&[cmd("drop", near, Attempt::Drop(Material::Copper))]);
    let again = world.step(&[cmd("copper", near, parse_attempt("gather copper").unwrap())]);
    assert_eq!(again.results[0].message, gathered);
    assert_eq!(world.material_items(Material::Copper)[0].1, Some(near));
    world.step(&[cmd("off", near, Attempt::Move(Dir::South))]);
    let refused = world.step(&[cmd("cold", near, parse_attempt("hammer").unwrap())]);
    assert_eq!(
        refused.results[0].message,
        "You need a fire or a stone to hammer the bead."
    );
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("hammer")));
    world.step(&[cmd("back", near, Attempt::Move(Dir::North))]);
    let yours = "You hammer the copper bead into a copper awl at the fire.";
    let seen = "Near hammers the copper bead into a copper awl at the fire.";
    let hit = world.step(&[cmd("hammer", near, parse_attempt("forge").unwrap())]);
    assert_eq!(hit.results[0].message, yours);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| { event.text != yours && event.text != seen }));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == seen));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("awl")));
    let arms = world.observe(near).unwrap().text;
    assert_eq!(arms_sentence(&arms), "You have a copper awl.");
    assert!(!arms.contains("copper bead"), "{arms}");
    assert_eq!(
        world.examine(near, ExamineTarget::Awl),
        "A copper awl is carried by you, under the rock overhang. It can pierce hide."
    );
    assert_eq!(
        world.examine(far, ExamineTarget::Awl),
        "A copper awl is carried by Near, under the rock overhang. It can pierce hide."
    );
    assert_eq!(
        world.examine(west, ExamineTarget::Awl),
        "You see no copper awl from here."
    );
    assert_eq!(world.material_tally(Material::Copper), 0);
    assert_eq!(world.material_items(Material::Awl)[0].1, Some(near));
    assert!(matches!(parse_attempt("sew").unwrap(), Attempt::Sew));
}

#[test]
fn a_carried_stone_hammers_the_bead_away_from_the_fire() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 1, 2).unwrap();
    let far = world.spawn_at("Far", "f", 4, 1).unwrap();
    let west = world.spawn_at("West", "w", 4, 4).unwrap();
    world.step(&[cmd("ore", near, Attempt::Gather(Material::CopperOre))]);
    for step in 0..5 {
        world.step(&[cmd(&format!("east-{step}"), near, Attempt::Move(Dir::East))]);
    }
    world.step(&[cmd("north", near, Attempt::Move(Dir::North))]);
    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    world.step(&[cmd("light", near, Attempt::Light)]);
    world.step(&[cmd("smelt", near, Attempt::Smelt)]);
    for _ in 0..4 {
        world.step(&[]);
    }
    world.step(&[cmd("bead", near, parse_attempt("gather bead").unwrap())]);
    world.step(&[cmd("south", near, Attempt::Move(Dir::South))]);
    for step in 0..4 {
        world.step(&[cmd(&format!("west-{step}"), near, Attempt::Move(Dir::West))]);
    }
    world.step(&[cmd("up", near, Attempt::Move(Dir::North))]);
    world.step(&[cmd("stone-cell", near, Attempt::Move(Dir::North))]);
    assert_eq!(world.actor_cell(near), Some((2, 0)));
    let bare = world.step(&[cmd("bare", near, Attempt::Hammer)]);
    assert_eq!(
        bare.results[0].message,
        "You need a fire or a stone to hammer the bead."
    );
    world.step(&[cmd("stone", near, Attempt::Gather(Material::Stone))]);
    let yours = "You hammer the copper bead into a copper awl with a stone.";
    let seen = "Near hammers the copper bead into a copper awl with a stone.";
    let hit = world.step(&[cmd("hammer", near, Attempt::Hammer)]);
    assert_eq!(hit.results[0].message, yours);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| { event.text != yours && event.text != seen }));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == seen));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("awl")));
    let arms = world.observe(near).unwrap().text;
    assert_eq!(arms_sentence(&arms), "You have a dry stone, a copper awl.");
    assert_eq!(
        world.examine(near, ExamineTarget::Awl),
        "A copper awl is carried by you, in the rain. It can pierce hide."
    );
    assert_eq!(
        world.examine(far, ExamineTarget::Awl),
        "A copper awl is carried by Near, in the rain. It can pierce hide."
    );
    assert_eq!(world.material_tally(Material::Copper), 0);
    assert_eq!(world.material_tally(Material::Stone), 1);
    assert_eq!(world.material_items(Material::Awl)[0].1, Some(near));
}

fn valley_with_an_awl_and_a_hide() -> (World, u32, u32, u32) {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let far = world.spawn_at("Far", "f", 5, 3).unwrap();
    let west = world.spawn_at("West", "w", 4, 4).unwrap();
    for (id, dir) in [("n1", Dir::North), ("n2", Dir::North), ("n3", Dir::North)] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    world.step(&[cmd("flint", near, Attempt::Gather(Material::Flint))]);
    world.step(&[cmd("knap", near, Attempt::Knap)]);
    for (id, dir) in [("e1", Dir::East), ("e2", Dir::East), ("s1", Dir::South)] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    world.step(&[cmd("haft", near, Attempt::Haft)]);
    for (step, dir) in (0..5).map(|step| (step, Dir::West)) {
        world.step(&[cmd(&format!("west-{step}"), near, Attempt::Move(dir))]);
    }
    world.step(&[cmd("to-ore", near, Attempt::Move(Dir::South))]);
    world.step(&[cmd("ore", near, Attempt::Gather(Material::CopperOre))]);
    world.step(&[cmd("from-ore", near, Attempt::Move(Dir::North))]);
    for step in 0..5 {
        world.step(&[cmd(&format!("back-{step}"), near, Attempt::Move(Dir::East))]);
    }
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    let lit = world.step(&[cmd("light", near, Attempt::Light)]);
    assert!(lit.results[0].success, "{}", lit.results[0].message);
    let smelt = world.step(&[cmd("smelt", near, Attempt::Smelt)]);
    assert!(smelt.results[0].success, "{}", smelt.results[0].message);
    for _ in 0..4 {
        world.step(&[]);
    }
    world.step(&[cmd("bead", near, parse_attempt("gather bead").unwrap())]);
    let hammer = world.step(&[cmd("hammer", near, Attempt::Hammer)]);
    assert!(hammer.results[0].success, "{}", hammer.results[0].message);
    let mut waited = 0;
    while world.deer_cell() != Some((6, 1)) && waited < 40 {
        world.step(&[]);
        waited += 1;
    }
    assert_eq!(
        world.deer_cell(),
        Some((6, 1)),
        "the deer returns under the rock"
    );
    let hunt = world.step(&[cmd("hunt", near, Attempt::Hunt)]);
    assert!(hunt.results[0].success, "{}", hunt.results[0].message);
    let cut = world.step(&[cmd("butcher", near, Attempt::Butcher)]);
    assert!(cut.results[0].success, "{}", cut.results[0].message);
    let hide = world.step(&[cmd("hide", near, Attempt::Gather(Material::Hide))]);
    assert!(hide.results[0].success, "{}", hide.results[0].message);
    (world, near, far, west)
}

#[test]
fn a_copper_awl_pierces_a_carried_deer_hide() {
    let (mut world, near, far, west) = valley_with_an_awl_and_a_hide();
    world.step(&[cmd("drop-awl", near, Attempt::Drop(Material::Awl))]);
    let bare = world.step(&[cmd("no-awl", near, parse_attempt("pierce hide").unwrap())]);
    assert_eq!(
        bare.results[0].message,
        "You are not carrying a copper awl."
    );
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("pierce")));
    world.step(&[cmd("awl", near, parse_attempt("gather awl").unwrap())]);
    let yours = "You pierce the deer hide with the copper awl. It is ready to sew.";
    let seen = "Near pierces the deer hide with the copper awl. It is ready to sew.";
    let hit = world.step(&[cmd("pierce", near, Attempt::Pierce)]);
    assert_eq!(hit.results[0].message, yours);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != yours));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == seen));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("pierce")));
    let arms = arms_sentence(&world.observe(near).unwrap().text);
    assert!(
        arms.contains("a pierced deer hide (ready to sew)"),
        "{arms}"
    );
    assert!(arms.contains("a copper awl"), "{arms}");
    assert_eq!(
        world.examine(near, ExamineTarget::Hide),
        "A pierced deer hide is carried by you, under the rock overhang. It is ready to sew."
    );
    assert_eq!(
        world.examine(far, ExamineTarget::Hide),
        "A pierced deer hide is carried by Near, under the rock overhang. It is ready to sew."
    );
    assert_eq!(
        world.examine(west, ExamineTarget::Hide),
        "You see no hide from here."
    );
    let again = world.step(&[cmd("again", near, Attempt::Pierce)]);
    assert_eq!(
        again.results[0].message,
        "The deer hide is already pierced."
    );
    assert_eq!(world.material_items(Material::Awl)[0].1, Some(near));
    assert_eq!(world.material_items(Material::Hide)[0].1, Some(near));
    assert!(matches!(parse_attempt("sew").unwrap(), Attempt::Sew));
}

#[test]
fn a_copper_awl_pierces_a_worn_deer_hide() {
    let (mut world, near, far, _west) = valley_with_an_awl_and_a_hide();
    let wore = world.step(&[cmd("wear", near, Attempt::Wear)]);
    assert_eq!(wore.results[0].message, "You wear the deer hide.");
    let yours = "You pierce the deer hide with the copper awl. It is ready to sew.";
    let seen = "Near pierces the deer hide with the copper awl. It is ready to sew.";
    let hit = world.step(&[cmd("pierce", near, Attempt::Pierce)]);
    assert_eq!(hit.results[0].message, yours);
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == seen));
    let text = world.observe(near).unwrap().text;
    assert!(
        text.contains("The pierced deer hide is on you. It is ready to sew."),
        "{text}"
    );
    let arms = arms_sentence(&text);
    assert!(
        arms.contains("a pierced deer hide on you (ready to sew)"),
        "{arms}"
    );
    assert!(arms.contains("a copper awl"), "{arms}");
    assert_eq!(
        world.examine(near, ExamineTarget::Hide),
        "A pierced deer hide is on you, under the rock overhang. It is ready to sew."
    );
    assert_eq!(world.material_items(Material::Awl)[0].1, Some(near));
    assert_eq!(world.material_items(Material::Hide)[0].1, Some(near));
}

#[test]
fn a_pierced_deer_hide_sews_into_a_cloak() {
    let (mut world, near, far, west) = valley_with_an_awl_and_a_hide();
    let early = world.step(&[cmd("early", near, parse_attempt("sew hide").unwrap())]);
    assert_eq!(early.results[0].message, "The deer hide is not scraped.");
    world.mark_carried_hides_scraped(near);
    let unpierced = world.step(&[cmd("unpierced", near, Attempt::Sew)]);
    assert_eq!(
        unpierced.results[0].message,
        "The deer hide is not pierced. Pierce it with the copper awl."
    );
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("sew")));
    world.step(&[cmd("pierce", near, Attempt::Pierce)]);
    let pierced = arms_sentence(&world.observe(near).unwrap().text);
    assert!(
        pierced.contains("a pierced deer hide (ready to sew)"),
        "{pierced}"
    );
    assert!(!pierced.contains(", ready to sew"), "{pierced}");
    let yours = "You sew the pierced deer hide into a cloak.";
    let seen = "Near sews the pierced deer hide into a cloak.";
    let sewn = world.step(&[cmd("sew", near, Attempt::Sew)]);
    assert_eq!(sewn.results[0].message, yours);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == yours));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == seen));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("sew")));
    let arms = arms_sentence(&world.observe(near).unwrap().text);
    assert!(arms.contains("a sewn deer-hide cloak"), "{arms}");
    assert!(arms.contains("a copper awl"), "{arms}");
    assert!(!arms.contains("pierced"), "{arms}");
    assert!(!arms.contains("ready to sew"), "{arms}");
    assert_eq!(
        world.examine(near, ExamineTarget::Hide),
        "A sewn deer-hide cloak is carried by you, under the rock overhang. It can be worn against the cold."
    );
    assert_eq!(
        world.examine(far, ExamineTarget::Hide),
        "A sewn deer-hide cloak is carried by Near, under the rock overhang. It can be worn against the cold."
    );
    let again = world.step(&[cmd("again", near, Attempt::Sew)]);
    assert_eq!(
        again.results[0].message,
        "The deer hide is already a cloak."
    );
    let wore = world.step(&[cmd("wear", near, Attempt::Wear)]);
    assert_eq!(
        wore.results[0].message,
        "You wear the sewn deer-hide cloak."
    );
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "You wear the sewn deer-hide cloak."));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Near wears the sewn deer-hide cloak."));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("cloak")));
    let text = world.observe(near).unwrap().text;
    assert!(
        text.contains("The sewn deer-hide cloak is on you."),
        "{text}"
    );
    assert!(text.contains("You wear a sewn deer-hide cloak."), "{text}");
    let worn_arms = arms_sentence(&text);
    assert!(!worn_arms.contains("cloak"), "{worn_arms}");
    assert_eq!(
        world.examine(near, ExamineTarget::Hide),
        "A sewn deer-hide cloak is on you, under the rock overhang. It keeps off the cold."
    );
    assert_eq!(world.material_items(Material::Awl)[0].1, Some(near));
    assert_eq!(world.material_items(Material::Hide)[0].1, Some(near));
}

fn cold_rank(word: &str) -> u8 {
    match word {
        "warm" => 0,
        "chilly" => 1,
        "cold" => 2,
        other => panic!("unknown cold word {other}"),
    }
}

#[test]
fn a_sewn_cloak_in_the_open_wind_stays_warmer_than_bare_skin() {
    let (mut world, near, _far, west) = valley_with_an_awl_and_a_hide();
    world.mark_carried_hides_scraped(near);
    world.step(&[cmd("pierce", near, Attempt::Pierce)]);
    world.step(&[cmd("sew", near, Attempt::Sew)]);
    let wore = world.step(&[cmd("wear", near, Attempt::Wear)]);
    assert_eq!(
        wore.results[0].message,
        "You wear the sewn deer-hide cloak."
    );
    for (id, dir) in [
        ("w1", Dir::East),
        ("w2", Dir::East),
        ("w3", Dir::North),
        ("w4", Dir::North),
        ("w5", Dir::North),
    ] {
        world.step(&[cmd(id, west, Attempt::Move(dir))]);
    }
    assert_eq!(world.actor_cell(west), Some((6, 1)));
    let mut waited = 0;
    while waited < 40 {
        let near_look = world.observe(near).unwrap();
        let west_look = world.observe(west).unwrap();
        let settled = near_look.self_info.cold == "warm"
            && west_look.self_info.cold == "warm"
            && !near_look.text.contains("A small fire burns here")
            && !near_look
                .text
                .contains("The warmth of the fire is still on you.")
            && !west_look
                .text
                .contains("The warmth of the fire is still on you.");
        if settled {
            break;
        }
        world.step(&[]);
        waited += 1;
    }
    assert!(
        waited < 40,
        "the rock should take the cold and the leftover fire off"
    );
    world.step(&[
        cmd("out-near", near, Attempt::Move(Dir::South)),
        cmd("out-west", west, Attempt::Move(Dir::South)),
    ]);
    assert_eq!(world.actor_cell(near), Some((6, 2)));
    assert_eq!(world.actor_cell(west), Some((6, 2)));
    let mut cloak = String::new();
    let mut bare = String::new();
    for _ in 0..16 {
        let cloak_now = world.observe(near).unwrap().self_info.cold.clone();
        let bare_now = world.observe(west).unwrap().self_info.cold.clone();
        assert!(
            world
                .observe(near)
                .unwrap()
                .text
                .contains("You are in the wind."),
            "cloak"
        );
        assert!(
            world
                .observe(west)
                .unwrap()
                .text
                .contains("You are in the wind."),
            "bare"
        );
        if cold_rank(&cloak_now) < cold_rank(&bare_now) {
            cloak = cloak_now;
            bare = bare_now;
            break;
        }
        world.step(&[]);
    }
    assert_ne!(cloak, "", "the cloak should stay warmer than bare skin");
    assert!(cold_rank(&cloak) < cold_rank(&bare), "{cloak} vs {bare}");
    let cloak_line = world.observe(near).unwrap().text;
    let bare_line = world.observe(west).unwrap().text;
    assert!(cloak_line.contains(&format!("You are {cloak}.")));
    assert!(bare_line.contains(&format!("You are {bare}.")));
    assert!(cloak_line.contains("The sewn deer-hide cloak is on you."));
    assert!(!bare_line.contains("cloak"));
}

fn wait_until_tiring(world: &mut World, actor: u32) {
    let mut ticks = 0;
    while world.observe(actor).unwrap().self_info.energy != "tiring" {
        world.step(&[]);
        ticks += 1;
        assert!(ticks < 80, "tiredness should arrive");
    }
}

#[test]
fn an_iron_knife_cuts_a_whole_hide_into_strips_and_stays() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 4, 0).unwrap();
    let far = world.spawn_at("Far", "f", 4, 1).unwrap();
    let west = world.spawn_at("West", "w", 0, 4).unwrap();
    assert!(matches!(
        parse_attempt("cut strips").unwrap(),
        Attempt::CutHide
    ));
    assert!(matches!(
        parse_attempt("set snare").unwrap(),
        Attempt::SetSnare
    ));
    assert!(matches!(
        parse_attempt("set trap").unwrap(),
        Attempt::SetSnare
    ));
    let no_strips = world.step(&[cmd("no-strips", near, Attempt::SetSnare)]);
    let refused = "You have no hide strips or willow cordage to set a snare.";
    assert_eq!(no_strips.results[0].message, refused);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != refused));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("snare")));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("snare")));
    let denied = world.step(&[cmd("no-knife", far, parse_attempt("cut hide").unwrap())]);
    assert_eq!(
        denied.results[0].message,
        "You need an iron knife to cut the hide."
    );
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("iron knife to cut the hide")));
    for (id, attempt) in [
        ("flint", Attempt::Gather(Material::Flint)),
        ("knap", Attempt::Knap),
        ("east-ore", Attempt::Move(Dir::East)),
        ("ore", Attempt::Gather(Material::IronOre)),
        ("east-rock", Attempt::Move(Dir::East)),
        ("south-rock", Attempt::Move(Dir::South)),
        ("bark", Attempt::Gather(Material::Bark)),
        ("light", Attempt::Light),
        ("branch", Attempt::Gather(Material::Branch)),
        ("haft", Attempt::Haft),
        ("feed", Attempt::Feed),
    ] {
        let step = world.step(&[cmd(id, near, attempt)]);
        assert!(step.results[0].success, "{}", step.results[0].message);
    }
    relight_charcoal(&mut world, near);
    let smelt = world.step(&[cmd("smelt", near, Attempt::SmeltIron)]);
    assert!(smelt.results[0].success, "{}", smelt.results[0].message);
    for _ in 0..4 {
        world.step(&[]);
    }
    let bloom = world.step(&[cmd("bloom", near, Attempt::Gather(Material::Iron))]);
    assert!(bloom.results[0].success, "{}", bloom.results[0].message);
    let knife = world.step(&[cmd("hammer", near, Attempt::HammerIron)]);
    assert!(knife.results[0].success, "{}", knife.results[0].message);
    let bare = world.step(&[cmd("no-hide", near, parse_attempt("cut strips").unwrap())]);
    assert_eq!(bare.results[0].message, "You have no deer hide to cut.");
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("no deer hide")));
    let mut waited = 0;
    while world.deer_cell() != Some((6, 1)) && waited < 40 {
        world.step(&[]);
        waited += 1;
    }
    assert_eq!(world.deer_cell(), Some((6, 1)));
    let hunt = world.step(&[cmd("hunt", near, Attempt::Hunt)]);
    assert!(hunt.results[0].success, "{}", hunt.results[0].message);
    let butcher = world.step(&[cmd("butcher", near, Attempt::Butcher)]);
    assert!(butcher.results[0].success, "{}", butcher.results[0].message);
    let yours = "You cut the deer hide into strips with the iron knife.";
    let seen = "Near cuts the deer hide into strips with the iron knife.";
    let cut = world.step(&[cmd("cut", near, Attempt::CutHide)]);
    assert_eq!(cut.results[0].message, yours);
    assert_eq!(world.material_tally(Material::Hide), 0);
    assert_eq!(world.material_tally(Material::Cord), 1);
    assert_eq!(world.material_tally(Material::Knife), 1);
    assert_eq!(world.material_items(Material::Knife)[0].1, Some(near));
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != yours));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == seen));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("strips")));
    let here = world.observe(near).unwrap().text;
    assert!(
        here.contains("Hide strips lie here. They are cordage."),
        "{here}"
    );
    let examined = world.examine(near, ExamineTarget::Cord);
    assert!(
        examined.contains("Hide strips lie here. They are cordage."),
        "{examined}"
    );
    let gathered = world.step(&[cmd("strips", near, Attempt::Gather(Material::Cord))]);
    assert_eq!(gathered.results[0].message, "You gather hide strips.");
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("gather hide strips")
            && !event.text.contains("gathers hide strips")));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Near gathers hide strips."));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("gathers hide strips")));
    let arms = arms_sentence(&world.observe(near).unwrap().text);
    assert!(arms.contains("hide strips"), "{arms}");
    let carried = world.examine(near, ExamineTarget::Cord);
    assert!(
        carried.contains("Hide strips are carried by you") && carried.contains("They are cordage."),
        "{carried}"
    );
    world.step(&[cmd("off-rock", near, Attempt::Move(Dir::South))]);
    assert_eq!(world.actor_cell(near), Some((6, 2)));
    let yours = "You set a snare of hide strips.";
    let witnessed = "Near sets a snare of hide strips.";
    let set = world.step(&[cmd("snare", near, parse_attempt("set trap").unwrap())]);
    assert_eq!(set.results[0].message, yours);
    assert_eq!(world.material_tally(Material::Cord), 0);
    let here = world.observe(near).unwrap();
    assert!(
        here.text.contains("A hide snare is set here."),
        "{}",
        here.text
    );
    assert!(here
        .events
        .iter()
        .all(|event| event.text != yours && !event.text.contains("Near sets")));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == witnessed));
    let far_look = world.observe(far).unwrap().text;
    assert!(
        far_look.contains("A hide snare is set on the valley floor to the south and east."),
        "{far_look}"
    );
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("snare")));
    let west_look = world.observe(west).unwrap().text;
    assert!(!west_look.contains("hide snare"), "{west_look}");
    assert!(!west_look.contains("snare is set"), "{west_look}");
    assert!(
        west_look.contains("Strip willow south of the brook, then set a snare."),
        "{west_look}"
    );
    let examined = world.examine(near, ExamineTarget::Snare);
    assert!(
        examined.contains("A hide snare is set here") && examined.contains("hide strips"),
        "{examined}"
    );
    let again = world.step(&[cmd("again", near, Attempt::SetSnare)]);
    assert_eq!(again.results[0].message, "A snare is already set here.");
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("already set")));
    let meat_before = world.material_tally(Material::Meat);
    for _ in 0..world.tuning.snare_ticks {
        world.step(&[]);
    }
    let caught = world.observe(near).unwrap();
    assert!(caught
        .events
        .iter()
        .any(|event| event.text == "The snare catches small game."));
    assert!(caught
        .events
        .iter()
        .all(|event| event.text != "The snare catches a hare."));
    assert_eq!(world.hare_cell(), Some((2, 4)));
    assert!(
        caught.text.contains("A portion of raw meat lies here."),
        "{}",
        caught.text
    );
    assert!(!caught.text.contains("hide snare"), "{}", caught.text);
    assert_eq!(world.material_tally(Material::Meat), meat_before + 1);
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "The snare catches small game."));
    let far_meat = world.observe(far).unwrap().text;
    assert!(
        far_meat.contains("A portion of raw meat lies on the valley floor to the south and east."),
        "{far_meat}"
    );
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("hare") && !event.text.contains("snare")));
    assert!(!world.observe(west).unwrap().text.contains("raw meat"));
    world.step(&[cmd("back", near, Attempt::Move(Dir::North))]);
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    let beside = world.observe(near).unwrap().text;
    assert!(
        beside.contains("A portion of raw meat lies on the valley floor to the south."),
        "{beside}"
    );
    assert!(
        beside.contains("South:") && beside.contains("a portion of raw meat"),
        "{beside}"
    );
}

#[test]
fn rest_under_a_lean_to_eases_tiredness_and_the_open_wind_still_refuses() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 5, 2).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let west = world.spawn_at("West", "w", 0, 4).unwrap();
    let refused = world.step(&[cmd("open", far, Attempt::Rest)]);
    assert_eq!(
        refused.results[0].message,
        "The wind will not let you rest."
    );
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != "The wind will not let you rest."));
    let built = world.step(&[cmd("build", near, Attempt::BuildLeanTo)]);
    assert_eq!(
        built.results[0].message,
        "You build a lean-to of three branches."
    );
    wait_until_tiring(&mut world, near);
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains("You are in a little wind."));
    let rested = world.step(&[cmd("rest", near, Attempt::Rest)]);
    assert_eq!(
        rested.results[0].message,
        "You rest under the lean-to. You are rested."
    );
    let here = world.observe(near).unwrap();
    assert_eq!(here.self_info.energy, "rested");
    assert!(here
        .events
        .iter()
        .any(|event| event.text == "You rest under the lean-to."));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Near rests under the lean-to."));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("rest")));
    let rock = {
        let mut sheltered = World::valley(Tuning::evening(), 1);
        let near = sheltered.spawn_at("Near", "n", 6, 1).unwrap();
        wait_until_tiring(&mut sheltered, near);
        sheltered.step(&[cmd("rock", near, Attempt::Rest)])
    };
    assert_eq!(
        rock.results[0].message,
        "You rest under the rock. You are rested."
    );
}

#[test]
fn rest_under_the_rock_eases_tiredness_and_the_open_wind_refuses() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let far = world.spawn_at("Far", "f", 5, 3).unwrap();
    assert!(world
        .observe(near)
        .unwrap()
        .allowed_actions
        .iter()
        .any(|action| action == "rest"));
    wait_until_tiring(&mut world, near);
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains("You are tiring."));
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains("You are out of the wind."));
    let rested = world.step(&[cmd("rest", near, Attempt::Rest)]);
    assert!(rested.results[0].success, "{}", rested.results[0].message);
    assert_eq!(
        rested.results[0].message,
        "You rest under the rock. You are rested."
    );
    let near_look = world.observe(near).unwrap();
    assert_eq!(near_look.self_info.energy, "rested");
    assert!(near_look.text.contains("You are rested."));
    assert!(near_look.text.contains("You are out of the wind."));
    assert!(near_look
        .events
        .iter()
        .any(|event| event.text == "You rest under the rock."));
    let far_look = world.observe(far).unwrap();
    assert!(far_look
        .events
        .iter()
        .any(|event| event.text == "Near rests under the rock."));
    assert!(far_look
        .events
        .iter()
        .all(|event| !event.text.contains("You are rested")));
    world.step(&[cmd("out", near, Attempt::Move(Dir::South))]);
    assert_eq!(world.actor_cell(near), Some((6, 2)));
    wait_until_tiring(&mut world, near);
    let open = world.observe(near).unwrap();
    assert!(open.text.contains("You are tiring."));
    assert!(open.text.contains("You are in the wind."));
    assert!(open.allowed_actions.iter().all(|action| action != "rest"));
    let refused = world.step(&[cmd("rest-open", near, Attempt::Rest)]);
    assert!(!refused.results[0].success);
    assert_eq!(
        refused.results[0].message,
        "The wind will not let you rest."
    );
    let after = world.observe(near).unwrap();
    assert_eq!(after.self_info.energy, "tiring");
    assert!(after.text.contains("You are tiring."));
    assert!(after.text.contains("You are in the wind."));
    assert!(after
        .events
        .iter()
        .all(|event| event.text != "The wind will not let you rest."));
    let far_after = world.observe(far).unwrap();
    assert!(far_after
        .events
        .iter()
        .all(|event| event.text != "The wind will not let you rest."));
    assert!(far_after
        .events
        .iter()
        .all(|event| event.text != "Near rests."));
    let slept = {
        let mut sheltered = World::valley(Tuning::evening(), 1);
        let near = sheltered.spawn_at("Near", "n", 6, 1).unwrap();
        wait_until_tiring(&mut sheltered, near);
        sheltered.step(&[cmd("sleep", near, Attempt::Rest)])
    };
    assert_eq!(
        slept.results[0].message,
        "You rest under the rock. You are rested."
    );
}

#[test]
fn leaving_the_rock_names_the_wind_and_the_way_back_keeps_the_shelter() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let out = world.step(&[cmd("out", near, Attempt::Move(Dir::South))]);
    assert_eq!(
        out.results[0].message,
        "You walk south, out from under the rock overhang, into the wind."
    );
    assert!(world.observe(west).unwrap().events.iter().any(|event| {
        event.text == "Near walks south, out from under the rock overhang, into the wind."
    }));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("walks")));
    let back = world.step(&[cmd("in", near, Attempt::Move(Dir::North))]);
    assert_eq!(
        back.results[0].message,
        "You walk north, under the rock overhang, out of the wind."
    );
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Near walks north, under the rock overhang, out of the wind."));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("walks")));
    while world.sim_time() < 16 {
        world.step(&[]);
    }
    let wet = world.step(&[cmd("out-wet", near, Attempt::Move(Dir::South))]);
    assert_eq!(
        wet.results[0].message,
        "You walk south, out from under the rock overhang, into the wind and the rain."
    );
    assert!(world.observe(west).unwrap().events.iter().any(|event| {
        event.text
            == "Near walks south, out from under the rock overhang, into the wind and the rain."
    }));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("into the wind")));
    let home = world.step(&[cmd("in-wet", near, Attempt::Move(Dir::North))]);
    assert_eq!(
        home.results[0].message,
        "You walk north, under the rock overhang, out of the wind and the rain."
    );
    let plain = world.step(&[cmd("plain", far, Attempt::Move(Dir::East))]);
    assert_eq!(plain.results[0].message, "You walk east.");
}

#[test]
fn a_lump_of_iron_ore_on_the_slope_can_be_carried() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 1).unwrap();
    let far = world.spawn_at("Far", "f", 5, 1).unwrap();
    let west = world.spawn_at("West", "w", 5, 3).unwrap();
    assert_eq!(world.material_tally(Material::IronOre), 1);
    assert_eq!(world.material_tally(Material::Ochre), 2);
    assert_eq!(world.material_tally(Material::Flint), 1);
    let from_near = world.observe(near).unwrap().text;
    assert!(
        from_near.contains("A lump of iron ore lies on the slope to the north and east."),
        "{from_near}"
    );
    assert!(
        from_near.contains("A flint nodule lies on the slope to the north."),
        "{from_near}"
    );
    let north = from_near
        .lines()
        .find(|line| line.starts_with("North:"))
        .unwrap_or("");
    assert_eq!(north, "North: slope, a flint nodule.");
    assert!(
        from_near.contains("A lump of ochre lies on the slope to the north and west."),
        "{from_near}"
    );
    assert_eq!(
        world.examine(near, ExamineTarget::IronOre),
        "A lump of iron ore lies on the slope to the north and east."
    );
    assert_eq!(
        world.examine(west, ExamineTarget::IronOre),
        "You see no iron ore from here."
    );
    assert!(!world.observe(west).unwrap().text.contains("iron"));
    let beside = world.observe(far).unwrap().text;
    assert!(
        beside.contains("North: slope, a lump of iron ore."),
        "{beside}"
    );
    assert!(
        beside.contains("A lump of iron ore lies on the slope to the north."),
        "{beside}"
    );
    world.step(&[cmd("east", near, Attempt::Move(Dir::East))]);
    world.step(&[cmd("north", near, Attempt::Move(Dir::North))]);
    assert_eq!(world.actor_cell(near), Some((5, 0)));
    let here = world.observe(near).unwrap().text;
    assert!(here.contains("A lump of iron ore lies here."), "{here}");
    assert!(here.contains("You are on the slope"), "{here}");
    assert_eq!(
        world.examine(near, ExamineTarget::IronOre),
        "A lump of iron ore lies here, on the slope."
    );
    let yours = "You gather a lump of iron ore.";
    let witnessed = "Near gathers a lump of iron ore.";
    let report = world.step(&[cmd("iron", near, Attempt::Gather(Material::IronOre))]);
    assert_eq!(report.results[0].message, yours);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != yours));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == witnessed));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("iron")));
    let after = world.observe(near).unwrap().text;
    assert!(!after.contains("A lump of iron ore lies here."), "{after}");
    assert!(
        arms_sentence(&after).contains("a lump of iron ore"),
        "{after}"
    );
    assert_eq!(world.material_items(Material::IronOre)[0].1, Some(near));
    assert_eq!(world.material_tally(Material::Ochre), 2);
    assert_eq!(world.material_tally(Material::Flint), 1);
    let refused = world.step(&[cmd("smelt", near, Attempt::Smelt)]);
    assert!(!refused.results[0].success);
    assert_eq!(refused.results[0].message, "There is no fire burning here.");
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("fire")));
    assert_eq!(world.material_items(Material::IronOre)[0].1, Some(near));
}

#[test]
fn carried_iron_ore_needs_a_hotter_fire_before_it_smelts() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 5, 0).unwrap();
    let far = world.spawn_at("Far", "f", 4, 1).unwrap();
    let west = world.spawn_at("West", "w", 4, 4).unwrap();
    world.step(&[cmd("ore", near, Attempt::Gather(Material::IronOre))]);
    world.step(&[cmd("east", near, Attempt::Move(Dir::East))]);
    world.step(&[cmd("south", near, Attempt::Move(Dir::South))]);
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    world.step(&[cmd("light", near, Attempt::Light)]);
    let cold = world.step(&[cmd("early", near, Attempt::Smelt)]);
    assert_eq!(
        cold.results[0].message,
        "Iron needs a hot charcoal fire. This fire will leave a lump of charcoal when it burns down, then light with that."
    );
    assert!(!cold.results[0].success);
    for witness in [far, west] {
        assert!(
            world
                .observe(witness)
                .unwrap()
                .events
                .iter()
                .all(|event| !event.text.contains("charcoal fire")),
            "{:?}",
            world
                .observe(witness)
                .unwrap()
                .events
                .iter()
                .map(|event| event.text.clone())
                .collect::<Vec<_>>()
        );
    }
    assert_eq!(world.material_items(Material::IronOre)[0].1, Some(near));
    let before = world.observe(near).unwrap().text;
    assert!(
        before.contains("A small fire burns here. It warms you."),
        "{before}"
    );
    assert!(!before.contains("burns hotter"), "{before}");
    let from_far = world.observe(far).unwrap().text;
    assert!(from_far.contains("A small fire burns there."), "{from_far}");
    assert!(!from_far.contains("burns hotter"), "{from_far}");
    assert!(
        !world.observe(west).unwrap().text.contains("burns hotter"),
        "{}",
        world.observe(west).unwrap().text
    );
    let fed = "You feed a dry branch into the fire. It burns hotter.";
    let seen_feed = "Near feeds a dry branch into the fire. It burns hotter.";
    let feed = world.step(&[cmd("feed", near, Attempt::Feed)]);
    assert_eq!(feed.results[0].message, fed);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == fed));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == seen_feed));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("feed") && !event.text.contains("hotter")));
    let hotter = world.observe(near).unwrap().text;
    assert!(
        hotter.contains("A fire burns hotter here. It warms you."),
        "{hotter}"
    );
    assert!(!hotter.contains("A small fire burns here"), "{hotter}");
    let seen_hot = world.observe(far).unwrap().text;
    assert!(
        seen_hot.contains("A fire burns hotter there."),
        "{seen_hot}"
    );
    assert!(!seen_hot.contains("A small fire burns"), "{seen_hot}");
    assert!(
        !world.observe(west).unwrap().text.contains("burns hotter"),
        "{}",
        world.observe(west).unwrap().text
    );
    assert!(
        world
            .examine(near, ExamineTarget::Fire)
            .starts_with("A fire burns hotter here,"),
        "{}",
        world.examine(near, ExamineTarget::Fire)
    );
    let still = world.step(&[cmd("fed-fire", near, Attempt::SmeltIron)]);
    assert_eq!(
        still.results[0].message,
        "Iron needs a hot charcoal fire. This fire will leave a lump of charcoal when it burns down, then light with that."
    );
    assert!(world.observe(near).unwrap().events.iter().any(|event| {
        event.text == "Iron needs a hot charcoal fire. This fire will leave a lump of charcoal when it burns down, then light with that."
    }));
    for witness in [far, west] {
        assert!(world
            .observe(witness)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("charcoal fire") }));
    }
    relight_charcoal(&mut world, near);
    let yours = "You set a lump of iron ore by the fire. It leaves your arms.";
    let witnessed = "Near sets a lump of iron ore by the fire. It leaves their arms.";
    let set = world.step(&[cmd("smelt", near, Attempt::SmeltIron)]);
    assert_eq!(
        set.results[0].message,
        "You set a lump of iron ore by the fire. It leaves your arms. It is not iron yet."
    );
    let after_set = world.sim_time();
    assert!(world.observe(near).unwrap().events.iter().all(|event| {
        event.text != yours && event.text != witnessed && !event.text.contains("not iron yet")
    }));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == witnessed));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("iron") && !event.text.contains("smelt")));
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains("A lump of iron ore is smelting in the hot charcoal fire."));
    assert!(!arms_sentence(&world.observe(near).unwrap().text).contains("iron"));
    for _ in 0..3 {
        world.step(&[]);
        assert_eq!(world.material_tally(Material::Iron), 0);
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("A lump of iron ore is smelting in the hot charcoal fire."));
    }
    world.step(&[]);
    assert_eq!(world.sim_time(), after_set + 4);
    assert_eq!(world.material_tally(Material::Iron), 1);
    assert_eq!(world.material_items(Material::Iron)[0].1, None);
    let bloom = "The iron ore smelts into a bloom.";
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == bloom));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == bloom));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != bloom));
    let here = world.observe(near).unwrap().text;
    assert!(
        here.contains("A bloom of iron lies here. It is metal from a hotter fire."),
        "{here}"
    );
    assert_eq!(
        world.examine(near, ExamineTarget::Iron),
        "A bloom of iron lies here. It is metal from a hotter fire."
    );
    assert!(!here.contains("is smelting in the fire."), "{here}");
    let gathered = "You gather a bloom of iron.";
    let seen = "Near gathers a bloom of iron.";
    let took = world.step(&[cmd("bloom", near, parse_attempt("gather bloom").unwrap())]);
    assert_eq!(took.results[0].message, gathered);
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == seen));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("bloom")));
    let arms = world.observe(near).unwrap().text;
    assert_eq!(
        arms_sentence(&arms),
        "You have a bloom of iron, metal from a hotter fire."
    );
    assert_eq!(
        world.examine(near, ExamineTarget::Iron),
        "A bloom of iron is carried by you, under the rock overhang. It is metal from a hotter fire."
    );
    assert_eq!(world.material_items(Material::Iron)[0].1, Some(near));
    world.step(&[cmd("off-fire", near, Attempt::Move(Dir::South))]);
    let refused = world.step(&[cmd("cold-hammer", near, Attempt::HammerIron)]);
    assert_eq!(
        refused.results[0].message,
        "You need a hotter fire or a stone to hammer the bloom."
    );
    assert!(!refused.results[0].success);
    for witness in [far, west] {
        assert!(world
            .observe(witness)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("hammer") && !event.text.contains("knife") }));
    }
    assert_eq!(world.material_items(Material::Iron)[0].1, Some(near));
    world.step(&[cmd("back-fire", near, Attempt::Move(Dir::North))]);
    let yours = "You hammer the bloom of iron into an iron knife at the fire.";
    let seen_hit = "Near hammers the bloom of iron into an iron knife at the fire.";
    let hit = world.step(&[cmd("hammer", near, Attempt::HammerIron)]);
    assert_eq!(hit.results[0].message, yours);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| { event.text != yours && event.text != seen_hit }));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == seen_hit));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("knife")));
    assert_eq!(world.material_tally(Material::Iron), 0);
    assert_eq!(world.material_tally(Material::Knife), 1);
    let arms = world.observe(near).unwrap().text;
    assert!(!arms.contains("bloom of iron"), "{arms}");
    assert_eq!(arms_sentence(&arms), "You have an iron knife.");
    assert_eq!(
        world.examine(near, ExamineTarget::Knife),
        "An iron knife is carried by you, under the rock overhang. It can cut."
    );
    assert_eq!(
        world.examine(far, ExamineTarget::Knife),
        "An iron knife is carried by Near, under the rock overhang. It can cut."
    );
    assert_eq!(
        world.examine(west, ExamineTarget::Knife),
        "You see no iron knife from here."
    );
    assert!(
        world
            .observe(near)
            .unwrap()
            .text
            .contains("A hot charcoal fire burns here."),
        "the look still does not promise iron"
    );
}

#[test]
fn a_bloom_of_iron_hammers_into_a_knife_with_a_stone() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 5, 0).unwrap();
    let far = world.spawn_at("Far", "f", 4, 1).unwrap();
    let west = world.spawn_at("West", "w", 4, 4).unwrap();
    world.step(&[cmd("ore", near, Attempt::Gather(Material::IronOre))]);
    world.step(&[cmd("east", near, Attempt::Move(Dir::East))]);
    world.step(&[cmd("south", near, Attempt::Move(Dir::South))]);
    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    world.step(&[cmd("light", near, Attempt::Light)]);
    world.step(&[cmd("feed", near, Attempt::Feed)]);
    relight_charcoal(&mut world, near);
    world.step(&[cmd("smelt", near, Attempt::SmeltIron)]);
    for _ in 0..4 {
        world.step(&[]);
    }
    world.step(&[cmd("bloom", near, Attempt::Gather(Material::Iron))]);
    for (id, dir) in [
        ("s", Dir::South),
        ("w1", Dir::West),
        ("w2", Dir::West),
        ("w3", Dir::West),
        ("w4", Dir::West),
        ("n1", Dir::North),
        ("n2", Dir::North),
    ] {
        let step = world.step(&[cmd(id, near, Attempt::Move(dir))]);
        assert!(step.results[0].success, "{}", step.results[0].message);
    }
    assert_eq!(world.actor_cell(near), Some((2, 0)));
    let stone = world.step(&[cmd("stone", near, Attempt::Gather(Material::Stone))]);
    assert!(stone.results[0].success, "{}", stone.results[0].message);
    let yours = "You hammer the bloom of iron into an iron knife with a stone.";
    let seen = "Near hammers the bloom of iron into an iron knife with a stone.";
    let hit = world.step(&[cmd("hammer", near, Attempt::HammerIron)]);
    assert_eq!(hit.results[0].message, yours);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| { event.text != yours && event.text != seen }));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == seen));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("knife")));
    assert_eq!(world.material_tally(Material::Iron), 0);
    assert_eq!(world.material_tally(Material::Knife), 1);
    assert_eq!(world.material_items(Material::Stone)[0].1, Some(near));
    let examined = world.examine(near, ExamineTarget::Knife);
    assert!(
        examined.contains("An iron knife is carried by you") && examined.contains("It can cut."),
        "{examined}"
    );
}

#[test]
fn an_iron_knife_butchers_a_carcass_and_stays_in_the_arms() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 4, 0).unwrap();
    let far = world.spawn_at("Far", "f", 4, 1).unwrap();
    let west = world.spawn_at("West", "w", 4, 4).unwrap();
    for (id, near_attempt, far_attempt, west_attempt) in [
        (
            "flint",
            Attempt::Gather(Material::Flint),
            Attempt::Move(Dir::East),
            Attempt::Move(Dir::East),
        ),
        (
            "knap",
            Attempt::Knap,
            Attempt::Move(Dir::East),
            Attempt::Move(Dir::North),
        ),
        (
            "to-ore",
            Attempt::Move(Dir::East),
            Attempt::Gather(Material::Berry),
            Attempt::Move(Dir::North),
        ),
        (
            "ore",
            Attempt::Gather(Material::IronOre),
            Attempt::Gather(Material::Berry),
            Attempt::Gather(Material::Branch),
        ),
        (
            "to-rock-e",
            Attempt::Move(Dir::East),
            Attempt::Gather(Material::Berry),
            Attempt::Move(Dir::North),
        ),
        (
            "to-rock-s",
            Attempt::Move(Dir::South),
            Attempt::Gather(Material::Berry),
            Attempt::Move(Dir::East),
        ),
    ] {
        let step = world.step(&[
            cmd(id, near, near_attempt),
            cmd(&format!("far-{id}"), far, far_attempt),
            cmd(&format!("west-{id}"), west, west_attempt),
        ]);
        assert!(
            step.results.iter().all(|result| result.success),
            "{:?}",
            step.results
                .iter()
                .map(|result| result.message.clone())
                .collect::<Vec<_>>()
        );
    }
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    for (id, near_attempt, west_attempt) in [
        (
            "bark",
            Attempt::Gather(Material::Bark),
            Attempt::Drop(Material::Branch),
        ),
        ("light", Attempt::Light, Attempt::Move(Dir::South)),
        (
            "branch",
            Attempt::Gather(Material::Branch),
            Attempt::Move(Dir::South),
        ),
        ("haft", Attempt::Haft, Attempt::Move(Dir::South)),
        ("feed", Attempt::Feed, Attempt::Move(Dir::West)),
    ] {
        let step = world.step(&[
            cmd(id, near, near_attempt),
            cmd(&format!("west-{id}"), west, west_attempt),
        ]);
        assert!(
            step.results.iter().all(|result| result.success),
            "{:?}",
            step.results
                .iter()
                .map(|result| result.message.clone())
                .collect::<Vec<_>>()
        );
    }
    world.step(&[cmd("west-on", west, Attempt::Move(Dir::West))]);
    assert_eq!(world.actor_cell(west), Some((4, 4)));
    relight_charcoal(&mut world, near);
    let smelt = world.step(&[cmd("smelt", near, Attempt::SmeltIron)]);
    assert!(smelt.results[0].success, "{}", smelt.results[0].message);
    for _ in 0..4 {
        world.step(&[]);
    }
    assert!(
        arms_sentence(&world.observe(near).unwrap().text).contains("fire-hardened flint tip"),
        "{}",
        world.observe(near).unwrap().text
    );
    let bloom = world.step(&[cmd("bloom", near, Attempt::Gather(Material::Iron))]);
    assert!(bloom.results[0].success, "{}", bloom.results[0].message);
    let knife = world.step(&[cmd("hammer", near, Attempt::HammerIron)]);
    assert!(knife.results[0].success, "{}", knife.results[0].message);
    let mut deer_wait = 0;
    while world.deer_cell() != Some((6, 1)) && deer_wait < 40 {
        world.step(&[]);
        deer_wait += 1;
    }
    assert_eq!(
        world.deer_cell(),
        Some((6, 1)),
        "the deer is under the rock at sim time {}",
        world.sim_time()
    );
    let hunt = world.step(&[cmd("hunt", near, Attempt::Hunt)]);
    assert!(hunt.results[0].success, "{}", hunt.results[0].message);
    assert_eq!(world.material_items(Material::Spear)[0].1, Some(near));
    assert_eq!(world.material_items(Material::Knife)[0].1, Some(near));
    let bare = world.step(&[cmd("no-knife", far, Attempt::Butcher)]);
    assert_eq!(
        bare.results[0].message,
        "You need a flint spear or an iron knife."
    );
    let no_bone = world.step(&[cmd("early", west, Attempt::CarveAwl)]);
    assert_eq!(no_bone.results[0].message, "You have no bone to carve.");
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("no bone")));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("no bone")));
    let yours =
        "You butcher the carcass with the iron knife into three portions of raw meat, a hide, a bone, and an antler.";
    let seen =
        "Near butchers the carcass with the iron knife into three portions of raw meat, a hide, a bone, and an antler.";
    let cut = world.step(&[cmd("butcher", near, Attempt::Butcher)]);
    assert_eq!(cut.results[0].message, yours);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != yours));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == seen));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("butcher")));
    assert_eq!(world.material_tally(Material::Carcass), 0);
    assert_eq!(world.material_tally(Material::Meat), 3);
    assert_eq!(world.material_tally(Material::Hide), 1);
    assert_eq!(world.material_tally(Material::Bone), 2);
    assert_eq!(world.material_tally(Material::Knife), 1);
    assert_eq!(world.material_tally(Material::Spear), 1);
    assert_eq!(world.material_items(Material::Knife)[0].1, Some(near));
    assert_eq!(world.material_items(Material::Spear)[0].1, Some(near));
    assert!(
        world
            .observe(near)
            .unwrap()
            .text
            .contains("Two bones lie here."),
        "{}",
        world.observe(near).unwrap().text
    );
    assert!(matches!(
        parse_attempt("carve awl").unwrap(),
        Attempt::CarveAwl
    ));
    assert!(matches!(
        parse_attempt("make awl").unwrap(),
        Attempt::CarveAwl
    ));
    world.step(&[cmd("drop-knife", near, Attempt::Drop(Material::Knife))]);
    let no_tool = world.step(&[cmd("tool", near, Attempt::CarveAwl)]);
    assert_eq!(
        no_tool.results[0].message,
        "You need a flint flake or an iron knife to carve the bone."
    );
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("to carve the bone")));
    world.step(&[cmd("knife", near, Attempt::Gather(Material::Knife))]);
    let carved_yours = "You carve a bone into a bone awl with the iron knife.";
    let carved_seen = "Near carves a bone into a bone awl with the iron knife.";
    let carved = world.step(&[cmd("carve", near, parse_attempt("make awl").unwrap())]);
    assert_eq!(carved.results[0].message, carved_yours);
    assert_eq!(world.material_tally(Material::Bone), 1);
    assert_eq!(world.material_tally(Material::Awl), 1);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != carved_yours && !event.text.contains("Near carves")));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == carved_seen));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("Near carves")));
    let arms = arms_sentence(&world.observe(near).unwrap().text);
    assert!(arms.contains("a bone awl"), "{arms}");
    let awl = world.examine(near, ExamineTarget::Awl);
    assert!(
        awl.contains("A bone awl is carried by you") && awl.contains("It can pierce hide."),
        "{awl}"
    );
    world.step(&[cmd("drop-spear", near, Attempt::Drop(Material::Spear))]);
    let lifted = world.step(&[cmd("hide", near, Attempt::Gather(Material::Hide))]);
    assert!(lifted.results[0].success, "{}", lifted.results[0].message);
    let pierce_yours = "You pierce the deer hide with the bone awl. It is ready to sew.";
    let pierce_seen = "Near pierces the deer hide with the bone awl. It is ready to sew.";
    let pierced = world.step(&[cmd("pierce", near, Attempt::Pierce)]);
    assert_eq!(pierced.results[0].message, pierce_yours);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != pierce_yours));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == pierce_seen));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("bone awl")));
    assert!(world
        .examine(near, ExamineTarget::Hide)
        .contains("ready to sew"));
    world.step(&[cmd("drop-hide", near, Attempt::Drop(Material::Hide))]);
    world.step(&[cmd("drop-awl", near, Attempt::Drop(Material::Awl))]);
    world.step(&[cmd("spear", near, Attempt::Gather(Material::Spear))]);
    let examined = world.examine(near, ExamineTarget::Knife);
    assert!(examined.contains("It can cut."), "{examined}");
    if !world.observe(near).unwrap().text.contains("fire burns") {
        let branch = world.step(&[cmd(
            "relight-branch",
            near,
            Attempt::Gather(Material::Branch),
        )]);
        assert!(branch.results[0].success, "{}", branch.results[0].message);
        let bark = world.step(&[cmd("relight-bark", near, Attempt::Gather(Material::Bark))]);
        assert!(bark.results[0].success, "{}", bark.results[0].message);
        let light = world.step(&[cmd("relight", near, Attempt::Light)]);
        assert!(light.results[0].success, "{}", light.results[0].message);
    }
    let cook = world.step(&[cmd("cook", near, Attempt::Cook)]);
    assert!(cook.results[0].success, "{}", cook.results[0].message);
    let again = world.step(&[cmd("again", near, Attempt::Butcher)]);
    assert_eq!(
        again.results[0].message,
        "There is no carcass here to butcher."
    );

    assert!(matches!(parse_attempt("cut meat").unwrap(), Attempt::Cut));
    let denied = world.step(&[cmd("no-cut", far, parse_attempt("cut meat").unwrap())]);
    assert_eq!(
        denied.results[0].message,
        "You need an iron knife to cut the meat."
    );
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("iron knife to cut")));

    let gathered = world.step(&[cmd("meat", near, Attempt::Gather(Material::Meat))]);
    assert!(
        gathered.results[0].success,
        "{}",
        gathered.results[0].message
    );
    let arms = arms_sentence(&world.observe(near).unwrap().text);
    assert!(arms.contains("a portion of cooked meat"), "{arms}");
    let yours = "You cut the cooked meat into two smaller pieces with the iron knife.";
    let seen = "Near cuts the cooked meat into two smaller pieces with the iron knife.";
    let split = world.step(&[cmd("cut-carried", near, Attempt::Cut)]);
    assert_eq!(split.results[0].message, yours);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != yours));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == seen));
    assert!(world
        .observe(west)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("cuts the")));
    let arms = arms_sentence(&world.observe(near).unwrap().text);
    assert!(
        arms.contains("two smaller pieces of cooked meat")
            && arms.contains("an iron knife")
            && arms.contains("fire-hardened flint tip"),
        "{arms}"
    );
    assert_eq!(world.material_tally(Material::Meat), 4);
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains("Two portions of raw meat lie here."));

    let raw = "You cut the raw meat into two smaller pieces with the iron knife.";
    let ground = world.step(&[cmd("cut-ground", near, Attempt::Cut)]);
    assert_eq!(ground.results[0].message, raw);
    let here = world.observe(near).unwrap().text;
    assert!(
        here.contains("A portion of raw meat lies here.")
            && here.contains("Two smaller pieces of raw meat lie here."),
        "{here}"
    );
    let last = world.step(&[cmd("cut-last", near, Attempt::Cut)]);
    assert_eq!(last.results[0].message, raw);
    let after = world.observe(near).unwrap().text;
    assert!(
        after.contains("Four smaller pieces of raw meat lie here."),
        "{after}"
    );
    let too_small = world.step(&[cmd("cut-small", near, Attempt::Cut)]);
    assert_eq!(
        too_small.results[0].message,
        "The piece of meat is already small."
    );
    assert_eq!(world.material_tally(Material::Knife), 1);
    assert_eq!(world.material_items(Material::Knife)[0].1, Some(near));
    assert_eq!(world.material_items(Material::Spear)[0].1, Some(near));
}

#[test]
fn deer_tracks_show_for_who_can_see_the_cell_and_then_fade() {
    let mut world = World::valley(Tuning::evening(), 1);
    let here = world.spawn_at("Near", "n", 2, 3).unwrap();
    let seen = world.spawn_at("West", "w", 4, 3).unwrap();
    let quiet = world.spawn_at("Far", "f", 6, 3).unwrap();
    assert!(!world.observe(here).unwrap().text.contains("Deer tracks"));
    world.step(&[]);
    assert_eq!(world.deer_cell(), Some((3, 3)));
    assert_eq!(world.sim_time(), 1);
    let on_the_cell = world.observe(here).unwrap().text;
    assert!(
        on_the_cell.contains("Deer tracks mark the ground here."),
        "{on_the_cell}"
    );
    assert!(
        !on_the_cell.contains("A deer stands here."),
        "{on_the_cell}"
    );
    let in_sight = world.observe(seen).unwrap().text;
    assert!(
        in_sight.contains("Deer tracks mark the ground to the west."),
        "{in_sight}"
    );
    let out = world.observe(quiet).unwrap().text;
    assert!(!out.contains("Deer tracks"), "{out}");
    for _ in 0..3 {
        world.step(&[]);
        assert!(world
            .observe(here)
            .unwrap()
            .text
            .contains("Deer tracks mark the ground here."));
    }
    assert_eq!(world.sim_time(), 4);
    assert_ne!(world.deer_cell(), Some((2, 3)));
    world.step(&[]);
    assert_eq!(world.sim_time(), 5);
    assert!(!world
        .observe(here)
        .unwrap()
        .text
        .contains("Deer tracks mark the ground here."));
}

#[test]
fn a_splash_is_heard_one_cell_past_sight() {
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 4, 0).unwrap();
    let seen = world.spawn_at("West", "w", 4, 2).unwrap();
    let heard = world.spawn_at("Far", "f", 5, 2).unwrap();
    let quiet = world.spawn_at("Otto", "o", 6, 2).unwrap();
    let missed = world.step(&[cmd("away", near, Attempt::Fish)]);
    assert_eq!(missed.results[0].message, "The brook is not here.");
    assert!(world
        .observe(heard)
        .unwrap()
        .events
        .iter()
        .all(|event| { !event.text.contains("brook") && !event.text.contains("splash") }));
    world.step(&[cmd("flint", near, Attempt::Gather(Material::Flint))]);
    world.step(&[cmd("knap", near, Attempt::Knap)]);
    for (id, dir) in [("e1", Dir::East), ("e2", Dir::East), ("s", Dir::South)] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    let light = world.step(&[cmd("light", near, Attempt::Light)]);
    assert!(light.results[0].success, "{}", light.results[0].message);
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    let haft = world.step(&[cmd("haft", near, Attempt::Haft)]);
    assert!(haft.results[0].success, "{}", haft.results[0].message);
    for _ in 0..4 {
        world.step(&[]);
    }
    for (id, dir) in [
        ("w1", Dir::West),
        ("w2", Dir::West),
        ("w3", Dir::West),
        ("w4", Dir::West),
        ("bank", Dir::South),
    ] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    assert_eq!(world.actor_cell(near), Some((2, 2)));
    let yours = "You spear a fish from the brook.";
    let seen_line = "Near spears a fish from the brook.";
    let splash = "You hear a splash to the west.";
    let cast = world.step(&[cmd("fish", near, Attempt::Fish)]);
    assert_eq!(cast.results[0].message, yours);
    let actor = world.observe(near).unwrap();
    assert!(actor.events.iter().any(|event| event.text == yours));
    assert!(actor.events.iter().all(|event| {
        event.text.starts_with("A beaver splashes away") || !event.text.contains("splash")
    }));
    let eyes = world.observe(seen).unwrap();
    assert!(eyes.events.iter().any(|event| event.text == seen_line));
    assert!(eyes
        .events
        .iter()
        .all(|event| !event.text.contains("splash")));
    let ears = world.observe(heard).unwrap();
    assert!(ears.events.iter().any(|event| event.text == splash));
    assert!(ears
        .events
        .iter()
        .all(|event| !event.text.contains("spears")));
    assert!(world
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| { !event.text.contains("splash") && !event.text.contains("fish") }));
    let again = world.step(&[cmd("again", near, Attempt::Fish)]);
    assert_eq!(
        again.results[0].message,
        "The fish are gone from this stretch of the brook."
    );
    assert_eq!(
        world
            .observe(heard)
            .unwrap()
            .events
            .iter()
            .filter(|event| event.text == splash)
            .count(),
        1
    );
    assert!(world
        .observe(seen)
        .unwrap()
        .events
        .iter()
        .all(|event| { !event.text.contains("gone from this stretch") }));
}

#[test]
fn night_in_the_open_lets_you_hear_the_wind() {
    let mut world = world_at(10_000);
    let open = world.spawn_at("Open", "o", 4, 3).unwrap();
    let rock = world.spawn_at("Rock", "r", 6, 1).unwrap();
    let bare = world.spawn_at("Bare", "b", 6, 1).unwrap();
    let thatch = world.spawn_at("Thatch", "t", 5, 2).unwrap();
    let built = world.step(&[cmd("build", thatch, Attempt::BuildLeanTo)]);
    assert!(built.results[0].success, "{}", built.results[0].message);
    for (id, dir) in [
        ("reed-w1", Dir::West),
        ("reed-w2", Dir::West),
        ("reed-w3", Dir::West),
        ("reed-w4", Dir::West),
        ("reed-w5", Dir::West),
        ("reed-s", Dir::South),
    ] {
        world.step(&[cmd(id, thatch, Attempt::Move(dir))]);
    }
    let reeds = world.step(&[cmd("reeds", thatch, Attempt::Gather(Material::Reed))]);
    assert!(reeds.results[0].success, "{}", reeds.results[0].message);
    for (id, dir) in [
        ("back-n", Dir::North),
        ("back-e1", Dir::East),
        ("back-e2", Dir::East),
        ("back-e3", Dir::East),
        ("back-e4", Dir::East),
        ("back-e5", Dir::East),
    ] {
        world.step(&[cmd(id, thatch, Attempt::Move(dir))]);
    }
    let covered = world.step(&[cmd("thatch", thatch, Attempt::Thatch)]);
    assert!(covered.results[0].success, "{}", covered.results[0].message);
    for i in 0..3 {
        let got = world.step(&[cmd(
            &format!("branch-{i}"),
            bare,
            Attempt::Gather(Material::Branch),
        )]);
        assert!(got.results[0].success, "{}", got.results[0].message);
    }
    world.step(&[cmd("south", bare, Attempt::Move(Dir::South))]);
    assert_eq!(world.actor_cell(bare), Some((6, 2)));
    let second = world.step(&[cmd("bare-build", bare, Attempt::BuildLeanTo)]);
    assert!(second.results[0].success, "{}", second.results[0].message);
    let day = world.observe(open).unwrap().text;
    assert!(day.contains("You are in the wind."), "{day}");
    assert!(!day.contains("You hear the wind rush."), "{day}");
    assert!(!day.contains("whisper"), "{day}");
    while world.sim_time() < world.tuning.night_at {
        world.step(&[]);
    }
    let night_open = world.observe(open).unwrap();
    assert!(
        night_open.text.contains("You hear the wind rush.")
            && night_open.text.contains("You are in the wind."),
        "{}",
        night_open.text
    );
    assert!(!night_open.text.contains("whisper"), "{}", night_open.text);
    let night_rock = world.observe(rock).unwrap().text;
    assert!(
        night_rock.contains("You are out of the wind."),
        "{night_rock}"
    );
    assert!(
        !night_rock.contains("You hear the wind rush."),
        "{night_rock}"
    );
    assert!(!night_rock.contains("whisper"), "{night_rock}");
    let night_bare = world.observe(bare).unwrap().text;
    assert!(
        night_bare.contains("You are in a little wind.")
            && night_bare.contains("The wind is a whisper here."),
        "{night_bare}"
    );
    assert!(
        !night_bare.contains("You hear the wind rush."),
        "{night_bare}"
    );
    let night_thatch = world.observe(thatch).unwrap().text;
    assert!(
        world.sim_time() < world.tuning.night_at + world.tuning.night_len,
        "{}",
        world.sim_time()
    );
    assert!(
        night_thatch.contains("You are out of the wind."),
        "{night_thatch}"
    );
    assert!(
        !night_thatch.contains("You hear the wind rush."),
        "{night_thatch}"
    );
    assert!(!night_thatch.contains("whisper"), "{night_thatch}");
    for person in [open, rock, bare, thatch] {
        assert!(world.observe(person).unwrap().events.iter().all(|event| {
            event.text != "You hear the wind rush." && !event.text.contains("whisper")
        }));
    }
    world.step(&[]);
    world.step(&[]);
    let still = world.observe(open).unwrap();
    assert!(
        still.text.contains("You hear the wind rush."),
        "{}",
        still.text
    );
    assert!(still.events.iter().all(|event| {
        event.text != "You hear the wind rush." && !event.text.contains("whisper")
    }));
    assert!(world.sim_time() < world.tuning.night_at + world.tuning.night_len);
}

#[test]
fn night_shortens_the_look_until_a_fire_is_nearby() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let west = world.spawn_at("West", "w", 6, 1).unwrap();
    let far = world.spawn_at("Far", "f", 6, 2).unwrap();
    let day = world.observe(near).unwrap().text;
    assert!(
        day.contains("A deer stands")
            && day.contains("Evening is coming.")
            && !day.contains("Night has fallen")
            && !day.contains("you cannot make out the ground"),
        "{day}"
    );
    let bark = world.step(&[cmd("bark", west, Attempt::Gather(Material::Bark))]);
    assert!(bark.results[0].success, "{}", bark.results[0].message);
    let branch = world.step(&[cmd("branch", west, Attempt::Gather(Material::Branch))]);
    assert!(branch.results[0].success, "{}", branch.results[0].message);
    while world.sim_time() < world.tuning.night_at {
        world.step(&[]);
    }
    assert!(world.sim_time() >= world.tuning.night_at);
    let dark = world.observe(near).unwrap().text;
    assert!(
        dark.contains("Night has fallen.")
            && dark.contains("The dark hides what lies farther off.")
            && dark.contains("The rock overhang is to the north and east.")
            && dark.contains("North: the rock overhang to the north and east.")
            && dark.contains("East: the rock overhang to the north and east.")
            && dark.contains("South: you cannot make out the ground.")
            && dark.contains("You are on the")
            && !dark.contains("A deer stands"),
        "{dark}"
    );
    assert!(world
        .observe(west)
        .unwrap()
        .text
        .contains("you cannot make out the ground"));
    assert!(world
        .observe(far)
        .unwrap()
        .text
        .contains("The dark hides what lies farther off."));
    let lit = world.step(&[cmd("light", west, Attempt::Light)]);
    assert!(lit.results[0].success, "{}", lit.results[0].message);
    let by_fire = world.observe(west).unwrap().text;
    assert!(
        by_fire.contains("Night has fallen.")
            && by_fire.contains("A small fire burns here.")
            && !by_fire.contains("you cannot make out the ground")
            && !by_fire.contains("The dark hides what lies farther off."),
        "{by_fire}"
    );
    let beside = world.observe(far).unwrap().text;
    assert!(
        beside.contains("Night has fallen.")
            && !beside.contains("you cannot make out the ground")
            && !beside.contains("The dark hides what lies farther off."),
        "{beside}"
    );
    let still = world.observe(near).unwrap().text;
    assert!(
        still.contains("The dark hides what lies farther off.")
            && still.contains("you cannot make out the ground")
            && still.contains("You are on the"),
        "{still}"
    );
    let dawn_at = world.tuning.night_at + world.tuning.night_len;
    while world.sim_time() < dawn_at {
        world.step(&[]);
    }
    assert_eq!(world.sim_time(), dawn_at);
    let morning = world.observe(near).unwrap().text;
    assert!(
        morning.contains("Morning is on the valley.")
            && morning.contains("North: valley floor")
            && !morning.contains("Night has fallen")
            && !morning.contains("you cannot make out the ground")
            && !morning.contains("The dark hides"),
        "{morning}"
    );
    if let (Some((dx, dy)), Some((nx, ny))) = (world.deer_cell(), world.actor_cell(near)) {
        let dist = (dx - nx).abs().max((dy - ny).abs());
        if dist > 0 && dist <= world.tuning.sight {
            assert!(
                morning.contains("A deer stands") || morning.contains("a deer"),
                "{morning}"
            );
        }
    }
    let next_night = dawn_at + world.tuning.day_len;
    while world.sim_time() < next_night {
        world.step(&[]);
    }
    assert_eq!(world.sim_time(), next_night);
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains("Night has fallen."));
}

#[test]
fn night_cold_in_the_open_is_faster_and_shelter_holds() {
    let mut world = World::valley(Tuning::evening(), 1);
    let open = world.spawn_at("Open", "o", 6, 1).unwrap();
    let rock = world.spawn_at("Rock", "r", 6, 1).unwrap();
    let lean = world.spawn_at("Lean", "l", 5, 2).unwrap();
    let built = world.step(&[cmd("build", lean, Attempt::BuildLeanTo)]);
    assert!(built.results[0].success, "{}", built.results[0].message);
    world.step(&[cmd("to-rock-e", lean, Attempt::Move(Dir::East))]);
    world.step(&[cmd("to-rock-n", lean, Attempt::Move(Dir::North))]);
    assert_eq!(world.actor_cell(lean), Some((6, 1)));
    let bark = world.step(&[cmd("bark", rock, Attempt::Gather(Material::Bark))]);
    assert!(bark.results[0].success, "{}", bark.results[0].message);
    let branch = world.step(&[cmd("branch", rock, Attempt::Gather(Material::Branch))]);
    assert!(branch.results[0].success, "{}", branch.results[0].message);

    world.step(&[cmd("day-out", open, Attempt::Move(Dir::South))]);
    for i in 0..3 {
        world.step(&[cmd(&format!("day-wait-{i}"), open, Attempt::Look)]);
    }
    let day = world.observe(open).unwrap();
    assert_eq!(day.self_info.cold, "chilly");
    assert!(day.text.contains("You are chilly."), "{}", day.text);
    assert!(!day.text.contains("You are cold."));
    world.step(&[cmd("day-back", open, Attempt::Move(Dir::North))]);
    assert_eq!(world.observe(open).unwrap().self_info.cold, "warm");
    assert_eq!(world.actor_cell(open), Some((6, 1)));

    while world.sim_time() < world.tuning.night_at {
        world.step(&[]);
    }
    assert!(world.sim_time() >= world.tuning.night_at);
    assert!(world
        .observe(open)
        .unwrap()
        .text
        .contains("Night has fallen."));
    world.step(&[
        cmd("night-open", open, Attempt::Move(Dir::South)),
        cmd("night-lean", lean, Attempt::Move(Dir::South)),
    ]);
    world.step(&[
        cmd("night-open-2", open, Attempt::Look),
        cmd("night-lean-2", lean, Attempt::Move(Dir::West)),
    ]);
    assert_eq!(world.actor_cell(lean), Some((5, 2)));
    for i in 0..2 {
        world.step(&[cmd(&format!("night-open-wait-{i}"), open, Attempt::Look)]);
    }
    let night_open = world.observe(open).unwrap();
    assert_eq!(night_open.self_info.cold, "cold");
    assert!(
        night_open.text.contains("You are cold.")
            && night_open.text.contains("You are in the wind."),
        "{}",
        night_open.text
    );
    let under_lean = world.observe(lean).unwrap();
    assert_eq!(under_lean.self_info.cold, "warm");
    assert!(
        under_lean.text.contains("You are warm.")
            && under_lean.text.contains("You are in a little wind."),
        "{}",
        under_lean.text
    );
    let under_rock = world.observe(rock).unwrap();
    assert_eq!(under_rock.self_info.cold, "warm");
    assert!(
        under_rock.text.contains("You are warm.")
            && under_rock.text.contains("You are out of the wind."),
        "{}",
        under_rock.text
    );
    assert!(night_open
        .events
        .iter()
        .any(|event| event.text == "You are cold. The wind is on you."));
    assert!(night_open
        .events
        .iter()
        .all(|event| !event.text.contains("Open is cold")));
    assert!(under_rock
        .events
        .iter()
        .any(|event| event.text == "Open is cold. The wind is on them."));
    assert!(under_rock
        .events
        .iter()
        .all(|event| event.text != "You are cold. The wind is on you."));
    assert!(world.sim_time() < world.tuning.night_at + world.tuning.night_len);
    let lit = world.step(&[
        cmd("light", rock, Attempt::Light),
        cmd("to-fire", open, Attempt::Move(Dir::North)),
    ]);
    assert!(lit.results[0].success, "{}", lit.results[0].message);
    assert_eq!(world.actor_cell(open), Some((6, 1)));
    world.step(&[]);
    world.step(&[]);
    let warmed = world.observe(open).unwrap();
    assert_eq!(warmed.self_info.cold, "warm");
    assert!(
        warmed.text.contains("You are warm.")
            && warmed
                .text
                .contains("A small fire burns here. It warms you."),
        "{}",
        warmed.text
    );
    assert!(world.sim_time() < world.tuning.night_at + world.tuning.night_len);
    assert!(warmed.text.contains("Night has fallen."));
    assert!(!world.observe(lean).unwrap().text.contains("You are cold."));
}

#[test]
fn a_fire_in_the_lean_to_holds_warm_through_the_night() {
    let mut world = World::valley(Tuning::evening(), 1);
    let alone = world.spawn_at("Alone", "a", 5, 2).unwrap();
    let builder = world.spawn_at("Build", "b", 6, 1).unwrap();
    let hearth = world.spawn_at("Hearth", "h", 6, 1).unwrap();
    let built = world.step(&[cmd("lean-a", alone, Attempt::BuildLeanTo)]);
    assert!(built.results[0].success, "{}", built.results[0].message);
    for i in 0..3 {
        let got = world.step(&[cmd(
            &format!("br-{i}"),
            builder,
            Attempt::Gather(Material::Branch),
        )]);
        assert!(got.results[0].success, "{}", got.results[0].message);
    }
    let bark = world.step(&[cmd("bark", hearth, Attempt::Gather(Material::Bark))]);
    assert!(bark.results[0].success, "{}", bark.results[0].message);
    let branch = world.step(&[cmd("fuel", hearth, Attempt::Gather(Material::Branch))]);
    assert!(branch.results[0].success, "{}", branch.results[0].message);
    world.step(&[cmd("to-side", builder, Attempt::Move(Dir::West))]);
    assert_eq!(world.actor_cell(builder), Some((5, 1)));
    let second = world.step(&[cmd("lean-b", builder, Attempt::BuildLeanTo)]);
    assert!(second.results[0].success, "{}", second.results[0].message);
    world.step(&[
        cmd("alone-e", alone, Attempt::Move(Dir::East)),
        cmd("build-e", builder, Attempt::Move(Dir::East)),
    ]);
    world.step(&[cmd("alone-n", alone, Attempt::Move(Dir::North))]);
    assert_eq!(world.actor_cell(alone), Some((6, 1)));
    assert_eq!(world.actor_cell(builder), Some((6, 1)));
    world.step(&[]);
    assert_eq!(world.observe(alone).unwrap().self_info.cold, "warm");
    assert_eq!(world.observe(hearth).unwrap().self_info.cold, "warm");

    while world.sim_time() < world.tuning.night_at {
        world.step(&[]);
    }
    assert!(world
        .observe(alone)
        .unwrap()
        .text
        .contains("Night has fallen."));
    world.step(&[
        cmd("out", alone, Attempt::Move(Dir::South)),
        cmd("to-shelter", hearth, Attempt::Move(Dir::West)),
    ]);
    assert_eq!(world.actor_cell(alone), Some((6, 2)));
    assert_eq!(world.actor_cell(hearth), Some((5, 1)));
    let lit = world.step(&[
        cmd("under", alone, Attempt::Move(Dir::West)),
        cmd("light", hearth, Attempt::Light),
    ]);
    assert!(
        lit.results
            .iter()
            .any(|result| result.success && result.message.contains("light")),
        "{lit:?}"
    );
    assert_eq!(world.actor_cell(alone), Some((5, 2)));
    let end = world.tuning.night_at + world.tuning.night_len - 1;
    while world.sim_time() < end {
        world.step(&[]);
    }
    assert_eq!(world.sim_time(), end);

    let warm = world.observe(hearth).unwrap();
    assert_eq!(warm.self_info.cold, "warm", "{}", warm.text);
    assert!(
        warm.text.contains("You are warm.")
            && warm.text.contains("You are in a little wind.")
            && warm.text.contains("A small fire burns here. It warms you.")
            && warm.text.contains("The lean-to keeps the fire.")
            && warm.text.contains("Night has fallen.")
            && !warm
                .text
                .contains("The heat of the fire does not reach you.")
            && !warm.text.contains("You are chilly."),
        "{}",
        warm.text
    );
    let uncovered = world.observe(alone).unwrap();
    assert_eq!(uncovered.self_info.cold, "chilly", "{}", uncovered.text);
    assert!(
        uncovered.text.contains("You are chilly.")
            && uncovered.text.contains("You are in a little wind.")
            && uncovered
                .text
                .contains("The heat of the fire does not reach you.")
            && uncovered.text.contains("Night has fallen.")
            && !uncovered.text.contains("It warms you.")
            && !uncovered.text.contains("You are warm."),
        "{}",
        uncovered.text
    );
}

#[test]
fn a_clear_night_names_the_stars_and_a_raining_night_does_not() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let far = world.spawn_at("Far", "f", 2, 3).unwrap();
    let stars = "Night has fallen. The stars are out over the valley.";
    let wet = "Night has fallen. Rain is falling on the valley.";
    let day = world.observe(near).unwrap().text;
    assert!(day.contains("Evening is coming."), "{day}");
    assert!(!day.contains("stars"), "{day}");
    assert!(!world.observe(far).unwrap().text.contains("stars"));

    while world.sim_time() < 20 {
        world.step(&[]);
    }
    let raining_day = world.observe(near).unwrap().text;
    assert!(
        raining_day.contains("Rain is falling on the valley.") && !raining_day.contains("stars"),
        "{raining_day}"
    );
    assert!(!world.observe(far).unwrap().text.contains("stars"));

    while world.sim_time() < world.tuning.night_at {
        world.step(&[]);
    }
    let raining_night = world.observe(near).unwrap().text;
    assert!(raining_night.contains(wet), "{raining_night}");
    assert!(!raining_night.contains("stars"), "{raining_night}");
    assert!(world.observe(far).unwrap().text.contains(wet));
    assert!(!world.observe(far).unwrap().text.contains("stars"));

    let dawn = world.tuning.night_at + world.tuning.night_len;
    while world.sim_time() < dawn + 1 {
        world.step(&[]);
    }
    assert_eq!(world.sim_time(), dawn + 1);
    assert!(!world.is_raining());
    let morning = world.observe(near).unwrap();
    assert!(
        morning.text.contains("Morning is on the valley.")
            && !morning.text.contains("stars")
            && !morning.text.contains("Rain is falling"),
        "{}",
        morning.text
    );
    assert!(
        morning
            .events
            .iter()
            .any(|event| event.text == "The rain lets up."),
        "{:?}",
        morning.events
    );
    let far_morning = world.observe(far).unwrap();
    assert!(!far_morning.text.contains("stars"));
    assert!(far_morning
        .events
        .iter()
        .any(|event| event.text == "The rain lets up."));

    let clear_at = dawn + world.tuning.day_len;
    while world.sim_time() < clear_at {
        world.step(&[]);
    }
    assert_eq!(world.sim_time(), clear_at);
    let near_sky = world.observe(near).unwrap().text;
    let far_sky = world.observe(far).unwrap().text;
    assert!(near_sky.contains(stars), "{near_sky}");
    assert!(far_sky.contains(stars), "{far_sky}");
    assert!(!near_sky.contains("Rain is falling"), "{near_sky}");
    assert!(!far_sky.contains("Rain is falling"), "{far_sky}");
}

#[test]
fn a_carried_basket_holds_two_more_things() {
    assert_eq!(parse_attempt("basket").unwrap(), Attempt::NameBasket);
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let seen = world.spawn_at("Seen", "s", 6, 1).unwrap();
    let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
    let named = world.step(&[cmd("name", near, Attempt::NameBasket)]);
    assert_eq!(
        named.results[0].message,
        "Weave a bark basket from a sheet of bark. It holds two more things."
    );
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("two more")));
    let woven = world.step(&[cmd("weave", near, Attempt::WeaveBasket)]);
    assert_eq!(woven.results[0].message, "You weave a bark basket.");
    assert!(world
        .observe(seen)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Near weaves a bark basket."));
    assert!(world
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("basket")));
    for i in 0..4 {
        let got = world.step(&[cmd(
            &format!("branch-{i}"),
            near,
            Attempt::Gather(Material::Branch),
        )]);
        assert!(got.results[0].success, "{}", got.results[0].message);
    }
    let berry = world.step(&[cmd("berry", near, Attempt::Gather(Material::Berry))]);
    assert_eq!(
        berry.results[0].message,
        "You gather a handful of berries. It goes into the bark basket."
    );
    assert!(world.observe(seen).unwrap().events.iter().any(|event| {
        event.text == "Near gathers a handful of berries. It goes into the bark basket."
    }));
    assert!(world.observe(near).unwrap().events.iter().all(|event| {
        event.text != "You gather a handful of berries. It goes into the bark basket."
            && event.text != "Near gathers a handful of berries. It goes into the bark basket."
    }));
    assert!(world
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("into the bark basket")));
    let seen_holding = world.observe(seen).unwrap().text;
    assert!(
        seen_holding.contains(
            "Near carries four dry branches and a bark basket holding a handful of berries."
        ) && !seen_holding.contains("a full bark basket"),
        "{seen_holding}"
    );
    let pot = world.step(&[cmd("pot", near, Attempt::Gather(Material::Pot))]);
    assert_eq!(
        pot.results[0].message,
        "You gather a pot from the ground under the rock overhang. It goes into the bark basket. The basket is full."
    );
    assert!(world
        .observe(seen)
        .unwrap()
        .events
        .iter()
        .any(|event| {
            event.text
                == "Near gathers a pot from the ground under the rock overhang. It goes into the bark basket. The basket is full."
        }));
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| { !event.text.contains("The basket is full.") }));
    assert!(world
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("The basket is full.")));
    let look = world.observe(near).unwrap().text;
    assert!(
        look.contains(
            "four dry branches, a full bark basket holding a handful of berries and a pot"
        ),
        "{look}"
    );
    let seen_look = world.observe(seen).unwrap().text;
    assert!(
        seen_look.contains(
            "Near is here. Near carries four dry branches and a full bark basket holding a handful of berries and a pot."
        ),
        "{seen_look}"
    );
    assert!(world
        .observe(seen)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("carries a full bark basket")));
    assert!(!world
        .observe(quiet)
        .unwrap()
        .text
        .contains("Near carries four dry branches"));
    let exam = world.examine(near, ExamineTarget::Basket);
    assert!(
        exam.contains("It holds a handful of berries and a pot."),
        "{exam}"
    );
    let full = "Your arms are full. Drop something first.";
    let refused = world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    assert_eq!(refused.results[0].message, full);
    assert!(!refused.results[0].success);
    for person in [near, seen, quiet] {
        assert!(world
            .observe(person)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != full));
    }
    let again = world.step(&[cmd("name-2", near, Attempt::NameBasket)]);
    assert!(
        again.results[0].message.contains("It cannot hold more."),
        "{}",
        again.results[0].message
    );
}

#[test]
fn a_bark_basket_holds_berries_that_would_not_fit_in_the_arms() {
    assert_eq!(parse_attempt("weave basket").unwrap(), Attempt::WeaveBasket);
    assert_eq!(parse_attempt("make basket").unwrap(), Attempt::WeaveBasket);
    let mut world = World::valley(Tuning::evening(), 1);
    let bare = world.spawn_at("Bare", "b", 4, 3).unwrap();
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let far = world.spawn_at("Far", "f", 6, 1).unwrap();
    let quiet = world.spawn_at("Quiet", "q", 2, 3).unwrap();

    let refused = world.step(&[cmd("no-bark", bare, Attempt::WeaveBasket)]);
    assert_eq!(
        refused.results[0].message,
        "You need a sheet of bark to weave a basket."
    );
    assert!(!refused.results[0].success);
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("sheet of bark")));
    assert!(world
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| { !event.text.contains("sheet of bark") && !event.text.contains("basket") }));

    let woven = world.step(&[cmd("weave", near, Attempt::WeaveBasket)]);
    assert_eq!(woven.results[0].message, "You weave a bark basket.");
    assert!(woven.results[0].success);
    let near_look = world.observe(near).unwrap();
    assert!(
        near_look.text.contains("an empty bark basket"),
        "{}",
        near_look.text
    );
    assert!(
        !near_look.text.contains("holding a handful"),
        "{}",
        near_look.text
    );
    assert!(near_look.text.contains("A sheet of bark lies here."));
    assert!(!near_look.text.contains("A pile of bark"));
    assert!(near_look.events.iter().all(|event| {
        event.text != "You weave a bark basket." && event.text != "Near weaves a bark basket."
    }));
    let empty = world.examine(near, ExamineTarget::Basket);
    assert!(
        empty.contains("an empty bark basket")
            && empty.contains("woven from a sheet of bark")
            && !empty.contains("It is empty."),
        "{empty}"
    );
    let seen = world.examine(far, ExamineTarget::Basket);
    assert!(
        seen.contains("Near carries an empty bark basket") && !seen.contains("It is empty."),
        "{seen}"
    );
    assert_eq!(
        world.examine(quiet, ExamineTarget::Basket),
        "You see no bark basket from here."
    );
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Near weaves a bark basket."));
    assert!(world
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("weaves") && !event.text.contains("basket")));

    for i in 0..3 {
        let got = world.step(&[
            cmd(&format!("br-{i}"), near, Attempt::Gather(Material::Branch)),
            cmd(
                &format!("far-berry-{i}"),
                far,
                Attempt::Gather(Material::Berry),
            ),
        ]);
        assert!(got.results[0].success, "{}", got.results[0].message);
        assert!(got.results[1].success, "{}", got.results[1].message);
    }
    let filled = world.step(&[
        cmd("pot", near, Attempt::Gather(Material::Pot)),
        cmd("bark", far, Attempt::Gather(Material::Bark)),
    ]);
    assert!(filled.results[0].success, "{}", filled.results[0].message);
    assert!(filled.results[1].success, "{}", filled.results[1].message);
    let berry = world.step(&[
        cmd("berries", near, Attempt::Gather(Material::Berry)),
        cmd("ochre", far, Attempt::Gather(Material::Ochre)),
    ]);
    assert!(berry.results[0].success, "{}", berry.results[0].message);
    assert!(berry.results[1].success, "{}", berry.results[1].message);
    let holding = world.observe(near).unwrap().text;
    assert!(
        holding.contains("a bark basket holding a handful of berries"),
        "{holding}"
    );
    let full = world.examine(near, ExamineTarget::Basket);
    assert!(full.contains("It holds a handful of berries."), "{full}");
    world.step(&[cmd("west", far, Attempt::Move(Dir::West))]);
    world.step(&[cmd("west-2", far, Attempt::Move(Dir::West))]);
    assert_eq!(world.actor_cell(far), Some((4, 1)));
    let no_room = world.step(&[cmd("poison", far, Attempt::Gather(Material::BitterBerry))]);
    assert_eq!(
        no_room.results[0].message,
        "Your arms are full. Drop something first."
    );
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("arms are full")));
}

#[test]
fn a_raw_fish_goes_into_the_bark_basket_when_the_arms_are_full() {
    assert_eq!(parse_attempt("put fish").unwrap(), Attempt::PutFish);
    assert_eq!(
        parse_attempt("put the fish in the basket").unwrap(),
        Attempt::PutFish
    );
    let mut world = world_at(10_000);
    let near = world.spawn_at("Near", "n", 4, 0).unwrap();
    let far = world.spawn_at("Far", "f", 3, 3).unwrap();
    let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();

    let refused = world.step(&[cmd("no-basket", quiet, Attempt::PutFish)]);
    assert_eq!(
        refused.results[0].message,
        "You are not carrying a bark basket."
    );
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("bark basket")));

    world.step(&[cmd("flint", near, Attempt::Gather(Material::Flint))]);
    world.step(&[cmd("knap", near, Attempt::Knap)]);
    for (id, dir) in [("e1", Dir::East), ("e2", Dir::East), ("s", Dir::South)] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    let light = world.step(&[cmd("light", near, Attempt::Light)]);
    assert!(light.results[0].success, "{}", light.results[0].message);
    world.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    let haft = world.step(&[cmd("haft", near, Attempt::Haft)]);
    assert!(haft.results[0].success, "{}", haft.results[0].message);
    for _ in 0..4 {
        world.step(&[]);
    }
    world.step(&[cmd("bark2", near, Attempt::Gather(Material::Bark))]);
    let woven = world.step(&[cmd("weave", near, Attempt::WeaveBasket)]);
    assert!(woven.results[0].success, "{}", woven.results[0].message);
    let branch = world.step(&[cmd("br", near, Attempt::Gather(Material::Branch))]);
    assert!(branch.results[0].success, "{}", branch.results[0].message);
    let pot = world.step(&[cmd("pot", near, Attempt::Gather(Material::Pot))]);
    assert!(pot.results[0].success, "{}", pot.results[0].message);
    let extra = world.step(&[cmd("ochre", near, Attempt::Gather(Material::Ochre))]);
    assert!(extra.results[0].success, "{}", extra.results[0].message);
    for (id, dir) in [
        ("w1", Dir::West),
        ("w2", Dir::West),
        ("w3", Dir::West),
        ("w4", Dir::West),
        ("bank", Dir::South),
    ] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    assert_eq!(world.actor_cell(near), Some((2, 2)));
    let cast = world.step(&[cmd("fish", near, Attempt::Fish)]);
    assert_eq!(cast.results[0].message, "You spear a fish from the brook.");
    let holding = world.observe(near).unwrap().text;
    assert!(
        holding.contains("a bark basket holding a raw fish"),
        "{holding}"
    );
    assert_eq!(holding.matches("raw fish").count(), 1, "{holding}");
    let exam = world.examine(near, ExamineTarget::Basket);
    assert!(exam.contains("It holds a raw fish."), "{exam}");
    let seen = world.examine(far, ExamineTarget::Basket);
    assert!(
        seen.contains("Near carries a bark basket") && seen.contains("It holds a raw fish."),
        "{seen}"
    );
    let fish_exam = world.examine(far, ExamineTarget::Fish);
    assert!(
        fish_exam.contains("A raw fish is in the bark basket carried by Near"),
        "{fish_exam}"
    );
    assert_eq!(
        world.examine(quiet, ExamineTarget::Basket),
        "You see no bark basket from here."
    );
    let quiet_events = world.observe(quiet).unwrap();
    assert!(
        quiet_events.events.iter().all(|event| {
            event.text == "You are not carrying a bark basket."
                || (!event.text.contains("fish") && !event.text.contains("basket"))
        }),
        "{:?}",
        quiet_events
            .events
            .iter()
            .map(|event| event.text.clone())
            .collect::<Vec<_>>()
    );

    let dropped = world.step(&[cmd("drop-branch", near, Attempt::Drop(Material::Branch))]);
    assert!(dropped.results[0].success, "{}", dropped.results[0].message);
    let out = world.step(&[cmd("drop-fish", near, Attempt::Drop(Material::Fish))]);
    assert!(out.results[0].success, "{}", out.results[0].message);
    let empty = world.examine(near, ExamineTarget::Basket);
    assert!(empty.contains("an empty bark basket"), "{empty}");
    assert!(!empty.contains("It is empty."), "{empty}");
    let gathered = world.step(&[cmd("gather-fish", near, Attempt::Gather(Material::Fish))]);
    assert_eq!(gathered.results[0].message, "You gather a fish.");
    let in_arms = world.observe(near).unwrap().text;
    assert!(
        in_arms.contains("a raw fish") && !in_arms.contains("holding a raw fish"),
        "{in_arms}"
    );
    let put = world.step(&[cmd("put", near, Attempt::PutFish)]);
    assert_eq!(
        put.results[0].message,
        "You put the raw fish in the bark basket."
    );
    let again = world.observe(near).unwrap().text;
    assert!(
        again.contains("a bark basket holding a raw fish"),
        "{again}"
    );
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "You put the raw fish in the bark basket."));
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Near puts the raw fish in the bark basket."));
    let after_put = world.observe(quiet).unwrap();
    assert!(
        after_put.events.iter().all(|event| {
            event.text == "You are not carrying a bark basket."
                || (!event.text.contains("puts") && !event.text.contains("basket"))
        }),
        "{:?}",
        after_put
            .events
            .iter()
            .map(|event| event.text.clone())
            .collect::<Vec<_>>()
    );

    let clay = world.step(&[cmd("clay", near, Attempt::Gather(Material::Clay))]);
    assert!(clay.results[0].success, "{}", clay.results[0].message);
    let second = world.step(&[cmd("clay-2", near, Attempt::Gather(Material::Clay))]);
    assert!(second.results[0].success, "{}", second.results[0].message);
    let blocked = world.step(&[cmd("mint", near, Attempt::Gather(Material::Mint))]);
    assert_eq!(
        blocked.results[0].message,
        "Your arms are full. Drop something first."
    );
    let spilled = world.step(&[cmd("spill", near, Attempt::Drop(Material::Fish))]);
    assert!(spilled.results[0].success, "{}", spilled.results[0].message);
    let tucked = world.step(&[cmd("tuck", near, Attempt::Gather(Material::Fish))]);
    assert_eq!(
        tucked.results[0].message,
        "You gather a fish. It goes into the bark basket. The basket is full."
    );
    let tucked_look = world.observe(near).unwrap().text;
    assert!(
        tucked_look.contains("a full bark basket holding a raw fish"),
        "{tucked_look}"
    );
    assert_eq!(tucked_look.matches("raw fish").count(), 1, "{tucked_look}");
    assert!(world
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("arms are full")));
}

#[test]
fn full_arms_name_drop_and_the_refusal_stays_with_that_person() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 5, 2).unwrap();
    let seen = world.spawn_at("Seen", "s", 6, 2).unwrap();
    let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
    for i in 0..4 {
        let got = world.step(&[cmd(
            &format!("branch-{i}"),
            near,
            Attempt::Gather(Material::Branch),
        )]);
        assert!(got.results[0].success, "{}", got.results[0].message);
    }
    world.step(&[cmd("east", near, Attempt::Move(Dir::East))]);
    world.step(&[cmd("north", near, Attempt::Move(Dir::North))]);
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    let fifth = world.step(&[cmd("fifth", near, Attempt::Gather(Material::Branch))]);
    assert!(fifth.results[0].success, "{}", fifth.results[0].message);
    let sentence = "Your arms are full. Drop something first.";
    let gather = world.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    assert_eq!(gather.results[0].message, sentence);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != sentence));
    let weave = world.step(&[cmd("basket", near, Attempt::WeaveBasket)]);
    assert_eq!(weave.results[0].message, sentence);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != sentence));
    for witness in [seen, quiet] {
        assert!(
            world
                .observe(witness)
                .unwrap()
                .events
                .iter()
                .all(|event| { event.text != sentence && !event.text.contains("arms are full") }),
            "{:?}",
            world
                .observe(witness)
                .unwrap()
                .events
                .iter()
                .map(|event| event.text.clone())
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn a_walk_is_footsteps_one_cell_past_sight() {
    let mut world = World::valley(Tuning::evening(), 1);
    let near = world.spawn_at("Near", "n", 4, 3).unwrap();
    let seen = world.spawn_at("Seen", "s", 6, 3).unwrap();
    let east = world.spawn_at("East", "e", 2, 3).unwrap();
    let south = world.spawn_at("South", "u", 5, 0).unwrap();
    let quiet = world.spawn_at("Quiet", "q", 0, 0).unwrap();

    let walked = world.step(&[cmd("east", near, Attempt::Move(Dir::East))]);
    assert_eq!(walked.results[0].message, "You walk east.");
    assert_eq!(world.actor_cell(near), Some((5, 3)));
    let actor = world.observe(near).unwrap();
    assert!(actor
        .events
        .iter()
        .all(|event| event.text != "You walk east."));
    assert!(actor
        .events
        .iter()
        .all(|event| { !event.text.contains("Near walks") && !event.text.contains("footsteps") }));
    let eyes = world.observe(seen).unwrap();
    assert!(eyes
        .events
        .iter()
        .any(|event| event.text == "Near walks east."));
    assert!(eyes
        .events
        .iter()
        .all(|event| !event.text.contains("footsteps")));
    let to_the_east = world.observe(east).unwrap();
    assert!(to_the_east
        .events
        .iter()
        .any(|event| event.text == "You hear footsteps to the east."));
    assert!(to_the_east
        .events
        .iter()
        .all(|event| !event.text.contains("walks")));
    let to_the_south = world.observe(south).unwrap();
    assert!(to_the_south
        .events
        .iter()
        .any(|event| event.text == "You hear footsteps to the south."));
    assert!(to_the_south
        .events
        .iter()
        .all(|event| !event.text.contains("walks")));
    assert!(world
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| { !event.text.contains("footsteps") && !event.text.contains("walks") }));

    world.step(&[]);
    assert_eq!(
        world
            .observe(east)
            .unwrap()
            .events
            .iter()
            .filter(|event| event.text == "You hear footsteps to the east.")
            .count(),
        1
    );

    let onto = world.step(&[cmd("onto", near, Attempt::Move(Dir::East))]);
    assert_eq!(onto.results[0].message, "You walk east.");
    assert_eq!(world.actor_cell(near), Some((6, 3)));
    let arrived = world.observe(seen).unwrap();
    assert!(arrived
        .events
        .iter()
        .any(|event| event.text == "Near walks east."));
    assert!(arrived
        .events
        .iter()
        .all(|event| !event.text.contains("footsteps")));
    assert_eq!(
        world
            .observe(east)
            .unwrap()
            .events
            .iter()
            .filter(|event| event.text.contains("footsteps"))
            .count(),
        1
    );
    let south_lines: Vec<_> = world
        .observe(south)
        .unwrap()
        .events
        .iter()
        .filter(|event| event.text.contains("footsteps"))
        .map(|event| event.text.clone())
        .collect();
    assert_eq!(
        south_lines,
        vec![
            "You hear footsteps to the south.".to_string(),
            "You hear footsteps to the south and east.".to_string(),
        ]
    );
    let refused = world.step(&[cmd("no", near, Attempt::Move(Dir::East))]);
    assert_eq!(
        refused.results[0].message,
        "The valley does not go that way."
    );
    assert_eq!(
        world
            .observe(south)
            .unwrap()
            .events
            .iter()
            .filter(|event| event.text.contains("footsteps"))
            .count(),
        2
    );
    let actor_after = world.observe(near).unwrap();
    assert!(actor_after
        .events
        .iter()
        .all(|event| event.text != "You walk east."));
    assert!(actor_after
        .events
        .iter()
        .all(|event| { !event.text.contains("Near walks") && !event.text.contains("footsteps") }));
    assert!(world
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| { !event.text.contains("footsteps") && !event.text.contains("walks") }));
}

#[test]
fn a_small_fire_leaves_charcoal_when_it_goes_out() {
    let mut rain = world_at(5);
    let nils = rain.spawn_at("Nils", "n", 6, 1).unwrap();
    let seen = rain.spawn_at("Seen", "s", 5, 2).unwrap();
    let quiet = rain.spawn_at("Quiet", "q", 0, 4).unwrap();
    let missing = rain.step(&[cmd("none", nils, Attempt::Gather(Material::Charcoal))]);
    assert_eq!(
        missing.results[0].message,
        "There is no charcoal here to gather."
    );
    assert!(rain
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("charcoal")));
    rain.step(&[cmd("branch", nils, Attempt::Gather(Material::Branch))]);
    rain.step(&[cmd("bark", nils, Attempt::Gather(Material::Bark))]);
    rain.step(&[cmd("west", nils, Attempt::Move(Dir::West))]);
    rain.step(&[cmd("light", nils, Attempt::Light)]);
    assert!(rain
        .observe(nils)
        .unwrap()
        .text
        .contains("A small fire burns here."));
    rain.step(&[]);
    let doused = rain.observe(nils).unwrap();
    assert!(doused
        .events
        .iter()
        .any(|event| event.text.contains("Rain puts out the fire.")));
    assert!(doused.text.contains("A lump of charcoal lies here."));
    assert!(!doused.text.contains("A small fire burns here."));
    let named = rain.examine(nils, ExamineTarget::Charcoal);
    assert!(
        named.contains("A lump of charcoal lies")
            && named.contains("The fire left it. It is cold."),
        "{named}"
    );
    let from_sight = rain.examine(seen, ExamineTarget::Charcoal);
    assert!(
        from_sight.contains("The fire left it. It is cold."),
        "{from_sight}"
    );
    assert_eq!(
        rain.examine(quiet, ExamineTarget::Charcoal),
        "You see no charcoal from here."
    );
    assert!(rain
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("Rain puts out") && !event.text.contains("charcoal")));
    let taken = rain.step(&[cmd("take", nils, Attempt::Gather(Material::Charcoal))]);
    assert_eq!(taken.results[0].message, "You gather a lump of charcoal.");
    let held = rain.observe(nils).unwrap();
    assert!(!held.text.contains("A lump of charcoal lies here."));
    assert!(held.text.contains("a lump of charcoal"), "{}", held.text);
    assert!(rain
        .observe(seen)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Nils gathers a lump of charcoal."));
    assert!(rain
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("charcoal")));

    let mut fuel = world_at(1_000);
    let linde = fuel.spawn_at("Linde", "l", 6, 1).unwrap();
    let near = fuel.spawn_at("Near", "n", 4, 1).unwrap();
    let far = fuel.spawn_at("Far", "f", 0, 4).unwrap();
    let life = fuel.tuning.fire_ticks;
    fuel.step(&[cmd("light", linde, Attempt::Light)]);
    for _ in 1..life {
        fuel.step(&[]);
    }
    assert!(fuel
        .observe(linde)
        .unwrap()
        .text
        .contains("A small fire burns here."));
    assert!(!fuel.observe(linde).unwrap().text.contains("charcoal"));
    fuel.step(&[]);
    let cold = fuel.observe(linde).unwrap();
    assert!(cold
        .events
        .iter()
        .any(|event| event.text.contains("The fire burns down.")));
    assert!(cold.text.contains("A lump of charcoal lies here."));
    assert!(fuel
        .observe(near)
        .unwrap()
        .text
        .contains("A lump of charcoal lies there."));
    assert_eq!(
        fuel.examine(far, ExamineTarget::Charcoal),
        "You see no charcoal from here."
    );
    assert!(fuel
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("burns down") && !event.text.contains("charcoal")));
}

#[test]
fn a_lump_of_charcoal_marks_the_rock_and_rain_washes_an_open_one() {
    let mut bare = world_at(1_000);
    let near = bare.spawn_at("Near", "n", 6, 1).unwrap();
    let seen = bare.spawn_at("Seen", "s", 6, 2).unwrap();
    let quiet = bare.spawn_at("Quiet", "q", 0, 4).unwrap();
    let none = bare.step(&[cmd("none", near, Attempt::Mark)]);
    assert_eq!(none.results[0].message, "You have nothing to mark with.");
    assert!(bare
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "You have nothing to mark with."));
    assert!(bare
        .observe(seen)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("mark")));
    assert!(bare
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("mark")));

    let mut rock = world_at(1_000);
    let near = rock.spawn_at("Near", "n", 6, 1).unwrap();
    let seen = rock.spawn_at("Seen", "s", 6, 2).unwrap();
    let quiet = rock.spawn_at("Quiet", "q", 0, 4).unwrap();
    let life = rock.tuning.fire_ticks;
    rock.step(&[cmd("light", near, Attempt::Light)]);
    for _ in 1..life {
        rock.step(&[]);
    }
    rock.step(&[]);
    assert!(rock
        .observe(near)
        .unwrap()
        .text
        .contains("A lump of charcoal lies here."));
    rock.step(&[cmd("take", near, Attempt::Gather(Material::Charcoal))]);
    assert_eq!(rock.material_tally(Material::Charcoal), 1);
    let marked = rock.step(&[cmd("mark", near, Attempt::MarkRock)]);
    assert_eq!(
        marked.results[0].message,
        "You mark the rock with charcoal."
    );
    let yours = "You mark the rock with charcoal.";
    let witnessed = "Near marks the rock with charcoal.";
    assert!(rock
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != yours));
    assert!(rock
        .observe(seen)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == witnessed));
    assert!(rock
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("charcoal")));
    let here = rock.observe(near).unwrap().text;
    assert!(
        here.contains("A black charcoal mark is on the rock here."),
        "{here}"
    );
    let beside = rock.observe(seen).unwrap().text;
    assert!(
        beside.contains("A black charcoal mark is on the rock to the north."),
        "{beside}"
    );
    assert!(beside.contains("a black charcoal mark"), "{beside}");
    assert!(!rock.observe(quiet).unwrap().text.contains("charcoal mark"));
    let named = rock.examine(near, ExamineTarget::Charcoal);
    assert!(
        named.contains("A black charcoal mark is on the rock here."),
        "{named}"
    );
    assert!(!named.contains("A lump of charcoal"), "{named}");
    let from_sight = rock.examine(seen, ExamineTarget::Charcoal);
    assert!(
        from_sight.contains("A black charcoal mark is on the rock to the north."),
        "{from_sight}"
    );
    assert_eq!(
        rock.examine(quiet, ExamineTarget::Charcoal),
        "You see no charcoal from here."
    );
    assert_eq!(rock.material_tally(Material::Charcoal), 0);
    let again = rock.step(&[cmd("again", near, Attempt::Mark)]);
    assert!(!again.results[0].success);
    assert_eq!(again.results[0].message, "A black mark is already here.");
    rock.step(&[cmd("west", near, Attempt::Move(Dir::West))]);
    let without = rock.step(&[cmd("empty", near, Attempt::Mark)]);
    assert_eq!(without.results[0].message, "You have nothing to mark with.");
    assert!(rock
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "You have nothing to mark with."));
    assert!(rock
        .observe(seen)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != "You have nothing to mark with."));
    assert!(rock
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != "You have nothing to mark with."));
    assert!(rock
        .observe(near)
        .unwrap()
        .text
        .contains("A black charcoal mark is on the rock to the east."));

    let mut rain = world_at(16);
    rain.tuning.fire_ticks = 1;
    let near = rain.spawn_at("Near", "n", 6, 1).unwrap();
    let seen = rain.spawn_at("Seen", "s", 5, 0).unwrap();
    let quiet = rain.spawn_at("Quiet", "q", 0, 4).unwrap();
    rain.step(&[cmd("branch", near, Attempt::Gather(Material::Branch))]);
    rain.step(&[cmd("bark", near, Attempt::Gather(Material::Bark))]);
    rain.step(&[cmd("west", near, Attempt::Move(Dir::West))]);
    assert_eq!(rain.actor_cell(near), Some((5, 1)));
    rain.step(&[cmd("light", near, Attempt::Light)]);
    for _ in 0..4 {
        if rain
            .observe(near)
            .unwrap()
            .text
            .contains("A lump of charcoal lies here.")
        {
            break;
        }
        rain.step(&[]);
    }
    rain.step(&[cmd("take", near, Attempt::Gather(Material::Charcoal))]);
    let off = rain.step(&[cmd("rock", near, Attempt::MarkRock)]);
    assert_eq!(off.results[0].message, "There is no rock here to mark.");
    assert_eq!(rain.material_tally(Material::Charcoal), 1);
    let marked = rain.step(&[cmd("mark", near, Attempt::Mark)]);
    assert_eq!(
        marked.results[0].message,
        "You mark the ground with charcoal."
    );
    assert!(rain
        .observe(near)
        .unwrap()
        .text
        .contains("A black charcoal mark is on the ground here."));
    assert!(rain
        .observe(seen)
        .unwrap()
        .text
        .contains("A black charcoal mark is on the ground on the valley floor to the south."));
    assert!(rain
        .observe(seen)
        .unwrap()
        .text
        .contains("a black charcoal mark"));
    let named = rain.examine(near, ExamineTarget::Charcoal);
    assert!(
        named.contains("A black charcoal mark is on the ground here, on the valley floor."),
        "{named}"
    );
    while rain.sim_time() < 16 {
        rain.step(&[]);
    }
    assert!(!rain.is_raining());
    assert!(rain
        .observe(near)
        .unwrap()
        .text
        .contains("A black charcoal mark is on the ground here."));
    rain.step(&[]);
    assert!(rain.is_raining());
    let washed = "The rain washes the black charcoal mark off the ground on the valley floor.";
    assert!(rain
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == washed));
    assert!(rain
        .observe(seen)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == washed));
    assert!(rain
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("charcoal mark")));
    assert!(!rain.observe(near).unwrap().text.contains("charcoal mark"));
    assert!(!rain.observe(seen).unwrap().text.contains("charcoal mark"));
}

#[test]
fn a_lump_of_charcoal_feeds_a_fire_hot_enough_for_iron() {
    let mut world = world_at(1_000);
    let near = world.spawn_at("Near", "n", 6, 1).unwrap();
    let seen = world.spawn_at("Seen", "s", 4, 1).unwrap();
    let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
    let cold = world.step(&[cmd("none", near, Attempt::Feed)]);
    assert_eq!(cold.results[0].message, "There is no fire burning here.");
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "There is no fire burning here."));
    for id in [seen, quiet] {
        assert!(world
            .observe(id)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("fire burning")));
    }

    let life = world.tuning.fire_ticks;
    world.step(&[cmd("light", near, Attempt::Light)]);
    for _ in 1..life {
        world.step(&[]);
    }
    world.step(&[]);
    world.step(&[cmd("take", near, Attempt::Gather(Material::Charcoal))]);
    assert_eq!(world.material_tally(Material::Charcoal), 1);
    let unlit = world.step(&[cmd("unlit", near, Attempt::Feed)]);
    assert_eq!(unlit.results[0].message, "There is no fire burning here.");
    assert_eq!(world.material_tally(Material::Charcoal), 1);
    assert!(world
        .observe(seen)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != "There is no fire burning here."));

    world.step(&[cmd("aside", near, Attempt::Move(Dir::West))]);
    world.step(&[cmd("down", near, Attempt::Drop(Material::Charcoal))]);
    world.step(&[cmd("back", near, Attempt::Move(Dir::East))]);
    world.step(&[cmd("light-2", near, Attempt::Light)]);
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains("A small fire burns here."));
    world.step(&[cmd("to-lump", near, Attempt::Move(Dir::West))]);
    world.step(&[cmd("lift", near, Attempt::Gather(Material::Charcoal))]);
    world.step(&[cmd("to-fire", near, Attempt::Move(Dir::East))]);
    let branches = world.material_tally(Material::Branch);
    let fed = "You feed a lump of charcoal into the fire. It burns hotter.";
    let seen_feed = "Near feeds a lump of charcoal into the fire. It burns hotter.";
    let feed = world.step(&[cmd("feed", near, Attempt::Feed)]);
    assert_eq!(feed.results[0].message, fed);
    assert_eq!(world.material_tally(Material::Charcoal), 0);
    assert_eq!(world.material_tally(Material::Branch), branches);
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == fed));
    assert!(world
        .observe(seen)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == seen_feed));
    assert!(world
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("feed") && !event.text.contains("hotter")));
    let hotter = world.observe(near).unwrap().text;
    assert!(
        hotter.contains("A hot charcoal fire burns here. It warms you."),
        "{hotter}"
    );
    assert!(!hotter.contains("A small fire burns here"), "{hotter}");
    let from_seen = world.observe(seen).unwrap().text;
    assert!(
        from_seen.contains("A hot charcoal fire burns there."),
        "{from_seen}"
    );
    assert!(!world.observe(quiet).unwrap().text.contains("burns hotter"));
    assert!(
        world
            .examine(near, ExamineTarget::Fire)
            .starts_with("A hot charcoal fire burns here,"),
        "{}",
        world.examine(near, ExamineTarget::Fire)
    );
    let again = world.step(&[cmd("again", near, Attempt::Feed)]);
    assert_eq!(
        again.results[0].message,
        "The fire is already hot enough for iron."
    );
    assert!(world
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("hot enough")));
}

#[test]
fn reeds_thatch_a_lean_to_and_put_the_body_out_of_the_wind() {
    let mut world = world_at(1_000);
    let near = world.spawn_at("Near", "n", 5, 2).unwrap();
    let seen = world.spawn_at("Seen", "s", 5, 3).unwrap();
    let bank = world.spawn_at("Bank", "b", 0, 4).unwrap();
    let quiet = world.spawn_at("Quiet", "q", 0, 0).unwrap();
    assert!(world
        .observe(near)
        .unwrap()
        .text
        .contains("You are in the wind."));
    assert!(!world.observe(near).unwrap().text.contains("little wind"));
    assert_eq!(parse_attempt("cover lean-to").unwrap(), Attempt::Thatch);
    assert_eq!(parse_attempt("cover fire").unwrap(), Attempt::Bank);
    let bare = world.step(&[cmd("none", near, Attempt::Thatch)]);
    assert_eq!(
        bare.results[0].message,
        "There is no lean-to here to thatch."
    );
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .any(|event| { event.text == "There is no lean-to here to thatch." }));
    assert!(world
        .observe(seen)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("thatch")));
    assert!(world
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("thatch")));

    let built = world.step(&[cmd("build", near, Attempt::BuildLeanTo)]);
    assert_eq!(
        built.results[0].message,
        "You build a lean-to of three branches."
    );
    let after_build = world.observe(near).unwrap().text;
    assert!(
        after_build.contains("You are in a little wind."),
        "{after_build}"
    );
    assert!(after_build.contains("A lean-to of branches stands here."));
    let no_reeds = world.step(&[cmd("empty", near, Attempt::Thatch)]);
    assert_eq!(
        no_reeds.results[0].message,
        "You have no reeds to thatch with."
    );
    assert!(world
        .observe(seen)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != "You have no reeds to thatch with."));
    assert!(world
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("reeds")));

    for (id, dir) in [
        ("w1", Dir::West),
        ("w2", Dir::West),
        ("w3", Dir::West),
        ("w4", Dir::West),
        ("w5", Dir::West),
        ("s", Dir::South),
    ] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    assert_eq!(world.actor_cell(near), Some((0, 3)));
    let growing = world.observe(near).unwrap().text;
    assert!(
        growing.contains("Reeds grow here, at the brook's edge."),
        "{growing}"
    );
    let gathered = world.step(&[cmd("reeds", near, Attempt::Gather(Material::Reed))]);
    assert_eq!(gathered.results[0].message, "You gather a bundle of reeds.");
    assert!(world
        .observe(near)
        .unwrap()
        .events
        .iter()
        .all(|event| event.text != "You gather a bundle of reeds."));
    assert!(world
        .observe(bank)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Near gathers a bundle of reeds."));
    assert!(world
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("reeds")));
    assert_eq!(world.material_tally(Material::Reed), 4);

    for (id, dir) in [
        ("n", Dir::North),
        ("e1", Dir::East),
        ("e2", Dir::East),
        ("e3", Dir::East),
        ("e4", Dir::East),
        ("e5", Dir::East),
    ] {
        world.step(&[cmd(id, near, Attempt::Move(dir))]);
    }
    assert_eq!(world.actor_cell(near), Some((5, 2)));
    let thatched = world.step(&[cmd("thatch", near, Attempt::Thatch)]);
    assert_eq!(
        thatched.results[0].message,
        "You thatch the lean-to with reeds."
    );
    assert_eq!(world.material_tally(Material::Reed), 3);
    let here = world.observe(near).unwrap();
    assert!(here.events.iter().all(|event| {
        event.text != "You thatch the lean-to with reeds."
            && event.text != "Near thatches the lean-to with reeds."
    }));
    assert!(
        here.text.contains("A thatched lean-to stands here."),
        "{}",
        here.text
    );
    assert!(
        here.text.contains("You are out of the wind."),
        "{}",
        here.text
    );
    assert!(!here.text.contains("little wind"), "{}", here.text);
    assert!(world
        .observe(seen)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "Near thatches the lean-to with reeds."));
    let from_seen = world.observe(seen).unwrap().text;
    assert!(
        from_seen.contains("A thatched lean-to stands on the valley floor to the north."),
        "{from_seen}"
    );
    assert!(world
        .observe(quiet)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("thatch")));
    assert!(!world.observe(quiet).unwrap().text.contains("thatched"));
    let again = world.step(&[cmd("again", near, Attempt::Thatch)]);
    assert_eq!(again.results[0].message, "The lean-to is already thatched.");
}

#[test]
fn a_thatched_lean_to_keeps_the_rain_off_and_a_bare_one_does_not() {
    let mut world = world_at(28);
    let thatch = world.spawn_at("Thatch", "t", 0, 3).unwrap();
    let bare = world.spawn_at("Bare", "b", 6, 1).unwrap();
    let open = world.spawn_at("Open", "o", 4, 3).unwrap();
    let seen = world.spawn_at("Seen", "s", 3, 2).unwrap();
    let rock = world.spawn_at("Rock", "r", 6, 1).unwrap();

    let go = |world: &mut World, id: &str, actor: u32, attempt: Attempt| {
        let step = world.step(&[cmd(id, actor, attempt)]);
        assert!(step.results[0].success, "{}", step.results[0].message);
    };

    go(&mut world, "reeds", thatch, Attempt::Gather(Material::Reed));
    for (n, dir) in [
        (1, Dir::East),
        (2, Dir::East),
        (3, Dir::East),
        (4, Dir::East),
        (5, Dir::East),
        (6, Dir::North),
    ] {
        go(&mut world, &format!("t{n}"), thatch, Attempt::Move(dir));
    }
    assert_eq!(world.actor_cell(thatch), Some((5, 2)));
    go(&mut world, "build-t", thatch, Attempt::BuildLeanTo);
    go(&mut world, "thatch", thatch, Attempt::Thatch);

    for n in 1..=3 {
        go(
            &mut world,
            &format!("wood{n}"),
            bare,
            Attempt::Gather(Material::Branch),
        );
    }
    go(&mut world, "bw1", bare, Attempt::Move(Dir::West));
    go(&mut world, "bw2", bare, Attempt::Move(Dir::West));
    go(&mut world, "bs", bare, Attempt::Move(Dir::South));
    assert_eq!(world.actor_cell(bare), Some((4, 2)));
    go(&mut world, "build-b", bare, Attempt::BuildLeanTo);
    assert!(!world.is_raining());

    let thatch_path = [Dir::West, Dir::West, Dir::West, Dir::North];
    let bare_path = [Dir::North, Dir::West, Dir::West];
    let open_path = [Dir::West, Dir::West, Dir::North, Dir::North];
    for (n, dir) in thatch_path.into_iter().enumerate() {
        let mut intents = vec![cmd(&format!("to-mud-t{n}"), thatch, Attempt::Move(dir))];
        if n < bare_path.len() {
            intents.push(cmd(
                &format!("to-mud-b{n}"),
                bare,
                Attempt::Move(bare_path[n]),
            ));
        }
        if n < open_path.len() {
            intents.push(cmd(
                &format!("to-mud-o{n}"),
                open,
                Attempt::Move(open_path[n]),
            ));
        }
        let step = world.step(&intents);
        for result in &step.results {
            assert!(result.success, "{}", result.message);
        }
    }
    assert_eq!(world.actor_cell(thatch), Some((2, 1)));
    assert_eq!(world.actor_cell(bare), Some((2, 1)));
    assert_eq!(world.actor_cell(open), Some((2, 1)));
    let gathered = world.step(&[
        cmd("mud-t", thatch, Attempt::Gather(Material::Mud)),
        cmd("mud-b", bare, Attempt::Gather(Material::Mud)),
        cmd("mud-o", open, Attempt::Gather(Material::Mud)),
    ]);
    for result in &gathered.results {
        assert_eq!(result.message, "You gather a handful of mud.");
    }
    let smeared = world.step(&[
        cmd("smear-t", thatch, Attempt::Smear),
        cmd("smear-b", bare, Attempt::Smear),
        cmd("smear-o", open, Attempt::Smear),
    ]);
    for result in &smeared.results {
        assert_eq!(result.message, "You smear mud on your skin.");
    }
    let back_t = [Dir::South, Dir::East, Dir::East, Dir::East];
    let back_b = [Dir::East, Dir::East, Dir::South];
    let back_o = [Dir::South, Dir::South, Dir::East, Dir::East];
    for (n, dir) in back_t.into_iter().enumerate() {
        let mut intents = vec![cmd(&format!("back-t{n}"), thatch, Attempt::Move(dir))];
        if n < back_b.len() {
            intents.push(cmd(&format!("back-b{n}"), bare, Attempt::Move(back_b[n])));
        }
        if n < back_o.len() {
            intents.push(cmd(&format!("back-o{n}"), open, Attempt::Move(back_o[n])));
        }
        let step = world.step(&intents);
        for result in &step.results {
            assert!(result.success, "{}", result.message);
        }
    }
    assert_eq!(world.actor_cell(thatch), Some((5, 2)));
    assert_eq!(world.actor_cell(bare), Some((4, 2)));
    assert_eq!(world.actor_cell(open), Some((4, 3)));
    assert_eq!(world.actor_cell(seen), Some((3, 2)));
    assert_eq!(world.actor_cell(rock), Some((6, 1)));
    assert_eq!(world.observe(thatch).unwrap().self_info.mud, "mud-smeared");
    assert_eq!(world.observe(bare).unwrap().self_info.mud, "mud-smeared");
    assert_eq!(world.observe(open).unwrap().self_info.mud, "mud-smeared");
    world.wet_season = false;
    while !world.is_raining() {
        world.step(&[]);
    }
    world.step(&[]);

    let under_thatch = world.observe(thatch).unwrap();
    let under_bare = world.observe(bare).unwrap();
    let in_rain = world.observe(open).unwrap();
    let under_rock = world.observe(rock).unwrap();
    let from_seen = world.observe(seen).unwrap();

    assert!(
        under_thatch
            .text
            .contains("You are on the valley floor, under a thatched lean-to."),
        "{}",
        under_thatch.text
    );
    assert!(
        under_thatch
            .text
            .contains("The rain does not wet your clothes."),
        "{}",
        under_thatch.text
    );
    assert!(
        under_thatch
            .text
            .contains("The rain does not reach you under it, and it does not wash you."),
        "{}",
        under_thatch.text
    );
    assert_eq!(under_thatch.self_info.wetness, "dry");
    assert_eq!(under_thatch.self_info.mud, "mud-smeared");
    assert!(under_thatch.events.iter().all(|event| {
        !event.text.contains("washes the mud off you")
            && !event.text.contains("washes the mud off Thatch")
    }));
    assert!(under_thatch
        .events
        .iter()
        .any(|event| event.text == "The rain washes the mud off Open."));

    assert!(
        under_bare
            .text
            .contains("You are on the valley floor, under a lean-to."),
        "{}",
        under_bare.text
    );
    assert!(!under_bare.text.contains("under a thatched lean-to"));
    assert!(
        under_bare
            .text
            .contains("A little rain dampens your clothes."),
        "{}",
        under_bare.text
    );
    assert!(
        under_bare
            .text
            .contains("Only a little rain reaches you under it, and it does not wash you."),
        "{}",
        under_bare.text
    );
    assert_eq!(under_bare.self_info.wetness, "damp");
    assert_eq!(under_bare.self_info.mud, "mud-smeared");
    assert!(under_bare.events.iter().all(|event| {
        !event.text.contains("washes the mud off you")
            && !event.text.contains("washes the mud off Bare")
    }));
    assert!(under_bare
        .events
        .iter()
        .any(|event| event.text == "The rain washes the mud off Open."));

    assert!(
        in_rain
            .text
            .contains("You are on the valley floor, in the rain."),
        "{}",
        in_rain.text
    );
    assert!(
        in_rain.text.contains("The rain wets your clothes."),
        "{}",
        in_rain.text
    );
    assert_eq!(in_rain.self_info.wetness, "wet");
    assert_eq!(in_rain.self_info.mud, "bare skin");
    assert!(in_rain
        .events
        .iter()
        .any(|event| event.text == "The rain washes the mud off you."));

    assert!(
        under_rock
            .text
            .contains("The rain does not wet your clothes."),
        "{}",
        under_rock.text
    );
    assert_eq!(under_thatch.self_info.wetness, under_rock.self_info.wetness);

    assert!(
        from_seen.text.contains(
            "A thatched lean-to stands on the valley floor to the east. The rain does not reach anyone under it."
        ),
        "{}",
        from_seen.text
    );
    assert!(
        from_seen.text.contains(
            "A lean-to of branches stands on the valley floor to the east. The rain does not wash anyone under it."
        ),
        "{}",
        from_seen.text
    );
    assert!(from_seen
        .events
        .iter()
        .any(|event| event.text == "The rain washes the mud off Open."));
    assert!(from_seen.events.iter().all(|event| {
        !event.text.contains("washes the mud off Thatch")
            && !event.text.contains("washes the mud off Bare")
    }));
    assert_eq!(world.actor_cell(seen), Some((3, 2)));
}

#[test]
fn a_thatched_lean_to_keeps_the_cold_off_like_the_rock() {
    let mut world = World::valley(Tuning::evening(), 1);
    let thatch = world.spawn_at("Thatch", "t", 5, 2).unwrap();
    let bare = world.spawn_at("Bare", "b", 6, 1).unwrap();
    let open = world.spawn_at("Open", "o", 4, 4).unwrap();
    let rock = world.spawn_at("Rock", "r", 6, 1).unwrap();
    let seen = world.spawn_at("Seen", "s", 3, 4).unwrap();

    let go = |world: &mut World, id: &str, actor: u32, attempt: Attempt| {
        let step = world.step(&[cmd(id, actor, attempt)]);
        assert!(
            step.results[0].success,
            "{}: {}",
            step.results[0].command_id, step.results[0].message
        );
    };
    go(&mut world, "build-t", thatch, Attempt::BuildLeanTo);
    for n in 1..=3 {
        go(
            &mut world,
            &format!("wood-{n}"),
            bare,
            Attempt::Gather(Material::Branch),
        );
    }
    go(&mut world, "bw1", bare, Attempt::Move(Dir::West));
    go(&mut world, "bw2", bare, Attempt::Move(Dir::West));
    go(&mut world, "bs", bare, Attempt::Move(Dir::South));
    assert_eq!(world.actor_cell(bare), Some((4, 2)));
    go(&mut world, "build-b", bare, Attempt::BuildLeanTo);
    go(&mut world, "south", thatch, Attempt::Move(Dir::South));
    for n in 1..=5 {
        go(
            &mut world,
            &format!("west-{n}"),
            thatch,
            Attempt::Move(Dir::West),
        );
    }
    assert_eq!(world.actor_cell(thatch), Some((0, 3)));
    go(&mut world, "reeds", thatch, Attempt::Gather(Material::Reed));
    for n in 1..=5 {
        go(
            &mut world,
            &format!("east-{n}"),
            thatch,
            Attempt::Move(Dir::East),
        );
    }
    go(&mut world, "north", thatch, Attempt::Move(Dir::North));
    assert_eq!(world.actor_cell(thatch), Some((5, 2)));
    go(&mut world, "thatch", thatch, Attempt::Thatch);
    assert!(world.sim_time() < world.tuning.night_at);

    while world.observe(bare).unwrap().self_info.cold != "chilly" {
        world.step(&[]);
        assert!(
            world.sim_time() < world.tuning.night_at,
            "bare lean-to should chill before night"
        );
    }

    let day_thatch = world.observe(thatch).unwrap();
    let day_bare = world.observe(bare).unwrap();
    let day_open = world.observe(open).unwrap();
    let day_rock = world.observe(rock).unwrap();
    assert!(
        day_thatch.text.contains("You are warm.")
            && day_thatch.text.contains("You are out of the wind."),
        "{}",
        day_thatch.text
    );
    assert_eq!(day_thatch.self_info.cold, "warm");
    assert_eq!(day_thatch.self_info.wind, "out of the wind");
    assert!(
        day_rock.text.contains("You are warm.")
            && day_rock.text.contains("You are out of the wind."),
        "{}",
        day_rock.text
    );
    assert_eq!(day_thatch.self_info.cold, day_rock.self_info.cold);
    assert_eq!(day_thatch.self_info.wind, day_rock.self_info.wind);
    assert!(
        day_bare.text.contains("You are chilly.")
            && day_bare.text.contains("You are in a little wind.")
            && !day_bare.text.contains("You are cold."),
        "{}",
        day_bare.text
    );
    assert!(
        day_open.text.contains("You are cold.") && day_open.text.contains("You are in the wind."),
        "{}",
        day_open.text
    );
    assert!(day_thatch
        .events
        .iter()
        .any(|event| event.text == "Bare is chilly."));
    assert!(day_thatch
        .events
        .iter()
        .all(|event| event.text != "You are chilly."));
    assert!(day_bare
        .events
        .iter()
        .any(|event| event.text == "You are chilly."));
    assert!(day_bare
        .events
        .iter()
        .all(|event| event.text != "Bare is chilly."));
    let saw_thatch = world.examine(seen, ExamineTarget::Person("Thatch".into()));
    let saw_bare = world.examine(seen, ExamineTarget::Person("Bare".into()));
    assert!(
        saw_thatch.contains("They are warm.") && saw_thatch.contains("They are out of the wind."),
        "{saw_thatch}"
    );
    assert!(
        saw_bare.contains("They are chilly.") && saw_bare.contains("They are in a little wind."),
        "{saw_bare}"
    );
    assert!(!world.observe(seen).unwrap().text.contains("You are warm."));

    let night_end = world.tuning.night_at + world.tuning.night_len - 1;
    while world.sim_time() < night_end {
        world.step(&[]);
    }
    assert_eq!(world.sim_time(), night_end);
    let night_thatch = world.observe(thatch).unwrap();
    let night_bare = world.observe(bare).unwrap();
    let night_open = world.observe(open).unwrap();
    let night_rock = world.observe(rock).unwrap();
    assert!(
        night_thatch.text.contains("Night has fallen.")
            && night_thatch.text.contains("You are warm.")
            && night_thatch.text.contains("You are out of the wind."),
        "{}",
        night_thatch.text
    );
    assert!(
        night_rock.text.contains("Night has fallen.")
            && night_rock.text.contains("You are warm.")
            && night_rock.text.contains("You are out of the wind."),
        "{}",
        night_rock.text
    );
    assert_eq!(night_thatch.self_info.cold, night_rock.self_info.cold);
    assert_eq!(night_thatch.self_info.wind, night_rock.self_info.wind);
    assert!(
        night_bare.text.contains("Night has fallen.")
            && night_bare.text.contains("You are cold.")
            && night_bare.text.contains("You are in a little wind.")
            && !night_bare.text.contains("You are warm."),
        "{}",
        night_bare.text
    );
    assert!(
        night_open.text.contains("Night has fallen.")
            && night_open.text.contains("You are cold.")
            && night_open.text.contains("You are in the wind."),
        "{}",
        night_open.text
    );
    assert!(night_thatch.events.iter().all(|event| {
        event.text != "You are cold." && event.text != "You are cold. The wind is on you."
    }));
    let night_saw = world.examine(seen, ExamineTarget::Person("Thatch".into()));
    let night_bare_saw = world.examine(seen, ExamineTarget::Person("Bare".into()));
    assert!(
        night_saw.contains("They are warm.") && night_saw.contains("They are out of the wind."),
        "{night_saw}"
    );
    assert!(
        night_bare_saw.contains("They are cold.")
            && night_bare_saw.contains("They are in a little wind."),
        "{night_bare_saw}"
    );
}
