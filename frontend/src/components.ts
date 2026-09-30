import { all, chapter, chapters, esc, href, storyCount, type Chapter, type Story } from "./data";
import { icon } from "./icons";
import { preview } from "./preview";

/** Chapters on the left, a story or the index beside. */
export function components(main: HTMLElement, path: string[]): () => void {
  const [pageSlug, storySlug] = path;
  const current = pageSlug ? chapter(pageSlug) : undefined;
  if (pageSlug && !current) return missing(main, `No chapter “${pageSlug}”.`);
  const story = current && (storySlug ? current.stories.find((s) => s.slug === storySlug) : current.stories[0]);
  if (current && !story) return missing(main, `No story “${storySlug}” in ${current.title}.`);

  main.innerHTML = `<div class="docs"><aside class="side" aria-label="Components"></aside><div class="body"></div></div>`;
  const side = main.querySelector<HTMLElement>(".side")!;
  const body = main.querySelector<HTMLElement>(".body")!;
  sidebar(side, current, story);

  if (!current || !story) {
    body.innerHTML = index();
    document.title = "Components · Ely";
    return () => {};
  }
  document.title = `${story.title} · ${current.title} · Ely`;
  body.innerHTML = detail(current, story);
  return preview(body.querySelector<HTMLElement>(".live")!, current.slug, story.slug, story.title);
}

function sidebar(side: HTMLElement, current?: Chapter, story?: Story): void {
  side.innerHTML = `<input type="search" placeholder="Filter components" aria-label="Filter components" /><div class="tree"></div>`;
  const input = side.querySelector("input")!;
  const tree = side.querySelector<HTMLElement>(".tree")!;
  const draw = () => {
    const query = input.value.trim().toLowerCase();
    const rows = chapters
      .map((c) => {
        const hits = query ? c.stories.filter((s) => matches(s, query)) : c === current ? c.stories : [];
        if (query && !hits.length && !c.title.toLowerCase().includes(query)) return "";
        const open = query || c === current;
        return `<a class="chapter" href="${href(c)}" aria-current="${c === current}"><span class="n">${String(c.number).padStart(2, "0")}</span>${esc(c.title)}</a>${
          open && hits.length
            ? `<ul>${hits.map((s) => `<li><a href="${href(c, s)}" ${s === story ? 'aria-current="page"' : ""}>${esc(s.title)}</a></li>`).join("")}</ul>`
            : ""
        }`;
      })
      .join("");
    tree.innerHTML = rows || `<p class="none">Nothing matches “${esc(input.value)}”.</p>`;
  };
  input.addEventListener("input", draw);
  draw();
  side.querySelector<HTMLElement>('[aria-current="page"]')?.scrollIntoView({ block: "center" });
}

export function matches(story: Story, query: string): boolean {
  return story.title.toLowerCase().includes(query) || story.components.some((c) => c.toLowerCase().includes(query));
}

function detail(current: Chapter, story: Story): string {
  const at = all.findIndex((e) => e.story === story && e.chapter === current);
  const [prev, next] = [all[at - 1], all[at + 1]];
  const pager = (entry: (typeof all)[number] | undefined, cls: string, word: string, arrow: string) =>
    entry
      ? `<a class="${cls}" href="${href(entry.chapter, entry.story)}"><span class="label">${cls === "prev" ? arrow : ""}${word}${cls === "next" ? arrow : ""}</span>${esc(entry.story.title)}</a>`
      : "";
  return `<article class="detail">
    <div class="crumb"><a href="/components/">Components</a> / <a href="${href(current)}">${esc(current.title)}</a></div>
    <h1>${esc(story.title)}</h1>
    ${story.components.length ? `<div class="chips">${story.components.map((c) => `<span>${esc(c)}</span>`).join("")}</div>` : `<div class="chips"></div>`}
    <div class="live"></div>
    <nav class="pager" aria-label="More stories">${pager(prev, "prev", "Previous", icon("ArrowLeft"))}${pager(next, "next", "Next", icon("ArrowRight"))}</nav>
  </article>`;
}

function index(): string {
  return `<section class="index">
    <h1>Components</h1>
    <p>${storyCount} stories in ${chapters.length} chapters. Each one runs live, compiled to WebAssembly.</p>
    <div class="grid">${chapters
      .map((c) => `<a href="${href(c)}"><span class="n">${String(c.number).padStart(2, "0")}</span><span><strong>${esc(c.title)}</strong><small>${c.stories.length} stories</small></span></a>`)
      .join("")}</div>
  </section>`;
}

function missing(main: HTMLElement, why: string): () => void {
  console.error(`components: ${why}`);
  main.innerHTML = `<section class="index"><h1>Not found</h1><p>${esc(why)}</p><a class="button" href="/components/">All components</a></section>`;
  document.title = "Not found · Ely";
  return () => {};
}
