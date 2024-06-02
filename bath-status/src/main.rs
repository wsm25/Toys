mod database;
mod webapi;

use std::time::SystemTime;

use database::BathTrunk;
use rusqlite::Result;

const LOOPS:u32=1_000_000;

fn _main()->Result<()> {
    let conn=rusqlite::Connection::open_in_memory()?;
    let mut db=database::DataBase::new(&conn)?;
    for _ in 0..LOOPS{
        db.ins_bathtrunk(&BathTrunk{
            id:0, building: "114".to_string(), cord:(1,2)
        })?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let begin=SystemTime::now();
    _main()?;
    let end=SystemTime::now();
    println!("{:?}/op", end.duration_since(begin).unwrap()/LOOPS);
    Ok(())
}