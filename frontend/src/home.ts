import Lenis from "lenis";
import "lenis/dist/lenis.css";
import { REPO, chapters, componentCount, esc, storyCount } from "./data";
import { icon } from "./icons";
import { app, bars, candles, chat, type Model } from "./iso/models";
import { stage } from "./iso/stage";
import { preview } from "./preview";
import { onMode } from "./theme";

interface Show {
  glyph: "circle" | "square" | "triangle";
  name: string;
  line: string;
  page: string;
  story: string;
  title: string;
  model: () => Model;
}

const SHOWS: Show[] = [
  {
    glyph: "circle",
    name: "Finance",
    line: "Candles, volume and a crosshair. Drag to pan, Cmd-scroll to zoom, switch the chart type.",
    page: "finance",
    story: "candlestickchart-ohlcchart-heikinashichart-linequotechart-volumechart-charttypeswitcher-timerangeselector-intervalselector",
    title: "CandlestickChart",
    model: candles,
  },
  {
    glyph: "square",
    name: "Charts",
    line: "A year in two series. Hover for values, drag across to zoom, press a name to hide it.",
    page: "charts",
    story: "linechart-chartaxis-chartgrid-chartlegend-charttooltip-chartcrosshair-chartannotation-chartzoom-chartexport",
    title: "LineChart",
    model: bars,
  },
  {
    glyph: "triangle",
    name: "AI chat",
    line: "Bubbles, avatars, math and code, the way a model answers.",
    page: "chat",
    story: "chatcontainer-messagelist-messagebubble-messageavatar-messageheader-messagefooter-dateseparator-scrolltobottombutton",
    title: "ChatContainer",
    model: chat,
  },
];

/** A miniature that grows to a marimba, then each model beside the real thing. */
export function home(main: HTMLElement): () => void {
  document.title = "Ely · GPUI components";
  main.innerHTML = `
    <section class="hero">
      <div class="words">
        <span class="glyphs" aria-hidden="true"><i class="circle"></i><i class="square"></i><i class="triangle"></i></span>
        <h1>A component library for GPUI.<strong>Every part runs on this page.</strong></h1>
        <p>${componentCount} components in ${chapters.length} chapters, written in Rust, compiled to WebAssembly. Point at the model. Turn the sound on.</p>
        <div class="cta">
          <a class="button primary" href="/components/">Browse components ${icon("ArrowRight")}</a>
          <a class="button" href="${REPO}" rel="noopener">${icon("GitHub")}GitHub</a>
        </div>
      </div>
      <div class="iso-host hero-iso" data-model="app"></div>
    </section>
    ${SHOWS.map(
      (s, i) => `<section class="show">
        <div class="pair">
          <header class="mark">
            <i class="${s.glyph}" aria-hidden="true"></i>
            <span class="n">${String(i + 1).padStart(2, "0")}</span>
            <h2>${esc(s.name)}</h2>
            <p>${esc(s.line)}</p>
          </header>
          <div class="iso-host" data-model="${s.page}"></div>
        </div>
        <div class="live" data-i="${i}"></div>
      </section>`,
    ).join("")}
    <section class="facts">
      <div><strong>${storyCount}</strong><span>stories, each its own page</span></div>
      <div><strong>${chapters.length}</strong><span>chapters, primitives to maps</span></div>
      <div><strong>2</strong><span>modes from one palette</span></div>
      <div><strong>0</strong><span>screenshots</span></div>
    </section>`;

  const still = matchMedia("(prefers-reduced-motion: reduce)").matches;
  const lenis = still ? null : new Lenis({ autoRaf: true, anchors: true });
  const iso = stage(still);
  iso.add(main.querySelector<HTMLElement>(".hero-iso")!, app());
  main.querySelectorAll<HTMLElement>(".pair .iso-host").forEach((host, i) => iso.add(host, SHOWS[i].model()));
  const stopMode = onMode(() => iso.paint());

  // A live example loads near the view and unloads far from it.
  const stops = new Map<HTMLElement, () => void>();
  const near = new IntersectionObserver(
    (entries) => {
      for (const e of entries) {
        const host = e.target as HTMLElement;
        const show = SHOWS[Number(host.dataset.i)];
        if (e.isIntersecting && !stops.has(host)) {
          stops.set(host, preview(host, show.page, show.story, show.title));
        } else if (!e.isIntersecting && stops.has(host)) {
          stops.get(host)!();
          stops.delete(host);
          host.innerHTML = "";
        }
      }
    },
    { rootMargin: "100% 0px" },
  );
  main.querySelectorAll<HTMLElement>(".live").forEach((host) => near.observe(host));

  return () => {
    lenis?.destroy();
    near.disconnect();
    stops.forEach((stop) => stop());
    stopMode();
    iso.stop();
  };
}
