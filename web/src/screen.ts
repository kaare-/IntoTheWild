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
