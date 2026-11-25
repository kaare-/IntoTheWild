import assert from "node:assert/strict";
import { isLookCommand, signMap, splitLook } from "./screen.ts";

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

console.log("screen ok");
