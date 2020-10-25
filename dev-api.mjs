import fs from 'fs';
import { performance } from 'perf_hooks';

import chalk from 'chalk';
import SQL from 'sql-template-strings';
import * as sqlite from 'sqlite';
import sqlite3 from 'sqlite3';

sqlite.open({
  filename: './sqlite.tmp',
  driver: sqlite3.Database,
}).then((db, _sql) => {
  let sql = async (strings, ...values) => {
    strings = [...strings];
    values = [...values];
    for (let i = 0; i < values.length; i += 1) {
      let value = values[i];
      if (value instanceof Object) {
        values[i] = JSON.stringify(sortKeys(value));
        strings[i] = `${strings[i]} json(`;
        strings[i + 1] = `)${strings[i + 1]} `
      }
    }
    let rows = await db.all(SQL(strings, ...values));
    let f = (value, key) => {
      if (typeof value?.json === 'string') {
        let onlyKey = (Object.keys(value).length === 1);
        try {
          let json = sortKeys(JSON.parse(value.json), f);
          if (onlyKey) {
            return json;
          }
          if (json instanceof Array) {
            return Object.assign({}, value, { json });
          } else if (json instanceof Object) {
            delete value.json;
            return Object.assign(Object.create(json), value, { json });
          } else {
            return Object.assign({}, value, { json });
          }
        } catch (error) {
          console.warn(chalk.dim.keyword('orange')(`Warning: Invalid .json property in result: ${error}`));
        }
      }
      return value;
    };
    return sortKeys(rows, f);
  };

  let console = Object.assign(Object.create(globalThis.console), {
    sql: async (strings, ...values) => {
      let pretty = chalk.rgb(0xFF, 0xFF, 0x7F)(dedent(strings.join('…').replace(/(^\n+|\s+$)/gm, '')));
      let details = [];
      try {
        for (let { id, detail, parent } of await db.all(SQL(strings.map((x, i) => i ? x : `explain query plan ${x}`), ...values))) {
          details.push({
            order: id + .0,
            value: chalk.rgb(0x20, 0x20, 0x00)(`${id}${parent ? `:${parent}` : ``}: `) + chalk.rgb(0xB0, 0xB0, 0x40)(detail)
          });
        }
        for (let { addr, opcode, p1, p2, p3, p4, p5, comment } of await db.all(SQL(strings.map((x, i) => i ? x : `explain ${x}`), ...values))) {
          let args = [p1, p2, p3, p4, p5, comment];
          while (args.length > 0) {
            let arg = args.pop();
            if (arg && !arg?.match?.(/^0+$/)) {
              args.push(arg);
              break;
            }
          }
          details.push({
            order: addr + .1,
            value: chalk.rgb(0x20, 0x20, 0x00)(`${addr}: ${opcode}(${args.map(JSON.stringify).join(', ')})`)
          });
        }
      } catch { }
      let ugly = '';
      details = details.sort((a, b) => a.order - b.order).map(x => x.value);
      if (details.length > 0) {
        ugly += (chalk.rgb(0x7F, 0x7F, 0x40)(' → ') + details.join(chalk.rgb(0x20, 0x20, 0x20)(', ')));
      }
      if (ugly) {
        pretty += ugly;
      }
      try {
        let before = performance.now();
        let value = await (sql)(strings, ...values);
        let elapsed = performance.now() - before;
        console.debug(chalk.underline.rgb(0 | Math.min(0xFF, 0x00 + 2 * elapsed), 0 | (Math.max(0, 0x80 - elapsed / 100)), 0x20)(`Query took ${elapsed.toFixed(1)}ms:\n`) + pretty, chalk.green('→'), value);
        console.debug();
        return value;
      } catch (error) {
        console.debug(chalk.underline.red(`Query failed:\n`) + pretty, chalk.red('→'), chalk.red(error));
        console.debug();
        throw error;
      }
    }
  });

  return main({ sql, console });
});

let main = async ({ sql, console }) => {
  let configureFastAndDumb = async () => {
    await console.sql`pragma synchronous = off`;
    await console.sql`pragma locking_mode = exclusive`;
    await console.sql`pragma journal_mode = memory`;
    await console.sql`pragma temp_store = memory`;
    await console.sql`pragma foreign_keys = on`;
    await console.sql`pragma recursive_triggers = on`;

    await console.sql`attach ':memory:' as dump`;

    await console.sql`pragma foreign_key_check`;
  };

  let createTables = async () => {
    await console.sql`
      create table User(
        [json]
        text
        not null
        ,
        [userId]
        text
        not null
        unique
        as (json_extract(json, '$.userId')) stored
        constraint [typeof(userId) = 'text'] check (typeof(userId) = 'text')
        constraint [length(userId) between 1 and 20] check (length(userId) between 1 and 20)
        ,
        [name]
        text
        not null
        as (json_extract(json, '$.name')) stored
        constraint [typeof(name) = 'text'] check (typeof(name) = 'text')
        constraint [length(name) between 3 and 15] check (length(name) between 3 and 15)
        ,
        [number]
        text
        not null
        as (json_extract(json, '$.number')) stored
        constraint [typeof(number) = 'text'] check (typeof(number) = 'text')
        constraint [length(number) = 4] check (length(number) = 4)
        constraint [number is 0000 or between 1000 and 9999] check (
          number = '0000' or(
            cast(number as integer) between 1000 and 9999) and
          number = cast(cast(number as integer) as string))
        ,
        [gamerTag]
        text
        unique
        not null
        as (case number when '0000' then name else name || '#' || number end) stored
        constraint [typeof(gamerTag) = 'text'] check (typeof(gamerTag) = 'text')
        constraint [length(gamerTag) between 3 and 20] check (length(gamerTag) between 3 and 20)
        ,
        [games]
        text
        as (json_extract(json, '$.games')) stored
        ,
        [lastActive]
        integer
        as (json_extract(json, '$.lastActive')) stored
      )
    `;
    await console.sql`
      create index [User(lower(name), number)] on User(lower(name), number)
    `;
    await console.sql`
    create table Game(
      [json]
        text
        not null
      ,
      [gameId]
        text
        unique
        not null
        as (json_extract(json, '$.gameId')) stored
        constraint [typeof(gameId) = 'text'] check (typeof(gameId) = 'text')
        constraint [length(gameId) = 36] check (length(gameId) = 36)
        constraint [gameId like '%rcp1'] check (gameId like '%rcp1')
      ,
      [name]
        text
        as (json_extract(json, '$.name')) stored
    )
      `;
    await console.sql`
      create table UserGame(
        [userId]
        text
        not null
        references User(userId) deferrable initially deferred
        ,
        [gameId]
        text
        not null
        references Game(gameId) deferrable initially deferred
        ,
        [secondsPlayed]
        integer
        ,
        [lastPlayed]
        integer
        ,
        unique(gameId, userId)
        unique(userId, gameId)
      )
    `;
    await console.sql`
      create trigger [UserGame from User: insert]
      after insert on User begin
      insert into UserGame(
          userId,
          gameId,
          secondsPlayed,
          lastPlayed
        )
        select
          NEW.userId,
          key,
          json_extract(value, '$.secondsPlayed'),
          json_extract(value, '$.lastPlayed')
        from json_each(NEW.games);
      end
    `;
    await console.sql`
      create trigger [UserGame from User: delete]
      after delete on User begin
        select raise(fail, 'user deletion not implemented');
      end
    `;
    await console.sql`
      create trigger [UserGame from User: update]
      after update on User begin
        select raise(fail, 'user updates not implemented');
      end
    `;
  };

  let importRecords = async () => {
    await console.sql`savepoint [import spidered records]`;
    try {
      console.debug(chalk.dim(`Loading ${chalk.cyan('records.json')} …`));
      for (let record of Object.values(JSON.parse(fs.readFileSync('./stadia.st/-/records.json')))) {
        if (record.type === 'user') {
          await sql`insert into User values(${record})`;
        } else if (record.type === 'game') {
          await sql`insert into Game values(${record})`;
        }
      }
    } finally {
      await console.sql`release [import spidered records]`;
    }

    await console.sql`savepoint [import identified users]`;
    try {
      let sourceDir = `${process.env['HOME']}/Desktop/street`;
      for (let source of fs.readdirSync(sourceDir).filter(s => s.endsWith('.json')).map(s => `${sourceDir}/${s}`)) {
        console.debug(chalk.dim(`Loading ${chalk.underline.cyan(source)} …`));
        for (let record of Object.values(JSON.parse(fs.readFileSync(source)))) {
          record.userId = record.id;
          delete record.id;
          await sql`
            insert into User values(${record})
            on conflict do nothing
        `;
        }
      }
    } finally {
      await console.sql`release [import identified users]`;
    }

    await console.sql`analyze`;
  };

  let use = async () => {
    await console.sql`
      select
        (select count(*) from Game),
        (select count(*) from User),
        (select count(*) from UserGame)
      `;

    await console.sql`
      select
        gamerTag,
        Game.name,
        secondsPlayed
      from UserGame
      inner join Game on Game.gameId = UserGame.gameId
      inner join User on User.userId = UserGame.userId
      where secondsPlayed > 0
      order by secondsPlayed desc
      limit 0, 1
    `;

    await console.sql`
      select
        lower(User.name) as name,
        count(*) as count,
        founder.userId as founderUserId
      from User user
      left join User founder on lower(founder.name) = lower(user.name) and founder.number = '0000'
      group by lower(User.name)
      order by count desc
      limit 0, 8
    `;

    await console.sql`
      create temporary table UserGamedForAnHour
      as select gameId, userId, secondsPlayed
      from UserGame
      where secondsPlayed is not null
      and secondsPlayed >= 60 * 60
    `;

    await console.sql`
      select
        Game.name as name,
        coalesce(sum(UserGamedForAnHour.secondsPlayed) / 60 / 60, 0) as hours,
        count(UserGamedForAnHour.secondsPlayed) as players
      from Game
      left join UserGamedForAnHour
      on Game.gameId = UserGamedForAnHour.gameId
      group by UserGamedForAnHour.gameId
      order by hours desc
      limit 0, 2
    `;

    await console.sql`
      drop table UserGamedForAnHour
    `

    await console.sql`
    select * from (
        select * from (select
          substr(lower(User.name), 1, 1) as prefix,
          count(*) as count
        from User user
        group by prefix
        order by count desc
        limit 0, 64)
      union
        select * from (select
          substr(lower(User.name), 1, 2) as prefix,
          count(*) as count
        from User user
        group by prefix
        order by count desc
        limit 0, 64)
      union
        select * from (select
          substr(lower(User.name), 1, 3) as prefix,
          count(*) as count
        from User user
        group by prefix
        order by count desc
        limit 0, 64)
      union
        select * from (select
          substr(lower(User.name), 1, 4) as prefix,
          count(*) as count
        from User user
        group by prefix
        order by count desc
        limit 0, 64)
      )
      order by count desc
      limit 0, 64
    `;
  };

  await configureFastAndDumb();

  try {
    await createTables();
  } catch (error) {
    console.warn(chalk.underline.keyword('orange')('Warning: Failed to create tables:\n') + error);
    console.debug();
  }

  try {
    await importRecords();
  } catch (error) {
    console.warn(chalk.underline.keyword('orange')('Warning: Failed to import records into database:\n') + error);
    console.debug();
  }

  await use();

  await console.sql`
    create table dump.User
      as select main.User.json
      from main.User
      order by main.User.userId asc
  `;

  await console.sql`
    create table dump.Game
      as select main.Game.json
      from main.Game
      order by main.Game.gameId asc
  `;

  await console.sql`vacuum main into ${'./sqlite.min.tmp'};`;
};

let sortKeys = (x, f = (value, _key) => value) => {
  if (x instanceof Array) {
    return x.map((value, key) => f(sortKeys(value), key));
  } else if (x instanceof Object) {
    return Object.fromEntries(
      Object.entries(x)
        .sort(([a, _a], [b, _b]) => {
          if (a < b) {
            return -1;
          } else if (a > b) {
            return +1;
          } else {
            return 0;
          }
        })
        .map(([key, value]) => [key, f(sortKeys(value), key)]));
  } else {
    return x;
  }
};

let dedent = s => {
  let lines = s.split(/\n/g);
  let minIndent = Infinity;
  for (let line of lines) {
    if (line.length > 0) {
      let indent = line.match(/\S+/)?.index || Infinity;
      if (indent && indent < minIndent) {
        minIndent = indent;
      }
    }
  }
  if (!isFinite(minIndent)) {
    minIndent = 0;
  }
  return lines.map(line => line.slice(minIndent)).join('\n');
}
