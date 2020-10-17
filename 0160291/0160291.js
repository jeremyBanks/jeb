Promise.resolve().then(async () => {
  let markdown = "";

  markdown += `\
# Data Collection

This process took place throughout September and October 2020.

1. A new Google account was created, with a new Stadia profile, with no Stadia friends, games, or activity history. This account was used for the rest of the process.
2. A set of random **Name Prefixes** were generated. Each were between between 2-4 characters, taken from a distribution roughly approximating English letter frequency, with digits included at a lower frequency.
3. Each Name Prefix was searched for using the the "Find players" interface in Stadia, and the results were collected into a set of **Candidate Players**.
4. Each Candidate Player's profile was examined, and if their list of played games was publicly visible, each of their game stats sub-page were also examined. If their play time for each game was visible, this information was collected and gathered into a set of **Visible Players**.
`;

  const interestingHours = 2;
  const interestingSeconds = interestingHours * 60 * 60;

  const monthDays = 32;// yes
  const monthSeconds = monthDays * 24 * 60 * 60;
  const minMonthlyTimestamp = Date.now() - monthSeconds * 1000;

  const yearDays = 365;
  const yearSeconds = yearDays * 24 * 60 * 60;
  const minYearlyTimestamp = Date.now() - yearSeconds * 1000;

  let prefix = `st/${Math.floor(Date.now() / 10_000_000)
    .toString(10)
    .padStart(7, "0")}`;

  const prefixed = (separator, id) =>
    `${prefix}${separator}${id.toString().padStart(7, "0")}`;

  const jsonStyles = {
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

  const cleanName = (name) =>
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
          (p ? c.toUpperCase() : c.toLowerCase()) + d.toLowerCase()
      )
      .replace(/_/g, '');

  const rawUrl = new URL(document.location).searchParams.get("raw-url");
  let visiblePlayersUrl = new URL(document.location).searchParams.get(
    "visible-players-url"
  );

  if (!rawUrl && !visiblePlayersUrl) {
    visiblePlayersUrl =
      "https://gist.githubusercontent.com/StadiaStreet/9b0d619e16caca2504599cd4478a2ab7/raw/1-visible-players.json";
  }

  if (!visiblePlayersUrl) {
    const rawResponse = await fetch(rawUrl);
    if (!rawResponse.ok) {
      throw new Error(rawResponse);
    }
    const json = await rawResponse.json();

    const records = Object.values(json);

    const games = Object.fromEntries(
      records.flatMap((record) =>
        record.type === "game" && record.gameId ? [[record.gameId, record]] : []
      )
    );

    const avatars = Object.fromEntries(
      records.flatMap((record) =>
        record.type === "avatar" && record.avatarId
          ? [[record.avatarId, record]]
          : []
      )
    );

    const players = records.filter(
      (record) => record.type === "user" && record.userId
    );

    const monthlyActivePlayers = players.filter(
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

    const yearlyActivePlayers = players.filter(
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

    const gameListVisiblePlayers = players.filter(
      (player) => player.gameIds
    );

    console.debug("users whose game lists are visible", {
      gameListVisiblePlayers,
    });

    const gameListCapturedPlayers = players.filter((player) => player.games);

    console.debug("users whose game lists have been fully captured", {
      gameListCapturedPlayers,
    });

    const candidatePlayers = [...players.entries()].flatMap(([i, record]) => {
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

    const visiblePlayers = [...candidatePlayers].flatMap((player) => {
      let visible = false;

      const playerGameStats = {};
      for (const [gameId, stats] of Object.entries(player.games || {})) {
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

    for (const [i, visiblePlayer] of visiblePlayers.entries()) {
      visiblePlayer.id = prefixed("/visible-player-", i + 1);
    }

    visiblePlayersUrl = window.URL.createObjectURL(
      new Blob([JSON.stringify(visiblePlayers, null, 2)], {
        type: "application/json;charset=utf-8",
      })
    );

    const w = open(visiblePlayersUrl, "1-visible-players.json", `location=0,width=${Math.floor(0.25 * screen.width)},left=${Math.floor(0.70 * screen.width)},height=${Math.floor(0.35 * screen.availHeight)},top=${Math.floor(0.15 * screen.height)}`);
    windows.push(w);
    w?.addEventListener("load", () => {
      w.document.title = `⬇️ ${prefix}/1-visible-players.json`;
      const link = w.document.createElement("a");
      link.href = visiblePlayersUrl;
      link.download = "1-visible-players.json";
      link.title = "click to download";
      Object.assign(link.style, jsonStyles);
      link.appendChild(w.document.body.firstElementChild);
      w.document.body.appendChild(link);
    });
  }

  const visiblePlayersResponse = await fetch(visiblePlayersUrl);
  if (!visiblePlayersResponse.ok) {
    throw new Error(visiblePlayersResponse);
  }
  const visiblePlayers = await visiblePlayersResponse.json();

  console.debug("players with playtime visible", { visiblePlayers });

  const interestingPlayers = visiblePlayers.flatMap((player) => {
    const interestingPlayer = {
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


  const interestingGamePlayerLists = {};
  for (const player of interestingPlayers) {
    for (const [game, info] of Object.entries(player.games)) {
      (interestingGamePlayerLists[game] ??= []).push(player);
    }
  }

  const popularGames = ([...Object.entries(interestingGamePlayerLists)].sort(([_a, a], [_b, b]) => b.length - a.length).slice(0, 32));

  console.debug("most widely-meaningfully-played games", {popularGames});

  document.title = "Game Popularity Data and Analysis";

  const pre = document.createElement("div");
  // Object.assign(document.body.style, {
  //   margin: 0,
  //   padding: 0,
  // });
  Object.assign(pre.style, {
    font: '16px sans-serif',
    margin: "16px",
    lineHeight: "2.0",
    outline: "none",
  });
  document.body.appendChild(pre);
  markdown += `
The Visible Players data set, with user identifiers removed for privacy, was published [for download here](${visiblePlayersUrl}).

# Analysis

- The data set consists of ${
    Object.keys(visiblePlayers).length
  } Visible Players.
- A Visible Player is considered to have **meaningfully played** a game if they have at least ${interestingHours} hours (${interestingSeconds} seconds) of time played in that game.
- An Visible Player is considered to be an **Interesting Player** if they have meaningfully played at least one game.
- The data set includes ${
    Object.keys(interestingPlayers).length
  } Interesting Players.
- A game's **popularity** refers to the number of Interesting Players that have meaningfully played that game. An avatar's popularity refers to the number of Interesting Players that are currently using that avatar.
- An Interesting Player is considered to be a **Founder** if their gamertag number is \`0000\`.
- An Interesting Player is considered to be a **Settler** if they are not a Founder, but also joined Stadia (created a Stadia profile and chose their Stadia gamertag) within one year of its launch (prior to November 19, 2020 in the \`America/Los_Angeles\` time zone).

## Game Popularity Association Matrix

The most popular games were selected for comparison in an association matrix, which is visualized below. Each cell indicates the percentage of meaningful players of the game to the left that have also meaningfully played the game to the top.

🖼️

## Most Popular Avatars

### Overall

🖼️

### Among Meaningful Players of Each Game

🖼️

The script that produced this analysis was published [for download here](https://gist.githubusercontent.com/StadiaStreet/9b0d619e16caca2504599cd4478a2ab7/raw/1.js).
`;

  pre.textContent = markdown;
  pre.innerHTML = window.markdownit().render(markdown);

  const markdownUrl = window.URL.createObjectURL(
    new Blob([markdown], { type: "text/plain;charset=utf-8" })
  );

  const w = open(markdownUrl, "1.md", `location=0,width=${Math.floor(0.25 * screen.width)},left=${Math.floor(0.70 * screen.width)},height=${Math.floor(0.35 * screen.availHeight)},top=${Math.floor(screen.height * 3 / 5)}`);
  windows.push(w);
  w?.addEventListener("load", () => {
    w.document.title = `⬇️ ${prefix}/1.md`;
    const link = w.document.createElement("a");
    link.href = markdownUrl;
    link.download = "1.md";
    link.title = "click to download";
    Object.assign(link.style, jsonStyles);
    link.appendChild(w.document.body.firstElementChild);
    w.document.body.appendChild(link);
  });
});
