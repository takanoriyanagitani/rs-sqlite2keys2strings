use std::io;
use std::path::Path;

use io::BufRead;

use io::BufWriter;
use io::Write;

use rusqlite::Connection;
use rusqlite::Statement;

use rusqlite::Row;
use rusqlite::Rows;

use rusqlite::types::ValueRef;

pub fn sqlite2key2val2sink<S>(
    stmt: &mut Statement,
    key: &str,
    sink: &mut S,
) -> Result<(), io::Error>
where
    S: FnMut(&str, Option<&str>) -> Result<(), io::Error>,
{
    let mut rows: Rows = stmt.query([key]).map_err(io::Error::other)?;

    let orow: Option<&Row> = rows.next().map_err(io::Error::other)?;
    let Some(row) = orow else { return Ok(()) };

    let vval: ValueRef = row.get_ref(0).map_err(io::Error::other)?;
    let oval: Option<&str> = vval.as_str_or_null().map_err(io::Error::other)?;

    sink(key, oval)?;

    Ok(())
}

pub fn brdr2keys2sqlite2values2sink<R, S>(
    stmt: &mut Statement,
    mut brdr: R,
    sink: &mut S,
) -> Result<(), io::Error>
where
    R: BufRead,
    S: FnMut(&str, Option<&str>) -> Result<(), io::Error>,
{
    let mut buf: String = String::default();
    loop {
        buf.clear();
        let cnt: usize = brdr.read_line(&mut buf)?;
        if 0 == cnt {
            return Ok(());
        }

        let nolr: &str = buf.trim_ascii_end();

        sqlite2key2val2sink(stmt, nolr, sink)?;
    }
}

pub fn wtr2sink4json<W>(mut wtr: W) -> impl FnMut(&str, Option<&str>) -> Result<(), io::Error>
where
    W: Write,
{
    move |key: &str, oval: Option<&str>| {
        serde_json::to_writer(&mut wtr, &(key, oval))?;
        writeln!(&mut wtr)?;
        Ok(())
    }
}

pub fn brdr2keys2path2sqlite2vals2sink<P, R, S>(
    path2sqlite: P,
    brdr: R,
    sink: &mut S,
) -> Result<(), io::Error>
where
    P: AsRef<Path>,
    R: BufRead,
    S: FnMut(&str, Option<&str>) -> Result<(), io::Error>,
{
    let conn: Connection = Connection::open(path2sqlite).map_err(io::Error::other)?;
    let mut stmt: Statement = conn
        .prepare(
            r#"
                SELECT val
                FROM kvpairs
                WHERE key=?1
            "#,
        )
        .map_err(io::Error::other)?;
    brdr2keys2sqlite2values2sink(&mut stmt, brdr, sink)?;
    stmt.finalize().map_err(io::Error::other)?;
    Ok(())
}

pub fn brdr2keys2path2sqlite2vals2wtr4json<P, R, W>(
    path2sqlite: P,
    brdr: R,
    mut wtr: W,
) -> Result<(), io::Error>
where
    P: AsRef<Path>,
    R: BufRead,
    W: Write,
{
    let mut sink = wtr2sink4json(&mut wtr);
    brdr2keys2path2sqlite2vals2sink(path2sqlite, brdr, &mut sink)?;
    drop(sink);
    wtr.flush()
}

pub fn stdin2keys2path2sqlite2vals2stdout4json<P>(path2sqlite: P) -> Result<(), io::Error>
where
    P: AsRef<Path>,
{
    let o = io::stdout();
    let mut ol = o.lock();
    brdr2keys2path2sqlite2vals2wtr4json(path2sqlite, io::stdin().lock(), BufWriter::new(&mut ol))?;
    ol.flush()
}
