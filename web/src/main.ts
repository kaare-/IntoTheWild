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
    under_roof: boolean;
    gripping: boolean;
    busy: boolean;
    carrying: { material: string; condition: string; count: number }[];
  };
  events: { id: number; time: number; text: string }[];
  allowed_actions: string[];
  work: { kind: string; detail: string; finishes_at: number } | null;
};

type Clock = {
  sim_time: number;
  raining: boolean;
  manual: boolean;
  window_secs: number;
  closes_in_ms: number | null;
  queued: number;
};

const app = document.querySelector<HTMLElement>("#app")!;
const store = sessionStorage;

function token(): string | null {
  return store.getItem("villmarks-token");
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
  app.innerHTML = `
    <section class="gate">
      <h1>Villmarks</h1>
      <p class="lede">A brook is already running. Rain is on its way. Take a name the valley has not heard, or resume a token the server still remembers.</p>
      <div class="row">
        <input id="name" placeholder="Name" maxlength="24" autocomplete="off" />
        <button id="enter" type="button">Enter</button>
      </div>
      <div class="row">
        <input id="resume" placeholder="Resume token" autocomplete="off" />
        <button id="resume-btn" class="ghost" type="button">Resume</button>
      </div>
      <p class="status" id="gate-error">${escapeHtml(error)}</p>
    </section>`;
  const name = app.querySelector<HTMLInputElement>("#name")!;
  name.focus();
  app.querySelector("#enter")!.addEventListener("click", () => enter(name.value));
  name.addEventListener("keydown", (event) => {
    if (event.key === "Enter") enter(name.value);
  });
  const resume = app.querySelector<HTMLInputElement>("#resume")!;
  app.querySelector("#resume-btn")!.addEventListener("click", () => resumeWith(resume.value));
}

async function enter(name: string) {
  try {
    const joined = await api<{ token: string; name: string }>("/v1/join", {
      method: "POST",
      body: JSON.stringify({ name }),
    });
    store.setItem("villmarks-token", joined.token);
    store.setItem("villmarks-name", joined.name);
    await showPlay();
  } catch (err) {
    renderGate(err instanceof Error ? err.message : "Could not enter.");
  }
}

async function resumeWith(value: string) {
  const trimmed = value.trim();
  if (!trimmed) return;
  store.setItem("villmarks-token", trimmed);
  try {
    await api<Observation>("/v1/observe");
    await showPlay();
  } catch (err) {
    store.removeItem("villmarks-token");
    renderGate(err instanceof Error ? err.message : "That token is not in this valley.");
  }
}

let socket: WebSocket | null = null;
let seenEvents = new Set<number>();
let localNotes: string[] = [];
let commandHistory: string[] = [];
let historyAt = 0;

async function showPlay() {
  let observation: Observation;
  try {
    observation = await api<Observation>("/v1/observe");
  } catch {
    store.removeItem("villmarks-token");
    renderGate("The valley does not know that token.");
    return;
  }
  app.innerHTML = `
    <section class="play">
      <div>
        <article class="card map-card">
          <pre id="map"></pre>
          <div class="legend" id="legend"></div>
        </article>
        <p class="token">This tab's token<br><code id="token"></code></p>
      </div>
      <div class="card">
        <div class="top">
          <div>
            <div class="who" id="who"></div>
            <div class="status" id="where"></div>
          </div>
          <div class="clock" id="clock"></div>
        </div>
        <p class="work" id="work"></p>
        <p class="prose" id="prose"></p>
        <ol class="log" id="log"></ol>
        <form class="command" id="form">
          <input id="line" placeholder="look, gather branch, drag e…" autocomplete="off" />
          <button type="submit">Try</button>
        </form>
        <div class="status" id="ack"></div>
        <div class="chips" id="chips"></div>
      </div>
    </section>`;
  paint(observation);
  const form = app.querySelector<HTMLFormElement>("#form")!;
  const line = app.querySelector<HTMLInputElement>("#line")!;
  form.addEventListener("submit", (event) => {
    event.preventDefault();
    void send(line.value);
    commandHistory.push(line.value);
    historyAt = commandHistory.length;
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
  void pollClock();
}

function paint(observation: Observation) {
  const map = app.querySelector("#map");
  if (!map) return;
  map.textContent = observation.map;
  app.querySelector("#legend")!.textContent = observation.legend;
  app.querySelector("#token")!.textContent = token() ?? "";
  const self = observation.self_info;
  app.querySelector("#who")!.textContent = self.name;
  const carry = self.carrying
    .map((item) => `${item.count} ${item.material} (${item.condition})`)
    .join(", ");
  app.querySelector("#where")!.textContent = [
    `(${self.x},${self.y})`,
    self.wetness,
    self.thirst,
    self.hunger,
    self.illness,
    self.under_roof ? "under a roof" : "open sky",
    self.gripping ? "gripping the log" : "",
    carry ? `carrying ${carry}` : "",
  ]
    .filter(Boolean)
    .join(" · ");
  app.querySelector("#prose")!.textContent = observation.text;
  const work = app.querySelector("#work")!;
  work.textContent = observation.work
    ? observation.work.detail
    : observation.raining
      ? "Rain is on the valley."
      : "";
  for (const event of observation.events) {
    if (!seenEvents.has(event.id)) {
      seenEvents.add(event.id);
      localNotes.push(`t${event.time}  ${event.text}`);
    }
  }
  const log = app.querySelector("#log")!;
  log.innerHTML = localNotes
    .slice(-30)
    .map((line) => `<li>${escapeHtml(line)}</li>`)
    .join("");
  log.scrollTop = log.scrollHeight;
  const chips = app.querySelector("#chips")!;
  chips.innerHTML = "";
  for (const action of observation.allowed_actions) {
    if (action === "help" || action === "say") continue;
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = action;
    button.addEventListener("click", () => {
      if (action === "say") return;
      void send(action);
    });
    chips.appendChild(button);
  }
}

async function send(line: string) {
  const trimmed = line.trim();
  if (!trimmed) return;
  const ack = app.querySelector("#ack");
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
    if (ack) {
      ack.textContent = result.immediate
        ? ""
        : result.late
          ? `Late. Waiting in the window at tick ${result.window}.`
          : `Queued for tick ${result.window}.`;
    }
    if (result.immediate) {
      const brief = result.message.split("\n").find((line) => line.trim()) ?? result.message;
      localNotes.push(brief);
      const log = app.querySelector("#log");
      if (log) {
        const item = document.createElement("li");
        item.textContent = brief;
        log.appendChild(item);
        log.scrollTop = log.scrollHeight;
      }
      if (trimmed === "look" || trimmed.startsWith("examine") || trimmed === "x" || trimmed === "l") {
        const observation = await api<Observation>("/v1/observe");
        paint(observation);
      }
    }
  } catch (err) {
    if (ack) ack.textContent = err instanceof Error ? err.message : "The attempt was refused.";
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
    } catch {
      /* ignore non-json */
    }
  });
}

async function pollClock() {
  while (token() && app.querySelector("#clock")) {
    try {
      const clock = await api<Clock>("/v1/clock");
      const node = app.querySelector("#clock");
      if (!node) return;
      if (clock.manual) {
        node.textContent = `tick ${clock.sim_time} · manual`;
      } else {
        const seconds = Math.ceil((clock.closes_in_ms ?? 0) / 1000);
        node.textContent = `tick ${clock.sim_time} · window ${seconds}s${clock.raining ? " · rain" : ""}`;
      }
    } catch {
      /* the next poll will retry */
    }
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }
}

function escapeHtml(value: string) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

if (token()) {
  void showPlay();
} else {
  renderGate();
}
