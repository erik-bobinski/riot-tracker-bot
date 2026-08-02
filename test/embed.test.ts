import { describe, expect, it } from "vitest";
import { matchEmbed, rankEmbed } from "../src/services/discord/embed.ts";
import { EpochMillis, MatchId, Puuid } from "../src/services/game/index.ts";

describe("match embeds", () => {
  it("renders valid UTF-8 without mojibake", () => {
    const embed = matchEmbed(
      {
        discordNames: ["One", "Two"],
        trackedPuuids: ["one"],
        match: {
          matchId: MatchId.make("match"),
          game: "lol",
          date: EpochMillis.make(1_000),
          mode: "Ranked Solo/Duo",
          map: "Summoner's Rift",
          durationSeconds: 1_234,
          surrendered: false,
          players: [
            {
              puuid: Puuid.make("one"),
              team: "100",
              riotName: "Mock",
              riotTag: "NA1",
              character: "Ahri",
              kills: 20,
              deaths: 1,
              assists: 9,
              stat: "200 CS · 30k dmg",
              sortKey: 29,
              flair: "Penta Kill",
            },
          ],
          teams: [{ id: "100", won: true, score: [13, 8] }],
        },
      },
      {},
    );
    const rendered = `${embed.title}\n${embed.description}\n${JSON.stringify(embed.fields)}`;
    expect(rendered).toContain("Victory — Ranked Solo/Duo · Summoner's Rift");
    expect(rendered).toContain("13–8");
    expect(embed.description).toContain("**Penta Kill** — Mock#NA1");
    expect(embed.fields).toEqual([
      { name: "Player", value: "**Mock#NA1** (Ahri)", inline: true },
      { name: "K/D/A", value: "20/1/9", inline: true },
      { name: "Stats", value: "200 CS · 30k dmg", inline: true },
    ]);
    expect(rendered).not.toMatch(/Â|â€|ðŸ/);
  });
  it("renders each team as an aligned three-column table", () => {
    const player = (overrides: {
      puuid: string;
      team: string;
      riotName: string;
      kills: number;
      sortKey: number;
    }) => ({
      puuid: Puuid.make(overrides.puuid),
      team: overrides.team,
      riotName: overrides.riotName,
      riotTag: "NA1",
      character: "Ahri",
      kills: overrides.kills,
      deaths: 2,
      assists: 3,
      stat: "100 CS · 10k dmg",
      sortKey: overrides.sortKey,
    });
    const embed = matchEmbed(
      {
        discordNames: ["One"],
        trackedPuuids: ["one"],
        match: {
          matchId: MatchId.make("two-teams"),
          game: "lol",
          date: EpochMillis.make(1_000),
          mode: "Ranked Solo/Duo",
          durationSeconds: 1_234,
          surrendered: false,
          players: [
            player({
              puuid: "ally-low",
              team: "100",
              riotName: "AllyLow",
              kills: 1,
              sortKey: 1,
            }),
            player({
              puuid: "one",
              team: "100",
              riotName: "Mock",
              kills: 20,
              sortKey: 29,
            }),
            player({
              puuid: "enemy-long",
              team: "200",
              riotName: "AVeryLongRiotNameIndeed",
              kills: 5,
              sortKey: 5,
            }),
          ],
          teams: [
            { id: "100", won: true },
            { id: "200", won: false },
          ],
        },
      },
      {},
    );
    const fields = embed.fields ?? [];
    expect(fields).toHaveLength(6);
    expect(fields.slice(0, 3).map((field) => field.name)).toEqual([
      "Player",
      "K/D/A",
      "Stats",
    ]);
    // Second team continues the table without repeating headers
    expect(fields.slice(3).map((field) => field.name)).toEqual([
      "​",
      "​",
      "​",
    ]);
    // Rows sort by sortKey and stay line-aligned across the three columns
    expect(fields[0]?.value).toBe("**Mock#NA1** (Ahri)\nAllyLow#NA1 (Ahri)");
    expect(fields[1]?.value).toBe("20/2/3\n1/2/3");
    expect(fields[2]?.value).toBe("100 CS · 10k dmg\n100 CS · 10k dmg");
    // Long names truncate so cells never wrap and desync the columns
    expect(fields[3]?.value).toBe("AVeryLongRiotName… (Ahri)");
    expect(fields.every((field) => field.inline)).toBe(true);
  });
  it("uses rank emojis without redundant rank labels in leaderboards", () => {
    const league = matchEmbed(
      {
        discordNames: ["One"],
        trackedPuuids: ["one"],
        match: {
          matchId: MatchId.make("league-rank"),
          game: "lol",
          date: EpochMillis.make(1_000),
          mode: "Ranked",
          durationSeconds: 1_234,
          surrendered: false,
          players: [
            {
              puuid: Puuid.make("one"),
              team: "100",
              riotName: "Mock",
              riotTag: "NA1",
              character: "Ahri",
              kills: 20,
              deaths: 1,
              assists: 9,
              stat: "200 CS",
              rank: "Diamond II",
              rankIconKey: "diamond",
              sortKey: 29,
            },
          ],
          teams: [{ id: "100", won: true }],
        },
      },
      { "lol.diamond": "<:diamond:1>" },
    );
    expect(league.fields?.[0]?.value).toContain("<:diamond:1> II **Mock#NA1**");
    expect(JSON.stringify(league)).not.toContain("Diamond II");

    const valorant = matchEmbed(
      {
        discordNames: ["One"],
        trackedPuuids: ["one"],
        match: {
          matchId: MatchId.make("valorant-rank"),
          game: "valorant",
          date: EpochMillis.make(1_000),
          mode: "Competitive",
          durationSeconds: 1_234,
          surrendered: false,
          players: [
            {
              puuid: Puuid.make("one"),
              team: "100",
              riotName: "Mock",
              riotTag: "NA1",
              character: "Jett",
              kills: 20,
              deaths: 1,
              assists: 9,
              stat: "285 ACS",
              rank: "Diamond 1",
              rankIconKey: "diamond_1",
              sortKey: 29,
            },
          ],
          teams: [{ id: "100", won: true }],
        },
      },
      { "valorant.diamond_1": "<:diamond_1:2>" },
    );
    expect(valorant.fields?.[0]?.value).toContain(
      "<:diamond_1:2> **Mock#NA1**",
    );
    expect(JSON.stringify(valorant)).not.toContain("Diamond 1");
  });
  it("renders compact game-specific rank embeds", () => {
    const valorant = rankEmbed({
      _tag: "Ranks",
      discordName: "syanx_",
      game: "valorant",
      ranks: [
        {
          label: "Diamond 3",
          pointsLabel: "73 RR",
          rankIconKey: "diamond_3",
        },
      ],
      iconUrl: "https://example.test/diamond-3.png",
    });
    expect(valorant).toMatchObject({
      title: "syanx_'s Valorant Rank",
      description: "**Diamond 3** \u00b7 73 RR",
      color: 0xff4655,
      thumbnail: { url: "https://example.test/diamond-3.png" },
    });

    const league = rankEmbed({
      _tag: "Ranks",
      discordName: "syanx_",
      game: "lol",
      ranks: [
        {
          label: "Diamond II",
          queueLabel: "Ranked Solo/Duo",
          pointsLabel: "64 LP",
        },
        {
          label: "Platinum I",
          queueLabel: "Ranked Flex",
          pointsLabel: "21 LP",
        },
      ],
      iconUrl: "https://example.test/diamond.png",
    });
    expect(league.title).toBe("syanx_'s League Rank");
    expect(league.description).toContain(
      "**Diamond II** \u00b7 64 LP (Solo/Duo)",
    );
    expect(league.description).toContain("**Platinum I** \u00b7 21 LP (Flex)");
    expect(league.color).toBe(0x0ac8b9);
    expect(league.image).toEqual({
      url: "https://example.test/diamond.png",
    });
    expect(league.thumbnail).toBeUndefined();
  });
});
