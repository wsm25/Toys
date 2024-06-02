use rusqlite::{Connection, Statement, Result};

use chrono::{DateTime, Utc};
use serde::Serialize;

pub struct DataBase<'a>{
    // static safety: stmt is bound to conn
    query_bt: Statement<'a>,
    query_comment: Statement<'a>,
    ins_bt: Statement<'a>,
    ins_comment: Statement<'a>,
}

impl<'a> DataBase<'a>{
    pub fn new(conn: &'a Connection)->Result<Self>{
        // configs
        let _=conn.execute_batch(
            // https://avi.im/blag/2021/fast-sqlite-inserts/
            // 
            // in scenarios where temprate high-speed write is required,
            // we suppose in-memory databases, and backup it to file afterwards. 
            // Reference speed: 2.5us(mem) | 2ms(file) per insert
            "
            PRAGMA locking_mode = EXCLUSIVE;
            PRAGMA temp_store = MEMORY;",
        );
        // try create tables
        let _=conn.execute(
            "CREATE TABLE trunks (
                id       INTEGER PRIMARY KEY,
                building TEXT,
                x        INTEGER,
                y        INTEGER
            )",()
        );
        let _=conn.execute(
            "CREATE INDEX idx_trunks_building on trunks(building)",
            ()
        );
        let _=conn.execute(
            "CREATE TABLE comments (
                id       INTEGER PRIMARY KEY,
                building INTEGER,
                score    INTEGER,
                time     INTEGER,
                message  TEXT
            )", ()
        );
        let _=conn.execute(
            "CREATE INDEX idx_comments_building on comments(building)",
            ()
        );
        // @param building: text
        let query_bt=conn.prepare(
            "SELECT id, building, x, y FROM trunks where building = ?1"
        )?;
        // @param building: int
        let query_comment = conn.prepare(
            "SELECT id, building, score, time, message FROM comments where building = ?1"
        )?;
        // @param text int int
        let ins_bt=conn.prepare(
            "INSERT INTO trunks (building, x, y) VALUES (?1, ?2, ?3)"
        )?;
        // @param building: int int int text
        let ins_comment=conn.prepare(
            "INSERT INTO comments (building, score, time, message) VALUES (?1, ?2, ?3, ?4)"
        )?;
        
        Ok(Self {
            query_bt,
            query_comment,
            ins_bt,
            ins_comment,
        })
    }

    pub fn query_bathtrunks<'b>(&'b mut self, building: &str)
        ->Result<impl Iterator<Item=Result<BathTrunk>>+'b>{
        self.query_bt.query_map([building],|row|{
            Ok(BathTrunk{
                id: row.get(0)?,
                building: row.get(1)?,
                cord: (row.get(2)?, row.get(3)?)
            })
        })
    }

    pub fn query_comments<'b>(&'b mut self, building: i32)
        ->Result<impl Iterator<Item=Result<Comment>>+'b>{
        self.query_comment.query_map([building],|row|{
            Ok(Comment{
                id: row.get(0)?,
                btid: row.get(1)?,
                score: row.get(2)?,
                time: DateTime::from_timestamp_millis(row.get(3)?).ok_or(
                    rusqlite::Error::InvalidColumnType(
                        8,
                        "Chrono error: out-of-range number of milliseconds".to_string(), 
                        rusqlite::types::Type::Integer
                    )
                )?, // TODO: wrap to error
                message: row.get(4)?
            })
        })
    }

    pub fn ins_bathtrunk(& mut self, bt: &BathTrunk)->Result<usize>{
        self.ins_bt.execute((
            &bt.building,
            &bt.cord.0,
            &bt.cord.1
        ))
    }

    pub fn ins_comment(& mut self, c: &Comment)->Result<usize>{
        self.ins_comment.execute((
            &c.btid,
            &c.score,
            &c.time.timestamp_millis(),
            &c.message
        ))
    }
}

#[derive(Debug, )]
pub struct BathTrunk{
    pub id: i32,
    pub building: String,
    pub cord: (i32, i32),
}

#[derive(Debug, )]
pub struct Query<T: Debug+Serialize>{
    pub succeed: bool,
    pub id: i32,
    pub 
    pub building: String,
    pub cord: (i32, i32),
}

pub struct QueryItem<T: Debug+Serialize>{
    pub id:isize,
    
}

#[derive(Debug)]
pub struct Comment{
    pub id: i32,
    pub btid: i32, // bath trunk id
    pub score: i32, // 0-9
    pub time: DateTime<Utc>, // store mili timestamp
    pub message: String,
}

#[derive(Debug)]
pub struct CreateComment{
    pub btid: i32, // bath trunk id
    pub score: i32, // 0-9
    pub time: DateTime<Utc>, // store mili timestamp
    pub message: String,
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn test_db_init()->Result<()>{
        let conn=Connection::open_in_memory()?;
        let _db=DataBase::new(&conn)?;
        Ok(())
    }

    #[test]
    fn test_db()->Result<()>{
        let conn=Connection::open_in_memory()?;
        let mut db=DataBase::new(&conn)?;
        // bt
        db.ins_bathtrunk(&BathTrunk{
            id:0, building: "114".to_string(), cord:(1,2)
        })?;
        assert_eq!(
            db.query_bathtrunks("114")?.next().unwrap()?.cord,
            (1,2)
        );
        // com
        db.ins_comment(&Comment{
            id:0, 
            btid: 0,
            score: 9,
            time: Utc::now(),
            message: "nice bathtrunk!".to_string(),
        })?;
        assert_eq!(
            db.query_comments(0)?.next().unwrap()?.message,
            "nice bathtrunk!"
        );
        Ok(())
    }
}