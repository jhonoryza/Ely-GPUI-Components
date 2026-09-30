import shots from "./shots.json";
import { esc } from "./data";
import { mode, onMode, type Mode } from "./theme";

const captured = new Set<string>(shots.stories);
const windows: Record<string, string[]> = shots.windows;

/** Whether the site shows this story as a native capture. */
export function isCaptured(page: string, story: string): boolean {
  return captured.has(`${page}/${story}`);
}

const src = (name: string, theme: Mode) => `/shots/${name}-${theme}.jpg`;

function figure(name: string, alt: string): string {
  // Captures are at twice the pixels, drawn at their size.
  return `<img class="shot" data-shot="${name}" srcset="${src(name, mode())} 2x" alt="${esc(alt)}" loading="lazy" />`;
}

/** Draws the story's capture, or the windows it opens, in the site's mode. */
export function native(host: HTMLElement, page: string, story: string, title: string): () => void {
  const key = `${page}/${story}`;
  if (captured.has(key)) {
    host.innerHTML = `<p class="label">Native capture · macOS</p>${figure(`${page}--${story}`, `${title}, captured from the native gallery`)}`;
  } else if (windows[key]) {
    host.innerHTML = `<p class="label">The windows it opens, natively</p><div class="shots">${windows[key]
      .map((name) => figure(name, `A window ${title} opens, captured natively`))
      .join("")}</div>`;
  } else {
    return () => {};
  }
  return onMode((next) => host.querySelectorAll<HTMLImageElement>("img[data-shot]").forEach((img) => (img.srcset = `${src(img.dataset.shot!, next)} 2x`)));
}
