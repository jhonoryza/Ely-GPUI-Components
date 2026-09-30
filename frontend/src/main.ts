import "@fontsource-variable/geist";
import "@fontsource-variable/geist-mono";
import "./style.css";
import { components } from "./components";
import { REPO } from "./data";
import { home } from "./home";
import { icon } from "./icons";
import { onSound, setSound, soundOn } from "./iso/marimba";
import { menu } from "./menu";
import { search } from "./search";
import { mode, setMode } from "./theme";

const bar = document.getElementById("bar")!;
const main = document.getElementById("main")!;
let leave = () => {};

function render(): void {
  leave();
  const path = location.pathname.split("/").filter(Boolean);
  const onDocs = path[0] === "components";
  const nav = bar.querySelector('[data-nav="components"]')!;
  if (onDocs) nav.setAttribute("aria-current", "page");
  else nav.removeAttribute("aria-current");
  if (onDocs) leave = components(main, path.slice(1));
  else if (path.length === 0) leave = home(main);
  else {
    console.error(`site: no page at ${location.pathname}`);
    leave = components(main, path);
  }
  scrollTo(0, 0);
}

export function go(path: string): void {
  if (path === location.pathname) return;
  history.pushState(null, "", path);
  render();
}

bar.innerHTML = `
  <a class="mark" href="/">Ely</a>
  <nav aria-label="Site">
    <a class="link" href="/components/" data-nav="components">Components</a>
    <button class="link" aria-haspopup="menu" aria-expanded="false" data-menu>Browse</button>
  </nav>
  <div class="end">
    <button class="find" data-find aria-label="Search components">${icon("Search")}<span>Search</span><kbd>⌘K</kbd></button>
    <button class="tool" data-sound></button>
    <button class="tool" data-mode></button>
    <a class="tool" href="${REPO}" rel="noopener" aria-label="Ely on GitHub">${icon("GitHub")}</a>
  </div>`;

document.getElementById("foot")!.innerHTML = `
  <span>MIT or Apache-2.0</span>
  <a href="${REPO}" rel="noopener">Source</a>
  <span>Built on <a href="https://github.com/zed-industries/zed/tree/main/crates/gpui" rel="noopener">GPUI</a></span>
  <span>Fonts: Geist and Geist Mono (OFL)</span>
  <span>Icons: <a href="https://reicon.dev" rel="noopener">Reicon</a> (MIT), based on Solar Icons by 480 Design (<a href="https://creativecommons.org/licenses/by/4.0/" rel="noopener">CC BY 4.0</a>)</span>`;

const modeButton = bar.querySelector<HTMLButtonElement>("[data-mode]")!;
const drawMode = () => {
  const dark = mode() === "dark";
  modeButton.innerHTML = icon(dark ? "Sun" : "Moon");
  modeButton.setAttribute("aria-label", dark ? "Light mode" : "Dark mode");
};
modeButton.addEventListener("click", () => {
  setMode(mode() === "dark" ? "light" : "dark");
  drawMode();
});
drawMode();

const soundButton = bar.querySelector<HTMLButtonElement>("[data-sound]")!;
const drawSound = (on: boolean) => {
  soundButton.innerHTML = icon(on ? "Sound" : "VolumeCross");
  soundButton.setAttribute("aria-label", on ? "Sound off" : "Sound on");
  soundButton.setAttribute("aria-pressed", String(on));
};
soundButton.addEventListener("click", () => setSound(!soundOn()));
onSound(drawSound);
drawSound(false);

bar.querySelector("[data-find]")!.addEventListener("click", search(go));
menu(bar.querySelector<HTMLButtonElement>("[data-menu]")!, go);

// Same-origin links route in place, except the gallery.
document.addEventListener("click", (event) => {
  const link = (event.target as HTMLElement).closest("a");
  if (!link || event.defaultPrevented || event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || link.target) return;
  const url = new URL(link.href, location.href);
  if (url.origin !== location.origin || url.pathname.startsWith("/gallery/") || url.hash) return;
  event.preventDefault();
  go(url.pathname);
});
addEventListener("popstate", render);
render();
