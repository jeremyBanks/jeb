async function street(volume) {
  PROSE`

# StadiaStreet's Stadia Stats - Volume ${volume}

---

# Data Collection

  CODE`;

  let recordsUrl = query.get("Records") || "./records.json";
  let recordsResponse = await fetch(recordsUrl);
  if (!recordsResponse.ok) throw new Error(recordsResponse);
  let records = Object.values(await recordsResponse.json());

  let Games = records
    .filter((r) => r.type === "game")
    .map(({ imageUrl, gameId, name }) => ({
      gameId,
      imageUrl,
      name: cleanName(name),
    }));
  theSetOf({ Games });
  let gamesById = Object.fromEntries(Games.map((game) => [game.gameId, game]));

  CODE`

This process took place throughout September and October 2020.

A new Google account was created, with a new Stadia profile, with no Stadia
friends, games, or activity history. This account was used for the rest of the
process.

A set of random **Name Prefixes** were generated. Each was between between 2-4
characters, taken from a distribution roughly approximating English letter
frequency, with digits included at a lower frequency.

  CODE`;

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

  CODE`

Each Name Prefix was searched for using the the "Find players" interface in
the Stadia web site, and the results were collected into a set of **Candidate
Players**. Of those Candidate Players, ${(
    (100 * candidateFounders.length) /
    CandidatePlayers.length
  ).toFixed(1)}% were "founders" and ${(
    (100 * candidateSettlers.length) /
    CandidatePlayers.length
  ).toFixed(1)}% were not.

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

  CODE`;

  let PlayersWithGames = CandidatePlayers.filter((p) => p.games !== undefined);
  let roughlyShuffledPlayers = [...PlayersWithGames.entries()]
    .sort(() => Math.random() - 0.5)
    .sort(() => Math.random() - 0.5);
  for (let [i, player] of roughlyShuffledPlayers) {
    let length = String(PlayersWithGames.length).length;
    player.userId = `${volume}${(i + 1).toString().padStart(length, "0")}`;
    if (player.number !== "0000") {
      player.number = Math.floor(Math.random() * 9999 + 1)
        .toString()
        .padStart(4, "0");
    }
    let nameLength = 3 + Math.floor(Math.random() * Math.random() * 13);
    player.name = "";
    for (let i = 0; i < nameLength; i++) {
      if (i > 0) {
        player.name +=
          lettersAndDigits[Math.floor(Math.random() * lettersAndDigits.length)];
      } else {
        player.name += letters[Math.floor(Math.random() * letters.length)];
      }
    }
  }
  let visibleFounders = PlayersWithGames.filter((p) => p.number === "0000");
  let visibleSettlers = PlayersWithGames.filter((p) => p.number !== "0000");

  CODE`

All Candidate Players whose game lists were visible were collected into
${theSetOf({ PlayersWithGames })}. Of those Players With Games,
${visibleFounders.length} (${(
    (100 * visibleFounders.length) /
    PlayersWithGames.length
  ).toFixed(1)}%) were "founders" and ${visibleSettlers.length} (${(
    (100 * visibleSettlers.length) /
    PlayersWithGames.length
  ).toFixed(1)}%) were not.

  CODE`;

  CODE`

# Analysis

## Avatars

### Most Popular

The top ten most popular avatars among ${theSetOf({ PlayersWithGames })} are:

  CODE`;

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
  })).sort((a, b) => b.players.length - a.players.length);

  for (let [i, { imageUrl, name, players }] of popularAvatars
    .slice(0, 10)
    .entries()) {
    PROSE`
${i + 1}. ![**${name}**](${imageUrl}) with ${players.length} players (${(
      (players.length / PlayersWithGames.length) *
      100
    ).toFixed(1)}%)
    PROSE`;
  }

  CODE`

### Least Popular

The bottom ten least popular avatars among ${theSetOf({
    PlayersWithGames,
  })} are:

  CODE`;

  let unpopularAvatars = [...popularAvatars].reverse();

  for (let [i, { imageUrl, name, players }] of unpopularAvatars
    .slice(0, 10)
    .entries()) {
    PROSE`
${i + 1}. ![**${name}**](${imageUrl}) with ${players.length} players (${(
      (players.length / PlayersWithGames.length) *
      100
    ).toFixed(1)}%)
    PROSE`;
  }

  CODE`

## Games

## Most Tried

The top ten most widely-tried games among ${theSetOf({ PlayersWithGames })} are:

1. ![**Destiny 2**](https://lh3.googleusercontent.com/0fwJoLxjhigQ1ScFZ1S27hEOKEPR8HH5Ac6nK2WOI5G0baQMgBjoeAha7zPNWE2-J6Zsm6mKmYOKhmVVndltPdOZTPtT178vBTw-T9VfKCW6Ph_qbnJPwukCgxtq=w640-h360-rw) with 1234 players (12%).

However, this list may be misleading because it doesn't distinguish between
players who have only ever opened a game for a few minutes, and those who have
played it every day for months.

## Most Played

  CODE`;

  let PlayersWithPlaytime = PlayersWithGames.filter((p) =>
    Object.values(p.games).some((g) => g.secondsPlayed)
  );

  CODE`

All Players With Games whose playtimes were visible were collected into
${theSetOf({ PlayersWithPlaytime })}.

The top ten games which the most users have played for at least two hours,
among ${theSetOf({ PlayersWithPlaytime })}, are:

1. ![**Destiny 2**](https://lh3.googleusercontent.com/0fwJoLxjhigQ1ScFZ1S27hEOKEPR8HH5Ac6nK2WOI5G0baQMgBjoeAha7zPNWE2-J6Zsm6mKmYOKhmVVndltPdOZTPtT178vBTw-T9VfKCW6Ph_qbnJPwukCgxtq=w640-h360-rw) with 1234 players (12%).

## Most Playtime

The top ten games with the most total playtime among
${theSetOf({ PlayersWithPlaytime })} are:

1. ![**Destiny 2**](https://lh3.googleusercontent.com/0fwJoLxjhigQ1ScFZ1S27hEOKEPR8HH5Ac6nK2WOI5G0baQMgBjoeAha7zPNWE2-J6Zsm6mKmYOKhmVVndltPdOZTPtT178vBTw-T9VfKCW6Ph_qbnJPwukCgxtq=w640-h360-rw) with 1234 players (12%).


## Most Played Together

The top ten pairs of games for which the most players have played at least two
hours of each among ${theSetOf({ PlayersWithPlaytime })} are:

1. ![**Destiny 2**](https://lh3.googleusercontent.com/0fwJoLxjhigQ1ScFZ1S27hEOKEPR8HH5Ac6nK2WOI5G0baQMgBjoeAha7zPNWE2-J6Zsm6mKmYOKhmVVndltPdOZTPtT178vBTw-T9VfKCW6Ph_qbnJPwukCgxtq=w640-h360-rw) and
![**Destiny 2**](https://lh3.googleusercontent.com/0fwJoLxjhigQ1ScFZ1S27hEOKEPR8HH5Ac6nK2WOI5G0baQMgBjoeAha7zPNWE2-J6Zsm6mKmYOKhmVVndltPdOZTPtT178vBTw-T9VfKCW6Ph_qbnJPwukCgxtq=w640-h360-rw) with 1234 players (12%).

---

All of the data sets were collected into ${theSetOf()}.

  CODE`;
}

let PROSE = (strings, ...values) => {
  let parts = [];
  for (let i = 0; i < strings.length; i++) {
    if (i < values.length) {
      parts.push(strings[i]);
      parts.push(values[i].toString());
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
  let url = query.get(key);
  let value = data[key];
  let values = sorted(JSON.parse(JSON.stringify(value)));

  if (!url) {
    let json = JSON.stringify({ [key]: values }, null, 2);
    url = window.URL.createObjectURL(
      new Blob([json], { type: "application/json;charset=utf-8" })
    );
  }

  if (theSetOf.allSets[key]) {
    return `the set of ${values.length} ${key.replace(/(.)([A-Z])/g, "$1 $2")}`;
  } else {
    theSetOf.allSets[key] = value;
    return `[a set of ${values.length} **${key.replace(
      /(.)([A-Z])/g,
      "$1 $2"
    )}**](${url})`;
  }
};

theSetOf.allSets = {};

let cleanName = (name) =>
  name
    .replace(/^PLAYERUNKNOWN'S BATTLEGROUNDS$/, "PUBG")
    .replace(/&|\+/g, "and")
    .replace(/™/g, "_")
    .replace(/®/g, "_")
    .replace(/[\:\-]? Remake$/g, "_")
    .replace(/[\:\-]? Tamriel Unlimited$/g, "_")
    .replace(/[\:\-]? Early Access$/g, "_")
    .replace(/[\:\-]? \w+ Edition$/g, "_")
    .replace(/\(\w+ Ver(\.|sion)\)$/g, "_")
    .replace(/™/g, "_")
    .replace(/\s{2,}/g, "_")
    .replace(/^\s+|\s+$/g, "")
    .normalize("NFKD")
    .replace(/\p{Mark}/gu, "")
    .replace(/[^a-z0-9A-Z_':]+/g, "_")
    .replace(/(:)([A-Za-z])+/g, "$1_$2")
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
          `st${volume}-${
            link.lastElementChild?.textContent?.replace(/ /g, "") || "data"
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

let volume = 333;
import("./markdown-it.js").finally(() =>
  street(volume).then(() => {
    if (!PROSE.url) return;

    let windows = [];
    window.onbeforeunload = () => {
      windows.forEach((w) => w?.close());
    };
    let w = open(
      PROSE.url,
      `${volume}.md`,
      `location = 0, width = ${Math.floor(
        0.25 * screen.width
      )}, left = ${Math.floor(0.7 * screen.width)}, height = ${Math.floor(
        0.35 * screen.availHeight
      )}, top = ${Math.floor(0.15 * screen.height)} `
    );
    windows.push(w);
    w?.addEventListener("load", () => {
      w.document.title = `⬇️ ${volume}.md`;
      let link = w.document.createElement("a");
      link.href = PROSE.url;
      link.download = "${volume}.json";
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
