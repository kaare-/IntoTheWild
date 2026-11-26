import { isLookCommand, meLine, signMap, sparsePlace, toInnerVoice } from "./screen.ts";

type Observation = {
  sim_time: number;
  raining: boolean;
  text: string;
  map: string;
  legend: string;
  self_info: {
    name: string;
    x: number;
    y: number;
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
    under_roof: boolean;
    gripping: boolean;
    feeling?: string;
    busy: boolean;
    carrying: { material: string; condition: string; count: number }[];
  };
  events: { id: number; time: number; text: string }[];
  allowed_actions: string[];
  work: { kind: string; detail: string; finishes_at: number } | null;
};

const app = document.querySelector<HTMLElement>("#app")!;
const TAB = sessionStorage;
const LAST = localStorage;
const TOKEN_KEY = "villmarks-token";
const NAME_KEY = "villmarks-name";

function token(): string | null {
  return TAB.getItem(TOKEN_KEY) || LAST.getItem(TOKEN_KEY);
}

function rememberToken(value: string, name?: string) {
  TAB.setItem(TOKEN_KEY, value);
  LAST.setItem(TOKEN_KEY, value);
  if (name) {
    TAB.setItem(NAME_KEY, name);
    LAST.setItem(NAME_KEY, name);
  }
}

function forgetToken() {
  TAB.removeItem(TOKEN_KEY);
  TAB.removeItem(NAME_KEY);
  LAST.removeItem(TOKEN_KEY);
  LAST.removeItem(NAME_KEY);
}

async function api<T>(path: string, init: RequestInit = {}): Promise<T> {
  const headers = new Headers(init.headers);
  if (token()) headers.set("Authorization", `Bearer ${token()}`);
  if (init.body && !headers.has("Content-Type")) {
    headers.set("Content-Type", "application/json");
  }
  const response = await fetch(path, { ...init, headers });
  const data = await response.json();
  if (!response.ok) {
    throw new Error(data.message || `request failed (${response.status})`);
  }
  return data as T;
}

function renderGate(error = "") {
  const remembered = LAST.getItem(TOKEN_KEY) || "";
  app.innerHTML = `
    <section class="gate">
      <p>villmarks</p>
      <form id="join">
        <label for="name">name</label>
        <input id="name" maxlength="24" autocomplete="off" spellcheck="false" />
      </form>
      <form id="resume-form">
        <label for="resume">token</label>
        <input id="resume" autocomplete="off" spellcheck="false" value="${escapeHtml(remembered)}" />
      </form>
      <p class="miss" id="gate-error">${escapeHtml(error)}</p>
    </section>`;
  const name = app.querySelector<HTMLInputElement>("#name")!;
  name.focus();
  app.querySelector("#join")!.addEventListener("submit", (event) => {
    event.preventDefault();
    void enter(name.value);
  });
  const resume = app.querySelector<HTMLInputElement>("#resume")!;
  app.querySelector("#resume-form")!.addEventListener("submit", (event) => {
    event.preventDefault();
    void resumeWith(resume.value);
  });
}

async function enter(name: string) {
  const trimmed = name.trim();
  if (!trimmed) return;
  try {
    const joined = await api<{ token: string; name: string }>("/v1/join", {
      method: "POST",
      body: JSON.stringify({ name: trimmed }),
    });
    rememberToken(joined.token, joined.name);
    await showPlay();
  } catch (err) {
    renderGate(err instanceof Error ? err.message : "Could not enter.");
  }
}

async function resumeWith(value: string) {
  const trimmed = value.trim();
  if (!trimmed) return;
  TAB.setItem(TOKEN_KEY, trimmed);
  try {
    const observation = await api<Observation>("/v1/observe");
    rememberToken(trimmed, observation.self_info.name);
    await showPlay();
  } catch (err) {
    forgetToken();
    renderGate(err instanceof Error ? err.message : "That token is not in this valley.");
  }
}

let socket: WebSocket | null = null;
let seenEvents = new Set<number>();
let seenReplies = new Set<string>();
let localNotes: string[] = [];
let commandHistory: string[] = [];
let historyAt = 0;

async function showPlay() {
  let observation: Observation;
  try {
    observation = await api<Observation>("/v1/observe");
  } catch {
    forgetToken();
    renderGate("The valley does not know that token.");
    return;
  }
  rememberToken(token() ?? "", observation.self_info.name);
  app.innerHTML = `
    <section class="play">
      <div class="left">
        <div class="transcript" id="log"></div>
        <p class="now" id="work"></p>
        <form class="command" id="form">
          <span class="prompt">&gt;</span>
          <input id="line" autocomplete="off" spellcheck="false" />
        </form>
        <button id="copy-token" class="token" type="button">token</button>
      </div>
      <div class="right">
        <pre class="map" id="map"></pre>
        <p class="place" id="place"></p>
      </div>
      <p class="body" id="body"></p>
    </section>`;
  paint(observation);
  app.querySelector("#copy-token")?.addEventListener("click", async () => {
    const value = token() ?? "";
    try {
      await navigator.clipboard.writeText(value);
      note("token copied");
    } catch {
      note(value);
    }
  });
  const form = app.querySelector<HTMLFormElement>("#form")!;
  const line = app.querySelector<HTMLInputElement>("#line")!;
  form.addEventListener("submit", (event) => {
    event.preventDefault();
    const typed = line.value;
    void send(typed);
    if (typed.trim()) {
      commandHistory.push(typed);
      historyAt = commandHistory.length;
    }
    line.value = "";
  });
  line.addEventListener("keydown", (event) => {
    if (event.key === "ArrowUp" && commandHistory.length) {
      historyAt = Math.max(0, historyAt - 1);
      line.value = commandHistory[historyAt] ?? "";
      event.preventDefault();
    } else if (event.key === "ArrowDown") {
      historyAt = Math.min(commandHistory.length, historyAt + 1);
      line.value = commandHistory[historyAt] ?? "";
      event.preventDefault();
    }
  });
  line.focus();
  connectSocket();
}

function paint(observation: Observation) {
  const map = app.querySelector("#map");
  if (!map) return;
  map.textContent = signMap(observation.map);
  app.querySelector("#place")!.textContent = sparsePlace(observation.text);
  app.querySelector("#body")!.textContent = meLine(observation.self_info);
  const work = app.querySelector("#work")!;
  work.textContent = observation.work ? toInnerVoice(observation.work.detail) : "";
  for (const event of observation.events) {
    if (seenEvents.has(event.id)) continue;
    seenEvents.add(event.id);
    localNotes.push(toInnerVoice(event.text));
  }
  renderNotes();
}

function note(text: string) {
  const trimmed = text.trim();
  if (!trimmed) return;
  localNotes.push(toInnerVoice(trimmed));
  renderNotes();
}

function renderNotes() {
  const log = app.querySelector("#log");
  if (!log) return;
  const lines = localNotes.slice(-80);
  log.replaceChildren();
  lines.forEach((text, index) => {
    const row = document.createElement("p");
    const age = lines.length - 1 - index;
    row.style.opacity = String(Math.max(0.14, 1 - age * 0.07));
    row.textContent = text;
    log.appendChild(row);
  });
  log.scrollTop = log.scrollHeight;
}

async function send(line: string) {
  const trimmed = line.trim();
  if (!trimmed) return;
  note(`> ${trimmed}`);
  try {
    const result = await api<{
      ok: boolean;
      message: string;
      immediate: boolean;
      duplicate: boolean;
      late: boolean;
      window: number;
    }>("/v1/intent", {
      method: "POST",
      body: JSON.stringify({
        command_id: crypto.randomUUID(),
        line: trimmed,
        window: null,
      }),
    });
    if (result.immediate) {
      if (!isLookCommand(trimmed)) note(result.message);
      const observation = await api<Observation>("/v1/observe");
      paint(observation);
    } else {
      note(
        result.late
          ? `Late. Waiting in the window at tick ${result.window}.`
          : `Queued for tick ${result.window}.`,
      );
    }
  } catch (err) {
    note(err instanceof Error ? err.message : "The attempt was refused.");
  }
}

function connectSocket() {
  socket?.close();
  const protocol = location.protocol === "https:" ? "wss" : "ws";
  socket = new WebSocket(`${protocol}://${location.host}/v1/ws?token=${encodeURIComponent(token() ?? "")}`);
  socket.addEventListener("message", (event) => {
    try {
      const payload = JSON.parse(String(event.data));
      if (payload.observation) paint(payload.observation as Observation);
      showReplies(payload.replies);
    } catch {
      /* ignore non-json */
    }
  });
}

function showReplies(replies: unknown) {
  if (!Array.isArray(replies)) return;
  for (const reply of replies) {
    if (!reply || typeof reply !== "object") continue;
    const row = reply as { command_id?: string; message?: string };
    if (!row.command_id || !row.message || seenReplies.has(row.command_id)) continue;
    seenReplies.add(row.command_id);
    note(row.message);
  }
}

function escapeHtml(value: string) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

if (token()) {
  void showPlay();
} else {
  renderGate();
}
