export type Mode = "light" | "dark";

const listeners = new Set<(mode: Mode) => void>();

export function mode(): Mode {
  return document.documentElement.classList.contains("dark") ? "dark" : "light";
}

export function setMode(next: Mode): void {
  document.documentElement.classList.toggle("dark", next === "dark");
  try {
    localStorage.setItem("ely-theme", next);
  } catch {}
  listeners.forEach((listen) => listen(next));
}

export function onMode(listen: (mode: Mode) => void): () => void {
  listeners.add(listen);
  return () => listeners.delete(listen);
}
