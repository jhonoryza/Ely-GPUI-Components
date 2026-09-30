export type Tone = "paper" | "ink" | "soft" | "accent";

/** A box on the table: cell, footprint, lift, height, tone, the beat it grows on. */
export interface Block {
  x: number;
  z: number;
  w: number;
  d: number;
  y: number;
  h: number;
  tone: Tone;
  beat: number;
}

export type Model = { blocks: Block[]; span: [number, number] };

const block = (x: number, z: number, w: number, d: number, h: number, tone: Tone, y = 0): Block => ({
  x, z, w, d, y, h, tone, beat: 0,
});

/** Beats from the far corner toward the eye, one diagonal a step. */
function sweep(blocks: Block[], pace = 1): Block[] {
  return blocks.map((b) => ({ ...b, beat: b.beat || Math.round((b.x + b.z) * pace) }));
}

/** A small app on its slab: a sidebar, a chart, a chat, a switch. */
export function app(): Model {
  const b: Block[] = [block(0, 0, 18, 12, 0.5, "paper")];
  b.push(block(0.5, 0.5, 17, 1, 0.3, "soft", 0.5));
  for (let i = 0; i < 3; i++) b.push(block(1 + i * 0.9, 0.8, 0.5, 0.4, 0.3, i === 0 ? "accent" : "ink", 0.8));
  for (let i = 0; i < 6; i++) b.push(block(0.8, 2.2 + i * 1.5, 3.4, 1, 0.3, i === 1 ? "accent" : "soft", 0.5));
  b.push(block(5, 2.2, 7.4, 1, 0.25, "soft", 0.5));
  b.push(block(5, 4, 7.4, 5.6, 0.12, "soft", 0.5));
  const bars = [0.9, 1.5, 1.2, 2.1, 1.7, 2.8, 2.3, 3.2];
  bars.forEach((h, i) => b.push(block(5.4 + i * 0.85, 7.2, 0.55, 1.2, h, i === 5 ? "accent" : "ink", 0.62)));
  b.push(block(5.4, 10.4, 6.6, 0.3, 0.15, "soft", 0.5));
  b.push(block(8.4, 10.3, 0.5, 0.5, 0.5, "accent", 0.5));
  for (let i = 0; i < 4; i++) {
    const mine = i % 2 === 1;
    b.push(block(mine ? 14.2 : 13, 2.2 + i * 1.7, mine ? 2.8 : 3.6, 1.2, 0.5, mine ? "ink" : "soft", 0.5));
  }
  b.push(block(13, 9.6, 1.6, 0.8, 0.35, "soft", 0.5));
  b.push(block(13.9, 9.6, 0.7, 0.8, 0.7, "accent", 0.5));
  return { blocks: sweep(b, 0.9), span: [18, 12] };
}

/** Twenty candles and their volume, rising in the accent, falling in ink. */
export function candles(): Model {
  const b: Block[] = [block(0, 0, 21, 8, 0.3, "paper")];
  const days: Array<[number, number, number, number]> = [];
  let close = 0;
  for (let i = 0; i < 20; i++) {
    const open = close;
    close = open + Math.sin(i * 1.7) * 1.1 + Math.cos(i * 0.6) * 0.6 + 0.25;
    days.push([open, close, Math.min(open, close) - 0.4 - (i % 3) * 0.2, Math.max(open, close) + 0.35 + (i % 2) * 0.3]);
  }
  // Prices fill 0.9 to 4.5 above the slab.
  const floor = Math.min(...days.map((d) => d[2]));
  const range = Math.max(...days.map((d) => d[3])) - floor;
  const lift = (price: number) => 0.9 + ((price - floor) / range) * 3.6;
  days.forEach(([open, close, low, high], i) => {
    const x = 0.7 + i;
    b.push({ ...block(x + 0.33, 2.73, 0.14, 0.14, lift(high) - lift(low), "soft", lift(low)), beat: 2 + i * 2 });
    const [bottom, top] = [lift(Math.min(open, close)), lift(Math.max(open, close))];
    b.push({ ...block(x, 2.4, 0.8, 0.8, Math.max(top - bottom, 0.2), close >= open ? "accent" : "ink", bottom), beat: 3 + i * 2 });
    b.push({ ...block(x, 5, 0.8, 1.6, 0.2 + ((i * 7) % 5) * 0.22, "soft", 0.3), beat: 3 + i * 2 });
  });
  return { blocks: sweep(b), span: [21, 8] };
}

/** Five series of eight months, one in the accent. */
export function bars(): Model {
  const b: Block[] = [block(0, 0, 11, 8, 0.3, "paper")];
  for (let z = 0; z < 5; z++) {
    for (let x = 0; x < 8; x++) {
      const h = 0.4 + 2.2 * (0.5 + 0.5 * Math.sin(x * 0.7 + z * 1.1)) * (1 + z * 0.12);
      const tone: Tone = z === 2 ? "accent" : z === 4 ? "ink" : "soft";
      b.push({ ...block(1 + x * 1.2, 1 + z * 1.3, 0.8, 0.8, h, tone, 0.3), beat: x * 2 + z });
    }
  }
  return { blocks: b, span: [11, 8] };
}

/** A conversation: turns left and right, and three dots still typing. */
export function chat(): Model {
  const b: Block[] = [block(0, 0, 10, 12, 0.3, "paper")];
  const turns: Array<[boolean, number]> = [[false, 6], [true, 4.4], [false, 7.2], [true, 3.2], [false, 5.4]];
  turns.forEach(([mine, w], i) => {
    const z = 1 + i * 2;
    if (!mine) b.push({ ...block(0.8, z + 0.3, 0.8, 0.8, 0.8, "ink", 0.3), beat: i * 4 });
    b.push({ ...block(mine ? 9.2 - w : 2, z, w, 1.4, 0.45, mine ? "accent" : "soft", 0.3), beat: i * 4 + 1 });
  });
  for (let i = 0; i < 3; i++) b.push({ ...block(2.2 + i * 0.7, 10.9, 0.45, 0.45, 0.45, "ink", 0.3), beat: 21 + i });
  return { blocks: b, span: [10, 12] };
}
