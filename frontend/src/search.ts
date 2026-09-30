import { matches } from "./components";
import { all, esc, href } from "./data";

const LIMIT = 50;

/** A palette over every story: ⌘K, Ctrl-K or /. */
export function search(go: (path: string) => void): () => void {
  let back: HTMLElement | null = null;
  const scrim = document.createElement("div");
  scrim.className = "scrim";
  scrim.innerHTML = `<div class="palette" role="dialog" aria-modal="true" aria-label="Search components">
    <input type="search" placeholder="Search stories and components" aria-label="Search" role="combobox" aria-expanded="true" aria-controls="hits" />
    <ol id="hits" role="listbox"></ol></div>`;
  const input = scrim.querySelector("input")!;
  const list = scrim.querySelector("ol")!;
  let hits: typeof all = [];
  let cursor = 0;

  const draw = () => {
    const query = input.value.trim().toLowerCase();
    hits = (query ? all.filter((e) => matches(e.story, query) || e.chapter.title.toLowerCase().includes(query)) : all).slice(0, LIMIT);
    cursor = Math.min(cursor, Math.max(hits.length - 1, 0));
    list.innerHTML = hits.length
      ? hits
          .map(
            (e, i) =>
              `<li role="option" id="hit-${i}" aria-selected="${i === cursor}"><a href="${href(e.chapter, e.story)}" tabindex="-1"><span>${esc(e.story.title)}</span><span>${esc(e.chapter.title)}</span></a></li>`,
          )
          .join("")
      : `<li class="none">Nothing matches “${esc(input.value)}”.</li>`;
    input.setAttribute("aria-activedescendant", hits.length ? `hit-${cursor}` : "");
    list.querySelector('[aria-selected="true"]')?.scrollIntoView({ block: "nearest" });
  };

  const close = () => {
    scrim.remove();
    back?.focus();
  };

  const open = () => {
    if (scrim.isConnected) return;
    back = document.activeElement as HTMLElement | null;
    document.body.append(scrim);
    input.value = "";
    cursor = 0;
    draw();
    input.focus();
  };

  input.addEventListener("input", () => ((cursor = 0), draw()));
  input.addEventListener("keydown", (event) => {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      cursor = (cursor + (event.key === "ArrowDown" ? 1 : hits.length - 1)) % Math.max(hits.length, 1);
      draw();
    } else if (event.key === "Enter" && hits[cursor]) {
      close();
      go(href(hits[cursor].chapter, hits[cursor].story));
    } else if (event.key === "Escape") {
      close();
    } else if (event.key === "Tab") {
      // The field is the dialog's one stop.
      event.preventDefault();
    }
  });
  scrim.addEventListener("click", (event) => {
    if (event.target === scrim) close();
    const link = (event.target as HTMLElement).closest("a");
    if (link) {
      event.preventDefault();
      close();
      go(link.getAttribute("href")!);
    }
  });
  addEventListener("keydown", (event) => {
    const typing = (event.target as HTMLElement).closest("input, textarea, [contenteditable]");
    if ((event.key === "k" && (event.metaKey || event.ctrlKey)) || (event.key === "/" && !typing)) {
      event.preventDefault();
      open();
    }
  });
  return open;
}
