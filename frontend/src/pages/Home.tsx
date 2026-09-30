import { Link } from "@tanstack/react-router";
import { useEffect } from "react";
import { chapters, componentCount } from "../data";

export function Home() {
  useEffect(() => {
    document.title = "Ely · GPUI components";
  }, []);
  return (
    <section className="hero">
      <h1>Components for GPUI.</h1>
      <p>
        {componentCount} components in {chapters.length} chapters, live in the browser.
      </p>
      <Link to="/components/" className="button">
        Browse
      </Link>
    </section>
  );
}
