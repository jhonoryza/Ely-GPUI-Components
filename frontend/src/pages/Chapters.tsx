import { Link } from "@tanstack/react-router";
import { useEffect } from "react";
import { chapters, pad } from "../data";

export function Chapters() {
  useEffect(() => {
    document.title = "Components · Ely";
  }, []);
  return (
    <section className="chapters">
      <h1>Components</h1>
      <ol>
        {chapters.map((c) => (
          <li key={c.slug}>
            <Link to="/components/$page/$story/" params={{ page: c.slug, story: c.stories[0].slug }}>
              <span className="n">{pad(c.number)}</span>
              <span className="name">{c.title}</span>
              <span className="count">{c.stories.length}</span>
            </Link>
          </li>
        ))}
      </ol>
    </section>
  );
}
