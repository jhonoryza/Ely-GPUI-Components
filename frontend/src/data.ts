import manifest from "../../examples/gallery/stories.json";

export interface Story {
  title: string;
  slug: string;
  components: string[];
}

export interface Chapter {
  number: number;
  slug: string;
  title: string;
  summary: string;
  stories: Story[];
}

export const chapters: Chapter[] = manifest.pages;

export const storyCount = chapters.reduce((sum, chapter) => sum + chapter.stories.length, 0);

export const componentCount = new Set(chapters.flatMap((c) => c.stories.flatMap((s) => s.components))).size;

export function chapter(slug: string): Chapter | undefined {
  return chapters.find((c) => c.slug === slug);
}

export function href(chapter: Chapter, story?: Story): string {
  return story ? `/components/${chapter.slug}/${story.slug}/` : `/components/${chapter.slug}/`;
}

/** Every story in reading order, for paging and search. */
export const all = chapters.flatMap((c) => c.stories.map((s) => ({ chapter: c, story: s })));

export function esc(text: string): string {
  return text.replace(/[&<>"']/g, (c) => `&#${c.charCodeAt(0)};`);
}

export const REPO = "https://github.com/ZacharyZhang-NY/Ely-GPUI-Components";
