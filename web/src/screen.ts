/** Fixed width of one signed cell. Empty ground is spaces, so columns stay aligned. */
export const SIGN_WIDTH = 3;

const BODY_STARTS = [
  "Your clothes ",
  "The rain wets your clothes",
  "The brook wets your clothes",
  "The rain does not wet your clothes",
  "A little rain dampens your clothes",
];

/** The numbered grid becomes a sign row. Unseen ground and an empty floor are blank. */
export function signMap(raw: string): string {
  const rows: string[] = [];
  for (const line of raw.split("\n")) {
    if (!/^\d/.test(line)) continue;
    const rest = line.replace(/^\d+\s*/, "");
    let signed = "";
    for (let i = 0; i < rest.length; i += 4) {
      const field = rest.slice(i, i + 4);
      if (!field.trim()) continue;
      signed += signCell(field);
    }
    rows.push(signed.replace(/\s+$/, ""));
  }
  return rows.join("\n");
}

function signCell(field: string): string {
  const raw = field.trimEnd();
  if (!raw || raw.startsWith("?")) return " ".repeat(SIGN_WIDTH);
  const left = raw[0] ?? " ";
  let mid = raw[1] ?? " ";
  const extra = raw[2] ?? " ";
  const person = left === "@" || left === "p";
  if (person && mid === ".") mid = " ";
  if (!person && left === "." && mid === " " && (extra === " " || extra === "")) {
    return " ".repeat(SIGN_WIDTH);
  }
  if (!person && left === ".") {
    return pad(mid + (extra === " " ? "" : extra));
  }
  return pad(left + (mid === " " ? "" : mid) + (extra === " " ? "" : extra));
}

function pad(value: string): string {
  return (value + " ".repeat(SIGN_WIDTH)).slice(0, SIGN_WIDTH);
}

/** Sky, place, and compass stay together. The self line is the body. */
export function splitLook(text: string): { place: string; body: string } {
  const place: string[] = [];
  const body: string[] = [];
  for (const line of text.split("\n")) {
    if (!line.trim()) continue;
    if (BODY_STARTS.some((start) => line.startsWith(start))) body.push(line);
    else place.push(line);
  }
  return { place: place.join("\n"), body: body.join("\n") };
}

/** A look or an examine is the place text, not a second copy on the action side. */
export function isLookCommand(line: string): boolean {
  const trimmed = line.trim().toLowerCase();
  return trimmed === "look" || trimmed === "l" || trimmed === "x" || trimmed.startsWith("examine");
}

/** Actor-facing You-lines, shown as inner voice. Witness lines stay as they are. */
export function toInnerVoice(text: string): string {
  return text
    .split(/(?<=\.)\s+/)
    .map(innerSentence)
    .join(" ");
}

function innerSentence(sentence: string): string {
  let s = sentence;
  if (s.startsWith("You are ")) s = "I am " + s.slice("You are ".length);
  else if (s.startsWith("You ")) s = "I " + s.slice(4);
  else if (s.startsWith("Your ")) s = "My " + s.slice(5);
  else if (s.startsWith("you are ")) s = "I am " + s.slice("you are ".length);
  else if (s.startsWith("you ")) s = "I " + s.slice(4);
  else if (s.startsWith("your ")) s = "my " + s.slice(5);
  s = s.replace(/\bYour\b/g, "My").replace(/\byour\b/g, "my");
  s = s.replace(/\byou\b/g, "me");
  return s;
}

const COMPASS = ["North", "East", "South", "West"];

/**
 * Sky, where I stand, one thing I hear, and the four directions with air between them.
 * The long stock sentences stay off this pane.
 */
export function sparsePlace(text: string): string {
  const lines = text
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
  const blocks: string[] = [];
  const sky = lines[0];
  if (sky && !sky.startsWith("You are on") && !COMPASS.some((dir) => sky.startsWith(`${dir}:`))) {
    blocks.push(toInnerVoice(sky));
  }
  const here = lines.find((line) => line.startsWith("You are on ") || line.startsWith("You are in "));
  if (here) {
    const first = here.split(/(?<=\.)\s+/)[0] ?? here;
    blocks.push(toInnerVoice(first));
  }
  const heard = lines.find(
    (line) => /^(Frogs croak|You hear |An owl|Birds)/.test(line) && line.length < 140,
  );
  if (heard) blocks.push(toInnerVoice(heard));

  const compass: string[] = [];
  for (const dir of COMPASS) {
    const line = lines.find((item) => item.startsWith(`${dir}:`));
    if (!line) continue;
    const rest = line.slice(dir.length + 1).trim();
    const first = (rest.split(/(?<=\.)\s+/)[0] ?? rest).replace(/\.$/, "");
    compass.push(dir, toInnerVoice(first), "");
  }
  if (compass.length > 0) {
    blocks.push(compass.join("\n").replace(/\n+$/, ""));
  }
  return blocks.join("\n\n");
}

export type BodyState = {
  wetness: string;
  thirst: string;
  hunger: string;
  illness: string;
  energy: string;
  cold: string;
  wind: string;
  sunburn: string;
  mud: string;
  ochre?: string;
  gripping?: boolean;
};

/** One sentence about how I am. Calm defaults stay out of it. */
export function meLine(self: BodyState): string {
  const bits: string[] = [];
  if (self.cold === "chilly" || self.cold === "cold") bits.push(self.cold);
  if (self.thirst === "thirsty" || self.thirst === "parched") bits.push(self.thirst);
  if (self.hunger === "hungry" || self.hunger === "starving") bits.push(self.hunger);
  if (self.wetness === "drying") bits.push("drying off");
  else if (self.wetness && self.wetness !== "dry") bits.push(self.wetness);
  if (self.energy === "tiring" || self.energy === "spent") bits.push(self.energy);
  if (self.illness && self.illness !== "well") bits.push(self.illness);
  if (self.sunburn === "pink") bits.push("pink from the sun");
  else if (self.sunburn === "burned") bits.push("burned");
  if (self.mud && self.mud !== "bare skin") bits.push(self.mud);
  if (self.ochre && self.ochre !== "bare") bits.push(self.ochre);

  let line: string;
  if (bits.length === 0) {
    line = self.wind === "out of the wind" ? "I am out of the wind." : "I feel all right.";
  } else {
    line = `I am ${joinAnd(bits)}.`;
    if (self.wind === "out of the wind") line = line.replace(/\.$/, ", out of the wind.");
    else if (self.wind === "in a little wind") line = line.replace(/\.$/, ", in a little wind.");
  }
  if (self.gripping) {
    if (bits.length === 0 && self.wind !== "out of the wind") return "I am gripping the heavy log.";
    return line.replace(/\.$/, ", and I grip the log.");
  }
  return line;
}

function joinAnd(parts: string[]): string {
  if (parts.length <= 1) return parts[0] ?? "";
  if (parts.length === 2) return `${parts[0]} and ${parts[1]}`;
  return `${parts.slice(0, -1).join(", ")}, and ${parts[parts.length - 1]}`;
}
