async function street() {
  PROSE`

# Stadia Street's Stadia Stats: Volume ${volume}

---

  CODE`

  let recordsUrl = query.get("Records") || "../-/records.json";
  let recordsResponse = await fetch(recordsUrl);
  if (!recordsResponse.ok) throw new Error(recordsResponse);
  let records = Object.values(await recordsResponse.json());

  let Games = records
    .filter((r) => r.type === "game")
    .map(({ imageUrl, gameId, name, skuId }) => ({
      gameId,
      skuId,
      imageUrl,
      name: cleanName(name),
    }));
  theSetOf({ Games });
  let gamesById = Object.fromEntries(Games.map((game) => [game.gameId, game]));

  CODE`

  ## Data

  CODE`

  let CandidatePlayers = Object.values(records)
    .filter((r) => r.type === "user")
    .map(({ avatarId, games, name, number, userId, lastActive }) => ({
      userId,
      name,
      number,
      avatarId,
      lastActive,
      games:
        games &&
        Object.fromEntries(
          Object.entries(games).map(
            ([gameId, { achievements, secondsPlayed, lastPlayed }]) => [
              gamesById[gameId].name,
              {
                achievements,
                secondsPlayed,
                lastPlayed,
              },
            ]
          )
        ),
    }));
  let candidateFounders = CandidatePlayers.filter((p) => p.number === "0000");
  let candidateSettlers = CandidatePlayers.filter((p) => p.number !== "0000");

  // let roughlyShuffledPlayers = [...CandidatePlayers.entries()]
  //   .sort(() => Math.random() - 0.5)
  //   .sort(() => Math.random() - 0.5);
  // for (let [i, player] of roughlyShuffledPlayers) {
  //   let length = String(CandidatePlayers.length).length;
  //   player.userId = `${volume}${(i + 1).toString().padStart(length, "0")}`;
  //   if (player.number !== "0000") {
  //     player.number = Math.floor(Math.random() * 9000 + 1000)
  //       .toString()
  //       .padStart(4, "0");
  //   }
  //   let nameLength = 3 + Math.floor(Math.random() * Math.random() * 13);
  //   player.name = "";
  //   for (let i = 0; i < nameLength; i++) {
  //     if (i > 0) {
  //       player.name +=
  //         lettersAndDigits[Math.floor(Math.random() * lettersAndDigits.length)];
  //     } else {
  //       player.name += letters[Math.floor(Math.random() * letters.length)];
  //     }
  //   }
  // }
  let PlayersWithGames = CandidatePlayers.filter((p) => p.games !== undefined);
  let visibleFounders = PlayersWithGames.filter((p) => p.number === "0000");
  let visibleSettlers = PlayersWithGames.filter((p) => p.number !== "0000");

  CODE`

Hmm before we anonymize it, let's take the set of all player numbers, so we can
look for patterns in those.

Discovered Stadia players were were collected into ${theSetOf({ CandidatePlayers })}.
Of those Candidate Players, ${count(candidateFounders.length)} (${(
      (100 * candidateFounders.length) /
      CandidatePlayers.length
    ).toFixed(1)}%) were "founders" and ${count(candidateSettlers.length)} (${(
      (100 * candidateSettlers.length) /
      CandidatePlayers.length
    ).toFixed(1)}%) were not.

For each Candidate Player, their user ID, gamertag name, gamertag number,
avatar, last played time (if visible), and list of game played (if visible)
were collected from their profile.

For each Candidate Players whose profile had a visible list of games, their
achievement count (if visible), playtime (if visible), and last-played time
(if visible) were collected for each game, from that game's detail subpage of
their profile.

For each Candidate Player, their user ID, gamertag name, and gamertag number
were erased and replaced with generated values to maintain privacy, except that
"founder" users' numbers remained unchanged as exclusively "0000".

All Candidate Players whose game lists were visible were collected into
${theSetOf({ PlayersWithGames })}. Of those Players With Games,
${count(visibleFounders.length)} (${(
      (100 * visibleFounders.length) /
      PlayersWithGames.length
    ).toFixed(1)}%) were "founders" and ${count(visibleSettlers.length)} (${(
      (100 * visibleSettlers.length) /
      PlayersWithGames.length
    ).toFixed(1)}%) were not.

  CODE`

  CODE`

---

## Avatars

Avatars ranked by popularity among ${theSetOf({ PlayersWithGames })}:

  CODE`

  let Avatars = records
    .filter((r) => r.type === "avatar")
    .map(({ avatarId, name }) => ({
      avatarId,
      name,
      imageUrl: `https://www.gstatic.com/stadia/gamers/avatars/mdpi/avatar_${avatarId}.png`,
    }));
  theSetOf({ Avatars });
  let playersByAvatarId = Object.fromEntries(
    Avatars.map((a) => [a.avatarId, []])
  );
  for (let player of PlayersWithGames) {
    playersByAvatarId[player.avatarId].push(player);
  }
  let popularAvatars = Avatars.map(({ avatarId, name, imageUrl }) => ({
    avatarId,
    imageUrl,
    name,
    players: playersByAvatarId[avatarId],
  }))
    .sort((a, b) => a.avatarId < b.avatarId ? -1 : +1)
    .sort((a, b) => b.players.length - a.players.length);

  for (let [i, { imageUrl, name, players }] of popularAvatars
    .entries()) {
    PROSE`
${i + 1}. [![](${imageUrl}) **${name}**](${imageUrl}) with ${count(players.length)} players (${(
        (players.length / PlayersWithGames.length) *
        100
      ).toFixed(1)}%)
    PROSE`
  }

  CODE`

---

## Games

  CODE`

  let gameIdsByGameName = Object.fromEntries(
    Games.map(({ gameId, name }) => [name, gameId])
  );
  let tryersByGameId = Object.fromEntries(
    Games.map(({ gameId }) => [gameId, []])
  );

  for (let player of PlayersWithGames) {
    for (let [i, [nameA, infoA]] of Object.entries(player.games).entries()) {
      let idA = gameIdsByGameName[nameA];
      tryersByGameId[idA].push(player);
    }
  }

  const mostTriedGames = Object.entries(tryersByGameId)
    .map(([gameId, players]) => ({
      ...gamesById[gameId],
      players,
    }))
    .sort((a, b) => b.players.length - a.players.length);

  CODE`

### Most Tried

The top most widely-tried games among ${theSetOf({ PlayersWithGames })} were:

  CODE`

  for (let [i, { imageUrl, name, players, skuId, gameId }] of mostTriedGames
    .slice(0, 16)
    .entries()) {
    PROSE`
${i + 1}. [![](${imageUrl}) **${name}**](https://stadia.google.com/readonlystoredetails/${gameId}/sku/${skuId}) with ${count(players.length)} players (${(
        (players.length / PlayersWithGames.length) *
        100
      ).toFixed(1)}%)
    PROSE`
  }

  CODE`

However, this list may be misleading because it doesn't distinguish between
players who have only ever opened a game for a few minutes, and those who have
played it every day for months.

### Most Played

  CODE`

  let PlayersWithPlaytime = PlayersWithGames.filter((p) =>
    Object.values(p.games).some((g) => g.secondsPlayed)
  );

  let significantPlayTime = 2 * 60 * 60;

  let PlayersWithSignificantPlaytime = PlayersWithPlaytime.filter((p) =>
    Object.values(p.games).some((g) => g.secondsPlayed > significantPlayTime)
  );

  let twoHourPlayersByGameId = Object.fromEntries(
    Games.map(({ gameId }) => [gameId, []])
  );
  let pairKey = (a, b) => [a, b].sort().join("-");
  let twoHourPlayersByGameIdPairs = {};
  for (let player of PlayersWithSignificantPlaytime) {
    for (let [i, [nameA, infoA]] of Object.entries(player.games).entries()) {
      let idA = gameIdsByGameName[nameA];
      if (infoA.secondsPlayed && infoA.secondsPlayed > significantPlayTime) {
        twoHourPlayersByGameId[idA].push(player);
        for (let [j, [nameB, infoB]] of Object.entries(
          player.games
        ).entries()) {
          if (j < i && infoB.secondsPlayed > significantPlayTime) {
            let idB = gameIdsByGameName[nameB];
            (twoHourPlayersByGameIdPairs[pairKey(idA, idB)] =
              twoHourPlayersByGameIdPairs[pairKey(idA, idB)] || []).push(
                player
              );
          }
        }
      }
    }
  }

  const mostTwoHourPlayedGames = Object.entries(twoHourPlayersByGameId)
    .map(([gameId, players]) => ({
      ...gamesById[gameId],
      players,
    }))
    .sort((a, b) => b.players.length - a.players.length);

  const mostTwoHourPlayedGamePairs = Object.entries(twoHourPlayersByGameIdPairs)
    .map(([gameIds, players]) => {
      let [idA, idB] = gameIds.split(/-/);
      let gameA = gamesById[idA];
      let gameB = gamesById[idB];
      if (
        twoHourPlayersByGameId[idA].length < twoHourPlayersByGameId[idB].length
      ) {
        let t = gameA;
        gameA = gameB;
        gameB = t;
      }
      return {
        gameA,
        gameB,
        players,
      };
    })
    .sort((a, b) => b.players.length - a.players.length);

  CODE`

All Players With Games whose playtimes were visible were collected into
${theSetOf({ PlayersWithPlaytime })}. Of those, players who had a
playtime of at least two hours in any game were collected into
${theSetOf({ PlayersWithSignificantPlaytime })}.

The top games which the most users have played for at least two hours,
among ${theSetOf({ PlayersWithSignificantPlaytime })}, were:

  CODE`

  for (let [i, { imageUrl, name, players, skuId, gameId }] of mostTwoHourPlayedGames
    .slice(0, 16)
    .entries()) {
    PROSE`
  ${i + 1}. [![](${imageUrl}) **${name}**](https://stadia.google.com/readonlystoredetails/${gameId}/sku/${skuId}) with ${count(players.length)} players (${(
        (players.length / PlayersWithSignificantPlaytime.length) *
        100
      ).toFixed(1)}%)
    PROSE`
  }

  CODE`

### Most Total Playtime

The top games by total playtime across
${theSetOf({ PlayersWithPlaytime })} are:

  CODE`

  let totalSeconds = 0;
  let totalSecondsByGameId = {};

  for (const [i, player] of PlayersWithPlaytime.entries()) {
    for (let [i, [name, info]] of Object.entries(player.games).entries()) {
      if (info.secondsPlayed) {
        let id = gameIdsByGameName[name];
        totalSecondsByGameId[id] ??= 0;
        totalSecondsByGameId[id] += info.secondsPlayed;
        totalSeconds += info.secondsPlayed;
      }
    }
  }

  for (const [i, [gameId, seconds]] of Object.entries(
    totalSecondsByGameId).sort((a, b) => b[1] - a[1]).slice(0, 16).entries()
  ) {
    let { imageUrl, skuId, name } = gamesById[gameId];
    PROSE`
${i + 1}. [![](${imageUrl}) **${name}**](https://stadia.google.com/readonlystoredetails/${gameId}/sku/${skuId}) with ${count(Math.floor(seconds / 60 / 60))} hours (${(
        (seconds / totalSeconds) *
        100
      ).toFixed(1)}%)
    PROSE`
  }

  CODE`

### Most Played Together

The top pairs of games for which the most players have played at least two
hours of each among ${theSetOf({ PlayersWithSignificantPlaytime })} were:

  CODE`

  for (let [i, { gameA, gameB, players }] of mostTwoHourPlayedGamePairs
    .slice(0, 16)
    .entries()) {
    PROSE`
  ${i + 1}. [![](${gameA.imageUrl}) **${gameA.name}**](https://stadia.google.com/readonlystoredetails/${gameA.gameId}/sku/${gameA.skuId}) and [![](${gameB.imageUrl
      }) **${gameB.name}**](https://stadia.google.com/readonlystoredetails/${gameB.gameId}/sku/${gameB.skuId}) with ${count(players.length)} players in common (${(
        (players.length / PlayersWithSignificantPlaytime.length) *
        100
      ).toFixed(1)}%)
    PROSE`
  }

  CODE`

---

*Stadia Street's Stadia Stats: Volume ${volume}* was generated by [this
script](https://gist.githubusercontent.com/StadiaStreet/${gist}/raw/${volume}.js)
and ${theSetOf()}.

  CODE`
}

let PROSE = (strings, ...values) => {
  let parts = [];
  for (let i = 0; i < strings.length; i++) {
    if (i < values.length) {
      parts.push(strings[i]);
      try {
        parts.push(values[i].toString());
      } catch (error) {
        console.error(error);
        debugger;
      }
    } else {
      parts.push(strings[i].replace(/\n *[A-Z]+$/, ""));
    }
  }

  PROSE.markdown += parts.join("");

  if (PROSE.url) {
    URL.revokeObjectURL(PROSE.url);
    PROSE.url = null;
  }

  PROSE.url = window.URL.createObjectURL(
    new Blob([PROSE.markdown], { type: "text/plain;charset=utf-8" })
  );

  try {
    document.body.innerHTML = markdownit().render(PROSE.markdown);
    document.body.style.whiteSpace = "";
  } catch (error) {
    console.error(error);
    document.body.textContent = PROSE.markdown;
    document.body.style.whiteSpace = "pre-wrap";
  }
};
PROSE.markdown = "";
PROSE.url = null;

let CODE = PROSE;

let sorted = (x) => {
  if (Array.isArray(x)) {
    return x.map(sorted).sort((a, b) => {
      a = JSON.stringify(a);
      b = JSON.stringify(b);
      if (a < b) {
        return -1;
      } else if (a > b) {
        return +1;
      } else {
        return 0;
      }
    });
  } else if (x && typeof x === "object") {
    let y = {};
    for (let key of Object.keys(x).sort((a, b) => {
      if (a < b) {
        return -1;
      } else if (a > b) {
        return +1;
      } else {
        return 0;
      }
    })) {
      y[key] = sorted(x[key]);
    }
    return y;
  } else {
    return x;
  }
};

let theSetOf = (
  data = {
    DataSets: Object.entries(theSetOf.allSets).map(([k, v]) => ({ [k]: v })),
  }
) => {
  if (Object.keys(data).length !== 1) {
    throw new TypeError("theSetOf what?");
  }
  let key = Object.keys(data)[0];
  let url;
  if (query.get("env") !== "dev") {
    url = `https://gist.githubusercontent.com/StadiaStreet/${gist}/raw/st${volume}-${key}.json`;
  } else {
    url = query.get(key);
  }

  let value = data[key];
  let values = sorted(JSON.parse(JSON.stringify(value)));

  if (!url) {
    let json = JSON.stringify({ [key]: values }, null, 2);
    url = window.URL.createObjectURL(
      new Blob([json], { type: "application/json;charset=utf-8" })
    );
  }

  if (theSetOf.allSets[key]) {
    return `the set of ${count(values.length)} ${key.replace(/(.)([A-Z])/g, "$1 $2")}`;
  } else {
    theSetOf.allSets[key] = value;
    return `[a set of ${count(values.length)} **${key.replace(
      /(.)([A-Z])/g,
      "$1 $2"
    )}**](${url})`;
  }
};

theSetOf.allSets = {};

let cleanName = (name) =>
  name
    .replace(/^PLAYERUNKNOWN'S BATTLEGROUNDS$/, "PUBG")
    .replace(/^Tom Clancy's/, "")
    .replace(/^Zombie Army 4: Dead War$/, "Zombie Army 4")
    .replace(/^HITMAN - World of Assassination$/, "Hitman")
    .replace(/^Red Dead Redemption 2$/, "RDR2")
    .replace(/^The Elder Scrolls Online: Tamriel Unlimited$/, "ESO")
    .replace(/&|\+/g, "and")
    .replace(/™/g, "_")
    .replace(/®/g, "_")
    .replace(/[\:\-]? Remake$/g, "_")
    .replace(/[\:\-]? Early Access$/g, "_")
    .replace(/[\:\-]? \w+ Edition$/g, "_")
    .replace(/\(\w+ Ver(\.|sion)\)$/g, "_")
    .replace(/™/g, "_")
    .replace(/\s{2,}/g, "_")
    .replace(/^\s+|\s+$/g, "")
    .normalize("NFKD")
    .replace(/\p{Mark}/gu, "")
    .replace(/[^a-z0-9A-Z_':]+/g, "_")
    .replace(/(:)([A-Za-z]+)/g, "$1_$2")
    .replace(/^\_+|\_+$/g, "")
    .replace(/_/g, " ")
    .replace(/[ ]+/g, " ")
    .trim();

let query = new URL(location).searchParams;

document.addEventListener(
  "click",
  (event) => {
    let link = event?.target?.closest("a");

    if (link?.href?.startsWith("blob:")) {
      if (
        event.ctrlKey ||
        event.altKey ||
        event.ctrlKey ||
        event.metaKey ||
        event.shiftKey ||
        event.button !== 0
      ) {
        link.removeAttribute("download");
      } else {
        link.setAttribute(
          "download",
          `st${volume}-${link.lastElementChild?.textContent?.replace(/ /g, "") || "data"
          }.json`
        );
      }
    }
  },
  {
    capture: true,
  }
);

let letters = `\
eeeeeeeeeeeetttttttttaaaaaaaaoooooooiiiiiinnnnnnssssssrrrrrrhhhhhddddll\
luuccmmffyywwggpbvkxqjzeeeeeeeeeeeetttttttttaaaaaaaaoooooooiiiiiinnnnnns\
sssssrrrrrrhhhhhddddllluuccmmffyywwggpbvkxqjz`;
let lettersAndDigits = `${letters}0123456789`;

let volume = 360;
let gist = "0e378320a54c4059c0ce846c0a00c5d9";

let count = n => new Intl.NumberFormat().format(n);

import("./markdown-it.js").finally(() =>
  street(volume).then(() => {
    if (!PROSE.url) return;

    let windows = [];
    window.onbeforeunload = () => {
      windows.forEach((w) => w?.close());
    };
    let filename = `st${volume}.md`;
    let w = open(
      PROSE.url,
      filename,
      `location = 0, width = ${Math.floor(
        0.25 * screen.width
      )}, left = ${Math.floor(0.7 * screen.width)}, height = ${Math.floor(
        0.35 * screen.availHeight
      )}, top = ${Math.floor(0.15 * screen.height)} `
    );
    windows.push(w);
    w?.addEventListener("load", () => {
      w.document.title = `⬇️ ${filename}`;
      let link = w.document.createElement("a");
      link.href = PROSE.url;
      link.download = filename;
      link.title = "click to download";
      Object.assign(link.style, {
        textDecoration: "none",
        padding: "4px 12px",
        display: "block",
        position: "absolute",
        top: "0",
        bottom: "0",
        left: "0",
        right: "0",
        background: "#246",
        fontWeight: "bold",
        color: "#FFC",
        userSelect: "none",
        cursor: "copy",
        overflow: "auto",
        fontSize: "8px",
      });
      link.appendChild(w.document.body.firstElementChild);
      w.document.body.appendChild(link);
    });
  })
);
