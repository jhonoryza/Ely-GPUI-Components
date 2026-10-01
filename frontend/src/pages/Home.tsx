import { Link } from "@tanstack/react-router";
import { useEffect } from "react";
import { chapters, componentCount } from "../data";
import { Iso } from "../iso/Iso";
import { app } from "../iso/models";

const APP = app();

export function Home() {
  useEffect(() => {
    document.title = "Ely · GPUI components";
  }, []);
  return (
    <section className="hero">
      <Iso model={APP} label="A miniature app growing cell by cell" className="hero-iso" />
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
