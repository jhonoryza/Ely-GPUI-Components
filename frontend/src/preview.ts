import { esc } from "./data";
import { icon } from "./icons";
import { mode, onMode } from "./theme";

const SPIN_AFTER = 400;
const GIVE_UP = 20000;

/** One story live, themed with the site; returns its stop. */
export function preview(host: HTMLElement, page: string, story: string, title: string): () => void {
  const src = (theme: string) => `/gallery/?page=${page}&story=${story}&theme=${theme}`;
  host.innerHTML = `
    <a class="stage-skip" href="#after-stage">Skip the live example</a>
    <div class="stage">
      <iframe title="${esc(title)}, live" src="${src(mode())}"></iframe>
      <div class="state" role="status">Starting…</div>
      <div class="actions">
        <button class="tool" data-act="reload" aria-label="Reload the example">${icon("Refresh")}</button>
        <a class="tool" href="${src(mode())}" target="_blank" rel="noopener" aria-label="Open the example alone">${icon("ArrowRightUp")}</a>
        <button class="tool" data-act="full" aria-label="Full screen">${icon("Maximize")}</button>
      </div>
    </div>
    <span id="after-stage" tabindex="-1"></span>`;
  const stage = host.querySelector<HTMLElement>(".stage")!;
  const frame = stage.querySelector("iframe")!;
  const state = stage.querySelector<HTMLElement>(".state")!;
  let timers: number[] = [];

  const starting = () => {
    timers.forEach(clearTimeout);
    state.hidden = false;
    state.textContent = "Starting…";
    timers = [
      window.setTimeout(() => (state.innerHTML = `<span class="spin"></span>Starting…`), SPIN_AFTER),
      window.setTimeout(() => {
        state.innerHTML = `Couldn't start the example. <button class="link" data-act="reload">Reload</button>`;
        console.error(`preview: ${page}/${story} sent no ready in ${GIVE_UP} ms`);
      }, GIVE_UP),
    ];
  };

  const ready = (event: MessageEvent) => {
    if (event.source !== frame.contentWindow || event.data?.ely !== "ready") return;
    timers.forEach(clearTimeout);
    state.hidden = true;
    // The host's mode may have turned while the wasm loaded.
    frame.contentWindow?.postMessage({ ely: "theme", theme: mode() }, location.origin);
  };

  const stopMode = onMode((next) => {
    frame.contentWindow?.postMessage({ ely: "theme", theme: next }, location.origin);
    stage.querySelector<HTMLAnchorElement>("a.tool")!.href = src(next);
  });

  const full = (on: boolean) => {
    stage.classList.toggle("full", on);
    document.body.style.overflow = on ? "hidden" : "";
    const button = stage.querySelector<HTMLElement>('[data-act="full"]')!;
    button.innerHTML = icon(on ? "Minimize" : "Maximize");
    button.setAttribute("aria-label", on ? "Leave full screen" : "Full screen");
  };

  const escape = (event: KeyboardEvent) => {
    if (event.key === "Escape" && stage.classList.contains("full")) full(false);
  };

  stage.addEventListener("click", (event) => {
    const act = (event.target as HTMLElement).closest<HTMLElement>("[data-act]")?.dataset.act;
    if (act === "reload") {
      starting();
      frame.src = src(mode());
    } else if (act === "full") {
      full(!stage.classList.contains("full"));
    }
  });
  addEventListener("message", ready);
  addEventListener("keydown", escape);
  starting();

  return () => {
    timers.forEach(clearTimeout);
    removeEventListener("message", ready);
    removeEventListener("keydown", escape);
    stopMode();
    document.body.style.overflow = "";
  };
}
