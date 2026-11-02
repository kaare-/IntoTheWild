# Design notes

Villmarks is one small valley and one authoritative process. The simulation crate has no network, database, wall clock, or model calls. The server batches intents and steps the core. Browser and Python clients share that API.

## Cell, stock, and construction

EmpireMUD 2.0 beta (`EmpireMUD/EmpireMUD-2.0-Beta`) stores a player landscape change as a new sector. `ACMD(do_lay)` in `src/building.c` checks that the room can take a road, spends resources, and calls `change_terrain`. `change_terrain` in `src/db.world.c` calls `perform_change_sect`, which writes both `SECT(room)` and `world_map[x][y].sector_type` and marks the map dirty. `show_map_to_char` in `src/mapview.c` then draws a glyph from the sector icon, with road, building, and crop icons able to replace it. The command, the stored terrain, and the glyph are one chain.

That chain is the wrong split for this valley. Bark on a frame is not a new kind of ground. A cell keeps its region (slope, brook, floor) and its rock overhang. Loose wood is a quantity until someone lifts it or builds with it, and then it is an item with a stable id. A frame is a third thing: supports, a beam, and optional bark, sitting on a cell. A fire is a fourth: its own id, the cell it burns on, and the moment it goes out. Lighting one spends a dry sheet of bark and a dry branch once; the fire is not a second copy of that wood, and bark already laid as a roof is not tinder. The map glyph is computed from what that person can see. A half-finished frame stays on the cell and does not become a terrain type. Material is counted once: pile count plus item count. Berries are a pile like branches. Cooking a handful over a burning fire is what eases hunger. Raw berries do not, and neither does the bark.

A deer is a fifth thing: one id, the cell it stands on, and an index on a fixed route. It is not a sector and not a second berry pile. Each tick it walks one cell, or, when berries are on the cell under its feet, eats one handful from that pile and stays. Eating uses the pile count only. The cell it leaves keeps hoofprints. Those marks are a set of cells, not a terrain change, and they do not decay in this valley yet. The glyph is `d` for the deer, ahead of wood and berries and behind a fire, and `"` for hoofprints when the cell would otherwise show only ground. A person sees the deer, the prints, and those events only inside their sight.

## One observation, two kinds of client

Bunnyland (the public how-it-works notes, client guide, and `thalismind/bunnyland-server` spec) puts human players and autonomous characters in the same rooms, with the same verbs and the same validation. Clients differ in presentation. An LLM is a controller for a character, not the world. Perception is character-scoped.

`World::observe` is that projection: prose, a fixed-width ASCII map, and JSON, including `allowed_actions`. Moisture numbers, tokens, hidden cells, and the cause log stay on the server. The Python client and the browser both call `POST /v1/intent` and `GET /v1/observe`. An optional model is configured with `VILLMARKS_MODEL_URL`, `VILLMARKS_MODEL_NAME`, `VILLMARKS_MODEL_KEY`, and `VILLMARKS_MODEL_TIMEOUT`. Its latency is only in the client. If the model is unset or fails, the scripted policy chooses. The server keeps sealing windows either way.

Bunnyland validates and commits per command. Villmarks collects every intent that arrived for the open window, sorts them by actor id and then command id, and commits one outcome. Receive order inside the window does not pick a winner.

## What the load test is

Aardwolf's public stats page, read on 2 November 2026, showed a most-online-since-reboot figure of 397 and an all-time figure of 853. Those reboot numbers move. GemStone III's published Simutronics timeline (Elonka Dunin) records more than 2000 simultaneous users in December 1996. Those are networked, long-lived games. They are context, not a target and not code we copied.

`cargo run --release -p villmarks-server --bin loadtest` times `World::step` for 20 windows. It does not open sockets, parse HTTP, or send WebSocket frames. The `sim+sqlite` rows also commit one SQLite transaction per window. Intent setup is outside the timer. Rain is disabled for the sample (`rain_starts_at = 10000`). The numbers are one machine, one run, not a capacity claim.

## Time and commands

Real time, simulation time, and the action window are separate. A window is 20 seconds unless `--window-secs` or `--manual` says otherwise. Manual play advances only on `POST /v1/advance`. The core itself never sleeps.

A command is an attempt. The core decides range, duration, and result. Look, examine, and help answer immediately and do not step the world. Anything that changes the valley is queued. A late hint (a client window number older than the current tick) still joins the open window; it is not dropped and it is not applied to a closed one. The same command id returns the stored result, including after a restart. A queued id that has not been sealed returns "Already queued". A parse failure is remembered for that id; send a new id to correct it.

One body attempt per actor per window (move, gather, drop, grab, drag, raise frame, lay bark, drink, boil, light a fire, cook). Say, interrupt, and release do not take that slot. Looking at the deer or its tracks is examine, not a new body attempt. Long work occupies the body until it finishes, is interrupted, or a precondition fails. Thirst, hunger, illness, and the deer all move on the schedule even when the window was empty. A gather in that window is resolved before the deer eats, so a person and the deer do not both take the same handful. A raw drink from the brook eases thirst and records a draw (`contest_hash` with the tag `drink`). When the draw falls under `illness_chance`, the illness amount jumps and the word becomes "ill". Each later tick lowers that amount until the word is "well". Boiling a drink of brook water over a burning fire eases thirst and records no illness draw. The boiled drink is not the cure. Two people pulling the same way can move the log. One person cannot. Two people grabbing one loose branch produce one item; the draw is `contest_hash(seed, time, tag, subject, actor)` and the hash is stored on the event. Two people lighting the same cell get one fire. Empty windows still rain, fade an illness, burn a fire down, walk the deer, and finish work, including with nobody present. Rain in the open puts a fire out. A rock overhang or a bark roof does not.

## Log and persistence

Events have an id, a simulation time, affected objects, and cause ids when a prior event contributed. A log shift cites the grips and the drag. Independent events may share a tick. Ties break by event sequence. Draws are stored so a run can be replayed. The client event list has id, time, and text only.

SQLite is WAL, one connection, one transaction per join and per sealed window. A snapshot is written every `snapshot_every` records (5 by default) and again on SIGINT. Restore is the latest snapshot plus later log rows. The snapshot includes the clock, grips, work, the future schedule, and the seed. Queued intents that were not sealed are gone; resending them runs them once.

## Follow-ups

1. Seasons that change the rain and the bark.
2. Decay of wet wood, of frames nobody maintains, and of hoofprints.
