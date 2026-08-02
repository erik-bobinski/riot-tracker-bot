// Posts previously rendered embed payloads to the notification channel.
// Usage: tsx --env-file=.env.dev scripts/post-embeds.ts scripts/out/embeds-before.json
import { readFileSync } from "node:fs";

const path = process.argv[2];
if (!path) throw new Error("usage: post-embeds.ts <payload.json>");
const { label, embeds } = JSON.parse(readFileSync(path, "utf8")) as {
  label: string;
  embeds: Record<string, unknown>;
};
const token = process.env["DISCORD_BOT_TOKEN"];
const channel = process.env["NOTIFICATION_CHANNEL_ID"];
if (!token || !channel) throw new Error("missing Discord env vars");

for (const [game, embed] of Object.entries(embeds)) {
  const response = await fetch(
    `https://discord.com/api/v10/channels/${channel}/messages`,
    {
      method: "POST",
      headers: {
        Authorization: `Bot ${token}`,
        "content-type": "application/json",
      },
      body: JSON.stringify({
        content: `\`${label}\` — ${game === "lol" ? "League of Legends" : "Valorant"}`,
        embeds: [embed],
      }),
    },
  );
  if (!response.ok) {
    throw new Error(`${game}: ${response.status} ${await response.text()}`);
  }
  console.log(`posted ${label} ${game}`);
}
