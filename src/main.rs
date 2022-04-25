use axum::{
    extract,
    routing::get,
    Router,
    response::Html,
    Json
};
use serde::{Serialize, Deserialize};
use mongodb::{Client, options::ClientOptions, Database};

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    let client = spawn_db().await?;
    let db = client.database("todo");
    //for db_name in db.list_collection_names(None).await? {
    //    println!("{}", db_name);
    //}


    let app = Router::new()
        .route("/", get(hello_handler))
        .route("/dynamic/:id", get(dynamic).post(post_dynamic))
        .layer(extract::Extension(db));

    axum::Server::bind(&"0.0.0.0:3001".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}

async fn spawn_db() -> mongodb::error::Result<Client> {
    let mut client_option = ClientOptions::parse("mongodb://localhost:27017").await?;

    client_option.app_name = Some("My App".to_string());

    let client = Client::with_options(client_option)?;

    Ok(client)
}

async fn hello_handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}


async fn dynamic(extract::Path(id): extract::Path<String>) -> Html<String> {
    Html(format!("<h1>The path parameter was: {}", id))
}

#[derive(Serialize, Deserialize, Debug)]
struct Item {
    text: String,
    complete: bool,
}

async fn post_dynamic(
    extract::Extension(db) : extract::Extension<Database>,
    extract::Path(id): extract::Path<String>, extract::Json(payload) : extract::Json<Item>
) -> Json<Item> {
   let return_item = Item {text: payload.text, complete: payload.complete};
    insert_db(&db, &return_item).await.unwrap();
    println!("Success deserializing {:#?}", return_item);
    Json(return_item)
}

async fn insert_db(db: &Database, item: &Item) -> mongodb::error::Result<()> {
    let typed_collection = db.collection::<Item>("items");
    typed_collection.insert_one(item, None).await?;
    Ok(())
}
