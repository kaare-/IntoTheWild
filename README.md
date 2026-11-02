# Villmarks

A small wilderness MUD. The valley is already there: a brook, a slope, a rock overhang, dry wood and berries under that rock, an open pile, one heavy log, and one deer on the trail. People and external agents use the same rules. Rain soaks whatever the sky can reach. Bark on a frame supported at both ends is a roof; a half-finished frame is not. The same bark, with a dry branch, is tinder for a fire.

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

Commands: `look`, `examine wood|bark|berries|water|fire|deer|tracks|log|frame|self|here|<name>`, `say`, `n` `s` `e` `w`, `gather branch|bark|berries|water`, `drop`, `grab`, `drag n|s|e|w`, `release`, `raise frame`, `lay bark`, `drink`, `boil`, `light fire`, `cook`, `interrupt`, `help`.

Thirst rises every tick, whether or not anyone is here. Rain wets clothes and does not quench it. `drink` works only while you are standing in the brook. That raw drink eases thirst and rolls a recorded draw: half the time, in this valley, your stomach turns and you are ill. The illness amount stays on the server. You hear "well" or "ill". It fades on its own, a little every tick, until you are well again. `gather water` at the brook, carry it to a fire, and `boil` it. The boiled drink eases thirst and does not roll that draw. It also does not cure an illness you already have. Hunger rises the same way thirst does. Bark is tinder. `light fire` takes one dry loose sheet of bark and one dry branch. Rain puts that fire out in the open; the rock overhang or a bark roof keeps it. `cook` works only while a fire is burning where you stand and a handful of berries is in your arms or at your feet. Raw berries do nothing. One raw drink, one boiled drink, or one cooked handful takes the edge off thirst or hunger. Those amounts stay on the server too. You hear "not thirsty", "thirsty", or "parched", and "not hungry", "hungry", or "starving".

A deer is already on the trail at (2,3). It walks one cell every tick, including when nobody has sent a command, along the trail and then to the berries under the rock. If it is standing on a pile of berries it eats one handful and stays for that tick. The handful leaves the pile. It is not a new item. The cell it walks off keeps hoofprints, and those prints stay. `examine deer` and `examine tracks` are the same examine command as the wood and the fire. There is no separate animal command. A person sees the deer, the prints, and what it just did only inside their own sight. Someone who arrives later can still find the prints.

The wet season arrives at tick 18, while the rain is already falling, whether or not anyone has sent a command. From then on, rain in the open soaks wood twice as fast. The rock overhang and a bark roof keep the same shelter. Nothing dries. Someone who can see that open wood reads "The rain is heavier on the wood this season." Someone who cannot see it does not get that line, or the event on that ground. The commands stay the same.

Loose branches left in the open rot. Every 8 ticks one of them is gone, with or without a command. The line is "Wood left in the open is rotting." and, when one goes, "A branch rots away in the open." A rock overhang or a bark roof stops that, because the sky is no longer open on that cell. A branch built into a frame is not loose, so it is not in the rot. Bark, berries, and the log are not this rot. Someone who cannot see the open wood does not get the line.

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
1 ?? ?? .  .  .  pH Rw
2 ?? ?? .  .  @L .W .
3 ?? ?? .  .  .  .  .
4 ?? ?? .  .  .  .  .
```

`H` is the bark roof, `Rw` is wood under the rock, `.W` is the soaked pile, `@L` is Sera on the log.

## Checks

Run from this workspace on 2 November 2026.

| Check | Result |
| --- | --- |
| `cargo test --workspace` | Pass. 19 simulation tests and the in-process restart test. |
| Waterborne illness through the player API (manual server, seed 1, two clients) | Pass. One raw drink at the brook made Sera ill. The same command id did not drink again. Otto did not see her stomach turn. Boiling over the rock fire left her ill and did not sicken her again. Five empty advances later she was well. |
| Deer through the player API (manual server, seed 1, two clients) | Pass. An empty advance moved the deer east and left hoofprints on (2,3). Sera saw it eat one handful under the rock; that pile went from 4 to 3. Otto at (0,0) had neither the deer nor that eat. He later stood on (2,3), found the hoofprints, and could not see the deer. Examine did not advance the clock. The scripted policy returned `e`, which was already allowed. |
| Wet season through the player API (manual server, seed 1, two clients) | Pass. Empty advances reached tick 18 with the open branches damp and no season line. The next empty advance soaked that pile and Sera read "The rain is heavier on the wood this season." The rock pile stayed dry. Otto at (0,4) still saw no wood and no heavier-rain line. The announcement did not repeat on the next tick. Examine did not advance the clock. The scripted policy returned `e` and `n`, both already allowed. |
| Rot through the player API (manual server, seed 1, two clients) | Pass. Before any advance, Sera read that the open branches were rotting and the rock pile was not. Empty advances held both piles at 4 through tick 8. The next empty advance left the open pile at 3, the rock pile at 4, and the event "A branch rots away in the open." Otto at (0,4) saw no wood and no rot line. The next tick did not rot another branch. Examine did not advance the clock. The scripted policy returned `e` and `n`, both already allowed. |
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

The original list is in the valley now: the evening, thirst, hunger and the fire, brook illness, the deer, the wet season, and rot of loose branches left in the open. Nothing further is started.
