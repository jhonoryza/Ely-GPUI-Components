import { Marked, type Tokens } from "marked";

const sources = import.meta.glob<string>("./*.md", { query: "?raw", import: "default", eager: true });
const examples = import.meta.glob<string>("../../../examples/docs/*.rs", { query: "?raw", import: "default", eager: true });

/** The guides in reading order, by file name. */
const ORDER = ["introduction"];

export interface Guide {
  /** The path segment; the introduction has none. */
  slug: string;
  title: string;
  html: string;
  /** The `##` headings, for the on-this-page list. */
  toc: { id: string; text: string }[];
}

const escape = (text: string) => text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

const slugify = (text: string) =>
  text
    .toLowerCase()
    .replace(/<[^>]+>/g, "")
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");

function read(file: string): Guide {
  const source = sources[`./${file}.md`];
  if (source === undefined) throw new Error(`docs: no ${file}.md`);
  let title = "";
  const toc: Guide["toc"] = [];
  const marked = new Marked({
    renderer: {
      heading({ tokens, depth }: Tokens.Heading) {
        const html = this.parser.parseInline(tokens);
        if (depth === 1) {
          title = html.replace(/<[^>]+>/g, "");
          return "";
        }
        const id = slugify(html);
        if (depth === 2) {
          if (toc.some((h) => h.id === id)) throw new Error(`docs: ${file}.md repeats heading ${id}`);
          toc.push({ id, text: html.replace(/<[^>]+>/g, "") });
        }
        return `<h${depth} id="${id}">${html}</h${depth}>`;
      },
      code({ text, lang }: Tokens.Code) {
        const [language = "", ...rest] = (lang ?? "").split(/\s+/);
        const example = rest.find((part) => part.startsWith("example="))?.slice("example=".length);
        let code = text;
        let caption = language;
        if (example) {
          const body = examples[`../../../examples/docs/${example}.rs`];
          if (body === undefined) throw new Error(`docs: ${file}.md names a missing example ${example}`);
          code = body.trimEnd();
          caption = `examples/docs/${example}.rs · cargo run --example docs_${example}`;
        }
        return `<figure class="code"><figcaption>${escape(caption)}</figcaption><button class="copy" type="button">Copy</button><pre><code>${escape(code)}</code></pre></figure>`;
      },
    },
  });
  const html = marked.parse(source, { async: false });
  if (!title) throw new Error(`docs: ${file}.md has no # title`);
  return { slug: file === "introduction" ? "" : file, title, html, toc };
}

export const guides: Guide[] = ORDER.map(read);
