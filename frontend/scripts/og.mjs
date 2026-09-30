// Renders public/og.png from the hero; needs pnpm preview.
import { chromium } from "@playwright/test";

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1200, height: 630 }, deviceScaleFactor: 2, colorScheme: "light", reducedMotion: "reduce" });
await page.goto(process.env.SITE ?? "http://localhost:4173/");
await page.addStyleTag({
  content: ".bar, .cta { display: none } .hero { min-height: 630px; padding: 56px } .words { zoom: 1.5 } .og { font-weight: 600; margin-bottom: 16px }",
});
await page.evaluate(() => {
  document.querySelector(".glyphs").insertAdjacentHTML("afterend", '<div class="og">Ely</div>');
  document.querySelector(".hero p").textContent = "Every component live in the browser, compiled from Rust.";
});
await page.waitForTimeout(1500);
await page.screenshot({ path: "public/og.png", clip: { x: 0, y: 0, width: 1200, height: 630 } });
await browser.close();
console.log("og: wrote public/og.png");
