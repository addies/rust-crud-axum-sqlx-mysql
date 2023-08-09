use axum::{
    extract::{Path, State},
    routing::{get, post, delete, put},
    Router,
    http::StatusCode,
    response::IntoResponse,
    Json
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower_http::cors::{Any, CorsLayer};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};



#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
#[allow(non_snake_case)]
pub struct MyTable {
    nomer: i64,
    nama: String,
    alamat: String,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct MyTableInsert {
    nama: String,
    alamat: String,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct MyTableUpdate {
    nama: String,
    alamat: String,
}



async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Simple CRUD API with Rust, SQLX, MySQL,and Axum";
    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}

async fn getall_mytable (State(pool):State<MySqlPool>) 
    -> Result<Json<Vec<MyTable>>, (StatusCode, String)> {
    let string_query = "SELECT * FROM mytable";
    let result = sqlx::query_as(string_query)
        .fetch_all(&pool)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, format!("Error is: {}", err)))?;

    Ok(Json(result))    
}

async fn get_mytable (State(pool):State<MySqlPool>, Path(nomer): Path<i64>) 
    -> Result<Json<Vec<MyTable>>, (StatusCode, String)> {
    let string_query = "SELECT * FROM mytable WHERE nomer = $1";
    let result = sqlx::query_as(string_query)
        .bind(nomer)
        .fetch_all(&pool)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, format!("Error is: {}", err)))?;

    Ok(Json(result))   
}

async fn create_mytable( State(pool):State<MySqlPool>, Json(my_table):Json<MyTableInsert>) 
    -> Result<Json<Value>, (StatusCode, String)> {
   // let string_query = "INSERT INTO mytable (nomer,nama,alamat) VALUES ($1, $2)";
    let string_query = "INSERT INTO mytable (nama,alamat) VALUES (?, ?)";
    let _result = sqlx::query(string_query)
    //.bind(&my_table.nomer)
        .bind(&my_table.nama.to_string())
        .bind(&my_table.alamat.to_string())
        .execute(&pool)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, format!("Error is: {}", err)))?;

    Ok(Json(json!({"msg": "Data inserted successfully"})))
}

async fn delete_mytable (State(pool):State<MySqlPool>, Path(nomer): Path<i64>) 
    -> Result<Json<Vec<MyTable>>, (StatusCode, String)> {
    let string_query = "DELETE FROM mytable WHERE nomer = ?";
    let result = sqlx::query_as(string_query)
        .bind(nomer)
        .fetch_all(&pool)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, format!("Error is: {}", err)))?;

    Ok(Json(result))   
}

async fn update_mytable( State(pool):State<MySqlPool>, Path(nomer): Path<i64>, Json(my_table):Json<MyTableUpdate>) 
    -> Result<Json<Value>, (StatusCode, String)> {
    let string_query = "UPDATE mytable SET nama = ?, alamat = ? WHERE nomer = ?";
    let _result = sqlx::query(string_query)
        .bind(&my_table.nama.to_string())
        .bind(&my_table.alamat.to_string())
        .bind(nomer)
        .execute(&pool)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, format!("Error is: {}", err)))?;

    Ok(Json(json!({"msg": "Data updated successfully"})))
}


#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    
    // initialize tracing
    tracing_subscriber::fmt::init();

    let cors: CorsLayer = CorsLayer::new()
        .allow_origin(Any);

    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool = match MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url) 
        .await
        {
            Ok(pool) => {
                println!("✅Connection to the database is successful!");
                pool
            }
            Err(err) => {
                println!("🔥 Failed to connect to the database: {:?}", err);
                std::process::exit(1);
            }
        };

    println!("🚀 Server started successfully");

    // build our application with a route
    let app = Router::new()
        .route("/", get(health_checker_handler))
        .route("/health_checker_handler", get(health_checker_handler))
        .route("/api/mytable", get(getall_mytable))
        .route("/api/mytable", post(create_mytable))
        .route("/api/mytable/:nomer", get(get_mytable))
        .route("/api/mytable/:nomer", delete(delete_mytable))
        .route("/api/mytable/:nomer", put(update_mytable))
        .with_state(pool)
        .layer(cors);
    
    println!("Listening on port {}", "0.0.0.0:3000");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

