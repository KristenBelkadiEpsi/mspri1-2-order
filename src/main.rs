use deadpool_postgres::{Config, Pool};
use model::{CreateOrderDTO, OrderModel, UpdateOrderDTO};

use actix_cors::Cors;
use actix_web::{
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use dotenv::dotenv;
use orders::model;
use serde::Deserialize;
use serde_json::json;
use tokio_postgres::{Error, NoTls};
use uuid::Uuid;

#[cfg(test)]
mod integration_test;
#[derive(Deserialize)]

struct Pagination {
    page: i32,
    per_page: i32,
}

async fn get_orders(app_data: Data<Pool>, pagination: web::Query<Pagination>) -> impl Responder {
    let client = app_data.get().await.unwrap();

    let offset = (pagination.page - 1) * pagination.per_page;

    let query = format!(
        "SELECT * FROM orders LIMIT {} OFFSET {}",
        pagination.per_page, offset
    );

    let total_count: i64 = client
        .query_one(r#"SELECT COUNT(*) AS total_count FROM "orders""#, &[])
        .await
        .unwrap()
        .get("total_count");
    let mut orders = Vec::new();
    for row in client.query(query.as_str(), &[]).await.unwrap() {
        let order = OrderModel {
            id: row.get("id"),
            created_at: row.get("created_at"),

            customer_id: row.get("customer_id"),
        };
        orders.push(order);
    }

    HttpResponse::Ok().json(json!({"value":orders,"total_count":total_count}))
}

async fn get_order_by_id(app_data: Data<Pool>, order_id: web::Path<Uuid>) -> impl Responder {
    let client = app_data.get().await.unwrap();

    let row = client
        .query_one("SELECT * FROM orders WHERE id = $1", &[&*order_id])
        .await
        .unwrap();

    let order = OrderModel {
        id: row.get("id"),
        created_at: row.get("created_at"),
        customer_id: row.get("customer_id"),
    };

    HttpResponse::Ok().json(order)
}

async fn create_order(
    app_data: Data<Pool>,
    new_order: web::Json<CreateOrderDTO>,
) -> impl Responder {
    let client = app_data.get().await.unwrap();
    let new_uuid = Uuid::new_v4();
    let query = "INSERT INTO orders (id, created_at, customer_id) VALUES ($1, $2, $3) RETURNING id";
    let result = client
        .query_one(
            query,
            &[&new_uuid, &new_order.created_at, &new_order.customer_id],
        )
        .await
        .unwrap();

    HttpResponse::Ok().body(format!(
        "Created order with id: {}",
        result.get::<usize, Uuid>(0)
    ))
}

async fn update_order(
    app_data: Data<Pool>,
    order_id: web::Path<Uuid>,
    updated_order: web::Json<UpdateOrderDTO>,
) -> impl Responder {
    let client = app_data.get().await.unwrap();

    let query = "UPDATE orders SET created_at = $1, customer_id = $2 WHERE id = $3";
    client
        .execute(
            query,
            &[
                &updated_order.created_at,
                &updated_order.customer_id,
                &*order_id,
            ],
        )
        .await
        .unwrap();

    HttpResponse::Ok().body(format!("Updated order with id: {}", order_id))
}

async fn delete_order(app_data: Data<Pool>, order_id: web::Path<Uuid>) -> impl Responder {
    let client = app_data.get().await.unwrap();

    client
        .execute("DELETE FROM orders WHERE id = $1", &[&*order_id])
        .await
        .unwrap();

    HttpResponse::Ok().body(format!("Deleted order with id: {}", order_id))
}

pub fn get_pool() -> Result<Pool, Error> {
    dotenv().ok();
    let url = std::env::var("DATABASE_URL").unwrap();
    let mut cfg = Config::new();
    cfg.url = Some(url);

    let pool = cfg.create_pool(None, NoTls).unwrap();
    Ok(pool)
}
pub fn get_pool_test() -> Result<Pool, Error> {
    dotenv().ok();
    let url = std::env::var("DATABASE_URL_TEST").unwrap();
    let mut cfg = Config::new();
    cfg.url = Some(url);
    let pool = cfg.create_pool(None, NoTls).unwrap();
    Ok(pool)
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = get_pool().unwrap();

    let conn = pool.get().await.unwrap();
    conn.execute(
        r#"CREATE TABLE IF NOT EXISTS orders (
    id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ,
    customer_id UUID
    )"#,
        &[],
    )
    .await
    .unwrap();
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
        App::new()
            .wrap(cors)
            .app_data(Data::new(pool.clone()))
            .route("/orders", web::get().to(get_orders))
            .route("/orders/{id}", web::get().to(get_order_by_id))
            .route("/orders", web::post().to(create_order))
            .route("/orders/{id}", web::put().to(update_order))
            .route("/orders/{id}", web::delete().to(delete_order))
    })
    .bind(("0.0.0.0", 8085))?
    .run()
    .await
}
