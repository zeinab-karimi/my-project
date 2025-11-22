use axum::{
    routing::get,Router,response::Json
};
use axum::{extract::{State,Path},response::Json as JsonResponse};
use rusqlite::{Connection, Result};
use serde::{Serialize,Deserialize};
use std::sync::{Arc,Mutex};
use std::{env,net::SocketAddr};
use tokio::net::TcpListener;
use dotenvy::dotenv;
use tracing::{info,Level};
use tracing_subscriber::FmtSubscriber;


#[derive(Clone)]
struct AppState{
    db: Arc<Mutex<Connection>>,
}

#[derive(Serialize)]
struct User{
    id:u32,
    name: String,
    age:u8,
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

async fn health()->&'static str{
"ok"
}

async fn root()-> &'static str{
    "Hello from UserService"
}

async fn get_user_by_id(Path(id):Path<i32>,State(state):State<AppState>,)->Result<Json<User>,String>{
    let conn=state.db.lock().unwrap();
    let mut stmt=conn.prepare("SELECT id,name,age FROM users WHERE id=?1")
        .map_err(|_|"Query error".to_string())?;

    let result=stmt.query_row([id],|row|{
        Ok(User{
            id:row.get(0)?,
            name:row.get(1)?,
            age:row.get(2)?,
        })
    });

    match result{
        Ok(user)=>Ok(Json(user)),
        Err(_)=>Err("User not found".to_string()),
    }
}
#[tokio::main]
async fn main()->Result<()> {



    dotenv().ok();
    let subscriber=FmtSubscriber::builder()
        .with_max_level(Level::INFO)
    .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");


    let host=env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    let port:u16=env::var("SERVER_PORT").unwrap_or_else(|_| "4000".to_string()).parse().unwrap();




    let conn=Connection::open("database.db")?;
    println!("Connection to SQlite established");

    let state=AppState{
        db:Arc::new(Mutex::new(conn)),
    };

    let app=Router::new()
        .route("/users/:id",get(get_user_by_id))
        .route("/",get(root))
        .route("/health",get(health))
    .route("/users", get(list_users).post(add_user))
        .with_state(state);


    let addr:SocketAddr=format!("{}:{}",host,port).parse().unwrap();


    println!("Listening on http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(),app)
    .await
    .unwrap();
    Ok(())
}

