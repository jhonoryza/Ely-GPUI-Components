import type { Block } from "./engine";

const b = (x: number, z: number, w: number, d: number, h: number, y = 0, accent = false): Block => ({ x, z, w, d, h, y, accent, beat: 0 });

/** Beats from the far corner toward the eye. */
const sweep = (blocks: Block[], pace: number) => blocks.map((k) => ({ ...k, beat: Math.round((k.x + k.z) * pace) }));

/** An app: title bar, sidebar, chart, chat, switch. */
export function app(): Block[] {
  const s: Block[] = [b(0, 0, 18, 12, 0.5)];
  s.push(b(0.6, 0.6, 16.8, 1, 0.25, 0.5));
  for (let i = 0; i < 3; i++) s.push(b(1 + i * 0.8, 0.85, 0.45, 0.45, 0.25, 0.75, i === 0));
  for (let i = 0; i < 6; i++) s.push(b(0.8, 2.2 + i * 1.5, 3.4, 1, 0.25, 0.5, i === 1));
  s.push(b(5, 2.2, 7.4, 1, 0.2, 0.5));
  s.push(b(5, 3.8, 7.4, 6.4, 0.08, 0.5));
  [0.9, 1.5, 1.2, 2.1, 1.7, 2.8, 2.3, 3.2].forEach((h, i) => s.push(b(5.5 + i * 0.85, 7.6, 0.55, 1.2, h, 0.58, i === 5)));
  for (let i = 0; i < 4; i++) s.push(b(i % 2 ? 14.2 : 13, 2.2 + i * 1.7, i % 2 ? 2.8 : 3.6, 1.2, 0.4, 0.5, i === 3));
  s.push(b(13, 9.8, 1.6, 0.8, 0.3, 0.5), b(13.9, 9.8, 0.7, 0.8, 0.6, 0.5, true));
  return sweep(s, 0.9);
}
