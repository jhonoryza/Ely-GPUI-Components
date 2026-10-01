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

export const componentCount = new Set(chapters.flatMap((c) => c.stories.flatMap((s) => s.components))).size;

/** Every story in reading order, for paging and search. */
export const all = chapters.flatMap((chapter) => chapter.stories.map((story) => ({ chapter, story })));

export const REPO = "https://github.com/ZacharyZhang-NY/Ely-GPUI-Components";

export const pad = (n: number) => String(n).padStart(2, "0");
