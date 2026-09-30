import "@fontsource-variable/geist";
import "@fontsource-variable/geist-mono";
import { createRootRoute, createRoute, createRouter, redirect, RouterProvider } from "@tanstack/react-router";
import { createRoot } from "react-dom/client";
import { Shell } from "./Shell";
import { chapters } from "./data";
import { Chapters } from "./pages/Chapters";
import { Home } from "./pages/Home";
import { NotFound } from "./pages/NotFound";
import { Story } from "./pages/Story";
import "./styles.css";

const root = createRootRoute({ component: Shell, notFoundComponent: () => <NotFound /> });
const home = createRoute({ getParentRoute: () => root, path: "/", component: Home });
const index = createRoute({ getParentRoute: () => root, path: "/components", component: Chapters });
const chapter = createRoute({
  getParentRoute: () => root,
  path: "/components/$page",
  // A chapter opens its first story.
  beforeLoad: ({ params }) => {
    const found = chapters.find((c) => c.slug === params.page);
    if (found) throw redirect({ to: "/components/$page/$story/", params: { page: found.slug, story: found.stories[0].slug } });
  },
  component: () => <NotFound what="No chapter by that name." />,
});
const story = createRoute({ getParentRoute: () => root, path: "/components/$page/$story", component: Story });

const router = createRouter({
  routeTree: root.addChildren([home, index, chapter, story]),
  trailingSlash: "always",
  scrollRestoration: true,
  defaultViewTransition: true,
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

createRoot(document.getElementById("root")!).render(<RouterProvider router={router} />);
