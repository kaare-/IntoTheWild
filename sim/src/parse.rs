//! Player lines become attempts. The core still decides whether they happen.

use crate::model::{Attempt, Dir, ExamineTarget, Material};

pub fn help_text() -> &'static str {
    "\
look                         where you are, and the local map. At night the ground stays dark. A fire, a smoke column, or the rock overhang that is still in sight keeps its direction
examine wood|bark|berries|dark berries|pale berries|mushrooms|pot|mud|water|stone|flint|ochre|iron ore|bloom of iron|iron knife|copper ore|copper bead|awl|flake|clay|bowl|spear|carcass|hide|brand|fire|deer|tracks|log|frame|rock|slope|eggs|moss|bow|paddle|hook|self|here|<name>
say <words>                  people nearby hear you
n s e w                      walk
gather branch|bark|berries|dark berries|pale berries|mushrooms|pot|mud|water|stone|flint|ochre|iron ore|bloom of iron|iron knife|copper ore|copper bead|awl|clay|bowl|spear|carcass|meat|hide|bone
drop branch|bark|berries|dark berries|pale berries|mushrooms|pot|mud|water|stone|flint|ochre|iron ore|bloom of iron|iron knife|copper ore|copper bead|awl|clay|bowl|spear|carcass|meat|hide
set clay                     a lump at a burning fire; it hardens into a bowl
bind spear                   a carried stone and a carried branch; hold it at a fire
knap                         a carried flint nodule into a sharp flake
haft                         a carried flint flake and a dry branch into a spear; hide strips bind the tip when carried, otherwise willow cordage, otherwise nettle cordage. Hold it at a fire
point branch                 a carried dry branch into a wooden spear. The point is still green. No flint and no stone
harden spear                 that wooden point, held in a burning fire. It is weaker than a flint spear. The fire is not used up
craft bow                    a dry branch and hide strips, willow cordage, or nettle cordage, from the arms if you carry them, or lying in this cell. Hide strips are spent before willow, and willow before nettle. The bow stays in the arms.
make bow                     the same bow
dry branch                   a damp or soaked branch at a burning fire; after the wait it is dry and can haft. The fire is not used up.
dry clothes                  wet, damp, or soaked clothes at a burning fire. You wait there until they are dry. Leaving the fire stops the wait. The fire is not used up.
wait by the fire             the same wait for wet clothes
take brand                   a burning brand from a fire already burning here; the fire stays lit. It goes out in the rain, or after a short wait.
grab                         take hold of the heavy log
drag n|s|e|w                 pull; one person cannot move the heavy log. A dugout canoe moves for one person, but not into the brook
hollow log                   the heavy log, at a fire on this cell. The fire chars it and is not used up
carve canoe                  a charred heavy log, with a flint flake or an iron knife. The tool is not used up. A dugout canoe lies here
launch                       a dugout canoe on the bank, into the brook beside you
board                        a dugout canoe floating in the brook here or on the next cell
paddle [n|s|e|w]             while aboard, move the dugout along the brook. You move with it. A bare paddle picks a brook cell. Holding a wooden paddle names it. The bank refuses
disembark                    leave the dugout onto the bank. leave canoe is the same. Walking off without this refuses
beach                        a floating dugout, onto the bank beside it. If you are aboard, you come ashore with it. drag canoe is the same
seal canoe                   a lump of pine resin on the dugout here, or the one afloat beside you. The lump is spent and the hull is pitched. pitch canoe and waterproof canoe are the same
release                      let go of the log
raise frame                  three branches, then three phases of work
lay bark                     bark over a frame supported at both ends
drink                        raw water where you stand, and on the bank beside the brook where a baited hook can take a fish. Still water freezes until the day thaws it. The brook does not freeze. A miss stays with you
boil                         a pot or a fired bowl, and a fire; unfired clay cannot
steep|brew mint            a handful of mint in a pot or a bark cup of water over a fire. You drink the mint tea. A sick stomach eases. A bowl or unfired clay is not enough. With no fire, no mint, no pot or cup, or no water in that pot or cup, only you hear which one is missing. Drinkable water one step away adds which way, and the water stays where it is. When the still beside you is frozen and that step is the brook, only you hear that the still is frozen
steep|brew pine            a handful of pine needles the same way. A chill eases a little. It does not settle a stomach. gather pine needles takes them from the pine south of the trail
steep|brew willow bark      a strip of willow bark the same way. A sunburn eases a little. It does not settle a stomach or a chill. The cordage is not the bark
light fire                   dry bark and a dry branch, from the arms if you carry them. Dry moss from the shade under the rock is tinder the same way, and is spent before bark when both are held. Wet moss is not tinder. A lump of charcoal you carry, or one on this ground, stands in for the branch, even when a dry branch lies here. That fire burns hot, and for twice as long. A lump of pine resin in the arms is tinder instead, and is spent before moss and bark when it is held. Resin on the ground is not tinder. The branch is still required when no charcoal is at hand. A soaked sheet in the arms is left there when dry bark is on the ground or in the pile at your feet or one step off, and the fire is lit on that dry bark
signal                       a column of smoke from a burning fire here. smoke signal is the same. It lasts while the fire burns. Someone who cannot see the fire, past the ordinary smoke, reads which way, and that someone is signaling with smoke. One cell farther, they smell it, and the same words. The ordinary plume stays Smoke rises, without that cue. Green reeds or pine needles may be laid on the fire. They do not change the column. With no fire, with nothing green when you asked for it, or with the column already rising, only you hear it
carve drill                  two dry branches and cordage into a bow drill. One branch is the spindle, one is the fireboard. Hide strips, then willow, then nettle. A spent bow drill in the arms is carved anew with one dry branch
light fire with drill        the bow drill stays in the arms, and the line names it spent. No bark, no resin, and no charcoal. light fire still wants tinder
cook                         heat a mushroom, or cook berries, a carcass, raw meat, or cattail flour into a flatbread
grind                        dried cattail roots, raw or cooked, with a stone in the arms and a stone under them. The rock is a stone. The stones stay. The roots become cattail flour.
hunt                         a hardened spear; the deer on your cell or the next, a wolf while it stands on your cell or the next, an otter on its cell or the next, a beaver on its cell or the next, a fox on its cell or the next, or a hare on its cell or the next. With no spear that can reach, a carried bow and a sharp flint flake reach as far as you can see. The flake is spent when the deer, the wolf, the otter, the beaver, the fox, or the hare comes down. A refusal leaves it. The bow stays.
fish                         a fire-hardened flint spear, or a fishing net, or a bone fish hook with hide strips, in the brook or beside it, and from a dugout on the brook. A bone fish hook and a worm, with no spear and no net, is the same cast as fish with hook, in the brook or in the still shallows. The spear and the net are not used up. The hide strips are spent as the line. The worm is spent as the bait. That stretch is quiet for a short wait. On frozen shallows with no hole, the reply names the ice before the worm
fish with hook               the same cast with a bone fish hook and hide strips in the arms. A worm baits the hook. In the still shallows the worm is the bait and the line is not required. The hook stays. The strips are spent when they are the line. A hole in the ice lets that cast through while the still is frozen. With no hole, the reply names the ice before the worm. With no hook, no worm in the shallows, or no water, only you hear it
break ice                    a stone, a bone, or an iron knife on frozen still water. A hole opens, and the tool stays. break and chop are the same. With no ice, no tool, or a hole already open, only you hear it
spear fish                   the same cast with the spear
butcher                      a carcass at your feet into three portions; a flint spear or an iron knife in your arms. With both, the knife cuts. Neither is used up. An otter, a fox, a hare, or a beaver is one portion of raw meat, that animal's hide, and a bone.
cut meat                     one portion of raw or cooked meat into two smaller pieces; an iron knife in your arms. The knife is not used up.
cut hide                     a whole deer hide into hide strips, with an iron knife. The knife is not used up.
carve awl                    a bone, with a flint flake or an iron knife, into a bone awl. The tool is not used up. The awl can pierce hide.
carve paddle                 a dry branch, from the arms or on the ground, with a flint flake or an iron knife. The tool is not used up. The wooden paddle stays in the arms. Holding it names the paddle when you paddle. Bare hands still paddle.
carve hook                   a bone, from the arms or on the ground, with a flint flake or an iron knife. The tool is not used up. The bone fish hook stays in the arms.
carve whistle                a bone the same way, into a bone whistle. One bone lies under the rock, and it lies there again after the same wait as salt. The tool is not used up. With no bone, or no flake or knife, only you hear which is missing
blow whistle                 the bone whistle in the arms. play whistle is the same. Someone who can see you hears the blow. Past sight, a look says which way, and one cell farther still. A hare within hearing bolts, a fox within hearing slips, and an otter or a beaver within hearing splashes, and those lines are not added to the blow. It is not a smoke signal. With no whistle, only you hear it
carve flute                  a bundle of reeds, from the arms or on the ground, with a flint flake or an iron knife, into a reed flute. The tool is not used up. With no reeds, or no flake or knife, only you hear which is missing
blow flute                   the reed flute in the arms. play flute is the same reach as the bone whistle. The hare does not bolt, the fox does not slip, and the otter and the beaver do not splash. With no flute, only you hear it
sew drum                     a hide, a dry branch bent into the hoop, and cordage. stretch drum is the same. The hide may be a deer hide, a hare hide, or another, from the arms or lying here. Hide strips, then willow, then nettle, then sinew. The branch and the cordage are spent. With one of them missing, only you hear which
beat drum                    the drum in the arms. play drum is the same. Someone who can see you hears the beat. Past sight, a look says which way, one cell farther than the bone whistle, and one cell farther still, and that someone is beating a drum as a signal. It is not birdsong. The hare does not bolt. A deer within hearing lifts its head, and that line is not added to the beat. With no drum, only you hear it
make awl                     the same bone awl
set snare                    hide strips, willow cordage, nettle cordage, or sinew, from the arms or the ground, into a snare in this cell. After a short wait it catches small game. The valley hare, on this cell or the next, leaves a hare carcass. The line is spent. Hide strips are spent before willow, willow before nettle, and nettle before sinew.
set trap                     the same snare
shape clay                   a lump in your arms or at your feet into an unfired clay pot. It is not fired.
fire pot                     an unfired pot at a burning fire; it hardens and can hold water. The fire is not used up.
build lean-to                three dry branches into a lean-to in this cell. It breaks the wind.
gather reeds                 a bundle at the brook's edge
gather cattail               cattail roots from the shallows of the still. dig cattail roots is the same. They are not the reeds. Raw roots do not feed until they are cooked.
gather cattail fluff         a handful of fluff from those cattails, when they are not under the ice. gather fluff is the same. It is not the roots. The cattails have no fluff once it is taken, then it hangs there again after the same wait as the roots. Ice holds that wait. It still lights a fire when it is damp, where moss does not. The open rain puts it out.
thatch lean-to               a carried bundle of reeds on a lean-to already standing here. cover lean-to is the same. You are out of the wind, and the rain does not reach you
weave basket                 a sheet of bark, from the arms or the pile at your feet, into a bark basket. It holds one handful of berries, or one raw fish, that would not fit in the arms.
fold bark                    a sheet of bark into a bark cup. make bark cup is the same. The cup holds water. A clay pot is still a pot.
seal bark cup                a lump of pine resin on a bark cup in the arms or at your feet. The lump is spent. pitch bark cup is the same. An unsealed cup lets a drink soak through after 8 ticks. A sealed cup keeps it. With no cup, or no pine resin, only you hear which one is missing.
weave net                    three bundles of reeds, from the arms or the pile at your feet, into a fishing net. fish with it takes a fish. The net is not used up.
set net                      a carried fishing net, in the brook. It leaves the arms. After a short wait it catches a fish. The net stays until you gather it.
set weir                     one bundle of reeds and cordage, hide strips or willow or nettle, in the brook. After a short wait a raw fish lies there. The reeds and the cordage are spent. The weir is gone. The brook is not fished out. This is not a spear, a hook, or a set net.
weave mat                    two bundles of reeds, from the arms or the pile at your feet, into a reed mat. Set it or drop it, then rest on it. The open wind becomes a little wind.
set mat                      a carried reed mat, on this ground. It leaves the arms. rest works on it, as under a lean-to.
make net                     the same net
gather charcoal              a lump left when a small fire goes out, from rain or from the fuel ending
bank                         dry branches lying on a burning fire, covered with a handful of mud. After a short wait the wood chars into charcoal and the fire goes out. A fire that would burn out first is held until that wait. If the fire is put out, the wood comes back and does not char. The branches in your arms stay there. smother and cover fire are the same
make basket                  the same weave
put fish                     a raw fish from the arms into that basket. Berries and a fish do not share it.
wear hide                    a carried hide, against the cold. A sewn cloak on you does not take a carry slot
remove hide                  a worn hide; it returns to the arms and counts again
eat berries                  sweet berries, raw
eat dark berries             the other patch of berries
eat pale berries             pale berries
eat meat                     cooked meat, or salted meat, or smoked meat; raw meat is not food
gather salt                  a salt crust from the slope, or from the dry margin of the still. The ground is bare, then the crust forms again.
gather honey                 a honeycomb from the bees' nest in the tree on the east slope. The bees sting.
gather eggs                  eggs from the bird's nest in the tree on the west slope. They must be cooked.
gather moss                  a handful from the damp bank of the brook. It can be pressed on a sting.
dig worms                    a worm from damp earth, the bank or a place with mud. gather worms is the same. The damp earth has no worms once it is dug, then they are there again after 24 ticks. On a bone fish hook the worm is spent, and that stretch of the brook can be fished again on the next tick. Without a worm the hook is unchanged. One cell off, on a side or a corner, the reply names that step and does not move you. A miss stays with you
gather flint                  a flint nodule from the slope. One cell off, on a side or a corner, the reply names that step and does not move you. Farther than one cell, it does not invent a step. A miss stays with you
gather quartz                a clear stone from the rock face. gather clear stone is the same. It can spark. It does not grow back. strike flint on it makes sparks jump. With dry bark, dry moss, or dry cattail fluff in the arms, and a dry branch, the sparks catch and a fire burns. The stone and the flint stay. Without that tinder, the sparks stay sparks. A miss stays with you.
gather resin                 a lump of pine resin from the pine south of the trail. It is pitch. The pine is bare, then the lumps lie there again.
strip willow                 a bundle of willow cordage from the willow south of the brook. gather cordage is the same. The willow is bare, then the bundles hang there again. A bone fish hook spends a bundle as the line when the arms hold no hide strips.
strip willow bark            a strip of bark from that same willow. It is not the cordage. gather willow bark is the same. The willow has no bark once the strip is taken, then the strip grows back.
strip nettle                 a bundle of nettle cordage from the nettles east of the trail. gather nettle is the same. Bare hands sting. Mud on the skin does not. The nettles are bare, then they grow again. The bundle is cordage where hide strips or willow cordage work.
pull sinew                   sinew from a carcass or from raw meat. strip sinew is the same. The carcass or the meat stays, and a second pull says it is already stripped. The length is cordage for a snare or a drum, and it is not willow or nettle. A miss stays with you. Bare sinew names the verb.
press moss                   a carried handful on a sting. The stings clear.
dress sting                  the same press
use moss                     the same press
smoke nest                   a burning brand, or a fire in this cell or the next. The bees quiet, and the next gather does not sting.
eat honey                    the honeycomb. Hunger eases.
eat eggs                     cooked eggs. Raw eggs are not food. Hunger eases once they are cooked.
salt meat                    salt on a portion of raw meat. The salt is spent. The meat can be eaten cold.
salt fish                    salt on a raw fish. The salt is spent. The fish can be eaten cold, and it does not spoil.
smoke fish                   a raw fish hung by a lit fire, or set on the next cell, from the arms or the ground. It is not smoked yet. When the wait ends it can be eaten cold, and it does not spoil. smoke is the same
smoke meat                   a portion of raw meat hung by a lit fire, or set on the next cell, from the arms or the ground. It is not smoked yet. When the wait ends it can be eaten cold, and it does not spoil. smoke the meat is the same
eat fish                     a cooked fish, or salted fish, or smoked fish; raw fish is not food
eat mushrooms                mushrooms
eat cattail roots            cooked cattail roots. Raw roots do not feed.
eat flatbread                a cattail flatbread cooked from cattail flour. Raw flour does not feed.
eat carcass                  a carcass must be cooked
eat bark                     bark does not feed you. The sheet stays. It still lights a fire
smear mud                    mud on skin against the sun
smear ochre                  red ochre on the skin, a mark unlike mud
mark                         a carried lump of ochre leaves a red mark on this ground. A lump of charcoal leaves a black mark instead, and is used up. With neither, you have nothing to mark with
mark rock                    the same mark on the rock, when you stand under it. Rain washes an open mark and leaves one under the rock
smelt                        a carried lump of copper ore at a burning fire; after the same wait as clay, a copper bead. A hot charcoal fire does it in half that wait. Look says the ore is smelting in the hot charcoal fire. With no ore, and a lump one step away, only you hear which way
smelt iron                   a carried lump of iron ore at a hot charcoal fire; after the same wait as clay, a bloom of iron. A plain fire refuses it, even after a dry branch is fed in. With no ore, and a lump one step away, only you hear which way
feed                         an extra dry branch, or a carried lump of charcoal, into a burning fire; the fire burns hotter. Charcoal fed in makes it a charcoal fire, which iron needs. With neither, you have nothing to feed the fire
hammer|forge                 a carried copper bead at a fire, or with a stone, into a copper awl
hammer bloom                 a carried bloom of iron at a hotter fire, or with a stone, into an iron knife
scrape hide                  a raw hide with a flint flake, a flint nodule, or a stone. The hide is scraped. The tool stays. An unscraped hide will not sew into a cloak
pierce hide                  a carried or worn deer hide, with a copper awl; the hide is ready to sew. A wolf hide uses the same awl. An otter hide uses a bone awl the same way. A fox hide uses a copper awl or a bone awl the same way. A hare hide uses a copper awl or a bone awl the same way. A beaver hide uses a copper awl or a bone awl the same way
sew hide                     a scraped and pierced deer hide into a cloak, worn against the cold. A scraped hide that is not pierced names the awl already in the arms, or the awl at the feet when the arms have none. An awl one cell off names that awl and the step. With none nearby, it names a copper awl or a bone awl. A pierced wolf hide sews into a warmer cloak. A pierced otter hide sews into a cloak that keeps the rain and the brook off. A pierced beaver hide sews into a cloak that sheds the rain and the brook. A pierced fox hide sews into a cloak that breaks the wind and cuts the cold the way a deer cloak does. A pierced hare hide sews into a light cloak that takes a little of the cold off and leaves the wind and the rain
rest|sleep                   under the rock, a lean-to, or on a reed mat, tiredness eases; the open wind refuses
interrupt                    stop the work your body is in
help"
}

fn parse_signal(rest: &str) -> Result<Attempt, String> {
    let rest = rest.trim();
    if rest.is_empty() {
        return Ok(Attempt::Signal(None));
    }
    let green = match rest {
        "with reeds"
        | "with green reeds"
        | "reeds"
        | "with a bundle of reeds"
        | "a bundle of reeds" => Some(Material::Reed),
        "with pine needles" | "with needles" | "with green leaves" | "with leaves"
        | "pine needles" | "needles" | "green leaves" | "leaves" => Some(Material::Needle),
        _ => {
            return Err(
                "Lay green reeds or pine needles on the fire, or send the smoke up plain.".into(),
            );
        }
    };
    Ok(Attempt::Signal(green))
}

pub fn parse_attempt(line: &str) -> Result<Attempt, String> {
    let line = line.trim();
    if line.is_empty() {
        return Err("Say what you try. Type help.".into());
    }
    let lower = line.to_ascii_lowercase();
    let mut parts = lower.split_whitespace();
    let head = parts.next().unwrap();
    let rest = lower[head.len()..].trim();
    match head {
        "help" | "?" => Err(help_text().into()),
        "inventory" | "inv" if rest.is_empty() => Ok(Attempt::Arms),
        "snare" | "trap" if rest.is_empty() => Ok(Attempt::NameSnare),
        "canoe" | "dugout" if rest.is_empty() => Ok(Attempt::NameCanoe),
        "sinew" if rest.is_empty() => Ok(Attempt::NameSinew),
        "look" | "l" | "orient" => {
            if rest.is_empty() {
                Ok(Attempt::Look)
            } else if let Some(target) = rest.strip_prefix("at ") {
                Ok(Attempt::Examine(parse_target(target.trim())))
            } else {
                Ok(Attempt::Examine(parse_target(rest)))
            }
        }
        "examine" | "exam" | "ex" | "x" => Ok(Attempt::Examine(parse_target(rest))),
        "say" | "'" => {
            let spoken = line[head.len()..].trim();
            if spoken.is_empty() {
                Err("Say something.".into())
            } else {
                Ok(Attempt::Say(spoken.to_string()))
            }
        }
        "n" | "north" => Ok(Attempt::Move(Dir::North)),
        "s" | "south" => Ok(Attempt::Move(Dir::South)),
        "e" | "east" => Ok(Attempt::Move(Dir::East)),
        "w" | "west" => Ok(Attempt::Move(Dir::West)),
        "move" | "go" | "walk" => parse_dir(rest).map(Attempt::Move),
        "strike"
            if rest.is_empty()
                || rest == "flint"
                || rest == "quartz"
                || rest == "flint on quartz"
                || rest == "the flint"
                || rest == "the quartz" =>
        {
            Ok(Attempt::Strike)
        }
        "signal" => parse_signal(rest),
        "smoke" if rest == "signal" || rest.starts_with("signal ") => {
            parse_signal(rest.trim_start_matches("signal").trim())
        }
        "smoke"
            if rest.is_empty()
                || rest == "fish"
                || rest == "the fish"
                || rest == "a fish"
                || rest == "raw fish"
                || rest == "a raw fish" =>
        {
            Ok(Attempt::SmokeFish)
        }
        "smoke"
            if rest == "meat"
                || rest == "the meat"
                || rest == "raw meat"
                || rest == "the raw meat"
                || rest == "a portion of meat"
                || rest == "a portion of raw meat"
                || rest == "the portion"
                || rest == "portion"
                || rest == "a smaller piece"
                || rest == "a smaller piece of meat"
                || rest == "a smaller piece of raw meat"
                || rest == "the smaller piece" =>
        {
            Ok(Attempt::SmokeMeat)
        }
        "smoke"
            if rest == "nest"
                || rest == "the nest"
                || rest == "bees"
                || rest == "the bees"
                || rest == "bees nest"
                || rest == "the bees nest"
                || rest == "bees' nest"
                || rest == "the bees' nest" =>
        {
            Ok(Attempt::SmokeNest)
        }
        "take"
            if rest == "brand"
                || rest == "ember"
                || rest == "a brand"
                || rest == "an ember"
                || rest == "a burning brand"
                || rest == "burning brand"
                || rest == "the brand"
                || rest == "the ember" =>
        {
            Ok(Attempt::TakeBrand)
        }
        "press"
            if rest == "moss"
                || rest == "the moss"
                || rest == "a handful of moss"
                || rest == "sting"
                || rest == "the sting"
                || rest == "stings" =>
        {
            Ok(Attempt::DressSting)
        }
        "dress"
            if rest == "sting"
                || rest == "the sting"
                || rest == "stings"
                || rest == "the stings"
                || rest == "moss"
                || rest == "the moss" =>
        {
            Ok(Attempt::DressSting)
        }
        "use" if rest == "moss" || rest == "the moss" => Ok(Attempt::DressSting),
        "strip" | "pull"
            if rest == "sinew"
                || rest == "the sinew"
                || rest == "sinew cordage"
                || rest == "a length of sinew" =>
        {
            Ok(Attempt::PullSinew)
        }
        "strip"
            if rest == "nettle"
                || rest == "nettles"
                || rest == "the nettles"
                || rest == "nettle cordage" =>
        {
            Ok(Attempt::Gather(Material::Nettle))
        }
        "dig" if rest == "worm" || rest == "worms" || rest == "a worm" || rest == "the worms" => {
            Ok(Attempt::Gather(Material::Worm))
        }
        "dig"
            if rest == "cattail"
                || rest == "cattails"
                || rest == "roots"
                || rest == "cattail roots"
                || rest == "the cattail roots" =>
        {
            Ok(Attempt::Gather(Material::Cattail))
        }
        "strip"
            if rest == "willow bark"
                || rest == "the willow bark"
                || rest == "bark from the willow" =>
        {
            Ok(Attempt::Gather(Material::WillowBark))
        }
        "strip"
            if rest == "willow"
                || rest == "the willow"
                || rest == "withies"
                || rest == "cordage"
                || rest == "willow cordage" =>
        {
            Ok(Attempt::Gather(Material::Willow))
        }
        "gather" | "get" | "take" => parse_material(rest).map(Attempt::Gather),
        "drop" => parse_material(rest).map(Attempt::Drop),
        "grab" | "grip" => Ok(Attempt::Grab),
        "beach" if rest.is_empty() || rest == "canoe" || rest == "the canoe" => Ok(Attempt::Beach),
        "drag" | "pull" if rest == "canoe" || rest == "the canoe" => Ok(Attempt::Beach),
        "drag" | "pull" => parse_dir(rest).map(Attempt::Drag),
        "hollow" if rest == "log" || rest == "the log" => Ok(Attempt::HollowLog),
        "carve"
            if rest == "canoe" || rest == "a canoe" || rest == "the canoe" || rest == "dugout" =>
        {
            Ok(Attempt::CarveCanoe)
        }
        "launch" if rest.is_empty() || rest == "canoe" || rest == "the canoe" => {
            Ok(Attempt::Launch)
        }
        "seal" | "pitch" | "waterproof"
            if rest == "bark cup"
                || rest == "the bark cup"
                || rest == "a bark cup"
                || rest == "cup"
                || rest == "the cup" =>
        {
            Ok(Attempt::SealCup)
        }
        "seal" | "pitch" | "waterproof"
            if rest.is_empty()
                || rest == "canoe"
                || rest == "the canoe"
                || rest == "dugout"
                || rest == "the dugout" =>
        {
            Ok(Attempt::SealCanoe)
        }
        "board" if rest.is_empty() || rest == "canoe" || rest == "the canoe" => Ok(Attempt::Board),
        "paddle" if rest.is_empty() => Ok(Attempt::Paddle(None)),
        "paddle" => parse_dir(rest).map(|dir| Attempt::Paddle(Some(dir))),
        "disembark" if rest.is_empty() || rest == "canoe" || rest == "the canoe" => {
            Ok(Attempt::Disembark)
        }
        "leave" if rest == "canoe" || rest == "the canoe" => Ok(Attempt::Disembark),
        "release" | "ungrab" => Ok(Attempt::Release),
        "let" if rest == "go" => Ok(Attempt::Release),
        "interrupt" | "stop" => Ok(Attempt::Interrupt),
        "raise" if rest == "frame" => Ok(Attempt::RaiseFrame),
        "build" if rest == "frame" => Ok(Attempt::RaiseFrame),
        "build"
            if rest == "lean-to"
                || rest == "leanto"
                || rest == "shelter"
                || rest == "a lean-to" =>
        {
            Ok(Attempt::BuildLeanTo)
        }
        "thatch"
            if rest.is_empty()
                || rest == "lean-to"
                || rest == "leanto"
                || rest == "the lean-to"
                || rest == "shelter" =>
        {
            Ok(Attempt::Thatch)
        }
        "craft" | "make" | "carve"
            if rest == "bow drill"
                || rest == "a bow drill"
                || rest == "drill"
                || rest == "a drill" =>
        {
            Ok(Attempt::CarveDrill)
        }
        "craft" | "make" if rest == "bow" || rest == "a bow" => Ok(Attempt::CraftBow),
        "grind"
            if rest.is_empty()
                || rest == "cattail"
                || rest == "cattails"
                || rest == "roots"
                || rest == "cattail root"
                || rest == "cattail roots"
                || rest == "the cattail roots"
                || rest == "flour"
                || rest == "cattail flour" =>
        {
            Ok(Attempt::Grind)
        }
        "fold"
            if rest == "bark"
                || rest == "the bark"
                || rest == "birch bark"
                || rest == "a sheet of bark"
                || rest == "sheet of bark"
                || rest == "bark cup"
                || rest == "a bark cup"
                || rest == "birch bark cup" =>
        {
            Ok(Attempt::FoldCup)
        }
        "make"
            if rest == "bark cup"
                || rest == "a bark cup"
                || rest == "birch bark cup"
                || rest == "bark cup from bark"
                || rest == "a bark cup from bark" =>
        {
            Ok(Attempt::FoldCup)
        }
        "sew"
            if rest == "bark"
                || rest == "bark cup"
                || rest == "a bark cup"
                || rest == "birch bark"
                || rest == "birch bark cup" =>
        {
            Ok(Attempt::FoldCup)
        }
        "weave" | "make" if rest == "basket" || rest == "a basket" || rest == "bark basket" => {
            Ok(Attempt::WeaveBasket)
        }
        "weave" | "make"
            if rest == "mat" || rest == "a mat" || rest == "reed mat" || rest == "a reed mat" =>
        {
            Ok(Attempt::WeaveMat)
        }
        "weave" | "make"
            if rest == "net"
                || rest == "a net"
                || rest == "fishing net"
                || rest == "a fishing net" =>
        {
            Ok(Attempt::WeaveNet)
        }
        "carve" | "make"
            if rest == "awl" || rest == "an awl" || rest == "bone awl" || rest == "a bone awl" =>
        {
            Ok(Attempt::CarveAwl)
        }
        "carve"
            if rest == "paddle"
                || rest == "a paddle"
                || rest == "the paddle"
                || rest == "wooden paddle"
                || rest == "a wooden paddle" =>
        {
            Ok(Attempt::CarvePaddle)
        }
        "carve"
            if rest == "whistle"
                || rest == "a whistle"
                || rest == "the whistle"
                || rest == "bone whistle"
                || rest == "a bone whistle" =>
        {
            Ok(Attempt::CarveWhistle)
        }
        "blow" | "play"
            if rest == "whistle"
                || rest == "a whistle"
                || rest == "the whistle"
                || rest == "bone whistle"
                || rest == "a bone whistle"
                || rest == "the bone whistle" =>
        {
            Ok(Attempt::BlowWhistle)
        }
        "carve"
            if rest == "flute"
                || rest == "a flute"
                || rest == "the flute"
                || rest == "reed flute"
                || rest == "a reed flute" =>
        {
            Ok(Attempt::CarveFlute)
        }
        "blow" | "play"
            if rest == "flute"
                || rest == "a flute"
                || rest == "the flute"
                || rest == "reed flute"
                || rest == "a reed flute"
                || rest == "the reed flute" =>
        {
            Ok(Attempt::BlowFlute)
        }
        "sew" | "stretch" | "make" | "craft"
            if rest == "drum"
                || rest == "a drum"
                || rest == "the drum"
                || rest == "hide drum"
                || rest == "a hide drum" =>
        {
            Ok(Attempt::StretchDrum)
        }
        "beat" | "play"
            if rest == "drum"
                || rest == "a drum"
                || rest == "the drum"
                || rest == "hide drum"
                || rest == "a hide drum" =>
        {
            Ok(Attempt::BeatDrum)
        }
        "carve"
            if rest == "hook"
                || rest == "a hook"
                || rest == "the hook"
                || rest == "fish hook"
                || rest == "a fish hook"
                || rest == "bone hook"
                || rest == "a bone hook" =>
        {
            Ok(Attempt::CarveHook)
        }
        "salt" if rest == "meat" || rest == "raw meat" || rest == "the meat" => {
            Ok(Attempt::SaltMeat)
        }
        "salt"
            if rest == "fish"
                || rest == "raw fish"
                || rest == "the fish"
                || rest == "a fish"
                || rest == "a raw fish" =>
        {
            Ok(Attempt::SaltFish)
        }
        "put"
            if matches!(
                rest,
                "fish"
                    | "a fish"
                    | "the fish"
                    | "raw fish"
                    | "a raw fish"
                    | "the raw fish"
                    | "fish in basket"
                    | "fish in the basket"
                    | "the fish in the basket"
                    | "raw fish in basket"
                    | "raw fish in the basket"
                    | "the raw fish in the basket"
                    | "fish into basket"
                    | "fish into the basket"
                    | "the fish into the basket"
            ) =>
        {
            Ok(Attempt::PutFish)
        }
        "lay" if rest == "bark" => Ok(Attempt::LayBark),
        "roof" => Ok(Attempt::LayBark),
        "drink" => Ok(Attempt::Drink),
        "boil" => Ok(Attempt::Boil),
        "brew" | "steep"
            if rest == "willow"
                || rest == "willow bark"
                || rest == "willow bark tea"
                || rest == "willow tea"
                || rest == "a strip of willow bark"
                || rest == "strip of willow bark" =>
        {
            Ok(Attempt::SteepWillow)
        }
        "brew" | "steep"
            if rest == "pine"
                || rest == "needles"
                || rest == "pine needles"
                || rest == "pine needle"
                || rest == "pine tea"
                || rest == "pine needle tea"
                || rest == "a handful of pine needles" =>
        {
            Ok(Attempt::SteepPine)
        }
        "brew" | "steep"
            if rest.is_empty()
                || rest == "mint"
                || rest == "tea"
                || rest == "mint tea"
                || rest == "a handful of mint" =>
        {
            Ok(Attempt::Brew)
        }
        "brew" | "steep" => Err("Steep a handful of mint, pine needles, or willow bark.".into()),
        "set" if rest == "clay" => Ok(Attempt::SetClay),
        "set" if rest == "mat" || rest == "a mat" || rest == "the mat" || rest == "reed mat" => {
            Ok(Attempt::SetMat)
        }
        "set"
            if rest == "net"
                || rest == "a net"
                || rest == "the net"
                || rest == "fishing net"
                || rest == "a fishing net" =>
        {
            Ok(Attempt::SetNet)
        }
        "set"
            if rest == "snare"
                || rest == "a snare"
                || rest == "the snare"
                || rest == "trap"
                || rest == "a trap"
                || rest == "the trap" =>
        {
            Ok(Attempt::SetSnare)
        }
        "set"
            if rest == "weir"
                || rest == "a weir"
                || rest == "the weir"
                || rest == "fish weir"
                || rest == "a fish weir"
                || rest == "reed weir"
                || rest == "a reed weir" =>
        {
            Ok(Attempt::SetWeir)
        }
        "weir" if rest.is_empty() || rest == "fish" || rest == "the brook" => Ok(Attempt::SetWeir),
        "bind" if rest == "spear" => Ok(Attempt::BindSpear),
        "knap"
            if rest.is_empty() || rest == "flint" || rest == "nodule" || rest == "flint nodule" =>
        {
            Ok(Attempt::Knap)
        }
        "haft" if rest.is_empty() || rest == "spear" || rest == "flake" || rest == "flint" => {
            Ok(Attempt::Haft)
        }
        "point"
            if rest.is_empty()
                || rest == "branch"
                || rest == "spear"
                || rest == "a branch"
                || rest == "the branch" =>
        {
            Ok(Attempt::PointSpear)
        }
        "carve" if rest == "spear" || rest == "wooden spear" => Ok(Attempt::PointSpear),
        "harden"
            if rest.is_empty()
                || rest == "spear"
                || rest == "point"
                || rest == "the point"
                || rest == "wooden spear" =>
        {
            Ok(Attempt::HardenSpear)
        }
        "fire" if rest == "harden" || rest == "harden spear" => Ok(Attempt::HardenSpear),
        "fish" if rest.is_empty() || rest == "spear" => Ok(Attempt::Fish),
        "fish"
            if rest == "with hook"
                || rest == "with a hook"
                || rest == "with the hook"
                || rest == "hook"
                || rest == "with a fish hook"
                || rest == "with the fish hook"
                || rest == "with fish hook" =>
        {
            Ok(Attempt::FishHook)
        }
        "spear" if rest == "fish" => Ok(Attempt::Fish),
        "break" | "chop"
            if rest.is_empty()
                || rest == "ice"
                || rest == "the ice"
                || rest == "a hole"
                || rest == "hole"
                || rest == "the hole"
                || rest == "ice hole"
                || rest == "the ice hole"
                || rest == "hole in the ice"
                || rest == "a hole in the ice"
                || rest == "the hole in the ice" =>
        {
            Ok(Attempt::BreakIce)
        }
        "dry" if rest == "clothes" || rest == "my clothes" || rest == "the clothes" => {
            Ok(Attempt::DryClothes)
        }
        "wait" if rest == "by the fire" || rest == "by fire" || rest == "at the fire" => {
            Ok(Attempt::DryClothes)
        }
        "dry" if rest.is_empty() || rest == "branch" || rest == "wood" || rest == "the branch" => {
            Ok(Attempt::DryBranch)
        }
        "hunt"
            if rest.is_empty()
                || rest == "deer"
                || rest == "wolf"
                || rest == "otter"
                || rest == "fox"
                || rest == "hare" =>
        {
            Ok(Attempt::Hunt)
        }
        "butcher" if rest.is_empty() || rest == "carcass" || rest == "deer" || rest == "wolf" => {
            Ok(Attempt::Butcher)
        }
        "cut" if rest == "meat" || rest == "raw meat" || rest == "cooked meat" => Ok(Attempt::Cut),
        "cut"
            if rest == "hide"
                || rest == "deer hide"
                || rest == "the hide"
                || rest == "strips"
                || rest == "hide strips" =>
        {
            Ok(Attempt::CutHide)
        }
        "shape" if rest.is_empty() || rest == "clay" || rest == "pot" || rest == "clay pot" => {
            Ok(Attempt::Shape)
        }
        "bake" if rest.is_empty() || rest == "pot" || rest == "clay pot" || rest == "the pot" => {
            Ok(Attempt::FirePot)
        }
        "fire" if rest == "pot" || rest == "clay pot" || rest == "the pot" => Ok(Attempt::FirePot),
        "wear" | "wrap"
            if rest.is_empty()
                || rest == "hide"
                || rest == "deer hide"
                || rest == "wolf hide"
                || rest == "otter hide"
                || rest == "fox hide"
                || rest == "beaver hide"
                || rest == "wolf cloak"
                || rest == "otter cloak"
                || rest == "fox cloak"
                || rest == "beaver cloak"
                || rest == "cloak" =>
        {
            Ok(Attempt::Wear)
        }
        "remove"
            if rest.is_empty()
                || rest == "hide"
                || rest == "deer hide"
                || rest == "wolf hide"
                || rest == "otter hide"
                || rest == "fox hide"
                || rest == "beaver hide"
                || rest == "wolf cloak"
                || rest == "otter cloak"
                || rest == "fox cloak"
                || rest == "beaver cloak"
                || rest == "cloak" =>
        {
            Ok(Attempt::Remove)
        }
        "light" | "kindle"
            if rest == "fire with drill"
                || rest == "fire with the drill"
                || rest == "fire with a drill"
                || rest == "fire with bow drill"
                || rest == "fire with the bow drill"
                || rest == "with drill"
                || rest == "with the drill"
                || rest == "with a bow drill"
                || rest == "with the bow drill"
                || rest == "drill"
                || rest == "the drill"
                || rest == "bow drill"
                || rest == "the bow drill" =>
        {
            Ok(Attempt::DrillFire)
        }
        "drill" if rest.is_empty() || rest == "fire" || rest == "the fire" => {
            Ok(Attempt::DrillFire)
        }
        "light" | "kindle" if rest.is_empty() || rest == "fire" => Ok(Attempt::Light),
        "cook" => Ok(Attempt::Cook),
        "eat" => parse_eat(rest),
        "smear" if rest.is_empty() || rest == "mud" => Ok(Attempt::Smear),
        "smear" if rest == "ochre" => Ok(Attempt::SmearOchre),
        "mark" if rest.is_empty() || rest == "ground" => Ok(Attempt::Mark),
        "mark" if rest == "rock" => Ok(Attempt::MarkRock),
        "smelt" if rest.is_empty() || rest == "ore" || rest == "copper" || rest == "copper ore" => {
            Ok(Attempt::Smelt)
        }
        "smelt" if rest == "iron" || rest == "iron ore" => Ok(Attempt::SmeltIron),
        "feed"
            if rest.is_empty()
                || rest == "fire"
                || rest == "branch"
                || rest == "the fire"
                || rest == "charcoal"
                || rest == "lump of charcoal" =>
        {
            Ok(Attempt::Feed)
        }
        "bank" | "smother"
            if rest.is_empty()
                || rest == "fire"
                || rest == "the fire"
                || rest == "branches"
                || rest == "wood" =>
        {
            Ok(Attempt::Bank)
        }
        "cover" if rest == "fire" || rest == "the fire" => Ok(Attempt::Bank),
        "cover"
            if rest == "lean-to"
                || rest == "the lean-to"
                || rest == "leanto"
                || rest == "a lean-to" =>
        {
            Ok(Attempt::Thatch)
        }
        "hammer" | "forge"
            if rest.is_empty() || rest == "bead" || rest == "copper" || rest == "copper bead" =>
        {
            Ok(Attempt::Hammer)
        }
        "hammer" | "forge"
            if rest == "bloom"
                || rest == "iron"
                || rest == "bloom of iron"
                || rest == "knife"
                || rest == "iron knife" =>
        {
            Ok(Attempt::HammerIron)
        }
        "scrape"
            if rest.is_empty()
                || rest == "hide"
                || rest == "deer hide"
                || rest == "wolf hide"
                || rest == "otter hide"
                || rest == "fox hide"
                || rest == "hare hide"
                || rest == "beaver hide" =>
        {
            Ok(Attempt::Scrape)
        }
        "pierce"
            if rest.is_empty()
                || rest == "hide"
                || rest == "deer hide"
                || rest == "wolf hide"
                || rest == "otter hide"
                || rest == "fox hide"
                || rest == "beaver hide" =>
        {
            Ok(Attempt::Pierce)
        }
        "sew"
            if rest.is_empty()
                || rest == "hide"
                || rest == "deer hide"
                || rest == "wolf hide"
                || rest == "otter hide"
                || rest == "fox hide"
                || rest == "beaver hide"
                || rest == "cloak"
                || rest == "wolf cloak"
                || rest == "otter cloak"
                || rest == "fox cloak"
                || rest == "beaver cloak" =>
        {
            Ok(Attempt::Sew)
        }
        "rest" | "sleep" if rest.is_empty() => Ok(Attempt::Rest),
        _ => Err("Unknown attempt. Type help.".into()),
    }
}

fn parse_dir(text: &str) -> Result<Dir, String> {
    match text.trim() {
        "n" | "north" => Ok(Dir::North),
        "s" | "south" => Ok(Dir::South),
        "e" | "east" => Ok(Dir::East),
        "w" | "west" => Ok(Dir::West),
        _ => Err("Name a direction: n, s, e, or w.".into()),
    }
}

fn parse_material(text: &str) -> Result<Material, String> {
    match text.trim() {
        "branch" | "branches" | "wood" | "firewood" => Ok(Material::Branch),
        "bark cup" | "a bark cup" | "birch bark cup" => Ok(Material::BarkCup),
        "bark" | "birch" | "birch bark" | "sheet of birch bark" => Ok(Material::Bark),
        "log" => Ok(Material::Log),
        "berry" | "berries" | "sweet berries" => Ok(Material::Berry),
        "bitter" | "bitter berry" | "bitter berries" | "poison" | "poison berry"
        | "poison berries" | "dark" | "dark berry" | "dark berries" => Ok(Material::BitterBerry),
        "mushroom" | "mushrooms" => Ok(Material::Mushroom),
        "pale" | "pale berry" | "pale berries" => Ok(Material::PaleBerry),
        "pot" => Ok(Material::Pot),
        "mud" => Ok(Material::Mud),
        "water" => Ok(Material::Water),
        "stone" | "stones" => Ok(Material::Stone),
        "flint" | "flint nodule" | "nodule" => Ok(Material::Flint),
        "ochre" => Ok(Material::Ochre),
        "iron" | "iron ore" => Ok(Material::IronOre),
        "bloom" | "iron bloom" | "bloom of iron" | "lump of iron" => Ok(Material::Iron),
        "knife" | "iron knife" => Ok(Material::Knife),
        "copper ore" | "ore" => Ok(Material::CopperOre),
        "copper" | "copper bead" | "bead" => Ok(Material::Copper),
        "awl" | "copper awl" | "bone awl" => Ok(Material::Awl),
        "bone" => Ok(Material::Bone),
        "whistle" | "bone whistle" | "a bone whistle" | "the bone whistle" => {
            Ok(Material::Whistle)
        }
        "flute" | "reed flute" | "a reed flute" | "the reed flute" => Ok(Material::Flute),
        "drum" | "a drum" | "the drum" | "hide drum" => Ok(Material::Drum),
        "salt" | "salt crust" => Ok(Material::Salt),
        "honey" | "honeycomb" | "bees nest" | "bee nest" => Ok(Material::Honey),
        "egg" | "eggs" => Ok(Material::Egg),
        "moss" | "handful of moss" => Ok(Material::Moss),
        "worm" | "worms" | "a worm" => Ok(Material::Worm),
        "quartz" | "clear stone" | "clear stone of quartz" | "quartz stone" => Ok(Material::Quartz),
        "mint" | "handful of mint" | "wild mint" => Ok(Material::Mint),
        "pine needles" | "needles" | "handful of pine needles" | "pine needle" | "pine" => {
            Ok(Material::Needle)
        }
        "bow drill" | "a bow drill" | "drill" | "a drill" => Ok(Material::Drill),
        "resin"
        | "pine resin"
        | "pitch"
        | "lump of pitch"
        | "lumps of pitch"
        | "lump of pine resin"
        | "lumps of pine resin" => Ok(Material::Resin),
        "nettle" | "nettles" | "nettle cordage" | "bundle of nettle cordage" => {
            Ok(Material::Nettle)
        }
        "sinew" | "sinew cordage" | "length of sinew" | "a length of sinew" => Ok(Material::Sinew),
        "willow bark" | "strip of willow bark" | "a strip of willow bark" => Ok(Material::WillowBark),
        "willow"
        | "willow cordage"
        | "bundle of willow cordage"
        | "withies"
        | "cordage" => Ok(Material::Willow),
        "bow" | "a bow" => Ok(Material::Bow),
        "clay" => Ok(Material::Clay),
        "bowl" => Ok(Material::Bowl),
        "spear" | "spears" => Ok(Material::Spear),
        "carcass" | "deer carcass" | "wolf carcass" => Ok(Material::Carcass),
        "meat" | "raw meat" | "cooked meat" => Ok(Material::Meat),
        "fish" | "raw fish" | "cooked fish" => Ok(Material::Fish),
        "brand" | "ember" | "burning brand" => Ok(Material::Brand),
        "hide" | "deer hide" | "wolf hide" | "otter hide" | "fox hide" | "hare hide"
        | "beaver hide" | "wolf cloak" | "otter cloak" | "fox cloak" | "hare cloak"
        | "beaver cloak" => Ok(Material::Hide),
        "basket" | "bark basket" => Ok(Material::Basket),
        "charcoal" | "lump of charcoal" => Ok(Material::Charcoal),
        "reed" | "reeds" | "bundle of reeds" => Ok(Material::Reed),
        "flour" | "cattail flour" | "flatbread" | "cattail flatbread" => Ok(Material::Flour),
        "fluff"
        | "cattail fluff"
        | "handful of cattail fluff"
        | "down"
        | "cattail down" => Ok(Material::Fluff),
        "cattail" | "cattails" | "cattail root" | "cattail roots" | "roots" => {
            Ok(Material::Cattail)
        }
        "net" | "fishing net" | "a fishing net" => Ok(Material::Net),
        "mat" | "reed mat" | "a reed mat" => Ok(Material::Mat),
        "canoe" | "dugout" | "dugout canoe" => Ok(Material::Canoe),
        "paddle" | "wooden paddle" | "a wooden paddle" => Ok(Material::Paddle),
        "hook" | "fish hook" | "a fish hook" | "bone hook" | "bone fish hook" => Ok(Material::Hook),
        "strips" | "hide strips" | "cord" => Ok(Material::Cord),
        _ => Err(
            "Name a material: branch, bark, berries, dark berries, pale berries, mushrooms, pot, mud, water, stone, flint, ochre, iron ore, bloom of iron, iron knife, copper ore, copper bead, awl, clay, bowl, spear, carcass, meat, hide, strips, or log."
                .into(),
        ),
    }
}

fn parse_eat(text: &str) -> Result<Attempt, String> {
    match text.trim() {
        "" | "berry" | "berries" | "sweet" | "sweet berries" => Ok(Attempt::Eat(Material::Berry)),
        "mushroom" | "mushrooms" => Ok(Attempt::Eat(Material::Mushroom)),
        "cattail" | "cattails" | "cattail root" | "cattail roots" | "roots" => {
            Ok(Attempt::Eat(Material::Cattail))
        }
        "flour" | "cattail flour" | "flatbread" | "cattail flatbread" | "the flatbread" => {
            Ok(Attempt::Eat(Material::Flour))
        }
        "pale" | "pale berry" | "pale berries" => Ok(Attempt::Eat(Material::PaleBerry)),
        "bitter" | "bitter berry" | "bitter berries" | "poison" | "poison berry"
        | "poison berries" | "dark" | "dark berry" | "dark berries" => {
            Ok(Attempt::Eat(Material::BitterBerry))
        }
        "bark" | "sheet" | "sheet of bark" | "sheets" | "sheets of bark" | "dry bark" => {
            Ok(Attempt::Eat(Material::Bark))
        }
        "wood" | "branch" | "branches" | "log" => Err("Bark is tinder, not food.".into()),
        "mud" => Err("Mud is for your skin, not your stomach.".into()),
        "carcass" | "deer carcass" => Ok(Attempt::Eat(Material::Carcass)),
        "meat" | "cooked meat" | "raw meat" | "salted meat" | "smoked meat" | "the smoked meat" => {
            Ok(Attempt::Eat(Material::Meat))
        }
        "fish" | "cooked fish" | "raw fish" => Ok(Attempt::Eat(Material::Fish)),
        "honey" | "honeycomb" => Ok(Attempt::Eat(Material::Honey)),
        "egg" | "eggs" | "cooked eggs" => Ok(Attempt::Eat(Material::Egg)),
        _ => Err("Name a food: berries, dark berries, pale berries, mushrooms, or carcass.".into()),
    }
}

fn parse_target(text: &str) -> ExamineTarget {
    match text.trim() {
        "" | "here" | "ground" | "around" => ExamineTarget::Here,
        "self" | "me" | "myself" => ExamineTarget::SelfBody,
        "wood" | "branch" | "branches" | "firewood" => ExamineTarget::Wood,
        "bark cup" | "birch bark cup" => ExamineTarget::BarkCup,
        "bark" => ExamineTarget::Bark,
        "berry" | "berries" | "fruit" | "sweet berries" => ExamineTarget::Berries,
        "bitter" | "bitter berries" | "bitter berry" | "poison" | "poison berries"
        | "poison berry" | "dark" | "dark berries" | "dark berry" => ExamineTarget::BitterBerries,
        "mud" => ExamineTarget::Mud,
        "mushroom" | "mushrooms" => ExamineTarget::Mushrooms,
        "pale" | "pale berries" | "pale berry" => ExamineTarget::PaleBerries,
        "pot" => ExamineTarget::Pot,
        "stone" | "stones" => ExamineTarget::Stone,
        "flint" | "flint nodule" | "nodule" => ExamineTarget::Flint,
        "ochre" => ExamineTarget::Ochre,
        "iron" | "iron ore" => ExamineTarget::IronOre,
        "bloom" | "iron bloom" | "bloom of iron" | "lump of iron" => ExamineTarget::Iron,
        "knife" | "iron knife" => ExamineTarget::Knife,
        "copper" | "copper ore" | "ore" => ExamineTarget::CopperOre,
        "copper bead" | "bead" => ExamineTarget::Copper,
        "awl" | "copper awl" => ExamineTarget::Awl,
        "flake" | "flint flake" | "sharp flake" => ExamineTarget::Flake,
        "clay" => ExamineTarget::Clay,
        "bowl" => ExamineTarget::Bowl,
        "spear" | "spears" => ExamineTarget::Spear,
        "carcass" | "deer carcass" | "wolf carcass" => ExamineTarget::Carcass,
        "hide" | "deer hide" | "wolf hide" | "otter hide" | "fox hide" | "hare hide"
        | "beaver hide" | "wolf cloak" | "otter cloak" | "fox cloak" | "hare cloak"
        | "beaver cloak" => ExamineTarget::Hide,
        "strips" | "hide strips" | "cord" => ExamineTarget::Cord,
        "sinew" | "sinew cordage" | "length of sinew" | "a length of sinew" => ExamineTarget::Sinew,
        "nettle" | "nettles" | "nettle cordage" => ExamineTarget::Nettle,
        "willow bark" | "strip of willow bark" => ExamineTarget::WillowBark,
        "willow" | "willow cordage" | "withies" | "cordage" => ExamineTarget::Willow,
        "meat" | "raw meat" | "cooked meat" | "salted meat" | "smoked meat" => ExamineTarget::Meat,
        "fish" | "raw fish" | "cooked fish" => ExamineTarget::Fish,
        "brand" | "ember" | "burning brand" => ExamineTarget::Brand,
        "fire" | "flame" => ExamineTarget::Fire,
        "water" | "brook" => ExamineTarget::Water,
        "deer" | "animal" => ExamineTarget::Deer,
        "hare" | "a hare" | "the hare" => ExamineTarget::Hare,
        "tracks" | "hoofprints" | "prints" => ExamineTarget::Tracks,
        "log" => ExamineTarget::Log,
        "frame" | "shelter" | "roof" => ExamineTarget::Frame,
        "rock" | "overhang" | "rock overhang" => ExamineTarget::Rock,
        "basket" | "bark basket" => ExamineTarget::Basket,
        "charcoal" | "lump of charcoal" => ExamineTarget::Charcoal,
        "reed" | "reeds" | "bundle of reeds" => ExamineTarget::Reeds,
        "fluff" | "cattail fluff" | "down" | "cattail down" => ExamineTarget::Fluff,
        "cattail" | "cattails" | "cattail root" | "cattail roots" => ExamineTarget::Cattail,
        "net" | "fishing net" | "a fishing net" => ExamineTarget::Net,
        "mat" | "reed mat" | "a reed mat" => ExamineTarget::Mat,
        "canoe" | "dugout" | "dugout canoe" => ExamineTarget::Canoe,
        "bone" => ExamineTarget::Bone,
        "salt" | "salt crust" => ExamineTarget::Salt,
        "honey" | "honeycomb" | "bees" | "bees nest" | "nest" => ExamineTarget::Honey,
        "egg" | "eggs" | "bird nest" | "bird's nest" | "birds nest" => ExamineTarget::Eggs,
        "moss" => ExamineTarget::Moss,
        "mint" | "wild mint" | "handful of mint" => ExamineTarget::Mint,
        "pine needles" | "needles" | "handful of pine needles" | "pine" => ExamineTarget::Needle,
        "bow drill" | "a bow drill" | "drill" | "a drill" => ExamineTarget::Drill,
        "resin" | "pine resin" | "pitch" | "lump of pitch" | "lump of pine resin" => {
            ExamineTarget::Resin
        }
        "bow" | "a bow" => ExamineTarget::Bow,
        "paddle" | "wooden paddle" | "a wooden paddle" => ExamineTarget::Paddle,
        "hook" | "fish hook" | "a fish hook" | "bone hook" | "bone fish hook" => {
            ExamineTarget::Hook
        }
        "bone awl" => ExamineTarget::Awl,
        "snare" | "trap" | "hide snare" => ExamineTarget::Snare,
        "weir" | "reed weir" | "fish weir" | "a reed weir" => ExamineTarget::Weir,
        "slope" | "the slope" => ExamineTarget::Slope,
        other => ExamineTarget::Person(other.to_string()),
    }
}
