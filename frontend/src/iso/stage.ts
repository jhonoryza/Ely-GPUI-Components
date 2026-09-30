import * as THREE from "three";
import { BEAT, strike } from "./marimba";
import type { Block, Model, Tone } from "./models";

const GROW = 260;
const BUMP = 420;

interface Scene {
  host: HTMLElement;
  model: Model;
  world: THREE.Scene;
  camera: THREE.OrthographicCamera;
  mesh: THREE.InstancedMesh;
  grid: THREE.LineSegments;
  started: number | null;
  struck: number;
  bumps: Map<number, number>;
}

const box = new THREE.BoxGeometry(1, 1, 1).translate(0.5, 0.5, 0.5);
const at = new THREE.Object3D();
const ray = new THREE.Raycaster();
const tones: Record<Tone, THREE.Color> = { paper: new THREE.Color(), ink: new THREE.Color(), soft: new THREE.Color(), accent: new THREE.Color() };
const lines = new THREE.Color();

/** Back-out easing: a block overshoots, then settles on its height. */
const rise = (t: number) => 1 + 2.2 * (t - 1) ** 3 + 1.2 * (t - 1) ** 2;

/** Isometric dioramas, one canvas over the page, each drawn in its host's box. */
export function stage(still: boolean) {
  const canvas = document.createElement("canvas");
  canvas.className = "iso";
  canvas.setAttribute("aria-hidden", "true");
  document.body.append(canvas);
  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: true });
  renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
  renderer.setScissorTest(true);
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type = THREE.PCFSoftShadowMap;
  const scenes: Scene[] = [];
  let frame = 0;

  const paint = () => {
    const css = getComputedStyle(document.documentElement);
    for (const tone of Object.keys(tones) as Tone[]) tones[tone].set(css.getPropertyValue(`--iso-${tone}`).trim());
    lines.set(css.getPropertyValue("--line").trim());
    for (const s of scenes) {
      s.model.blocks.forEach((b, i) => s.mesh.setColorAt(i, tones[b.tone]));
      s.mesh.instanceColor!.needsUpdate = true;
      (s.grid.material as THREE.LineBasicMaterial).color.copy(lines);
    }
  };

  const place = (s: Scene, now: number) => {
    let moving = false;
    s.model.blocks.forEach((b: Block, i) => {
      const since = s.started === null ? -1 : now - s.started - b.beat * BEAT;
      const grown = still ? 1 : since < 0 ? 0 : since >= GROW ? 1 : rise(since / GROW);
      const bump = s.bumps.get(i);
      const lift = bump === undefined ? 0 : Math.max(0, 1 - (now - bump) / BUMP);
      if (bump !== undefined && lift === 0) s.bumps.delete(i);
      moving ||= (since >= 0 && since < GROW) || lift > 0 || (s.started !== null && since < 0);
      at.position.set(b.x, b.y, b.z);
      at.scale.set(b.w, Math.max(b.h * grown * (1 + 0.5 * Math.sin(lift * Math.PI)), 0.0001), b.d);
      at.updateMatrix();
      s.mesh.setMatrixAt(i, at.matrix);
    });
    s.mesh.instanceMatrix.needsUpdate = true;
    // One note for each beat that starts a block.
    if (s.started !== null && !still) {
      const beat = Math.floor((now - s.started) / BEAT);
      for (; s.struck <= beat; s.struck++) {
        const first = s.model.blocks.findIndex((b) => b.beat === s.struck);
        if (first >= 0) strike(s.struck + Math.round(s.model.blocks[first].h));
      }
    }
    return moving;
  };

  const draw = (now: number) => {
    frame = 0;
    const { innerWidth: w, innerHeight: h } = window;
    renderer.setSize(w, h, false);
    renderer.setScissor(0, 0, w, h);
    renderer.clear();
    let again = false;
    for (const s of scenes) {
      const r = s.host.getBoundingClientRect();
      if (r.bottom < 0 || r.top > h || r.width === 0) continue;
      again = place(s, now) || again;
      fit(s, r.width / r.height);
      renderer.setViewport(r.left, h - r.bottom, r.width, r.height);
      renderer.setScissor(r.left, h - r.bottom, r.width, r.height);
      renderer.render(s.world, s.camera);
    }
    if (again) request();
  };
  const request = () => {
    if (!frame) frame = requestAnimationFrame(draw);
  };

  const seen = new IntersectionObserver((entries) => {
    for (const e of entries) {
      const s = scenes.find((x) => x.host === e.target);
      if (s && e.isIntersecting && s.started === null) {
        s.started = performance.now();
        console.info(`iso: ${s.host.dataset.model} grows`);
      }
    }
    request();
  }, { threshold: 0.35 });

  addEventListener("scroll", request, { passive: true });
  addEventListener("resize", request);

  return {
    /** Grows `model` in `host` once it shows; pointing at a block strikes it. */
    add(host: HTMLElement, model: Model) {
      const world = new THREE.Scene();
      world.add(new THREE.HemisphereLight(0xffffff, 0x9a9a9a, 1.6));
      const [w, d] = model.span;
      const sun = new THREE.DirectionalLight(0xffffff, 1.9);
      sun.position.set(w / 2 - 12, 30, d / 2 + 7);
      sun.target.position.set(w / 2, 0, d / 2);
      sun.castShadow = true;
      sun.shadow.mapSize.set(2048, 2048);
      sun.shadow.radius = 4;
      sun.shadow.bias = -0.0006;
      Object.assign(sun.shadow.camera, { left: -w, right: w, top: w, bottom: -w, near: 1, far: 90 });
      world.add(sun, sun.target);
      const mesh = new THREE.InstancedMesh(box, new THREE.MeshLambertMaterial(), model.blocks.length);
      mesh.castShadow = mesh.receiveShadow = true;
      // The table: its grid, and a floor that shows only the shadow the model casts.
      const floor = new THREE.Mesh(new THREE.PlaneGeometry(w * 3, d * 3).rotateX(-Math.PI / 2), new THREE.ShadowMaterial({ opacity: 0.12 }));
      floor.position.set(w / 2, -0.001, d / 2);
      floor.receiveShadow = true;
      const grid = new THREE.LineSegments(table(w, d), new THREE.LineBasicMaterial({ transparent: true, opacity: 0.9 }));
      world.add(mesh, floor, grid);
      const s: Scene = { host, model, world, camera: new THREE.OrthographicCamera(), mesh, grid, started: null, struck: 0, bumps: new Map() };
      scenes.push(s);
      paint();
      place(s, performance.now());
      host.addEventListener("pointermove", (e) => {
        const r = host.getBoundingClientRect();
        ray.setFromCamera(new THREE.Vector2(((e.clientX - r.left) / r.width) * 2 - 1, -((e.clientY - r.top) / r.height) * 2 + 1), s.camera);
        const hit = ray.intersectObject(mesh)[0];
        if (hit?.instanceId === undefined || hit.instanceId === 0 || s.bumps.has(hit.instanceId)) return;
        s.bumps.set(hit.instanceId, performance.now());
        strike(hit.instanceId + Math.round(model.blocks[hit.instanceId].h), 0.5);
        request();
      });
      host.addEventListener("click", () => {
        s.started = performance.now();
        s.struck = 0;
        request();
      });
      seen.observe(host);
    },
    paint: () => (paint(), request()),
    stop() {
      cancelAnimationFrame(frame);
      seen.disconnect();
      removeEventListener("scroll", request);
      removeEventListener("resize", request);
      for (const s of scenes) (s.mesh.material as THREE.Material).dispose();
      renderer.dispose();
      canvas.remove();
    },
  };
}

/** Grid lines a cell apart around the slab, a drafting table under the model. */
function table(w: number, d: number): THREE.BufferGeometry {
  const points: number[] = [];
  const [x0, x1, z0, z1] = [-4, w + 4, -4, d + 4];
  for (let x = x0; x <= x1; x++) points.push(x, 0, z0, x, 0, z1);
  for (let z = z0; z <= z1; z++) points.push(x0, 0, z, x1, 0, z);
  return new THREE.BufferGeometry().setAttribute("position", new THREE.Float32BufferAttribute(points, 3));
}

/** A true isometric view that keeps the model's table and tallest block in the box. */
function fit(s: Scene, aspect: number) {
  const [w, d] = s.model.span;
  const tall = Math.max(...s.model.blocks.map((b) => b.y + b.h));
  const centre = new THREE.Vector3(w / 2, tall / 3, d / 2);
  s.camera.position.copy(centre).add(new THREE.Vector3(1, 1, 1).multiplyScalar(60));
  s.camera.up.set(0, 1, 0);
  s.camera.lookAt(centre);
  // The table's diagonal and the tallest block set the half extents.
  const across = ((w + d) * Math.SQRT1_2) / 2 + 0.6;
  const high = ((w + d) * 0.41) / 2 + tall * 0.82 / 2 + 0.6;
  const half = Math.max(across / aspect, high);
  s.camera.left = -half * aspect;
  s.camera.right = half * aspect;
  s.camera.top = half;
  s.camera.bottom = -half;
  s.camera.near = 0.1;
  s.camera.far = 400;
  s.camera.updateProjectionMatrix();
}
