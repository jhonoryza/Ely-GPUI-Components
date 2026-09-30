import { Link, useParams } from "@tanstack/react-router";
import { useEffect } from "react";
import { Live } from "../Live";
import { Capture, Windows, isCaptured } from "../Native";
import { all, chapters } from "../data";
import { NotFound } from "./NotFound";

export function Story() {
  const { page, story } = useParams({ strict: false });
  const chapter = chapters.find((c) => c.slug === page);
  const found = chapter?.stories.find((s) => s.slug === story);
  useEffect(() => {
    if (chapter && found) document.title = `${found.title} · ${chapter.title} · Ely`;
  }, [chapter, found]);
  if (!chapter || !found) return <NotFound what={`No story at ${page}/${story}.`} />;
  const at = all.findIndex((e) => e.story === found);
  const [prev, next] = [all[at - 1], all[at + 1]];
  return (
    <article className="story">
      <Link to="/components/" className="crumb">
        {chapter.title}
      </Link>
      <h1>{found.title}</h1>
      {isCaptured(chapter.slug, found.slug) ? (
        <Capture page={chapter.slug} story={found.slug} title={found.title} />
      ) : (
        <>
          <Live key={`${chapter.slug}/${found.slug}`} page={chapter.slug} story={found.slug} title={found.title} />
          <Windows page={chapter.slug} story={found.slug} title={found.title} />
        </>
      )}
      <nav className="pager" aria-label="More stories">
        {prev && (
          <Link to="/components/$page/$story/" params={{ page: prev.chapter.slug, story: prev.story.slug }}>
            {prev.story.title}
          </Link>
        )}
        {next && (
          <Link className="next" to="/components/$page/$story/" params={{ page: next.chapter.slug, story: next.story.slug }}>
            {next.story.title}
          </Link>
        )}
      </nav>
    </article>
  );
}
