import { useEffect, useRef } from "react";
import { useMode } from "../theme";
import { miniature } from "./draw";
import type { Block } from "./engine";

/** Grows once it shows, and again when pressed. */
export function Iso({ model, label, className }: { model: Block[]; label: string; className?: string }) {
  const canvas = useRef<HTMLCanvasElement>(null);
  const mini = useRef<ReturnType<typeof miniature> | null>(null);
  const theme = useMode();

  useEffect(() => {
    const still = matchMedia("(prefers-reduced-motion: reduce)").matches;
    const m = miniature(canvas.current!, model, still);
    mini.current = m;
    const seen = new IntersectionObserver(([e]) => e.isIntersecting && m.start(), { threshold: 0.35 });
    seen.observe(canvas.current!);
    return () => {
      seen.disconnect();
      m.stop();
    };
  }, [model]);

  useEffect(() => mini.current?.repaint(), [theme]);

  return <canvas ref={canvas} className={className} role="img" aria-label={label} onClick={() => mini.current?.replay()} />;
}
