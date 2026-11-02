use villmarks_sim::{
    contest_hash, parse_attempt, Attempt, Dir, ExamineTarget, Intent, Material, Tuning, World,
};

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
    assert_eq!(world.material_tally(Material::Branch), 4);
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
    assert!(eased
        .events
        .iter()
        .any(|event| event.text.contains("drinks from the brook")));
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
        .any(|event| event.text.contains("no fire")));
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
        1,
        "the deer eats the pile; the carried handful remains"
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
    assert!(lit
        .events
        .iter()
        .any(|event| event.text.contains("lights a fire with bark")));
    assert_eq!(world.material_tally(Material::Bark), bark - 1);
    assert_eq!(
        world.material_tally(Material::Branch),
        branches_before_light - 1
    );
    assert_eq!(world.material_tally(Material::Berry), 1);
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
    assert!(eased
        .events
        .iter()
        .any(|event| event.text.contains("cooks a handful of berries")));
    assert_eq!(world.material_tally(Material::Berry), 0);
    assert_eq!(world.material_tally(Material::Bark), bark - 1);
    assert!(world
        .observe(otto)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("cooks a handful of berries")));

    world.step(&[cmd("meal", sera, Attempt::Cook)]);
    assert_eq!(world.material_tally(Material::Berry), 0);

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
    slope.step(&[cmd("damp", sera, Attempt::Light)]);
    let seen: Vec<_> = slope
        .observe(sera)
        .unwrap()
        .events
        .iter()
        .map(|event| event.text.clone())
        .collect();
    assert!(
        seen.iter().any(|text| text.contains("too damp")),
        "{seen:?}"
    );
    assert_eq!(slope.material_tally(Material::Bark), bark);

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
    brief.step(&[cmd("light", linde, Attempt::Light)]);
    for _ in 0..7 {
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
        parse_attempt("light fire").unwrap(),
        Attempt::Light
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
        parse_attempt("gather berries").unwrap(),
        Attempt::Gather(Material::Berry)
    ));
    assert!(parse_attempt("eat").unwrap_err().contains("tinder"));
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
    world.step(&[cmd("pot", sera, Attempt::Boil)]);
    let boiled = world.observe(sera).unwrap();
    assert_eq!(boiled.self_info.illness, "well");
    assert_eq!(boiled.self_info.thirst, "not thirsty");
    assert!(boiled
        .events
        .iter()
        .any(|event| event.text.contains("boils a drink of water")));
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
fn a_deer_walks_without_a_command_eats_berries_and_leaves_hoofprints() {
    let mut near = world_at(10_000);
    let sera = near.spawn_at("Sera", "s", 4, 3).unwrap();
    let otto = near.spawn_at("Otto", "o", 0, 0).unwrap();
    assert_eq!(near.deer_cell(), Some((2, 3)));
    assert!(!near.hoofprints_at(2, 3));
    let before = near.observe(sera).unwrap();
    assert!(before.map.contains('d'));
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
    assert_eq!(near.material_tally(Material::Berry), 0);
    assert_eq!(near.deer_cell(), Some((6, 1)));
    near.step(&[]);
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
    assert!(before.contains("damp"), "{before}");
    assert!(!before.contains("soaked"), "{before}");
    assert!(before.contains("dry"), "{before}");
    assert!(!before.contains("heavier"));
    assert!(!world.observe(sera).unwrap().text.contains("heavier"));
    assert_eq!(
        world.examine(otto, ExamineTarget::Wood),
        "You see no wood from here."
    );

    world.step(&[]);
    assert!(world.wet_season);
    let after = world.examine(sera, ExamineTarget::Wood);
    assert!(after.contains("soaked"), "{after}");
    assert!(after.contains("dry"), "{after}");
    assert!(
        after.contains("The rain is heavier on it this season."),
        "{after}"
    );
    assert!(after.contains("Rain does not reach it."));
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
    assert!(world
        .observe(sera)
        .unwrap()
        .text
        .contains("Wood left in the open is rotting."));
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
    assert_eq!(world.material_tally(Material::Branch), 8);
    world.step(&[]);
    assert_eq!(world.material_tally(Material::Branch), 7);
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
    assert_eq!(open.branch_pile, 3);
    assert_eq!(rock.branch_pile, 4);
    assert!(seen
        .events
        .iter()
        .any(|event| event.text == "A branch rots away in the open."));
    let far = world.observe(otto).unwrap();
    assert!(far.events.iter().all(|event| !event.text.contains("rots")));
    assert!(!far.text.contains("rotting"));

    while world.sim_time() < 33 {
        world.step(&[]);
    }
    assert_eq!(world.material_tally(Material::Branch), 4);
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
        4
    );
    assert!(!gone.text.contains("Wood left in the open is rotting."));
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
