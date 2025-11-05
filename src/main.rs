use axum::{
    routing::get,Router,response::Json
};
use axum::{extract::{State},response::Json as JsonResponse};
use rusqlite::{Connection, Result};
use serde::{Serialize,Deserialize};
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

#[derive (Deserialize)]
struct NewUser{
    name: String,
    age: i32,
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


async fn add_user(State(state):State<AppState>,
Json(new_user):Json<NewUser>,)->JsonResponse<serde_json::Value> {
    let conn = state.db.lock().unwrap();

    conn.execute(
        "INSERT INTO users(name,age)VALUES(?1,?2)", (&new_user.name, &new_user.age),
    )
        .unwrap();

    JsonResponse(serde_json::json!({
    "status":"User added successfully!"
    }))
}
#[tokio::main]
async fn main()->Result<()> {

    let conn=Connection::open("database.db")?;
    println!("Connection to SQlite established");

    let state=AppState{
        db:Arc::new(Mutex::new(conn)),
    };

    let app=Router::new()
    .route("/users", get(list_users).post(add_user))
        .with_state(state);


    let addr=SocketAddr::from(([127,0,0,1], 3000));

    println!("Listening on http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(),app)
    .await
    .unwrap();
    Ok(())
}

