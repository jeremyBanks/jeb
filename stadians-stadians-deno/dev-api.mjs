import fs from 'fs';
import util from 'util';
import { performance } from 'perf_hooks';

import chalk from 'chalk';
import SQL from 'sql-template-strings';
import * as sqlite from 'sqlite';
import sqlite3 from 'sqlite3';

let db = sqlite.open({
  filename: './sqlite.tmp',
  driver: sqlite3.Database,
});

let main = async ({ db, sql: qsql, vsql: sql }) => {
  console.log(SQL`test ${SQL`hello`}`);

  {
    await qsql`pragma foreign_keys = on`;
    await qsql`pragma synchronous = off`;
    await qsql`pragma locking_mode = exclusive`;
    await qsql`pragma recursive_triggers = off`;
    await qsql`pragma foreign_key_check`;
  }

  let requiresInitialization = false;
  {
    let [{application_id: oldApplicationId}] = await qsql`pragma application_id`;
    let [{user_version: oldUserVersion}] = await qsql`pragma user_version`;
    if (oldApplicationId) oldApplicationId = `0x${oldApplicationId.toString(16).toUpperCase().padStart(8, '0')}`;
    if (oldUserVersion) oldUserVersion = `0x${oldUserVersion.toString(16).toUpperCase().padStart(8, '0')}`;

    let applicationId = '0x57AD1A57';
    let userVersion = `0x${(Math.floor(0x57570000 + (Date.now() - 1574164800000) / (60 * 60 * 1000)).toString(16).toUpperCase().padStart(8, '0'))}`;
    await db.exec(`
      pragma application_id = ${applicationId};
      pragma user_version = ${userVersion};
    `);

    requiresInitialization =
      requiresInitialization ||
      applicationId !== oldApplicationId ||
      userVersion !== oldUserVersion;

    console.debug({
      oldApplicationId,
      applicationId,
      oldUserVersion,
      userVersion,
      requiresInitialization,
    });
  }

  if (requiresInitialization) {
    await sql`
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
    await sql`
      create index [User(lower(name), number)] on User(lower(name), number)
    `;
    await sql`
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
    await sql`
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
    await sql`
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
    await sql`
      create trigger [UserGame from User: delete]
      after delete on User begin
        select raise(fail, 'user deletion not implemented');
      end
    `;
    await sql`
      create trigger [UserGame from User: update]
      after update on User begin
        select raise(fail, 'user updates not implemented');
      end
    `;


    // PACKING and UNPACKING
    await sql`
      create view please
             as select null as packed`;
    await sql`
      create table [User.json](
             json text not null check (json_type(json) = 'object'))`;
    await sql`
      create table [Game.json](
             json text not null check (json_type(json) = 'object'))`;
    await sql`
      create trigger [please set packed = false]
             instead of update of packed on please when new.PACKED is false
             begin insert into Game (json)
                          select json from [Game.json] where true
                          on conflict (gameId) do update
                             set json = json_patch(json, excluded.json);
                   delete from [Game.json];
             end`;
    await sql`
      create trigger [please set packed = true]
      instead of update of packed on please when new.PACKED is true
              begin insert into [Game.json] (json)
                           select json from Game where true;
                    delete from Game;
              end`;

    await sql`pragma recursive_triggers = on`;
    await sql`update please set packed = false`;
    await sql`update please set packed = true`;
    await sql`pragma recursive_triggers = off`;
};

  let importRecords = async () => {
    await sql`savepoint [import spidered records]`;
    try {
      console.debug(chalk.dim(`Loading ${chalk.cyan('records.json')} …`));
      for (let record of Object.values(JSON.parse(fs.readFileSync('./stadia.st/-/records.json')))) {
        if (record.type === 'user') {
          await qsql`insert into User values(${record})`;
        } else if (record.type === 'game') {
          await qsql`insert into Game values(${record})`;
        }
      }
    } finally {
      await sql`release [import spidered records]`;
    }

    await sql`savepoint [import identified users]`;
    try {
      let sourceDir = `${process.env['HOME']}/Desktop/street`;
      for (let source of fs.readdirSync(sourceDir).filter(s => s.endsWith('.json')).map(s => `${sourceDir}/${s}`)) {
        console.debug(chalk.dim(`Loading ${chalk.underline.cyan(source)} …`));
        for (let record of Object.values(JSON.parse(fs.readFileSync(source)))) {
          record.userId = record.id;
          delete record.id;
          await qsql`
            insert into User values(${record})
            on conflict do nothing
        `;
        }
      }
    } finally {
      await sql`release [import identified users]`;
    }

    await sql`analyze`;
  };

  let use = async () => {
    await sql`
      select
        (select count(*) from Game),
        (select count(*) from User),
        (select count(*) from UserGame)
      `;

    await sql`
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

    await sql`
      select
        lower(User.name) as name,
        count(*) as count,
        founder.userId as founderUserId
      from User user
      left join User founder on lower(founder.name) = lower(user.name) and founder.number = '0000'
      group by lower(User.name)
      order by count desc
      limit 0, 64
    `;

    return false;

    await sql`
      create temporary table UserGamedForAnHour
      as select gameId, userId, secondsPlayed
      from UserGame
      where secondsPlayed is not null
      and secondsPlayed >= 60 * 60
    `;

    await sql`
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

    await sql`
      drop table UserGamedForAnHour
    `

    await sql`
    select * from (
        select * from (select
          substr(lower(User.name || '#'), 1, 1) as prefix,
          count(distinct lower(User.name)) as names,
          count(*) as users
        from User user
        group by prefix
        order by users desc
        limit 0, 96)
        union
        select * from (select
          substr(lower(User.name || '#'), 1, 2) as prefix,
          count(distinct lower(User.name)) as names,
          count(*) as users
        from User user
        group by prefix
        order by users desc
        limit 0, 96)
      union
        select * from (select
          substr(lower(User.name || '#'), 1, 3) as prefix,
          count(distinct lower(User.name)) as names,
          count(*) as users
        from User user
        group by prefix
        order by users desc
        limit 0, 96)
      union
        select * from (select
          substr(lower(User.name || '#'), 1, 4) as prefix,
          count(distinct lower(User.name)) as names,
          count(*) as users
        from User user
        group by prefix
        order by users desc
        limit 0, 96)
      union
        select * from (select
          substr(lower(User.name || '#'), 1, 5) as prefix,
          count(distinct lower(User.name)) as names,
          count(*) as users
        from User user
        group by prefix
        order by users desc
        limit 0, 96)
      union
        select * from (select
          substr(lower(User.name || '#'), 1, 6) as prefix,
          count(distinct lower(User.name)) as names,
          count(*) as users
        from User user
        group by prefix
        order by users desc
        limit 0, 96)
      union
        select * from (select
          substr(lower(User.name || '#'), 1, 7) as prefix,
          count(distinct lower(User.name)) as names,
          count(*) as users
        from User user
        group by prefix
        order by users desc
        limit 0, 96)
      union
        select * from (select
          substr(lower(User.name || '#'), 1, 8) as prefix,
          count(distinct lower(User.name)) as names,
          count(*) as users
        from User user
        group by prefix
        order by users desc
        limit 0, 96)
      union
        select * from (select
          substr(lower(User.name || '#'), 1, 9) as prefix,
          count(distinct lower(User.name)) as names,
          count(*) as users
        from User user
        group by prefix
        order by users desc
        limit 0, 96)
      union
        select * from (select
          substr(lower(User.name || '#'), 1, 10) as prefix,
          count(distinct lower(User.name)) as names,
          count(*) as users
        from User user
        group by prefix
        order by users desc
        limit 0, 96)
      union
        select * from (select
          substr(lower(User.name || '#'), 1, 11) as prefix,
          count(distinct lower(User.name)) as names,
          count(*) as users
        from User user
        group by prefix
        order by users desc
        limit 0, 96)
      union
        select * from (select
          substr(lower(User.name || '#'), 1, 12) as prefix,
          count(distinct lower(User.name)) as names,
          count(*) as users
        from User user
        group by prefix
        order by users desc
        limit 0, 96)
      union
        select * from (select
          substr(lower(User.name || '#'), 1, 13) as prefix,
          count(distinct lower(User.name)) as names,
          count(*) as users
        from User user
        group by prefix
        order by users desc
        limit 0, 96)
      union
        select * from (select
          substr(lower(User.name || '#'), 1, 14) as prefix,
          count(distinct lower(User.name)) as names,
          count(*) as users
        from User user
        group by prefix
        order by users desc
        limit 0, 96)
      union
        select * from (select
          substr(lower(User.name || '#'), 1, 15) as prefix,
          count(distinct lower(User.name)) as names,
          count(*) as users
        from User user
        group by prefix
        order by users desc
        limit 0, 96)
      )
      order by users desc, names asc
      limit 0, 96
    `;



  await sql`select count(*) from User`;
};

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

  await sql`vacuum main into ${'./sqlite.min.tmp'};`;
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

db.then((db) => {
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

  let vsql = async (strings, ...values) => {
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
      console.debug(chalk.underline.rgb(0 | Math.min(0xFF, 0x00 + 2 * elapsed), 0 | (Math.max(0, 0x80 - elapsed / 100)), 0x20)(`Query took ${elapsed.toFixed(1)}ms:\n`) + pretty, chalk.green('→'), util.inspect(value, { maxArrayLength: null, colors: true }));
      console.debug();
      return value;
    } catch (error) {
      console.debug(chalk.underline.red(`Query failed:\n`) + pretty, chalk.red('→'), chalk.red(error));
      console.debug();
      throw error;
    }
  };

  return main({ db, sql, vsql });
});
