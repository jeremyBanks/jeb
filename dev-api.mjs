import fs from 'fs';

import SQL from 'sql-template-strings';
import * as sqlite from 'sqlite';
import sqlite3 from 'sqlite3';

sqlite.open({
  filename: '/mnt/c/Users/_/Desktop/data.sqlite3',
  driver: sqlite3.Database,
}).then((db, _sql) => main({
  sql: _sql = (strings, ...values) => {
    strings = [...strings];
    values = [...values];
    for (let i = 0; i < values.length; i += 1) {
      let value = values[i];
      if (value && typeof value === 'object') {
        values[i] = JSON.stringify(value);
        strings[i] = `${strings[i]} json(`;
        strings[i + 1] = `)${strings[i + 1]} `
      }
    }
    return db.all(SQL(strings, ...values));
  },

  console: Object.assign(Object.create(console), {
    sql: async (strings, ...values) => {
      let results = await _sql(strings, ...values);
      console.debug(strings.join('…').trim(), '→', results);
      return results;
    }
  })
}));

let main = async ({ sql, console }) => {
  let configureFastAndDumb = async () => {
    await console.sql`pragma synchronous = off`;
    await console.sql`pragma locking_mode = exclusive`;
    await console.sql`pragma journal_mode = memory`;
    await console.sql`pragma temp_store = memory`;

    await console.sql`pragma foreign_key_check`;
    await console.sql`pragma foreign_keys = on`;
  };

  let createTables = async () => {
    await sql`
      create table Player(
        [json]
        text not null,

        [userId]
        text
        not null
        unique
        as (json_extract(json, '$.userId')) stored
        constraint [type of Player.userId] check (typeof (userId) = 'text')
        constraint [length of Player.userId] check (length(userId) between 1 and 20),

        [name]
        text
        not null
        as (json_extract(json, '$.name')) stored
        constraint [type of Player.name] check (typeof (name) = 'text')
        constraint [length of Player.userId] check (length(name) between 3 and 15),

        [number]
        text
        not null
        as (json_extract(json, '$.number')) stored
        constraint [type of Player.number] check (typeof (number) = 'text')
        constraint [length of Player.number] check (length(number) = 4)
        constraint [range of Player.number] check (
          number = '0000' or (
            cast(number as integer) between 1000 and 9999) and
            number = cast(cast(number as integer) as string)),

        [gamerTag]
        text
        unique
        not null
        as (case number when '0000' then name else name || '#' || number end) stored
        constraint [type of Player.gamerTag] check (typeof (gamerTag) = 'text')
        constraint [length of Player.gamerTag] check (length(gamerTag) between 3 and 20),

        [games]
        text
        as (json_extract(json, '$.games')) stored
      );
    `;

    await sql`
      create table Game(
        [json]
        text
        not null,

        [gameId]
        text
        unique
        not null
        as (json_extract(json, '$.gameId')) stored
        constraint [type of Player.gameId] check (typeof (gameId) = 'text')
        constraint [length of Player.gameId] check (length(gameId) = 36)
        constraint [suffix of Game.gameId] check (gameId like '%rcp1'),

        [name]
        text
        as (json_extract(json, '$.name')) stored
      )
    `;

    await sql`
      create table PlayerGame(
        [userId]
        text
        not null,

        [gameId]
        text
        not null,

        unique (gameId, userId),
        unique (userId, gameId),

        foreign key (gameId) references Game (gameId),
        foreign key (userId) references Player (userId)
      )
    `;

    await sql`
      create trigger PlayerGameFromPlayerInserted
        after insert on Player begin
          select raise (rollback, 'player insertion not implemented');
        end
    `;

    await sql`
      create trigger PlayerGameFromPlayerDeleted
        after delete on Player begin
          select raise (rollback, 'player deletion not implemented');
        end
    `;

    await sql`
      create trigger PlayerGameFromPlayerUpdated
        after update on Player begin
          select raise (rollback, 'player updates not implemented');
        end
    `;
  };

  let importRecords = async () => {
    await sql`savepoint loading`;

    console.debug(`Loading records.json...`);
    for (let record of Object.values(JSON.parse(fs.readFileSync('./stadia.st/-/records.json')))) {
      if (record.type === 'user') {
        await sql`
          insert into Player values(${record})
        `;
      } else if (record.type === 'game') {
        await sql`
          insert into Game values(${record})
        `;
      }
    }

    for (let source of fs.readdirSync('/mnt/c/Users/_/Desktop/street').map(s => `/mnt/c/Users/_/Desktop/street/${s}`)) {
      console.debug(`Loading ${source}...`);
      for (let record of Object.values(JSON.parse(fs.readFileSync(source)))) {
        record.userId = record.id;
        delete record.id;
        await sql`
          insert into Player values(${record})
          on conflict do nothing
      `;
      }
    }

    await sql`release loading`;
  };

  let use = async () => {
    await console.sql`
      select Player.number, count(Player.json)
      as count from Player
      group by Player.number
      order by count desc
      limit 0, 4
    `;

    await console.sql`select count(Game.json) as games from Game`;
    await console.sql`select count(Player.json) as players from Player`;

    await console.sql`
      select Player.gamerTag, Game.name as gameName, json_extract(PlayerGame.value, '$.secondsPlayed') as secondsPlayed, (json_extract(PlayerGame.value, '$.secondsPlayed') / 60 / 60) as hoursPlayed
      from Player
      left join json_each(Player.games) as PlayerGame
      left join Game on Game.gameId = PlayerGame.key
      where gameId is not null and secondsPlayed is not null
      order by secondsPlayed desc
      limit 0, 4
    `;
  };

  await configureFastAndDumb();

  try {
    await createTables();
  } catch (error) {
    console.warn('failed to create tables', error);
  }

  try {
    await importRecords();
  } catch (error) {
    console.warn('failed to import records into database', error);
  }

  await use();
};
