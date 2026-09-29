"""Writes examples/gallery/stories.json: every gallery page and its sections, in the order they draw.

A section is a story the web gallery shows alone at ?page=<page slug>&story=<story slug>. Its
components are the names in its title that are entries in tasks/*.md, so "Sizes" names none.
`--check` fails if the committed file differs.
"""

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PAGES = ROOT / "examples/gallery/pages"
OUT = ROOT / "examples/gallery/stories.json"


def strip(text):
    """Blanks comments and literals' insides, keeping offsets, so braces and calls parse."""
    out, i, n = list(text), 0, len(text)

    def blank(a, b):
        for k in range(a, b):
            if out[k] != "\n":
                out[k] = " "

    while i < n:
        if text.startswith("//", i):
            j = text.find("\n", i)
            j = n if j < 0 else j
            blank(i, j)
            i = j
        elif text.startswith("/*", i):
            j = text.index("*/", i) + 2
            blank(i, j)
            i = j
        elif m := re.match(r'b?r(#*)"', text[i:]):
            end = '"' + m.group(1)
            j = text.index(end, i + m.end()) + len(end)
            blank(i + m.end(), j - len(end))
            i = j
        elif text[i] == '"':
            j = i + 1
            while text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            blank(i + 1, j)
            i = j + 1
        elif m := re.match(r"'(\\.[^']*|[^'\\])'", text[i:]):
            blank(i, i + m.end())
            i += m.end()
        else:
            i += 1
    return "".join(out)


def literal(text, at):
    """The string literal starting at `at`, decoded."""
    m = re.compile(r'"((?:[^"\\]|\\.)*)"', re.S).match(text, at)
    return json.loads('"' + re.sub(r"\\\n\s*", "", m.group(1)) + '"')


def functions(folder):
    """Every `fn` in a page's folder: (module, name) -> (source, stripped, body start, body end)."""
    found = {}
    for file in sorted(folder.rglob("*.rs")):
        module = file.stem
        text = file.read_text()
        bare = strip(text)
        for m in re.finditer(r"\bfn\s+(\w+)\b", bare):
            depth, j = 0, bare.index("{", m.end())
            if ";" in bare[m.end():j]:
                continue
            k = j
            while True:
                depth += {"{": 1, "}": -1}.get(bare[k], 0)
                if depth == 0:
                    break
                k += 1
            found[(module, m.group(1))] = (text, bare, j, k)
    return found


def sections(folder):
    """Section titles in the order the page's render reaches them; fails on any it never reaches."""
    fns = functions(folder)
    order, seen = [], set()

    def walk(module, name):
        if (module, name) in seen:
            return
        seen.add((module, name))
        text, bare, start, end = fns[(module, name)]
        call = re.compile(r"(?<![\w.])(?:(\w+)::)?(\w+)\s*\(")
        for m in call.finditer(bare, start, end):
            owner, callee = m.group(1) or module, m.group(2)
            if callee == "section" and m.group(1) is None:
                at = m.end() + len(bare[m.end():]) - len(bare[m.end():].lstrip())
                if text[at] != '"':
                    sys.exit(f"{folder.name}/{module}.rs: a section whose title is not a literal")
                order.append((literal(text, at), module))
            elif (owner, callee) in fns:
                walk(owner, callee)

    walk("mod", "render")
    reached = {title for title, _ in order}
    for (module, _), (text, bare, start, end) in fns.items():
        for m in re.finditer(r"(?<![\w.])section\s*\(\s*\"", bare[start:end]):
            title = literal(text, start + m.end() - 1)
            if title not in reached:
                sys.exit(f"{folder.name}/{module}.rs: render never reaches section {title!r}")
    return [title for title, _ in order]


def slug(title):
    """As the gallery's ui::slug: ASCII letters and digits, lowercased, dashes between."""
    out = re.sub(r"[^a-z0-9]+", "-", "".join(c.lower() if c.isascii() else " " for c in title))
    return out.strip("-")


def entries():
    """Names the task lists give components, keyed by letters alone."""
    names = {}
    for file in sorted((ROOT / "tasks").glob("*.md")):
        for line in file.read_text().splitlines():
            if m := re.match(r"- \[[ x]\] (.+?)(?: — | → | \(T\d|$)", line):
                for name in m.group(1).split(" / "):
                    names[key(name)] = re.sub(r"\s*\([^)]*\)", "", name).strip()
    return names


def key(name):
    return re.sub(r"[^a-z0-9]", "", re.sub(r"\([^)]*\)", "", name).lower())


def components(title, known):
    """The task lists' names for what the title names: "Icon button" is IconButton."""
    names = []
    for part in re.sub(r"\s*\([^)]*\)", "", title).split(" · "):
        for name in part.split(" → ")[0].split(" / "):
            if key(name) in known and known[key(name)] not in names:
                names.append(known[key(name)])
    return names


def pages():
    listing = (PAGES / "mod.rs").read_text()
    order = re.findall(r"^\s+(\w+)::PAGE,$", listing, re.M)
    known = entries()
    out = []
    for module in order:
        folder = PAGES / module
        head = (folder / "mod.rs").read_text()
        field = lambda name: re.search(rf"^\s+{name}: (.+),$", head, re.M).group(1)
        titles = sections(folder)
        slugs = [slug(title) for title in titles]
        for twice in {s for s in slugs if slugs.count(s) > 1}:
            sys.exit(f"{module}: two stories answer to {twice}")
        out.append(
            {
                "number": int(field("number")),
                "slug": json.loads(field("slug")),
                "title": json.loads(field("title")),
                "summary": json.loads(field("summary")),
                "stories": [
                    {"title": t, "slug": s, "components": components(t, known)}
                    for t, s in zip(titles, slugs)
                ],
            }
        )
    return out


def main():
    rows = [json.dumps(page, ensure_ascii=False) for page in pages()]
    text = '{"pages": [\n' + ",\n".join(rows) + "\n]}\n"
    if sys.argv[1:] == ["--check"]:
        if OUT.read_text() != text:
            sys.exit(f"{OUT.relative_to(ROOT)} is stale: run python3 scripts/stories.py")
    elif sys.argv[1:]:
        sys.exit("usage: python3 scripts/stories.py [--check]")
    else:
        OUT.write_text(text)


if __name__ == "__main__":
    main()
