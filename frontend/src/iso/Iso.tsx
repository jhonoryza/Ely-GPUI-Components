import { useEffect, useRef, useSyncExternalStore } from "react";
import { useMode } from "../theme";
import { miniature } from "./draw";
import type { Block } from "./engine";

const REDUCED = matchMedia("(prefers-reduced-motion: reduce)");

function useStill(): boolean {
  return useSyncExternalStore(
    (listen) => (REDUCED.addEventListener("change", listen), () => REDUCED.removeEventListener("change", listen)),
    () => REDUCED.matches,
  );
}

/** Grows once it shows; a press grows it again. */
export function Iso({ model, label, className }: { model: Block[]; label: string; className?: string }) {
  const canvas = useRef<HTMLCanvasElement>(null);
  const mini = useRef<ReturnType<typeof miniature> | null>(null);
  const theme = useMode();
  const still = useStill();

  useEffect(() => {
    const m = miniature(canvas.current!, model, still);
    mini.current = m;
    const seen = new IntersectionObserver(([e]) => e.isIntersecting && m.start(), { threshold: 0.35 });
    seen.observe(canvas.current!);
    return () => {
      seen.disconnect();
      m.stop();
    };
  }, [model, still]);

  useEffect(() => mini.current?.repaint(), [theme]);

  // Still, there is nothing to replay.
  if (still) return <canvas ref={canvas} className={`iso ${className ?? ""}`} role="img" aria-label={label} />;
  return (
    <button className={`iso ${className ?? ""}`} aria-label={`${label}. Grow it again`} onClick={() => mini.current?.replay()}>
      <canvas ref={canvas} />
    </button>
  );
}
