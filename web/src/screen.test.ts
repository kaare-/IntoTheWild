import assert from "node:assert/strict";
import { isLookCommand, meLine, signMap, sparsePlace, splitLook, toInnerVoice } from "./screen.ts";

const raw = [
  "  0   1   2   3   4   5   6   ",
  "0 ??  ??  ??  ??  ??  ??  ??  ",
  "1 ??  ??  =   .m  .y  .   Rw+ ",
  "2 ??  ??  .c  .L  .   .w  .   ",
  "3 ??  ??  .d  .   @.  .   .   ",
  "4 ??  ??  .r  .   .   .x  .   ",
].join("\n");

const signed = signMap(raw).split("\n");
assert.equal(signed.length, 5);
assert.equal(signed[0].trim(), "");
assert.ok(signed[1].includes("="));
assert.ok(signed[1].includes("m"));
assert.ok(signed[1].includes("y"));
assert.ok(signed[1].includes("Rw+"));
assert.equal(signed[1].includes("?"), false);
assert.equal(signed[1].includes("0"), false);
assert.ok(signed[3].includes("@"));
assert.equal(signed[3].includes("."), false);
assert.ok(signed[3].includes("d"));

const look = [
  "The valley is still dry. The sky is clear. Evening is coming.",
  "You are on the valley floor, in the open. A deer trail passes here.",
  "Your clothes are dry. You are not thirsty. You are not hungry. You are well. Your arms are empty.",
  "A deer stands two cells west, in the open.",
  "Around you:",
  "North: valley floor.",
  "South: valley floor, wild onions.",
].join("\n");

const split = splitLook(look);
assert.ok(split.body.startsWith("Your clothes are dry."));
assert.ok(split.body.includes("Your arms are empty."));
assert.equal(split.place.includes("Your clothes"), false);
assert.ok(split.place.includes("Evening is coming."));
assert.ok(split.place.includes("Around you:"));
assert.ok(split.place.includes("South: valley floor, wild onions."));

assert.equal(splitLook("The rain wets your clothes. You are thirsty.").place, "");
assert.ok(splitLook("The brook wets your clothes. You are warm.").body.startsWith("The brook"));

assert.equal(isLookCommand("look"), true);
assert.equal(isLookCommand("L"), true);
assert.equal(isLookCommand("examine deer"), true);
assert.equal(isLookCommand("gather berries"), false);

assert.equal(toInnerVoice("You walk east."), "I walk east.");
assert.equal(toInnerVoice("You wade west."), "I wade west.");
assert.equal(toInnerVoice("Your arms are empty."), "My arms are empty.");
assert.equal(toInnerVoice("It warms you."), "It warms me.");
assert.equal(
  toInnerVoice("The deer bolts. Your flint spear does not land."),
  "The deer bolts. My flint spear does not land.",
);
assert.equal(toInnerVoice("Ash walks west."), "Ash walks west.");
assert.equal(toInnerVoice("you cannot make out the ground"), "I cannot make out the ground");
assert.equal(toInnerVoice("A deer steps east."), "A deer steps east.");

const crowded = [
  "The valley is still dry. The sky is clear. Evening is coming.",
  "You hear frogs croak to the north and west.",
  "Wood left in the open is rotting two cells north and one cell east.",
  "You are on the valley floor, in the open. A deer trail passes here.",
  "Worms are in the dry ground here. Carve an antler into an antler pick to dig them.",
  "A deer stands one cell north and one cell west, in the open.",
  "Around you:",
  "North: valley floor, the heavy log.",
  "East: valley floor.",
  "South: you cannot make out the ground.",
  "West: hoofprints mark the outside floor. The last branch is gone from the next square south.",
].join("\n");
const sparse = sparsePlace(crowded);
assert.ok(sparse.startsWith("The valley is still dry. The sky is clear. Evening is coming."));
assert.ok(sparse.includes("I am on the valley floor, in the open."));
assert.ok(sparse.includes("I hear frogs croak to the north and west."));
assert.equal(sparse.includes("Worms"), false);
assert.equal(sparse.includes("Carve"), false);
assert.equal(sparse.includes("rotting"), false);
assert.equal(sparse.includes("Around you"), false);
assert.ok(sparse.includes("North\nvalley floor, the heavy log\n\nEast\nvalley floor"));
assert.ok(sparse.includes("South\nI cannot make out the ground"));
assert.ok(sparse.includes("West\nhoofprints mark the outside floor"));
assert.equal(sparse.includes("next square"), false);

const calm = {
  wetness: "dry",
  thirst: "not thirsty",
  hunger: "not hungry",
  illness: "well",
  energy: "rested",
  cold: "warm",
  wind: "in the wind",
  sunburn: "unburned",
  mud: "bare skin",
};
assert.equal(meLine(calm), "I feel all right.");
assert.equal(
  meLine({ ...calm, cold: "chilly", thirst: "thirsty" }),
  "I'm all right, although I am a little cold and thirsty.",
);
assert.equal(
  meLine({ ...calm, cold: "cold", thirst: "thirsty", hunger: "hungry", illness: "ill" }),
  "I feel ill and I'm cold, thirsty, hungry and my belly hurts.",
);
assert.equal(meLine({ ...calm, cold: "cold", thirst: "thirsty" }), "I'm cold and thirsty.");
assert.equal(
  meLine({ ...calm, wetness: "soaked", cold: "cold", thirst: "thirsty" }),
  "I'm cold, thirsty and soaked.",
);
assert.equal(meLine({ ...calm, wind: "out of the wind" }), "I'm out of the wind.");
assert.equal(
  meLine({ ...calm, cold: "chilly", wind: "out of the wind" }),
  "I'm all right, although I am a little cold, out of the wind.",
);
assert.equal(meLine({ ...calm, gripping: true }), "I'm gripping the heavy log.");
assert.equal(
  meLine({ ...calm, feeling: "I'm all right, although I am thirsty." }),
  "I'm all right, although I am thirsty.",
);

console.log("screen ok");
