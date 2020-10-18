async function street() {
  PROSE`

# Game Popularity Data and Analysis

---

# Data Collection

  CODE`

  let recordsUrl = query.get("Records") || './records.json';
  let recordsResponse = await fetch(recordsUrl);
  if (!recordsResponse.ok) throw new Error(recordsResponse);
  let records = Object.values(await recordsResponse.json());

  let gameNames = Object.fromEntries(records
    .filter(r => r.type === 'game')
    .map(game => [game.gameId, cleanName(game.name)]));

  CODE`

This process took place throughout September and October 2020.

A new Google account was created, with a new Stadia profile, with no Stadia
friends, games, or activity history. This account was used for the rest of the
process.

A set of random **Name Prefixes** were generated. Each was between between 2-4
characters, taken from a distribution roughly approximating English letter
frequency, with digits included at a lower frequency.

  CODE`

  let CandidatePlayers =
    Object.values(records)
      .filter(r => r.type === 'user')
      .map(({ avatarId, games, name, number, userId, lastActive }) => ({
        userId,
        name,
        number,
        avatarId,
        lastActive,
        games: games && Object.fromEntries(Object.entries(games)
          .map(([gameId, { achievements,
            secondsPlayed,
            lastPlayed, }]) => [gameNames[gameId], {
              achievements,
              secondsPlayed,
              lastPlayed,
            }]))
      }));
  let candidateFounders = CandidatePlayers.filter(p => p.number === '0000');
  let candidateSettlers = CandidatePlayers.filter(p => p.number !== '0000');

  CODE`

Each Name Prefix was searched for using the the "Find players" interface in
the Stadia web site, and the results were collected into a set of **Candidate
Players**. Of those Candidate Players, ${(
      100 * candidateFounders.length / CandidatePlayers.length).toFixed(1)
    }% were "founders" and ${(
      100 * candidateSettlers.length / CandidatePlayers.length).toFixed(1)
    }% were not.

For each Candidate Player, their user ID, gamertag name, gamertag number,
avatar, last played time (if visible), and (if visible) list of game played
were collected from their profile.

For each Candidate Players whose profile had a visible list of games, their
achievement count (if visible), playtime (if visible), and last-played time
(if visible) were collected for each game, from that game's detail subpage of
their profile.

  CODE`
  for (let player of CandidatePlayers) {
    player.userId =
      Math.floor(Math.random() * Number.MAX_SAFE_INTEGER).toString(10);
    if (player.number !== '0000') {
      player.number =
        Math.floor(Math.random() * 9999 + 1).toString().padStart(4, '0');
    }
    player.name =
      Math.random()
        .toString(36)
        .replace(/^[^A-Za-z]+/, '')
        .slice(0, 3 + Math.floor(13 * Math.random(), 2));
  }

  CODE`

For each Candidate Player, their user ID, gamertag name, and gamertag number
were erased and replaced with randomly-generated values to maintain privacy,
except that "founder" users' numbers remained unchanged as exclusively "0000".

  CODE`

  let VisiblePlayers = CandidatePlayers.filter(p => p.games !== undefined);
  let visibleFounders = VisiblePlayers.filter(p => p.number === "0000");
  let visibleSettlers = VisiblePlayers.filter(p => p.number !== "0000");

  CODE`

All Candidate Players whose game lists were visible were collected into
${theSetOf({ VisiblePlayers })}. Of those Visible Players,
${visibleFounders.length
    } (${(100 * visibleFounders.length / VisiblePlayers.length).toFixed(1)
    }%) were "founders" and ${visibleSettlers.length
    } (${(100 * visibleSettlers.length / VisiblePlayers.length).toFixed(1)
    }%) were not.

  CODE`

  let minMonthlyTimestamp = Date.now() - 32 * 24 * 60 * 60 * 1000;
  let MonthlyActivePlayers = VisiblePlayers.filter(
    p => (p.lastActive && p.lastActive >= minMonthlyTimestamp) || (
      p.games && Object.values(p.games).some(g => g.lastPlayed >= minMonthlyTimestamp)
    ));

  CODE`

# Analysis

## Avatars

## Most Popular

The top ten most popular avatars among ${theSetOf({ VisiblePlayers })} are:

  CODE`

  let avatars = records.filter(r => r.type === 'avatar');
  let playersByAvatarId =
    Object.fromEntries(avatars.map(a => [a.avatarId, []]));
  for (let player of VisiblePlayers) {
    playersByAvatarId[player.avatarId].push(player);
  }
  let popularAvatars = avatars.map(({ avatarId, name }) => ({
    avatarId,
    name,
    players: playersByAvatarId[avatarId],
  })).sort((a, b) => b.players.length - a.players.length);
  console.debug({ avatars, popularAvatars });

  for (const { avatarId, name, players } of popularAvatars.slice(0, 10)) {
    PROSE`
1. ![${name
      }](https://www.gstatic.com/stadia/gamers/avatars/mdpi/avatar_${avatarId
      }.png) with ${players.length} players (${(
        players.length / VisiblePlayers.length * 100).toFixed(1)
      }%)
    CODE`
  }

  CODE`

### Least Popular

The bottom ten least popular avatars among ${theSetOf({ VisiblePlayers })} are:

  CODE`

  const unpopularAvatars = [...popularAvatars].reverse();

  for (const { avatarId, name, players } of unpopularAvatars.slice(0, 10)) {
    PROSE`
1. ![${name
      }](https://www.gstatic.com/stadia/gamers/avatars/mdpi/avatar_${avatarId
      }.png) with ${players.length} players (${(
        players.length / VisiblePlayers.length * 100).toFixed(1)
      }%)
    CODE`
  }

  CODE`

From ${theSetOf({ VisiblePlayers })}, those whose last-online timestamp were
visible and within the last 32 days were collected into
${theSetOf({ MonthlyActivePlayers })}.

## Most-Tried Games

## Top-Playtime Games

All of the data sets linked above were collected into ${theSetOf()}.
`
  /*
    let games = Object.fromEntries(
      records.flatMap((record) =>
        record.type === "game" && record.gameId ? [[record.gameId, record]] : []
      )
    );

    let avatars = Object.fromEntries(
      records.flatMap((record) =>
        record.type === "avatar" && record.avatarId
          ? [[record.avatarId, record]]
          : []
      )
    );

    let players = records.filter(
      (record) => record.type === "user" && record.userId
    );



    let interestingHours = 2;
    let interestingSeconds = interestingHours * 60 * 60;

    let yearDays = 365;
    let yearSeconds = yearDays * 24 * 60 * 60;
    let minYearlyTimestamp = Date.now() - yearSeconds * 1000;


    let prefixed = (separator, id) =>
      `${prefix}${separator}${id.toString().padStart(7, "0")}`;

    let jsonStyles = {
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
    };

    let windows = [];
    window.onbeforeunload = () => {
      windows.forEach(w => w?.close());
    };

    let monthlyActivePlayers = players.filter(
      (player) =>
        (player.lastActive && player.lastActive >= minMonthlyTimestamp) ||
        (player.games &&
          Object.values(player.games).some((g) => {
            return g.lastPlayed >= minMonthlyTimestamp;
          }))
    );

    console.debug("users known to be active in the last month", {
      monthlyActivePlayers,
    });

    let yearlyActivePlayers = players.filter(
      (player) =>
        (player.lastActive && player.lastActive >= minYearlyTimestamp) ||
        (player.games &&
          Object.values(player.games).some(
            (g) => g.lastPlayed >= minYearlyTimestamp
          ))
    );

    console.debug("users known to be active in the last year", {
      yearlyActivePlayers,
    });

    let gameListVisiblePlayers = players.filter(
      (player) => player.gameIds
    );

    console.debug("users whose game lists are visible", {
      gameListVisiblePlayers,
    });

    let gameListCapturedPlayers = players.filter((player) => player.games);

    console.debug("users whose game lists have been fully captured", {
      gameListCapturedPlayers,
    });

    let candidatePlayers = [...players.entries()].flatMap(([i, record]) => {
      if (record.type !== "user") return [];
      return [
        {
          ...record,
        },
      ];
    });

    candidatePlayers // ha ha ha
      .sort((a, b) => Math.random() - 0.5)
      .sort((a, b) => Math.random() - 0.5)
      .sort((a, b) => Math.random() - 0.5)
      .sort((a, b) => Math.random() - 0.5);

    console.debug("all known players", { candidatePlayers });

    let visiblePlayers = [...candidatePlayers].flatMap((player) => {
      let visible = false;

      let playerGameStats = {};
      for (let [gameId, stats] of Object.entries(player.games || {})) {
        if (stats.secondsPlayed) {
          visible = true;
          playerGameStats[cleanName(games[gameId].name)] = {
            secondsPlayed: stats.secondsPlayed,
            lastPlayed: stats.lastPlayed,
            achievements: stats.achievements,
          };
        }
      }

      if (!visible) {
        return [];
      }

      return [
        {
          id: "",
          avatarId: player.avatarId || undefined,
          isFounder: player.number === "0000" || undefined,
          games: playerGameStats || undefined,
          lastActive: player.lastActive || undefined,
        },
      ];
    });

    for (let [i, visiblePlayer] of visiblePlayers.entries()) {
      visiblePlayer.id = prefixed("/visible-player-", i + 1);
    }

    visiblePlayersUrl = window.URL.createObjectURL(
      new Blob([JSON.stringify(visiblePlayers, null, 2)], {
        type: "application/json;charset=utf-8",
      })
    );

    let w = open(visiblePlayersUrl, "1-visible-players.json", `location = 0, width = ${Math.floor(0.25 * screen.width)}, left = ${Math.floor(0.70 * screen.width)}, height = ${Math.floor(0.35 * screen.availHeight)}, top = ${Math.floor(0.15 * screen.height)} `);
    windows.push(w);
    w?.addEventListener("load", () => {
      w.document.title = `⬇️ ${prefix} /1-visible-players.json`;
      let link = w.document.createElement("a");
      link.href = visiblePlayersUrl;
      link.download = "1-visible-players.json";
      link.title = "click to download";
      Object.assign(link.style, jsonStyles);
      link.appendChild(w.document.body.firstElementChild);
      w.document.body.appendChild(link);
    });

    console.debug("players with playtime visible", { visiblePlayers });

    let interestingPlayers = visiblePlayers.flatMap((player) => {
      let interestingPlayer = {
        ...player,
        games: Object.fromEntries(
          Object.entries(player.games).filter(
            ([id, { secondsPlayed }]) => secondsPlayed >= interestingSeconds
          )
        ),
      };

      if (Object.keys(interestingPlayer.games).length > 0) {
        return [interestingPlayer];
      } else {
        return [];
      }
    });

    console.debug("players with meaningful playtime visible", {
      interestingPlayers,
    });




    let interestingGamePlayerLists = {};
    for (let player of interestingPlayers) {
      for (let [game, info] of Object.entries(player.games)) {
        (interestingGamePlayerLists[game] ??= []).push(player);
      }
    }

    document.title = "Game Popularity Data and Analysis";

    markdown += `
  The Visible Players data set (n = ${Object.keys(visiblePlayers).length}), with user identifiers removed for privacy, was published [for download here](${visiblePlayersUrl}).

  ## Game Popularity Association Matrix

  Each cell indicates the percentage of players of the game to the left that have also played the game to the top.

  ### Meaningfully Played Games

  Here we only look at game that players have played for at least ${interestingHours} hours. However, the sample size is very small (n = ${interestingPlayers.length}) because most players' privacy settings do not allow us to know their playtime.

  ### Tried Games

  Here we consider all games that players have ever opened, even if it was only for a couple of minutes. This is less meaningful, but we have much more data (n = ${12344456434642}) because the default privacy settings allow us to see a player's list of games.

  `;



    let popularColumns = 8;
    let popularRows = 24;

    let popularGames = ([...Object.entries(interestingGamePlayerLists)].sort(([_a, a], [_b, b]) => b.length - a.length)).map(([name, players]) => ({ name, players }));

    console.debug("most widely-meaningfully-played games", { popularGames });

    markdown += `\n|  |`;
    for (let [column, columnGame] of popularGames.slice(0, popularColumns).entries()) {
      if (column === 0) {
        markdown += ' |';
      } else {
        markdown += ` **${columnGame.name.slice(0, 8).trim()}** |`;
      }
    }
    markdown += `\n|--:|`;
    for (let [column, columnGame] of popularGames.slice(0, popularColumns).entries()) {
      markdown += `--:|`;
    }
    for (let [row, rowGame] of popularGames.slice(0, popularRows).entries()) {
      markdown += `\n| `;
      if (row === 0) {
        markdown += ` |`;
      } else {
        markdown += ` **${rowGame.name.slice(0, 8).trim()}** |`;
      }
      for (let [column, columnGame] of popularGames.slice(0, popularColumns).entries()) {
        if (rowGame === columnGame) {
          markdown += ` **${rowGame.name.slice(0, 8).trim()}** |`;
        } else {
          let rowPlayers = new Set(rowGame.players);
          let commonPlayers = new Set(columnGame.players.filter(p => rowPlayers.has(p)));;
          let percent = (100 * commonPlayers.size / columnGame.players.length).toFixed(0);
          let description = `${percent}% (${commonPlayers.size} of ${columnGame.players.length}) of ${columnGame.name} players also play ${rowGame.name}`;
          markdown += ` \`${percent}\`[%](mailto:street@hey.com?subject=${encodeURIComponent(description)} "${description}") |`;
        }
      }
    }


    CODE`


  ## Most Popular Avatars

  ### Overall

  We have a lot of data for this (n ≫ 1).
  `;

    `
  ### Among Meaningful Players of Each Game

  🖼️

  The script that produced this analysis was published [for download here](https://gist.githubusercontent.com/StadiaStreet/9b0d619e16caca2504599cd4478a2ab7/raw/1.js).
  `;


    let markdownUrl = window.URL.createObjectURL(
      new Blob([markdown], { type: "text/plain;charset=utf-8" })
    );

    let w2 = open(markdownUrl, "1.md", `location=0,width=${Math.floor(0.25 * screen.width)},left=${Math.floor(0.70 * screen.width)},height=${Math.floor(0.35 * screen.availHeight)},top=${Math.floor(screen.height * 3 / 5)}`);
    windows.push(w2);
    w2?.addEventListener("load", () => {
      w2.document.title = `⬇️ ${prefix}/1.md`;
      let link = w2.document.createElement("a");
      link.href = markdownUrl;
      link.download = "1.md";
      link.title = "click to download";
      Object.assign(link.style, jsonStyles);
      link.appendChild(w.document.body.firstElementChild);
      w2.document.body.appendChild(link);
    });
    */
};


let PROSE = (strings, ...values) => {
  let parts = [];
  for (let i = 0; i < strings.length; i++) {
    if (i < values.length) {
      parts.push(strings[i]);
      parts.push(values[i].toString());
    } else {
      parts.push(strings[i].replace(/\n *CODE$/, ''));
    }
  }
  PROSE.markdown += parts.join('');

  try {
    document.body.innerHTML = markdownit().render(PROSE.markdown)
    document.body.style.whiteSpace = '';
  } catch (error) {
    console.error(error);
    document.body.textContent = PROSE.markdown;
    document.body.style.whiteSpace = 'pre-wrap';
  }
};
PROSE.markdown = '';
let CODE = PROSE;

let prefix = `st/${Math.floor(Date.now() / 10_000_000)
  .toString(10)
  .padStart(7, "0")}`;

let sorted = x => {
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
  } else if (x && typeof x === 'object') {
    let keys = Object.keys(x).sort();
    let y = {};
    for (let key of keys) {
      y[key] = sorted(x[key]);
    }
    return y;
  } else {
    return x;
  }
}

let theSetOf = (data = { DataSets: Object.entries(theSetOf.allSets).map(([k, v]) => ({ [k]: v })) }) => {
  if (Object.keys(data).length !== 1) {
    throw new TypeError('theSetOf what?');
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
    return `the set of ${values.length} ${key.replace(/(.)([A-Z])/g, '$1 $2')}`;
  } else {
    theSetOf.allSets[key] = value;
    return `[a set of ${values.length} **${key.replace(/(.)([A-Z])/g, '$1 $2')}**](${url})`;
  }
};
theSetOf.allSets = {};

let cleanName = (name) =>
  name
    .replace(/™/g, "_")
    .replace(/®/g, "_")
    .replace(/[\:\-]? Early Access$/g, "_")
    .replace(/[\:\-]? \w+ Edition$/g, "_")
    .replace(/\(\w+ Ver(\.|sion)\)$/g, "_")
    .replace(/™/g, "_")
    .replace(/\s{2,}/g, "_")
    .replace(/^\s+|\s+$/g, "")
    .normalize("NFKD")
    .replace(/\p{Mark}/gu, "")
    .replace(/'/g, "")
    .replace(/[^a-z0-9A-Z_]+/g, "_")
    .replace(/^\_+|\_+$/g, "")
    .replace(
      /(_|^)([a-zA-Z0-9])([a-zA-Z0-9]*)/g,
      (_1, p, c, d) =>
        p + (p ? c : c) + d
    )
    .replace(/_/g, ' ')
    .replace(/[ ]+/g, ' ')
    .trim();

let query = new URL(location).searchParams;

import("./markdown-it.js").finally(street);
