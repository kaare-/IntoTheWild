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
    assert_eq!(world.material_tally(Material::Branch), 3);
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
    assert!(lit.events.iter().any(|event| {
        event
            .text
            .contains("lights a fire with bark and a dry branch from the pile at the feet")
    }));
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
        parse_attempt("eat mushrooms").unwrap(),
        Attempt::Eat(Material::Mushroom)
    ));
    assert!(matches!(
        parse_attempt("eat carcass").unwrap(),
        Attempt::Eat(Material::Carcass)
    ));
    assert!(parse_attempt("eat bark").unwrap_err().contains("tinder"));
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
    assert_eq!(near.material_tally(Material::Berry), 0);
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
            .filter(|text| *text == "A deer eats the last handful of berries.")
            .count(),
        1
    );
    assert!(near
        .observe(sera)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text == "A deer eats the last handful of berries."));
    assert!(near
        .observe(otto)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("handful")));
    let witness = near.spawn_at("Witness", "w", 6, 1).unwrap();
    assert!(!near
        .observe(witness)
        .unwrap()
        .text
        .contains("Berries lie here"));
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
        .contains("Wood left in the open is rotting one cell north and one cell east."));
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
            "A pile of 2 branches is in contact with the ground one cell north and two cells east, in the open."
        ),
        "{from_mid}"
    );
    assert!(!from_mid.contains("to the north and east"), "{from_mid}");
    let from_rock = world.examine(under, ExamineTarget::Wood);
    assert!(
        from_rock.contains(
            "A pile of 2 branches is in contact with the ground one cell south and one cell west, in the open."
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
        .contains("Wood left in the open is rotting one cell north and two cells east."));
    assert!(world
        .observe(under)
        .unwrap()
        .text
        .contains("Wood left in the open is rotting one cell south and one cell west."));
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
        .any(|event| { event.text == "Nils gathers a branch. You hear branches breaking." }));
    assert!(taken
        .observe(far)
        .unwrap()
        .events
        .iter()
        .all(|event| !event.text.contains("branches breaking")));
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
    pool.step(&[cmd("pool", sera, Attempt::Drink)]);
    let ill = pool.observe(sera).unwrap();
    assert_eq!(ill.self_info.illness, "ill");
    assert_eq!(ill.self_info.thirst, "not thirsty");
    assert!(ill
        .events
        .iter()
        .any(|event| event.text.contains("drinks the still water")));
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
    running.step(&[cmd("run", sera, Attempt::Drink)]);
    let well = running.observe(sera).unwrap();
    assert_eq!(well.self_info.illness, "well");
    assert!(well
        .events
        .iter()
        .any(|event| event.text.contains("drinks from the brook")));

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
fn sweet_berries_give_energy_and_the_other_food_must_be_cooked() {
    let mut world = world_at(10_000);
    let sera = world.spawn_at("Sera", "s", 6, 1).unwrap();
    let nils = world.spawn_at("Nils", "n", 3, 1).unwrap();
    let linde = world.spawn_at("Linde", "l", 4, 1).unwrap();
    assert!(world
        .examine(sera, ExamineTarget::Berries)
        .contains("They are good to eat raw."));
    assert!(world
        .examine(nils, ExamineTarget::Mushrooms)
        .contains("Raw mushrooms do not feed you."));
    assert!(world
        .examine(linde, ExamineTarget::BitterBerries)
        .contains("They are poisonous."));
    assert_eq!(world.sim_time(), 0);

    world.step(&[cmd("raw-mushroom", nils, Attempt::Eat(Material::Mushroom))]);
    assert_eq!(world.material_tally(Material::Mushroom), 2);
    assert!(world
        .observe(nils)
        .unwrap()
        .events
        .iter()
        .any(|event| { event.text.contains("Raw mushrooms do not feed you.") }));
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
    world.step(&[cmd("eat", sera, Attempt::Eat(Material::Berry))]);
    let ate = world.observe(sera).unwrap();
    assert_eq!(ate.self_info.energy, "rested");
    assert_eq!(ate.self_info.hunger, hunger);
    assert!(ate
        .events
        .iter()
        .any(|event| event.text.contains("eats a handful of berries")));
    assert!(!ate.text.contains("energy:"));

    world.step(&[cmd("pick", nils, Attempt::Gather(Material::Mushroom))]);
    for n in 0..3 {
        world.step(&[cmd(&format!("to-fire-{n}"), nils, Attempt::Move(Dir::East))]);
    }
    assert_eq!(world.actor_cell(nils), Some((6, 1)));
    world.step(&[cmd("light", nils, Attempt::Light)]);
    world.step(&[cmd("meal", nils, Attempt::Cook)]);
    let cooked = world.observe(nils).unwrap();
    assert!(cooked
        .events
        .iter()
        .any(|event| event.text.contains("cooks a mushroom")));
    assert_eq!(cooked.self_info.hunger, "not hungry");
    assert_eq!(world.material_tally(Material::Mushroom), 1);
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
        .any(|event| event.text.contains("The wind is on them")));
    assert!(under
        .events
        .iter()
        .all(|event| !event.text.contains("wind is on")));

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
    assert!(refused
        .events
        .iter()
        .any(|event| event.text.contains("nothing here to cook")));
    assert_eq!(cook.material_tally(Material::BitterBerry), 2);
    assert!(parse_attempt("eat bark").unwrap_err().contains("tinder"));

    let mut pool = World::valley(Tuning::evening(), 1);
    pool.tuning.chill_at = 0;
    pool.tuning.rain_starts_at = 10_000;
    let sera = pool.spawn_at("Sera", "s", 2, 1).unwrap();
    pool.step(&[cmd("ice", sera, Attempt::Drink)]);
    let frozen = pool.observe(sera).unwrap();
    assert!(frozen
        .events
        .iter()
        .any(|event| event.text.contains("The still water is frozen.")));
    assert_eq!(frozen.self_info.illness, "well");
    assert!(frozen.visible_cells.iter().any(|cell| cell.frozen));
    let otto = pool.spawn_at("Otto", "o", 0, 1).unwrap();
    pool.step(&[cmd("run", otto, Attempt::Drink)]);
    assert!(pool
        .observe(otto)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text.contains("drinks from the brook")));

    let mut thaw = world_at(10_000);
    thaw.tuning.chill_at = 0;
    let sera = thaw.spawn_at("Sera", "s", 6, 1).unwrap();
    thaw.step(&[cmd("bark", sera, Attempt::Gather(Material::Bark))]);
    thaw.step(&[cmd("branch", sera, Attempt::Gather(Material::Branch))]);
    for n in 0..4 {
        thaw.step(&[cmd(&format!("west-{n}"), sera, Attempt::Move(Dir::West))]);
    }
    assert_eq!(thaw.actor_cell(sera), Some((2, 1)));
    thaw.step(&[cmd("ice", sera, Attempt::Drink)]);
    assert!(thaw
        .observe(sera)
        .unwrap()
        .events
        .iter()
        .any(|event| event.text.contains("frozen")));
    thaw.step(&[cmd("light", sera, Attempt::Light)]);
    thaw.step(&[cmd("thaw", sera, Attempt::Drink)]);
    let open = thaw.observe(sera).unwrap();
    assert!(open
        .events
        .iter()
        .any(|event| event.text.contains("drinks the still water")));
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
    assert!(lit.text.contains("warms anyone standing at it"));
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
        && ticks < 12
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
    assert!(world
        .observe(sera)
        .unwrap()
        .text
        .contains("warms anyone standing"));
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
    for _ in 0..6 {
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
    assert!(seen.events.iter().any(|event| {
        event.text == "Near lights a fire with the bark and the branch from the arms."
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
        wood.contains("A pile of 4 branches") || wood.contains("A pile of 4"),
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
    assert!(wood.contains("A pile of 4 branches"), "{wood}");

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
            assert_eq!(
                drawn.results[0].message,
                "You bring the deer down. A carcass lies where it stood."
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
    kill.step(&[cmd("bark2", hunter, Attempt::Gather(Material::Bark))]);
    kill.step(&[cmd("branch2", hunter, Attempt::Gather(Material::Branch))]);
    let relit = kill.step(&[cmd("relight", hunter, Attempt::Light)]);
    assert!(relit.results[0].success, "{}", relit.results[0].message);
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
    let near = world.spawn_at("Near", "n", 3, 1).unwrap();
    world.step(&[cmd("m1", near, Attempt::Gather(Material::Mushroom))]);
    world.step(&[cmd("m2", near, Attempt::Gather(Material::Mushroom))]);
    for i in 0..3 {
        world.step(&[cmd(&format!("e{i}"), near, Attempt::Move(Dir::East))]);
    }
    assert_eq!(world.actor_cell(near), Some((6, 1)));
    let picked = world.step(&[cmd("berry", near, Attempt::Gather(Material::Berry))]);
    assert!(picked.results[0].success, "{}", picked.results[0].message);

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
        "You cook a mushroom. You are not hungry. You are rested."
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
        "You cook a mushroom. You are not hungry."
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
        "Two dry mushrooms are carried by you, under the rock overhang. They must be cooked. Raw mushrooms do not feed you."
    );
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
    assert!(looked.text.contains("North: slope."), "{}", looked.text);
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
    assert!(apart
        .text
        .contains("Wood left in the open is rotting one cell south and one cell west."));
    assert!(apart.text.contains("a deer"));
    assert!(!apart.text.contains("on the rotting wood"));
    let left = world.examine(near, ExamineTarget::Deer);
    assert_eq!(left, "A deer stands one cell west, in the open.");
    assert!(!left.contains("rotting"), "{left}");
    assert_eq!(
        world.examine(far, ExamineTarget::Deer),
        "You cannot see a deer from here."
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
    for _ in 0..12 {
        if world.material_tally(Material::Berry) == 0 {
            break;
        }
        world.step(&[]);
        if world.deer_cell() == Some((6, 1)) && world.material_tally(Material::Berry) > 0 {
            let text = world.observe(near).unwrap().text;
            assert!(
                text.contains("A deer is eating the berries here."),
                "{text}"
            );
            assert!(!text.contains("Berries lie here"), "{text}");
            assert!(!text.contains("A deer stands here."), "{text}");
            let far_text = world.observe(far).unwrap().text;
            assert!(!far_text.contains("eating the berries"), "{far_text}");
            let count = world.material_tally(Material::Berry);
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
    assert_eq!(world.material_tally(Material::Berry), 0);
    assert_eq!(world.deer_cell(), Some((6, 1)));
    let clock = world.sim_time();
    assert_eq!(
        world.examine(near, ExamineTarget::Berries),
        "The berries that were here are gone. The deer here ate them."
    );
    assert_eq!(world.sim_time(), clock);
    assert_eq!(world.material_tally(Material::Berry), 0);
    let gone = world.observe(near).unwrap().text;
    assert!(!gone.contains("Berries lie here"), "{gone}");
    assert!(!gone.contains("eating the berries"), "{gone}");
    assert!(gone.contains("A deer stands here."), "{gone}");
    assert!(!world
        .observe(far)
        .unwrap()
        .text
        .contains("eating the berries"));
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
        "The berries that were here are gone. A deer ate them."
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
