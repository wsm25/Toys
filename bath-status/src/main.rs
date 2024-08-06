mod database;
mod webapi;
use std::time::SystemTime;

#[deprecated = "dead"]
fn main(){
    let conn=rusqlite::Connection::open("1.db").unwrap();
    let mut db=database::DataBase::new(&conn).unwrap();
    const LOOP:u32=10000;
    let begin=SystemTime::now();
    for i in 0..LOOP{
        db.ins_bathtrunk(
            &database::BathTrunk { building: i.to_string(), cord: (0, 0) }
        ).unwrap();
    }
    let end=SystemTime::now();
    println!("insert: {:?}/op", end.duration_since(begin).unwrap()/LOOP);
    for i in 0..LOOP{
        db.query_bathtrunks(&i.to_string()).unwrap().count();
    }
    let end=SystemTime::now();
    println!("select: {:?}/op", end.duration_since(begin).unwrap()/LOOP);
    
}

fn _main(){
    std::thread::spawn(||{
        tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build().unwrap()
            .block_on(
            tokio::task::LocalSet::new()
                .run_until(webapi::daemon())
        )
    }).join().unwrap();
}