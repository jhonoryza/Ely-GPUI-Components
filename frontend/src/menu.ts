import { chapters, esc, href, type Chapter } from "./data";
import { icon } from "./icons";

const OPEN_AFTER = 100;
const CLOSE_AFTER = 300;

type Point = { x: number; y: number };

/** Whether `p` lies in the triangle a, b, c. */
export function inside(p: Point, a: Point, b: Point, c: Point): boolean {
  const side = (u: Point, v: Point, w: Point) => (u.x - w.x) * (v.y - w.y) - (v.x - w.x) * (u.y - w.y);
  const [d1, d2, d3] = [side(p, a, b), side(p, b, c), side(p, c, a)];
  return !((d1 < 0 || d2 < 0 || d3 < 0) && (d1 > 0 || d2 > 0 || d3 > 0));
}

/** The header's menu: chapters, each with its stories. */
export function menu(trigger: HTMLButtonElement, go: (path: string) => void): void {
  const root = document.createElement("div");
  const sub = document.createElement("div");
  root.className = sub.className = "menu";
  root.setAttribute("role", "menu");
  sub.setAttribute("role", "menu");
  root.innerHTML = chapters
    .map((c, i) => `<button role="menuitem" aria-haspopup="menu" aria-expanded="false" data-i="${i}"><span>${esc(c.title)}</span>${icon("ChevronRight")}</button>`)
    .join("");
  let timer = 0;
  let shown: HTMLElement | null = null;
  let last: Point = { x: 0, y: 0 };

  const later = (ms: number, act: () => void) => {
    clearTimeout(timer);
    timer = window.setTimeout(act, ms);
  };

  const place = (el: HTMLElement, x: number, y: number) => {
    document.body.append(el);
    const r = el.getBoundingClientRect();
    el.style.left = `${Math.min(x, innerWidth - r.width - 8)}px`;
    el.style.top = `${Math.max(8, Math.min(y, innerHeight - r.height - 8))}px`;
  };

  const showSub = (row: HTMLElement, focus: boolean) => {
    const chapter: Chapter = chapters[Number(row.dataset.i)];
    shown?.setAttribute("aria-expanded", "false");
    shown = row;
    row.setAttribute("aria-expanded", "true");
    sub.innerHTML = chapter.stories.map((s) => `<a role="menuitem" href="${href(chapter, s)}">${esc(s.title)}</a>`).join("");
    const r = row.getBoundingClientRect();
    place(sub, r.right + 4, r.top - 4);
    if (focus) sub.querySelector<HTMLElement>("a")?.focus();
  };

  const close = (refocus: boolean) => {
    clearTimeout(timer);
    root.remove();
    sub.remove();
    shown = null;
    trigger.setAttribute("aria-expanded", "false");
    if (refocus) trigger.focus();
  };

  const open = (focusFirst: boolean) => {
    const r = trigger.getBoundingClientRect();
    trigger.setAttribute("aria-expanded", "true");
    place(root, r.left, r.bottom + 4);
    if (focusFirst) root.querySelector<HTMLElement>("button")?.focus();
  };

  // A pointer heading for the open submenu keeps it.
  const aiming = (p: Point) => {
    if (!sub.isConnected) return false;
    const r = sub.getBoundingClientRect();
    return inside(p, last, { x: r.left, y: r.top - 24 }, { x: r.left, y: r.bottom + 24 });
  };

  root.addEventListener("pointermove", (event) => {
    const p = { x: event.clientX, y: event.clientY };
    const row = (event.target as HTMLElement).closest<HTMLElement>("[data-i]");
    if (row && row !== shown) later(aiming(p) ? CLOSE_AFTER : OPEN_AFTER, () => showSub(row, false));
    last = p;
  });
  for (const el of [root, sub, trigger]) {
    el.addEventListener("pointerenter", () => clearTimeout(timer));
    el.addEventListener("pointerleave", () => later(CLOSE_AFTER, () => close(false)));
  }
  trigger.addEventListener("click", () => (root.isConnected ? close(false) : open(false)));
  trigger.addEventListener("keydown", (event) => {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      open(true);
    }
  });
  const keys = (list: HTMLElement, event: KeyboardEvent) => {
    const items = [...list.querySelectorAll<HTMLElement>("[role=menuitem]")];
    const at = items.indexOf(document.activeElement as HTMLElement);
    const move = (to: number) => items[(to + items.length) % items.length]?.focus();
    if (event.key === "ArrowDown") move(at + 1);
    else if (event.key === "ArrowUp") move(at - 1);
    else if (event.key === "Home") move(0);
    else if (event.key === "End") move(items.length - 1);
    else if (event.key === "ArrowRight" && list === root && at >= 0) showSub(items[at], true);
    else if (event.key === "ArrowLeft" && list === sub) (sub.remove(), shown?.focus());
    else if (event.key === "Escape" || event.key === "Tab") close(event.key === "Escape");
    else return;
    if (event.key !== "Tab") event.preventDefault();
  };
  root.addEventListener("keydown", (event) => keys(root, event));
  sub.addEventListener("keydown", (event) => keys(sub, event));
  root.addEventListener("click", (event) => {
    const row = (event.target as HTMLElement).closest<HTMLElement>("[data-i]");
    if (row) showSub(row, event.detail === 0);
  });
  sub.addEventListener("click", (event) => {
    const link = (event.target as HTMLElement).closest("a");
    if (!link) return;
    event.preventDefault();
    close(false);
    go(link.getAttribute("href")!);
  });
  addEventListener("pointerdown", (event) => {
    const t = event.target as Node;
    if (root.isConnected && !root.contains(t) && !sub.contains(t) && !trigger.contains(t)) close(false);
  });
}
