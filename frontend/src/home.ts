import Lenis from "lenis";
import "lenis/dist/lenis.css";
import { REPO, chapters, componentCount, storyCount } from "./data";
import { icon } from "./icons";
import { preview } from "./preview";
import { onMode } from "./theme";

const FEATURED = { page: "forms", story: "input-textfield-clearableinput", title: "Input / TextField · ClearableInput" };

/** Folding tiles, a few facts, one component live. */
export function home(main: HTMLElement): () => void {
  document.title = "Ely · GPUI components";
  main.innerHTML = `
    <section class="hero"><div class="pin">
      <canvas aria-hidden="true"></canvas>
      <div class="words">
        <h1>Ely, a component library for GPUI.<strong>Every one of them runs here, live.</strong></h1>
        <p>${componentCount} components in ${chapters.length} chapters, in light and dark, written in Rust and compiled to WebAssembly for this page.</p>
        <div class="cta">
          <a class="button primary" href="/components/">Browse components ${icon("ArrowRight")}</a>
          <a class="button" href="${REPO}" rel="noopener">${icon("GitHub")}GitHub</a>
        </div>
      </div>
    </div></section>
    <section class="band">
      <div class="label">In numbers</div>
      <h2>One crate, one theme, every part tested at 280 pixels wide.</h2>
      <div class="facts">
        <div><strong>${storyCount}</strong><span>stories, each its own page</span></div>
        <div><strong>${chapters.length}</strong><span>chapters, from primitives to maps</span></div>
        <div><strong>2</strong><span>modes, light and dark, from one palette</span></div>
        <div><strong>0</strong><span>screenshots: what you see is running</span></div>
      </div>
    </section>
    <section class="band">
      <div class="label">Try one</div>
      <h2>A text field, live. Type in it.</h2>
      <div class="home-stage"></div>
    </section>`;

  const still = matchMedia("(prefers-reduced-motion: reduce)").matches;
  const lenis = still ? null : new Lenis({ autoRaf: true, anchors: true });
  const hero = main.querySelector<HTMLElement>(".hero")!;
  // The pinned hero folds as it scrolls.
  const fold = () => Math.min(1, Math.max(0, scrollY / Math.max(hero.offsetHeight - innerHeight, 1)));
  let stops: Array<() => void> = [];
  let sceneStop = () => {};
  // three.js loads only here.
  import("./scene").then(({ scene }) => {
    if (!hero.isConnected) return;
    const s = scene(main.querySelector("canvas")!, fold, still);
    sceneStop = s.stop;
    stops.push(onMode(() => s.paint()));
  });

  const stage = main.querySelector<HTMLElement>(".home-stage")!;
  const lazy = new IntersectionObserver(([entry]) => {
    if (!entry.isIntersecting) return;
    lazy.disconnect();
    stops.push(preview(stage, FEATURED.page, FEATURED.story, FEATURED.title));
  }, { rootMargin: "200px" });
  lazy.observe(stage);

  return () => {
    lenis?.destroy();
    lazy.disconnect();
    sceneStop();
    stops.forEach((stop) => stop());
    stops = [];
  };
}
