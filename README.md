# Villmarks

A small wilderness MUD. The valley is already there: a brook, a slope, a rock overhang, dry wood under that rock, an open pile, and one heavy log. People and external agents use the same rules. Rain soaks whatever the sky can reach. Bark on a frame supported at both ends is a roof; a half-finished frame is not.

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

Commands: `look`, `examine wood|bark|log|frame|self|here|<name>`, `say`, `n` `s` `e` `w`, `gather branch|bark`, `drop`, `grab`, `drag n|s|e|w`, `release`, `raise frame`, `lay bark`, `interrupt`, `help`.

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
| `cargo test --workspace` | Pass. 11 simulation tests and the in-process restart test. |
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

1. Thirst, and drinking from the brook.
2. Hunger.
3. Waterborne disease from that brook.
4. Animals on the trails.
5. Seasons.
6. Decay of wet wood and abandoned frames.
