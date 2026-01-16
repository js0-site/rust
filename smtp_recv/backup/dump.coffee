#!/usr/bin/env coffee

> path > join resolve
  @3-/read
  @3-/write

{
  MYSQL_DB
} = process.env

PWD = import.meta.dirname
ROOT = resolve PWD,'../../..'
MOD = join ROOT, 'mod'
DUMP_SQL = join PWD, 'db_'+MYSQL_DB+'.txt'

li = []
for i from read(DUMP_SQL).replace(
  / AUTO_INCREMENT=\d+/
  ''
).replace(
   /\/\*!\s*\d*\s*DEFINER=`[^`]+`@`[^`]+`\s*\*\/\s*/g
  ''
).replace(
   /\s+DEFINER=`[^`]+`@`[^`]+`\s+/g
  ' '
).replaceAll(
  '/*!50003 CREATE*/ /*!50003 '
  'CREATE '
).replaceAll(
  '/*!50106 CREATE*/ /*!50106 '
  'CREATE '
).replaceAll(
  '*/;;'
  ';;'
).replaceAll(
  '*/ ;;'
  ';;'
).replace(
  /\s*AUTO_INCREMENT=\d+/g
  ''
).replaceAll(
  'CREATE PROCEDURE'
  'CREATE OR REPLACE PROCEDURE'
).split('\n')
  for kind from [
    'TRIGGER'
    'EVENT'
    'TABLE'
  ]
    i = i.replace(
      'CREATE '+kind+' '
      'CREATE '+kind+' IF NOT EXISTS '
    ).replace(
      ' IF NOT EXISTS IF NOT EXISTS '
      ' IF NOT EXISTS '
    )

  i = i.trim()
  if not i or i.startsWith('--') or i.startsWith('/*')
    continue
  i = i.replace(/ ENGINE=\w+/g,'').replace(' DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;',';')
  li.push i

write(
  DUMP_SQL
  li.join('\n')
)
