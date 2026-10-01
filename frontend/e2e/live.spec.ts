import { expect, test, type Page } from "@playwright/test";
import manifest from "../../examples/gallery/stories.json" with { type: "json" };

const STARTS = 45_000;

async function live(page: Page, path: string) {
  await page.goto(path);
  await expect(page.locator(".live iframe")).toHaveCount(1);
  // The host drops its state once the gallery says it is ready.
  await expect(page.locator(".live .live-state")).toHaveCount(0, { timeout: STARTS });
  return page.frameLocator(".live iframe");
}

test("a story runs live and follows the site's mode", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "light" });
  const frame = await live(page, "/components/buttons/button/");
  await expect(frame.locator("canvas")).toBeVisible();
  await expect(frame.locator("html")).not.toHaveClass(/dark/);
  await page.getByRole("button", { name: "Dark mode" }).click();
  await expect(frame.locator("html")).toHaveClass(/dark/);
});

test("a starting story leaves focus with the host", async ({ page, isMobile }) => {
  test.skip(isMobile, "no hardware keyboard");
  await page.goto("/");
  await page.getByRole("link", { name: "Browse" }).first().focus();
  await page.locator(".feature").first().scrollIntoViewIfNeeded();
  await expect(page.locator(".feature .live .live-state")).toHaveCount(0, { timeout: STARTS });
  await expect(page.getByRole("link", { name: "Browse" }).first()).toBeFocused();
});

test("the skip control passes the live example", async ({ page, isMobile }) => {
  test.skip(isMobile, "no hardware keyboard");
  await page.goto("/components/buttons/button/");
  const skip = page.getByRole("button", { name: "Skip the live example" });
  await skip.focus();
  await expect(skip).toBeVisible();
  await page.keyboard.press("Enter");
  await expect(page.locator(".live + span")).toBeFocused();
});

test("typing reaches a live field after a press", async ({ page, isMobile }) => {
  test.skip(isMobile, "no hardware keyboard");
  const frame = await live(page, "/components/forms/input-textfield-clearableinput/");
  const box = (await page.locator(".live iframe").boundingBox())!;
  // The first field sits under the section's note, as the gallery lays it.
  await page.mouse.click(box.x + 120, box.y + 72);
  const before = await frame.locator("canvas").screenshot();
  await page.keyboard.type("Ada");
  await expect(frame.locator("textarea")).toBeFocused();
  await expect.poll(async () => (await frame.locator("canvas").screenshot()).equals(before)).toBe(false);
});

test("full screen covers the page and Escape leaves it", async ({ page }) => {
  await live(page, "/components/buttons/button/");
  await page.locator(".live").hover();
  await page.getByRole("button", { name: "Full screen" }).click();
  await expect(page.locator(".live")).toHaveClass(/full/);
  await page.keyboard.press("Escape");
  await expect(page.locator(".live")).not.toHaveClass(/full/);
});

test("home frames run only near the view", async ({ page }) => {
  await page.goto("/");
  await page.locator(".close").scrollIntoViewIfNeeded();
  await expect(page.locator(".feature iframe")).toHaveCount(0);
});

// One story from every fifth chapter, each drawn in its frame.
for (const chapter of manifest.pages.filter((_, i) => i % 5 === 0)) {
  test(`the first ${chapter.title} story draws`, async ({ page }) => {
    const frame = await live(page, `/components/${chapter.slug}/`);
    const shot = await frame.locator("canvas").screenshot();
    expect(shot.length).toBeGreaterThan(4000);
  });
}
