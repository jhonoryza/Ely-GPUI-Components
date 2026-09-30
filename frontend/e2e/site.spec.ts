import { expect, test, type Page } from "@playwright/test";
import manifest from "../../examples/gallery/stories.json" with { type: "json" };

/** Fails the test on any console error or uncaught exception. */
function watch(page: Page): string[] {
  const errors: string[] = [];
  page.on("console", (m) => m.type() === "error" && errors.push(m.text()));
  page.on("pageerror", (e) => errors.push(e.message));
  return errors;
}

test("the home page shows the pitch, the scene and the credits", async ({ page }) => {
  const errors = watch(page);
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1 })).toContainText("component library for GPUI");
  await expect(page.locator("canvas.iso")).toBeAttached();
  await expect(page.locator(".hero-iso")).toBeVisible();
  await expect(page.locator("#foot")).toContainText("Solar Icons by 480 Design");
  expect(errors).toEqual([]);
});

test("the mode toggles, and a reload keeps it", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "light" });
  await page.goto("/components/");
  const html = page.locator("html");
  await expect(html).not.toHaveClass(/dark/);
  await page.getByRole("button", { name: "Dark mode" }).click();
  await expect(html).toHaveClass(/dark/);
  await page.reload();
  await expect(html).toHaveClass(/dark/);
});

test("the index lists every chapter with its story count", async ({ page }) => {
  await page.goto("/components/");
  const rows = page.locator(".grid a");
  await expect(rows).toHaveCount(manifest.pages.length);
  const first = manifest.pages[0];
  await expect(rows.first()).toContainText(`${first.stories.length} stories`);
});

test("a chapter opens its first story, and paging walks on", async ({ page }) => {
  const [chapter] = manifest.pages;
  await page.goto(`/components/${chapter.slug}/`);
  await expect(page.getByRole("heading", { level: 1 })).toHaveText(chapter.stories[0].title);
  await page.locator(".pager a.next").click();
  await expect(page).toHaveURL(`/components/${chapter.slug}/${chapter.stories[1].slug}/`);
  await expect(page.getByRole("heading", { level: 1 })).toHaveText(chapter.stories[1].title);
});

test("the sidebar filters by story and component name", async ({ page }) => {
  await page.goto("/components/");
  await page.getByPlaceholder("Filter components").fill("Toast");
  await expect(page.locator(".side li a").first()).toContainText("Toast");
  await page.getByPlaceholder("Filter components").fill("zzzz");
  await expect(page.locator(".side .none")).toContainText("Nothing matches");
});

test("search opens with the keyboard and goes where Enter points", async ({ page, isMobile }) => {
  test.skip(isMobile, "no hardware keyboard");
  await page.goto("/components/");
  await page.keyboard.press("/");
  const dialog = page.getByRole("dialog", { name: "Search components" });
  await expect(dialog).toBeVisible();
  await page.keyboard.type("breadcrumb");
  await page.keyboard.press("Enter");
  await expect(dialog).toBeHidden();
  await expect(page).toHaveURL(/\/components\/navigation\/.*breadcrumb/);
});

test("the Browse menu opens a chapter's stories beside it", async ({ page, isMobile }) => {
  test.skip(isMobile, "hover menu");
  await page.goto("/components/");
  await page.getByRole("button", { name: "Browse" }).click();
  await page.getByRole("menuitem", { name: "Charts" }).hover();
  const story = page.getByRole("menuitem", { name: /^RadarChart/ });
  await expect(story).toBeVisible();
  await story.click();
  await expect(page).toHaveURL(/\/components\/charts\/radarchart/);
});

test("an unknown chapter says so", async ({ page }) => {
  const errors = watch(page);
  await page.goto("/components/nowhere/");
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Not found");
  expect(errors.join()).toContain("No chapter");
});

test("a story the web cannot run shows its native capture, in the site's mode", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "light" });
  await page.goto("/components/forms/pathinput/");
  const shot = page.locator("img.shot");
  await expect(shot).toHaveAttribute("srcset", /pathinput-light\.jpg/);
  await expect(page.locator(".stage iframe")).toHaveCount(0);
  await page.locator("[data-mode]").click();
  await expect(shot).toHaveAttribute("srcset", /pathinput-dark\.jpg/);
  await expect.poll(() => shot.evaluate((img: HTMLImageElement) => img.naturalWidth)).toBeGreaterThan(0);
});

test("a live story that opens a window shows that window natively", async ({ page }) => {
  await page.goto("/components/shell/aboutdialog/");
  await expect(page.locator(".stage iframe")).toHaveCount(1);
  await expect(page.locator(".native img.shot")).toHaveAttribute("srcset", /shell-about-window/);
});
