// Builds a Discord-dark-theme HTML replica of rendered embed payloads so
// before/after screenshots can be captured without a Discord client.
// Usage: tsx scripts/build-preview.ts scripts/out/embeds-before.json
import { readFileSync, writeFileSync } from "node:fs";
import { basename } from "node:path";

const path = process.argv[2];
if (!path) throw new Error("usage: build-preview.ts <payload.json>");
const { label, embeds } = JSON.parse(readFileSync(path, "utf8")) as {
  label: string;
  embeds: Record<
    string,
    {
      title: string;
      description: string;
      color: number;
      thumbnail?: { url: string };
      fields?: ReadonlyArray<{ name: string; value: string; inline?: boolean }>;
    }
  >;
};

const escapeHtml = (value: string) =>
  value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");

const markdown = (value: string) =>
  escapeHtml(value)
    .replace(
      /&lt;:(\w+):(\d+)&gt;/g,
      '<img class="emoji" src="https://cdn.discordapp.com/emojis/$2.png?size=48" alt="$1">',
    )
    .replace(/&lt;t:(\d+):t&gt;/g, '<span class="ts">6:35 PM</span>')
    .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
    .replaceAll("\n", "<br>");

const fieldRows = (
  fields: ReadonlyArray<{ name: string; value: string; inline?: boolean }>,
) => {
  const rows: Array<Array<(typeof fields)[number]>> = [];
  for (const field of fields) {
    const last = rows.at(-1);
    if (field.inline && last && last.length < 3 && last[0]?.inline) {
      last.push(field);
    } else {
      rows.push([field]);
    }
  }
  return rows;
};

const embedHtml = (game: string, embed: (typeof embeds)[string]) => `
  <div class="message">
    <img class="avatar" src="https://cdn.discordapp.com/embed/avatars/0.png" alt="">
    <div>
      <div class="header"><span class="bot">riot-tracker-dev</span><span class="badge">APP</span><span class="when">6:35 PM</span></div>
      <div class="content"><code>${escapeHtml(label)}</code> — ${game === "lol" ? "League of Legends" : "Valorant"}</div>
      <div class="embed" style="border-left-color:#${embed.color.toString(16).padStart(6, "0")}">
        <div class="embed-grid">
          <div>
            <div class="title">${markdown(embed.title)}</div>
            <div class="desc">${markdown(embed.description)}</div>
            ${
              embed.fields
                ? fieldRows(embed.fields)
                    .map(
                      (row) =>
                        `<div class="fields cols-${row.length}">${row
                          .map(
                            (field) =>
                              `<div><div class="fname">${markdown(field.name)}</div><div class="fvalue">${markdown(field.value)}</div></div>`,
                          )
                          .join("")}</div>`,
                    )
                    .join("")
                : ""
            }
          </div>
          ${embed.thumbnail ? `<img class="thumb" src="${embed.thumbnail.url}" alt="">` : ""}
        </div>
      </div>
    </div>
  </div>`;

const html = `<!doctype html>
<html><head><meta charset="utf-8"><style>
  body { background: #313338; margin: 0; padding: 24px; width: 640px;
    font-family: "gg sans", "Segoe UI", "Helvetica Neue", sans-serif;
    color: #dbdee1; font-size: 14px; line-height: 1.35; }
  .message { display: grid; grid-template-columns: 40px 1fr; gap: 16px; margin-bottom: 28px; }
  .avatar { width: 40px; height: 40px; border-radius: 50%; }
  .header { display: flex; align-items: center; gap: 6px; margin-bottom: 2px; }
  .bot { font-weight: 600; color: #f2f3f5; }
  .badge { background: #5865f2; color: #fff; font-size: 10px; font-weight: 600;
    border-radius: 4px; padding: 1px 5px; }
  .when { color: #949ba4; font-size: 12px; }
  .content { margin-bottom: 6px; }
  .content code { background: #1e1f22; border-radius: 4px; padding: 1px 4px; font-size: 13px; }
  .embed { background: #2b2d31; border-left: 4px solid; border-radius: 4px;
    padding: 12px 16px 14px 14px; max-width: 520px; }
  .embed-grid { display: grid; grid-template-columns: 1fr auto; gap: 16px; }
  .title { font-weight: 700; color: #f2f3f5; font-size: 16px; margin-bottom: 8px; }
  .desc { white-space: normal; }
  .thumb { width: 80px; height: 80px; border-radius: 8px; object-fit: cover; }
  .fields { display: grid; gap: 8px; margin-top: 10px; }
  .cols-1 { grid-template-columns: 1fr; }
  .cols-2 { grid-template-columns: 1fr 1fr; }
  .cols-3 { grid-template-columns: 1.6fr 0.8fr 1.2fr; }
  .fname { font-weight: 700; color: #f2f3f5; font-size: 13px; margin-bottom: 3px; min-height: 15px; }
  .fvalue { color: #dbdee1; }
  .emoji { width: 20px; height: 20px; vertical-align: -5px; }
  .ts { background: #3f4248; border-radius: 3px; padding: 0 2px; }
</style></head><body>
${Object.entries(embeds)
  .map(([game, embed]) => embedHtml(game, embed))
  .join("\n")}
</body></html>`;

const out = path.replace(/\.json$/, ".html");
writeFileSync(out, html);
console.log(`wrote ${out} from ${basename(path)}`);
