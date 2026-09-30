import { expect, test, type Page } from "@playwright/test";
import manifest from "../../examples/gallery/stories.json" with { type: "json" };

const STARTS = 45_000;

async function live(page: Page, path: string) {
  await page.goto(path);
  const frame = page.frameLocator(".stage iframe");
  // The host hides its state once the gallery says it is ready.
  await expect(page.locator(".stage .state")).toBeHidden({ timeout: STARTS });
  return frame;
}

test("a story runs live and follows the site's mode", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "light" });
  const frame = await live(page, "/components/buttons/button/");
  await expect(frame.locator("canvas")).toBeVisible();
  await expect(frame.locator("html")).not.toHaveClass(/dark/);
  await page.locator("[data-mode]").click();
  await expect(frame.locator("html")).toHaveClass(/dark/);
});

test("the live story leaves focus with the host until it is pressed", async ({ page, isMobile }) => {
  test.skip(isMobile, "no hardware keyboard");
  await live(page, "/components/forms/input-textfield-clearableinput/");
  await page.waitForTimeout(1000);
  expect(await page.evaluate(() => document.activeElement?.tagName)).not.toBe("IFRAME");
  await page.keyboard.press("/");
  await expect(page.getByRole("dialog", { name: "Search components" })).toBeVisible();
});

test("a skip link passes the live example", async ({ page, isMobile }) => {
  test.skip(isMobile, "no hardware keyboard");
  await page.goto("/components/buttons/button/");
  const skip = page.getByRole("link", { name: "Skip the live example" });
  await skip.focus();
  await expect(skip).toBeVisible();
  await page.keyboard.press("Enter");
  await expect(page.locator("#after-stage")).toBeFocused();
});

test("typing reaches a live field after a press", async ({ page, isMobile }) => {
  test.skip(isMobile, "no hardware keyboard");
  const frame = await live(page, "/components/forms/input-textfield-clearableinput/");
  const box = (await page.locator(".stage iframe").boundingBox())!;
  // The first field sits under the section's note, as the gallery lays it.
  await page.mouse.click(box.x + 120, box.y + 72);
  const before = await frame.locator("canvas").screenshot();
  await page.keyboard.type("Ada");
  await expect(frame.locator("textarea")).toBeFocused();
  await expect.poll(async () => (await frame.locator("canvas").screenshot()).equals(before)).toBe(false);
});

test("full screen covers the page and Escape leaves it", async ({ page }) => {
  await live(page, "/components/buttons/button/");
  await page.locator(".stage").hover();
  await page.getByRole("button", { name: "Full screen" }).click();
  await expect(page.locator(".stage")).toHaveClass(/full/);
  await page.keyboard.press("Escape");
  await expect(page.locator(".stage")).not.toHaveClass(/full/);
});

// One story from every fifth chapter, each drawn in its frame.
for (const chapter of manifest.pages.filter((_, i) => i % 5 === 0)) {
  test(`the first ${chapter.title} story draws`, async ({ page }) => {
    const frame = await live(page, `/components/${chapter.slug}/`);
    const shot = await frame.locator("canvas").screenshot();
    expect(shot.length).toBeGreaterThan(4000);
  });
}
