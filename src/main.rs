use axum::{
    routing::get,Router,response::Json ,extract::State
};
use rusqlite::{Connection, Result};
use serde::Serialize;
use std::sync::{Arc,Mutex};
use std::net::SocketAddr;


#[derive(Clone)]
struct AppState{
    db: Arc<Mutex<Connection>>,
}

#[derive(Serialize)]
struct User{
    id: i32,
    name: String,
    age:i32,
}


async fn list_users(State(state):State<AppState>)->Json<Vec<User>>{
    let conn=state.db.lock().unwrap();
    let mut stmt=conn.prepare("SELECT id, name, age FROM users").unwrap();
    let rows=stmt.query_map([],|row|{
        Ok(User{
            id:row.get(0)?,
            name:row.get(1)?,
            age:row.get(2)?,
        })
    })
    .unwrap();

    let users:Vec<User>=rows.map(|u|u.unwrap()).collect();

    Json(users)
}


#[tokio::main]
async fn main()->Result<()> {

    let conn=Connection::open("database.db")?;
    println!("Connection to SQlite established");

    let state=AppState{
        db:Arc::new(Mutex::new(conn)),
    };

    let app=Router::new()
    .route("/users", get(list_users))
        .with_state(state);


    let addr=SocketAddr::from(([127,0,0,1], 3000));

    println!("Listening on http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(),app)
    .await
    .unwrap();
    Ok(())
}

