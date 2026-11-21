//! Player lines become attempts. The core still decides whether they happen.

use crate::model::{Attempt, Dir, ExamineTarget, Material};

pub fn help_text() -> &'static str {
    "\
look                         where you are, and the local map
examine wood|bark|berries|dark berries|pale berries|mushrooms|pot|mud|water|stone|flint|ochre|iron ore|bloom of iron|iron knife|copper ore|copper bead|awl|flake|clay|bowl|spear|carcass|hide|brand|fire|deer|tracks|log|frame|rock|slope|eggs|moss|bow|paddle|hook|self|here|<name>
say <words>                  people nearby hear you
n s e w                      walk
gather branch|bark|berries|dark berries|pale berries|mushrooms|pot|mud|water|stone|flint|ochre|iron ore|bloom of iron|iron knife|copper ore|copper bead|awl|clay|bowl|spear|carcass|meat|hide
drop branch|bark|berries|dark berries|pale berries|mushrooms|pot|mud|water|stone|flint|ochre|iron ore|bloom of iron|iron knife|copper ore|copper bead|awl|clay|bowl|spear|carcass|meat|hide
set clay                     a lump at a burning fire; it hardens into a bowl
bind spear                   a carried stone and a carried branch; hold it at a fire
knap                         a carried flint nodule into a sharp flake
haft                         a carried flint flake and a dry branch into a spear; hide strips, if carried, bind the tip. Hold it at a fire
craft bow                    a dry branch and hide strips, from the arms if you carry them, or lying in this cell. Both are spent. The bow stays in the arms.
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
drink                        raw water where you stand; still water freezes until the day thaws it. The brook does not freeze
boil                         a pot or a fired bowl, and a fire; unfired clay cannot
light fire                   dry bark and a dry branch, from the arms if you carry them
cook                         heat a mushroom, or cook berries, a carcass, or raw meat
hunt                         a hardened spear; the deer on your cell or the next, or a wolf while it stands on your cell or the next. With no spear that can reach, a carried bow and a sharp flint flake reach as far as you can see. The flake is spent when the deer or the wolf comes down. A refusal leaves it. The bow stays.
fish                         a fire-hardened flint spear, or a fishing net, or a bone fish hook with hide strips, in the brook or beside it, and from a dugout on the brook. The spear and the net are not used up. The hide strips are spent as the line. That stretch is quiet for a short wait
fish with hook               the same cast with a bone fish hook and hide strips in the arms. The hook stays. The strips are spent.
spear fish                   the same cast with the spear
butcher                      a carcass at your feet into three portions; a flint spear or an iron knife in your arms. With both, the knife cuts. Neither is used up.
cut meat                     one portion of raw or cooked meat into two smaller pieces; an iron knife in your arms. The knife is not used up.
cut hide                     a whole deer hide into hide strips, with an iron knife. The knife is not used up.
carve awl                    a bone, with a flint flake or an iron knife, into a bone awl. The tool is not used up. The awl can pierce hide.
carve paddle                 a dry branch, from the arms or on the ground, with a flint flake or an iron knife. The tool is not used up. The wooden paddle stays in the arms. Holding it names the paddle when you paddle. Bare hands still paddle.
carve hook                   a bone, from the arms or on the ground, with a flint flake or an iron knife. The tool is not used up. The bone fish hook stays in the arms.
make awl                     the same bone awl
set snare                    hide strips, from the arms or the ground, into a snare in this cell. After a short wait it catches a hare. The strips are spent.
set trap                     the same snare
shape clay                   a lump in your arms or at your feet into an unfired clay pot. It is not fired.
fire pot                     an unfired pot at a burning fire; it hardens and can hold water. The fire is not used up.
build lean-to                three dry branches into a lean-to in this cell. It breaks the wind.
gather reeds                 a bundle at the brook's edge
thatch lean-to               a carried bundle of reeds on a lean-to already standing here. You are out of the wind, and the rain does not reach you
weave basket                 a sheet of bark, from the arms or the pile at your feet, into a bark basket. It holds one handful of berries, or one raw fish, that would not fit in the arms.
weave net                    three bundles of reeds, from the arms or the pile at your feet, into a fishing net. fish with it takes a fish. The net is not used up.
set net                      a carried fishing net, in the brook. It leaves the arms. After a short wait it catches a fish. The net stays until you gather it.
weave mat                    two bundles of reeds, from the arms or the pile at your feet, into a reed mat. Set it or drop it, then rest on it. The open wind becomes a little wind.
set mat                      a carried reed mat, on this ground. It leaves the arms. rest works on it, as under a lean-to.
make net                     the same net
gather charcoal              a lump left when a small fire goes out, from rain or from the fuel ending
make basket                  the same weave
put fish                     a raw fish from the arms into that basket. Berries and a fish do not share it.
wear hide                    a carried deer hide, against the cold
remove hide                  a worn deer hide; it returns to the arms
eat berries                  sweet berries, raw
eat dark berries             the other patch of berries
eat pale berries             pale berries
eat meat                     cooked meat, or salted meat, or smoked meat; raw meat is not food
gather salt                  a salt crust from the slope. The slope is bare, then the crust forms again.
gather honey                 a honeycomb from the bees' nest in the tree on the east slope. The bees sting.
gather eggs                  eggs from the bird's nest in the tree on the west slope. They must be cooked.
gather moss                  a handful from the damp bank of the brook. It can be pressed on a sting.
gather resin                 a lump of pine resin from the pine south of the trail. It is pitch. The pine is bare, then the lumps lie there again.
press moss                   a carried handful on a sting. The stings clear.
dress sting                  the same press
use moss                     the same press
smoke nest                   a burning brand, or a fire in this cell or the next. The bees quiet, and the next gather does not sting.
eat honey                    the honeycomb. Hunger eases.
eat eggs                     cooked eggs. Raw eggs are not food. Hunger eases once they are cooked.
salt meat                    salt on a portion of raw meat. The salt is spent. The meat can be eaten cold.
salt fish                    salt on a raw fish. The salt is spent. The fish can be eaten cold, and it does not spoil.
smoke fish                   a raw fish at a lit fire, from the arms or the ground. It can be eaten cold, and it does not spoil. smoke the fish is the same
smoke meat                   a portion of raw meat at a lit fire, from the arms or the ground. It can be eaten cold, and it does not spoil. smoke the meat is the same
eat fish                     a cooked fish, or salted fish, or smoked fish; raw fish is not food
eat mushrooms                mushrooms
eat carcass                  a carcass must be cooked
eat bark                     bark does not feed you. The sheet stays. It still lights a fire
smear mud                    mud on skin against the sun
smear ochre                  red ochre on the skin, a mark unlike mud
mark                         a carried lump of ochre leaves a red mark on this ground. A lump of charcoal leaves a black mark instead, and is used up. With neither, you have nothing to mark with
mark rock                    the same mark on the rock, when you stand under it. Rain washes an open mark and leaves one under the rock
smelt                        a carried lump of copper ore at a burning fire; after the same wait as clay, a copper bead
smelt iron                   a carried lump of iron ore; a small fire refuses it, and a fire fed an extra dry branch yields a bloom of iron
feed                         an extra dry branch, or a carried lump of charcoal, into a burning fire; the fire burns hot enough for iron. With neither, you have nothing to feed the fire
hammer|forge                 a carried copper bead at a fire, or with a stone, into a copper awl
hammer bloom                 a carried bloom of iron at a hotter fire, or with a stone, into an iron knife
pierce hide                  a carried or worn deer hide, with a copper awl; the hide is ready to sew. A wolf hide uses the same awl
sew hide                     a pierced deer hide into a cloak, worn against the cold. A pierced wolf hide sews into a warmer cloak
rest|sleep                   under the rock, a lean-to, or on a reed mat, tiredness eases; the open wind refuses
interrupt                    stop the work your body is in
help"
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
        "smoke"
            if rest == "fish"
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
                || rest == "a portion of raw meat" =>
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
        "craft" | "make" if rest == "bow" || rest == "a bow" => Ok(Attempt::CraftBow),
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
        "bind" if rest == "spear" => Ok(Attempt::BindSpear),
        "knap"
            if rest.is_empty() || rest == "flint" || rest == "nodule" || rest == "flint nodule" =>
        {
            Ok(Attempt::Knap)
        }
        "haft" if rest.is_empty() || rest == "spear" || rest == "flake" || rest == "flint" => {
            Ok(Attempt::Haft)
        }
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
        "dry" if rest == "clothes" || rest == "my clothes" || rest == "the clothes" => {
            Ok(Attempt::DryClothes)
        }
        "wait" if rest == "by the fire" || rest == "by fire" || rest == "at the fire" => {
            Ok(Attempt::DryClothes)
        }
        "dry" if rest.is_empty() || rest == "branch" || rest == "wood" || rest == "the branch" => {
            Ok(Attempt::DryBranch)
        }
        "hunt" if rest.is_empty() || rest == "deer" || rest == "wolf" => Ok(Attempt::Hunt),
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
                || rest == "wolf cloak"
                || rest == "cloak" =>
        {
            Ok(Attempt::Wear)
        }
        "remove"
            if rest.is_empty()
                || rest == "hide"
                || rest == "deer hide"
                || rest == "wolf hide"
                || rest == "wolf cloak"
                || rest == "cloak" =>
        {
            Ok(Attempt::Remove)
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
        "pierce"
            if rest.is_empty() || rest == "hide" || rest == "deer hide" || rest == "wolf hide" =>
        {
            Ok(Attempt::Pierce)
        }
        "sew"
            if rest.is_empty()
                || rest == "hide"
                || rest == "deer hide"
                || rest == "wolf hide"
                || rest == "cloak"
                || rest == "wolf cloak" =>
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
        "bark" => Ok(Material::Bark),
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
        "salt" | "salt crust" => Ok(Material::Salt),
        "honey" | "honeycomb" | "bees nest" | "bee nest" => Ok(Material::Honey),
        "egg" | "eggs" => Ok(Material::Egg),
        "moss" | "handful of moss" => Ok(Material::Moss),
        "resin"
        | "pine resin"
        | "pitch"
        | "lump of pitch"
        | "lumps of pitch"
        | "lump of pine resin"
        | "lumps of pine resin" => Ok(Material::Resin),
        "bow" | "a bow" => Ok(Material::Bow),
        "clay" => Ok(Material::Clay),
        "bowl" => Ok(Material::Bowl),
        "spear" | "spears" => Ok(Material::Spear),
        "carcass" | "deer carcass" | "wolf carcass" => Ok(Material::Carcass),
        "meat" | "raw meat" | "cooked meat" => Ok(Material::Meat),
        "fish" | "raw fish" | "cooked fish" => Ok(Material::Fish),
        "brand" | "ember" | "burning brand" => Ok(Material::Brand),
        "hide" | "deer hide" | "wolf hide" | "wolf cloak" => Ok(Material::Hide),
        "basket" | "bark basket" => Ok(Material::Basket),
        "charcoal" | "lump of charcoal" => Ok(Material::Charcoal),
        "reed" | "reeds" | "bundle of reeds" => Ok(Material::Reed),
        "net" | "fishing net" | "a fishing net" => Ok(Material::Net),
        "mat" | "reed mat" | "a reed mat" => Ok(Material::Mat),
        "canoe" | "dugout" | "dugout canoe" => Ok(Material::Canoe),
        "paddle" | "wooden paddle" | "a wooden paddle" => Ok(Material::Paddle),
        "hook" | "fish hook" | "a fish hook" | "bone hook" | "bone fish hook" => Ok(Material::Hook),
        "strips" | "hide strips" | "cord" | "cordage" => Ok(Material::Cord),
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
        "hide" | "deer hide" | "wolf hide" | "wolf cloak" => ExamineTarget::Hide,
        "strips" | "hide strips" | "cord" | "cordage" => ExamineTarget::Cord,
        "meat" | "raw meat" | "cooked meat" | "salted meat" | "smoked meat" => ExamineTarget::Meat,
        "fish" | "raw fish" | "cooked fish" => ExamineTarget::Fish,
        "brand" | "ember" | "burning brand" => ExamineTarget::Brand,
        "fire" | "flame" => ExamineTarget::Fire,
        "water" | "brook" => ExamineTarget::Water,
        "deer" | "animal" => ExamineTarget::Deer,
        "tracks" | "hoofprints" | "prints" => ExamineTarget::Tracks,
        "log" => ExamineTarget::Log,
        "frame" | "shelter" | "roof" => ExamineTarget::Frame,
        "rock" | "overhang" | "rock overhang" => ExamineTarget::Rock,
        "basket" | "bark basket" => ExamineTarget::Basket,
        "charcoal" | "lump of charcoal" => ExamineTarget::Charcoal,
        "reed" | "reeds" | "bundle of reeds" => ExamineTarget::Reeds,
        "net" | "fishing net" | "a fishing net" => ExamineTarget::Net,
        "mat" | "reed mat" | "a reed mat" => ExamineTarget::Mat,
        "canoe" | "dugout" | "dugout canoe" => ExamineTarget::Canoe,
        "bone" => ExamineTarget::Bone,
        "salt" | "salt crust" => ExamineTarget::Salt,
        "honey" | "honeycomb" | "bees" | "bees nest" | "nest" => ExamineTarget::Honey,
        "egg" | "eggs" | "bird nest" | "bird's nest" | "birds nest" => ExamineTarget::Eggs,
        "moss" => ExamineTarget::Moss,
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
        "slope" | "the slope" => ExamineTarget::Slope,
        other => ExamineTarget::Person(other.to_string()),
    }
}
