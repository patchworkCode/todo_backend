use axum::{
    extract,
    routing::get,
    Router,
    response::{Html,IntoResponse},
    Json, http::StatusCode
};
use serde::{Serialize, Deserialize};
use mongodb::{Client,options::ClientOptions,bson::doc, Database, results::DeleteResult};
use futures::{TryStreamExt, TryFutureExt};
use uuid::Uuid;
use bson::serde_helpers::uuid_as_binary;
use std::collections::HashMap;


#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    let client = spawn_db().await?;
    let db = client.database("todo");
    //for db_name in db.list_collection_names(None).await? {
    //    println!("{}", db_name);
    //}


    let app = Router::new()
        .route("/", get(hello_handler))
        .route("/todo/", get(get_all_todo).post(post_one_todo).put(replace_one_todo).delete(delete_one_todo))
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


async fn get_all_todo(
    extract::Extension(db) : extract::Extension<Database>) -> Json<Vec<PayloadItem>> {
    let all_items = retrieve_all(&db).await.unwrap();
    Json(all_items)
}

async fn retrieve_all(db: &Database) -> mongodb::error::Result<Vec<PayloadItem>> {
    let mut all_items: Vec<PayloadItem> = vec![];
    let typed_collection = db.collection::<PayloadItem>("items");
    let mut cursor = typed_collection.find(None, None).await?;

    while let Some(item) = cursor.try_next().await? {
        all_items.push(item);
    };

    Ok(all_items)
}

#[derive(Serialize, Deserialize, Debug)]
struct PayloadItem {
    text: String,
    complete: bool,
}

impl From <Item> for PayloadItem {
    fn from(i: Item) -> Self {
        Self {
            text : i.text,
            complete : i.complete,
        } 
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Item {
    #[serde(with = "uuid_as_binary")]
    id : Uuid,
    text : String,
    complete : bool
}

impl From<PayloadItem> for Item {
    fn from(p: PayloadItem) -> Self {
        Self {
            id : Uuid::new_v4(),
            text  : p.text,
            complete : p.complete,
        }
    }
}

async fn post_one_todo(
    extract::Extension(db) : extract::Extension<Database>,
    extract::Json(payload) : extract::Json<PayloadItem>
) -> Json<PayloadItem> {
   let return_item = Item::from(payload);
    insert_db(&db,&return_item).await.unwrap();
    println!("Success deserializing {:#?}", return_item);
    Json(PayloadItem::from(return_item))
}

async fn insert_db(db: &Database, item: &Item) -> mongodb::error::Result<()> {
    let typed_collection = db.collection::<Item>("items");
    typed_collection.insert_one(item, None).await?;
    Ok(())
}

async fn replace_one_todo(
    extract::Query(params) : extract::Query<HashMap<String, String>>,
    extract::Extension(db) : extract::Extension<Database>,
    extract::Json(payload) : extract::Json<PayloadItem>
) -> Json<PayloadItem> {
    println!("{:#?}", params);
    let item = retrieve_one(&db, &params, payload).await.unwrap();
    Json(PayloadItem::from(item))
}

async fn retrieve_one (db : &Database, id: &HashMap<String, String>, item: PayloadItem) -> mongodb::error::Result<Item> {
    let typed_collection = db.collection::<Item>("items");
    let id = id.get("id").unwrap();
    let id_as_uuid : Uuid = Uuid::parse_str(&id[..]).unwrap();
    let filter = doc! {"id" : id_as_uuid};
    let update = doc! {"$set": {"text" : item.text, "complete" : item.complete}};
    let cursor = typed_collection.find_one_and_update(filter, update, None).await?;
    Ok(cursor.unwrap())

}

async fn delete_one_todo(
    extract::Query(params) : extract::Query<HashMap<String, String>>,
    extract::Extension(db) : extract::Extension<Database>,
) -> impl IntoResponse {
    let id;
    match params.get("id") {
        Some(value) => id = value,
        None => return StatusCode::NOT_FOUND,
    }
    let id_as_uuid : Uuid = Uuid::parse_str(&id[..]).unwrap();
    let deleted_count = delete_from_db(&db, &id_as_uuid).await.unwrap();
    StatusCode::OK
    
}

async fn delete_from_db(db: &Database, id: &Uuid) -> mongodb::error::Result<DeleteResult> {
    let typed_collection = db.collection::<Item>("items");
    let filter = doc! {"id" : id};
    let cursor = typed_collection.delete_one(filter, None).await?;
    Ok(cursor)

}
