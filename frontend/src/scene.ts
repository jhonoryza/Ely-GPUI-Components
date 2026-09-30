import * as THREE from "three";
import { RoundedBoxGeometry } from "three/addons/geometries/RoundedBoxGeometry.js";
import { chapters } from "./data";

const STEP = 1.2;

/** A tile per story, folding into chapter columns. */
export function scene(canvas: HTMLCanvasElement, fold: () => number, still: boolean): { stop: () => void; paint: () => void } {
  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: true });
  renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
  const camera = new THREE.PerspectiveCamera(32, 1, 0.1, 200);
  const world = new THREE.Scene();
  world.add(new THREE.HemisphereLight(0xffffff, 0x888888, 2.2));
  const sun = new THREE.DirectionalLight(0xffffff, 1.2);
  sun.position.set(-6, 10, 12);
  world.add(sun);

  const tiles = chapters.flatMap((c, col) => c.stories.map((_, row) => ({ col, row })));
  const mesh = new THREE.InstancedMesh(
    new RoundedBoxGeometry(1, 1, 0.24, 3, 0.16),
    new THREE.MeshStandardMaterial({ roughness: 0.7, metalness: 0 }),
    tiles.length,
  );
  world.add(mesh);

  // At rest a near-square lattice; folded, one column per chapter.
  const across = Math.ceil(Math.sqrt(tiles.length * 1.6));
  const rest = tiles.map((_, i) => new THREE.Vector3(((i % across) - across / 2) * STEP, (Math.floor(i / across) - tiles.length / across / 2) * -STEP, 0));
  const tallest = Math.max(...chapters.map((c) => c.stories.length));
  const folded = tiles.map(({ col, row }) => new THREE.Vector3((col - chapters.length / 2) * STEP * 0.9, (row - tallest / 2) * -STEP * 0.5, 0));
  // The tiles' widest and tallest reach, kept in view.
  const reach = { w: Math.max(across, chapters.length * 0.9) * STEP + 2, h: Math.max(tiles.length / across, tallest * 0.5) * STEP + 2 };
  let distance = 34;

  const paint = () => {
    const css = getComputedStyle(document.documentElement);
    const [ink, accent] = [new THREE.Color(css.getPropertyValue("--line").trim()), new THREE.Color(css.getPropertyValue("--accent").trim())];
    // About one tile in ten wears the accent.
    tiles.forEach((_, i) => mesh.setColorAt(i, (i * 7) % 10 === 3 ? accent : ink));
    mesh.instanceColor!.needsUpdate = true;
  };

  const size = () => {
    const { clientWidth: w, clientHeight: h } = canvas;
    renderer.setSize(w, h, false);
    camera.aspect = w / Math.max(h, 1);
    camera.updateProjectionMatrix();
    const k = 2 * Math.tan(THREE.MathUtils.degToRad(camera.fov / 2));
    distance = Math.max(reach.w / (k * camera.aspect), reach.h / k);
  };

  const at = new THREE.Object3D();
  const spot = new THREE.Vector3();
  let running = true;
  let frame = 0;
  const draw = (time: number) => {
    const t = still ? 0 : fold();
    const ease = t * t * (3 - 2 * t);
    tiles.forEach((_, i) => {
      spot.lerpVectors(rest[i], folded[i], ease);
      const wave = still ? 0 : Math.sin(time / 1400 + rest[i].x * 0.35 + rest[i].y * 0.2) * 0.35 * (1 - ease);
      at.position.set(spot.x, spot.y, wave);
      at.rotation.set(-0.5 * (1 - ease), 0, 0);
      at.scale.setScalar(1 - 0.45 * ease);
      at.updateMatrix();
      mesh.setMatrixAt(i, at.matrix);
    });
    mesh.instanceMatrix.needsUpdate = true;
    camera.position.set(0, -0.12 * distance * (1 - ease), distance);
    camera.lookAt(0, 0, 0);
    renderer.render(world, camera);
    if (running && !still) frame = requestAnimationFrame(draw);
  };
  const start = () => {
    cancelAnimationFrame(frame);
    frame = requestAnimationFrame(draw);
  };

  const seen = new IntersectionObserver(([entry]) => {
    running = entry.isIntersecting && !document.hidden;
    if (running) start();
  });
  seen.observe(canvas);
  const visible = () => {
    running = !document.hidden;
    if (running) start();
  };
  const resized = new ResizeObserver(() => (size(), start()));
  resized.observe(canvas);
  document.addEventListener("visibilitychange", visible);
  paint();
  size();
  start();

  const stop = () => {
    running = false;
    cancelAnimationFrame(frame);
    seen.disconnect();
    resized.disconnect();
    document.removeEventListener("visibilitychange", visible);
    renderer.dispose();
    mesh.geometry.dispose();
    (mesh.material as THREE.Material).dispose();
  };
  return { stop, paint: () => (paint(), start()) };
}
