#!/bin/bash

wsm="./sqlite2keys2strings.wasm"

hdir="$PWD"
gdir="/guest.d"
bname="example.sqlite.db3"
gpath="${gdir}/${bname}"
hpath="${hdir}/${bname}"

run_wasi(){
    wasmtime \
      run \
      --env ENV_SQLITE_DB_FILE_PATH="${gpath}" \
      --dir "${hdir}::${gdir}" \
      "${wsm}"
}

geninput(){
  echo generating example sqlite file...

  sqlite3 "${hpath}" "
    CREATE TABLE IF NOT EXISTS kvpairs (
      key TEXT PRIMARY KEY,
      val TEXT
    )
    WITHOUT ROWID
  "

  sqlite3 "${hpath}" "INSERT INTO kvpairs VALUES('hello', 'world')"
  sqlite3 "${hpath}" "INSERT INTO kvpairs VALUES('helo',  'wrld')"
}

input1(){
  echo helo
  echo NOT_FOUND
  echo hello
  echo hello
  echo hello
}

test -f "${hpath}" || geninput

input1 | run_wasi | jq -c
