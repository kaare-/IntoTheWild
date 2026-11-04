# Villmarks

A small wilderness MUD. The valley is already there: a brook, a still pool, a slope, a rock overhang, dry wood and berries under that rock, a pot, poison berries, mushrooms, an open pile, one heavy log, one loose stone on the slope at (2,0), two lumps of clay on the bank at (2,2), and one deer on the trail. People and external agents use the same rules. Rain soaks whatever the sky can reach. Bark on a frame supported at both ends is a roof; a half-finished frame is not. The same bark, with a dry branch, is tinder for a fire.

The simulation is a Rust library with no network, database, clock, or model calls. One Tokio and Axum process owns the valley. A TypeScript page and a Python client are players, not special cases. Design choices taken from EmpireMUD, Bunnyland, Aardwolf, and GemStone III are in [DESIGN.md](DESIGN.md). Nothing here is deployed.

## Run it

Rust 1.99.1 is pinned in `rust-toolchain.toml`. You need Node and Python 3.

```bash
cargo build -p villmarks-server
cd web && npm install && npm run build && cd ..
cargo run -p villmarks-server
```

The valley listens on [http://127.0.0.1:18431](http://127.0.0.1:18431). The action window is 20 seconds (`config/evening.toml`). Open a second browser tab and take a different name. `sessionStorage` keeps each tab's token.

```bash
python3 agent/play.py --name Linde
```

That client uses a scripted policy and no model key. To let an external model pick from the same allowed actions, set `VILLMARKS_MODEL_URL`, `VILLMARKS_MODEL_NAME`, and `VILLMARKS_MODEL_KEY`. The world does not wait on the model.

Useful flags:

```bash
cargo run -p villmarks-server -- --help
cargo run -p villmarks-server -- --manual          # advance with POST /v1/advance
cargo run -p villmarks-server -- --window-secs 2   # shorter windows
cargo run -p villmarks-server -- --data data/valley.sqlite
```

Delete the sqlite file to reseed the valley. A living file keeps the tuning it was born with. The web dev server, if you want it, is `npm run dev` in `web/` on port 47321 and proxies the API.

Commands: `look`, `examine wood|bark|berries|poison berries|mushrooms|pot|mud|water|stone|clay|bowl|fire|deer|tracks|log|frame|self|here|<name>`, `say`, `n` `s` `e` `w`, `gather branch|bark|berries|poison berries|mushrooms|pot|mud|water|stone|clay|bowl`, `drop`, `grab`, `drag n|s|e|w`, `release`, `raise frame`, `lay bark`, `drink`, `boil`, `set clay`, `light fire`, `cook`, `eat berries`, `smear mud`, `interrupt`, `help`.

One loose stone lies on the slope at (2,0). `gather stone` lifts it. From then on it is one carried thing with a stable id, and it is no longer on the ground. `drop stone` puts that same stone back where you stand. Someone outside sight does not see it. It is spoken as one stone: "A stone lies in contact with the ground here, in the open. The stone is dry. The sky is open above it. It is a found stone." In the arms, examine is one sentence: "A dry stone is carried by you, in the open, and it is a found stone." The id stays on the server. Branches and berries stay quantities.

A carried sheet of bark is one sentence: "A dry sheet of bark is carried by you, under the rock overhang." A carried pot is one sentence: "A dry pot is carried by you, under the rock overhang, and it can boil water over a fire." Dry sits inside the sentence. Look says "a dry sheet of bark" and "a dry pot". One pot on the ground is a pot. Piles of branches and berries stay quantities.

Two lumps of clay lie on the bank at (2,2), on the valley floor beside the brook and the still pool. `gather clay` lifts one lump. `set clay` while you stand at a burning fire puts that lump on the ground. Four ticks later (`clay_ticks` in `config/evening.toml`), if the fire is still burning, the clay hardens into a bowl. `gather bowl` lifts it. `boil` with the bowl works the same as with the pot: the water is drunk, and the bowl is not used up. Unfired clay cannot boil water. If the fire is out when those four ticks are up, the clay stays soft. Someone outside sight does not see the clay. There is no kiln, no wheel, and no metal. The seeded pot stays under the rock.

Thirst rises every tick, whether or not anyone is here. Rain wets clothes and does not quench it. `drink` works in the brook or in the still pool at (2,1). A raw drink eases thirst and rolls a recorded draw. Running water, the brook, turns your stomach only occasionally (two times in ten, in this valley). Still water does it almost every time (nine times in ten). The illness amount stays on the server. You hear "well" or "ill". It fades on its own, a little every tick, until you are well again. `gather water` at either source, carry it to a fire, and `boil` it. Boiling needs the pot that sits with the supplies under the rock, or a bowl fired from the clay. The pot is not used up, and neither is the bowl. Unfired clay cannot boil water. The boiled drink eases thirst and does not roll that draw. It also does not cure an illness you already have. Hunger rises the same way thirst does. Energy does too: "rested", "tiring", or "spent". Bark is tinder. `light fire` spends one dry sheet of bark and one dry branch from the arms when those are carried, and from the pile at the feet only when the arms do not already hold that fuel. The line says which one was used. Rain puts that fire out in the open; the rock overhang or a bark roof keeps it. Sweet berries, the pile under the rock, can be eaten raw and give energy. They do not ease hunger. The handfuls at (4,1) are poisonous. Eating them turns your stomach and does not feed you, and cooking them does not make them food. Mushrooms at (3,1) must be heat-treated; raw mushrooms are refused and stay where they were. `cook` works only while a fire is burning where you stand, and it takes a mushroom first, then sweet berries, from your arms or from the ground at your feet. Bark is still tinder, not food. One raw drink, one boiled drink, one raw handful of sweet berries, or one cooked portion takes the edge off. Those amounts stay on the server. You hear "not thirsty", "thirsty", or "parched", and "not hungry", "hungry", or "starving".

A deer is already on the trail at (2,3). It walks one cell every tick, including when nobody has sent a command, along the trail and then to the berries under the rock. If it is standing on a pile of berries it eats one handful and stays for that tick. The handful leaves the pile. It is not a new item. The cell it walks off keeps hoofprints, and those prints stay. `examine deer` and `examine tracks` are the same examine command as the wood and the fire. There is no separate animal command. A person sees the deer, the prints, and what it just did only inside their own sight. Someone who arrives later can still find the prints.

The wet season arrives at tick 18, while the rain is already falling, whether or not anyone has sent a command. From then on, rain in the open soaks wood twice as fast. The rock overhang and a bark roof keep the same shelter. Nothing dries. Someone who can see that open wood reads "The rain is heavier on the wood this season." Someone who cannot see it does not get that line, or the event on that ground. The commands stay the same.

Loose branches left in the open rot. Every 8 ticks one of them is gone, with or without a command. The line is "Wood left in the open is rotting." and, when one goes, "A branch rots away in the open. You hear branches breaking." A rock overhang or a bark roof stops that, because the sky is no longer open on that cell. A branch built into a frame is not loose, so it is not in the rot. Bark, berries, mushrooms, and the log are not this rot. Someone who cannot see the open wood does not get the line, and does not hear the break.

A nearby character also hears "You hear branches breaking." when the deer steps through loose branches, and when someone gathers a branch. A deer step on empty trail is quiet, and so is eating berries or lighting a fire. The sentence is part of the event that already happened. Sight is still two cells. A distant character does not get it.

A body in the open gets cold, and the wind is on them. The same exposure the rain already uses decides it: the rock overhang stops it, a bark roof lets a little through, open sky lets the wind double the cold. A body standing on a burning fire warms, and that tick does not also add the sky's cold. For a few ticks after they step off, or after the fire goes out, the sky's cold is halved, and the wind still cools them. While that leftover is what keeps the word warm, the look says so, and the sentence goes when the word leaves warm. The next cell does not warm. Someone standing there reads that the heat does not reach them. Rain still puts an open fire out. The sun on that body is unchanged. The sun burns bare skin while the rain is still off. `gather mud` at the brook or the still pool, then `smear mud`. A smear leaves a quarter of that sun. Mud flakes off, and rain in the open washes it off. The words are "warm", "chilly", or "cold"; "out of the wind", "in a little wind", or "in the wind"; "unburned", "pink", or "burned"; "bare skin", "thin mud", or "mud-smeared". From tick 12 the still pool is frozen unless a fire burns on it. The brook keeps running. The amounts stay on the server.

## An evening

Rain starts at tick 16. One person can still roof a frame before that if they take three branches and bark from under the rock and build on open ground. The open pile at (5,2) and the dry pile under the rock are the comparison. Two people can drag the log; one cannot.

This session was played through the player API against `villmarks-server --manual` (three clients, then a fourth who walked out of sight). Sera and Otto took the log. Linde built the roof. The interrupted drag did not move the log; the next one did.

```text
t7  The heavy log is in contact with the ground here, in the open. It is dry.
    Otto and Sera are gripping it.
t14 A frame stands here, under a bark roof. Supports: 2. It has a beam.
    It is supported at both ends. Bark lies over it and is dry.
    The wood in the frame is dry. Only a little rain reaches it.
```

After the rain, Linde examined the wood:

```text
A pile of 1 branch is in contact with the ground to the east, under the rock
overhang. The wood is dry. Rain does not reach it.
A pile of 4 branches is in contact with the ground to the south, in the rain.
The wood is soaked. Rain reaches it.
```

The frame's own wood was damp, under soaked bark: wet bark is a worse roof than dry bark, and it is still less than open sky. Nils, standing at the west edge, got `You see no wood from here.` His map had no wood glyph.

Sera's map after the rain:

```text
  0  1  2  3  4  5  6
0 ?? ?? ^  ^  ^  ^  ^
1 ?? ?? =  .  .  pH Rw
2 ?? ?? .  .  @L .W .
3 ?? ?? .  .  .  .  .
4 ?? ?? .  .  .  .  .
```

`H` is the bark roof, `Rw` is wood under the rock, `.W` is the soaked pile, `@L` is Sera on the log.

## Checks

Run from this workspace on 2 November 2026.

| Check | Result |
| --- | --- |
| `cargo test --workspace` | Pass, 4 November 2026. 32 simulation tests and the in-process restart test. |
| Waterborne illness through the player API (manual server, seed 1, two clients) | Pass. One raw drink at the brook made Sera ill. The same command id did not drink again. Otto did not see her stomach turn. Boiling over the rock fire left her ill and did not sicken her again. Five empty advances later she was well. |
| Deer through the player API (manual server, seed 1, two clients) | Pass. An empty advance moved the deer east and left hoofprints on (2,3). Sera saw it eat one handful under the rock; that pile went from 4 to 3. Otto at (0,0) had neither the deer nor that eat. He later stood on (2,3), found the hoofprints, and could not see the deer. Examine did not advance the clock. The scripted policy returned `e`, which was already allowed. |
| Wet season through the player API (manual server, seed 1, two clients) | Pass. Empty advances reached tick 18 with the open branches damp and no season line. The next empty advance soaked that pile and Sera read "The rain is heavier on the wood this season." The rock pile stayed dry. Otto at (0,4) still saw no wood and no heavier-rain line. The announcement did not repeat on the next tick. Examine did not advance the clock. The scripted policy returned `e` and `n`, both already allowed. |
| Rot through the player API (manual server, seed 1, two clients) | Pass. Before any advance, Sera read that the open branches were rotting and the rock pile was not. Empty advances held both piles at 4 through tick 8. The next empty advance left the open pile at 3, the rock pile at 4, and the event "A branch rots away in the open." Otto at (0,4) saw no wood and no rot line. The next tick did not rot another branch. Examine did not advance the clock. The scripted policy returned `e` and `n`, both already allowed. |
| Sound, still water, and food through the player API (manual server, seed 1, port 18509) | Pass, 3 November 2026. At tick 0, examine left the clock alone and told sweet berries, bitter berries, mushrooms, and the still pool apart. A later examine of the brook, also without moving the clock, calls it running water that sickens only occasionally. The client had no chance number and no draw. The first deer step was "A deer steps east." At tick 4, Sera heard "A deer steps east. You hear branches breaking." and Otto at (0,4) did not. At tick 9 she heard "A branch rots away in the open. You hear branches breaking." Otto did not. "Nils gathers a branch. You hear branches breaking." reached Sera only. At tick 25 Nils was tiring and hungry; `eat berries` left him rested and still hungry, and the handful was gone. Raw mushrooms and bitter berries were refused and the piles stayed at 2. Cooking a mushroom took hunger from hungry to not hungry. Boiling with no pot failed and kept the water. With the pot, the boiled drink eased thirst, did not turn a stomach, and left illness well. The still-pool drink turned Nils's stomach. Otto's brook drink left him well. A later look and examine stayed on tick 51. The scripted policy returned `eat berries`, then `cook`, then `gather mushrooms`, and never `eat mushrooms`. |
| Cold, wind, sun, mud, and poison berries through the player API (manual server, seed 1) | Pass, 4 November 2026. At tick 0 everyone was warm, unburned, and bare, and examine did not move the clock. The open body reached cold and burned. The body that walked onto the rock at tick 5 stayed chilly and unburned, out of the wind, through tick 12. A matched pair in the open: after "Muddy smears mud on their skin." the smeared body was pink at tick 12 while the bare body was burned. Before tick 12 the still pool could be drunk. At tick 12 it was frozen, the brook was not, and a fire on the pool let the drink through. Poison berries sickened and were not cooked. Bark is still tinder. The scripted policy returned `smear mud`, then a step toward shelter, then `gather mushrooms`, and never `eat poison berries`. |
| A fire warms the body standing on it (manual server, seed 1, port 18533) | Pass, 4 November 2026. Warm stood in the open at (5,1), cold, in the wind, pink. Lighting the fire made them chilly on that tick, with "Warm is warming at the fire." They were warm at tick 14 while the fire still burned. Near, off that cell, stayed cold. Sunburn went from pink to burned and did not improve. Look and examine left the clock alone. Rain put the fire out at tick 16, and by tick 20 Warm was chilly in the wind again. The scripted policy stays on a fire when cold, walks toward a known fire, and does not eat poison berries. |
| Leftover warmth, and the next cell (manual server, seed 1, port 18547) | Pass, 4 November 2026. Beside at (5,2) read "The heat of the fire does not reach you." Look and examine left the clock alone. Beside stayed cold and never read the leftover sentence. Warm stepped off the fire, still warm and in the wind, and read "The warmth of the fire is still on you." Rain put the open fire out at tick 16. Warm stayed warm through tick 21 and was chilly at tick 22. The fire did not burn down through the rain. Sunburn went pink to burned and did not improve. On the rock, Warm was out of the wind. The scripted policy stays on a fire when cold, and does not eat poison berries. |
| The leftover sentence and the rock (manual server, seed 1, port 18559) | Pass, 4 November 2026. While the fire burned, the leftover sentence was absent and the fire line was present. Rain put the open fire out at tick 16. The sentence was in the look at ticks 17, 18, 19, 20, and 21, every tick the word stayed warm, and it was absent when the word became chilly at tick 22. The fire did not burn down through the rain. On the live warm observation at (5,1), the scripted policy returned east, toward the rock. It stays on a fire when cold, walks to the rock when warm or chilly in the wind, and does not eat poison berries. |
| One loose stone on the slope (manual server, seed 1, port 18571) | Pass, 4 November 2026. Before anyone walked, Near at (4,3) and Far at (3,3) both got "You see no stone from here." Look and examine left the clock alone. Near walked to (2,0) and read "You are on the slope, in the open. A loose stone lies here." The cell was stone_pile 1, and the map showed s. Examine called it a found stone. After gather stone, the arms held one dry stone and the ground was clear. East to (3,0) and drop stone put loose_stone 1 back on that cell. Far still saw no stone. Gathering it again left one stone in the arms and nothing on the ground. |
| One stone, spoken as one stone (manual server, seed 1, port 18583) | Pass, 4 November 2026. From (4,3), berries were still "A pile of 4 handfuls of berries", and the stone was not in sight. On the slope, examine was "A stone lies in contact with the ground here, in the open. The stone is dry. The sky is open above it. It is a found stone." In the arms, look said "You have a dry stone." and examine was one sentence: "A dry stone is carried by you, in the open, and it is a found stone." After the drop, the ground sentence was the same. Look and examine left the clock alone. |
| Clay on the bank, fired into a bowl (manual server, seed 1, port 18597) | Pass, 4 November 2026. Two lumps at (2,2). Far at (3,3) could see them; after walking to (6,4), examine was "You see no clay from here." Near read "Clay lies on the bank here." and "A pile of 2 lumps of clay" that "cannot boil water." Look and examine left the clock alone. One lump gathered, one stayed. At the rock fire, boil said "Unfired clay cannot boil water." and kept the clay, the water, and the pot. Set at tick 19. Four ticks later, tick 23: "The clay hardens into a bowl." The bowl boiled, stayed in the arms, and the pot was still on (6,1). Rain did not put the rock fire out. Far at (6,4) never saw the clay or the bowl. |
| A fire spends carried fuel before the pile (manual server, seed 1, port 18611) | Pass, 4 November 2026. Near reached the rock with clay, water, bark, and a branch, and the piles at (6,1) were bark 1, branch 3, pot 1. Lighting said "You light a fire with the bark and the branch from the arms." The piles stayed. Bark and branch left the arms. `gather pot` then succeeded. After "The fire burns down." at tick 21, the next lighting said "You light a fire with bark and a dry branch from the pile at the feet." Bark went from 1 to 0 and branches from 3 to 2. The pot stayed in the arms. Rain did not put the rock fire out. Look and examine left the clock alone. |
| A carried sheet and a carried pot (manual server, seed 1, port 18623) | Pass, 4 November 2026. At the rock, berries were still "A pile of 4 handfuls of berries" and branches "A pile of 4 branches". The pot on the ground was "A pot lies in contact with the ground here, under the rock overhang. The pot is dry. Rain does not reach it. It can boil water over a fire." Look said "A pot sits here." After gather, the arms were "You have a dry sheet of bark, a dry pot." Out of sight of the piles, examine was "A dry sheet of bark is carried by you, in the open." and "A dry pot is carried by you, in the open, and it can boil water over a fire." After the drop, the ground pot was "A pot lies in contact with the ground here, in the open. The pot is dry. The sky is open above it. It can boil water over a fire." Look and examine left the clock alone. |
| Rain evening through the player API (`python3 agent/evening_session.py` against `--manual`) | Pass. Dry rock pile, soaked open pile, bark roof, distant slice, duplicate grab id. |
| Opposite HTTP receive order for the same drag (`python3 agent/order_check.py`, seed 7) | Pass. Both orders finished at (4,2). |
| One branch, and an interrupted drag | Pass in `sim/tests/evening.rs`. The evening session also interrupted a live drag before it landed. |
| Stop and start while two people hold the log (`python3 agent/restart_check.py`, SIGINT) | Pass. Grips, cell, rain, and a drag due at tick 18 came back. One more advance moved the log to (4,2). The same grab id was not applied again. |
| Release load test | Pass as a local experiment. See below. |
| Two browser tabs and `agent/play.py` | Pass, on `--window-secs 2`. See below. |

Load test, release binary, 20 windows, rain not firing. Network is not included. SQLite is included only in the `sim+sqlite` rows.

```text
spread-move  actors=1000 mode=sim         windows=20 mean_us=1078 max_us=1263 exceeded_20s_window=false
spread-move  actors=1000 mode=sim+sqlite  windows=20 mean_us=5165 max_us=29485 exceeded_20s_window=false
brook-say    actors=100  mode=sim         windows=20 mean_us=63 max_us=74 exceeded_20s_window=false
brook-say    actors=100  mode=sim+sqlite  windows=20 mean_us=217 max_us=937 exceeded_20s_window=false
```

No window took longer than 20 seconds. The slowest sqlite window in the spread was about 29 milliseconds. This does not say what a hosted game of Aardwolf or GemStone's size would do.

The browser check used Chrome against `http://127.0.0.1:18431` with `--window-secs 2`. Two tabs joined as Sera and Otto. Sera looked, examined the wood, walked north from (4,3) to (4,2), and spoke. Otto heard her, walked onto the log, and showed up on Sera's map as `p`. On a longer 2 second run the same page's examine showed the rock pile dry and the open pile soaked. `python3 agent/play.py --name Linde` used that timed server as well: it queued moves, gathered under the rock, and was still there when rain began. The map type is 18px at the root and about 21px for the grid, so it stays readable when a project pane draws the page smaller. A 390px-wide viewport stacks the map over the log and leaves the command field in the page instead of covering the prose.

## Still unverified

- A live model call. The client path is in `agent/policy.py`. No key was set, so that branch was not executed.
- A checkout on another machine. The commands above were run here after `cargo build` and `npm run build`.
- The default 20 second window was not the clock for these passes. The scripted evening used `--manual`. The browser and `play.py` used `--window-secs 2`. `config/evening.toml` still says 20, and nothing in the measured load test exceeded that.

## Follow-ups

A carried sheet of bark and a carried pot are each one sentence, and one pot is a pot. Nothing further is started.
