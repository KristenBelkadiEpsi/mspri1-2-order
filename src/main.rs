use model::{CreateOrderDTO, OrderModel, UpdateOrderDTO};

use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use chrono::DateTime;
use dotenv::dotenv;
use orders::model;
/* use postgres::{Client, Error, NoTls}; */
use serde::Deserialize;
use std::str::FromStr;
use tokio_postgres::{Client, Connection, Error, NoTls};
use uuid::Uuid;

#[derive(Deserialize)]
struct Pagination {
    page: i32,
    per_page: i32,
}

async fn get_orders(pagination: web::Query<Pagination>) -> impl Responder {
    let mut client = get_connection().await.unwrap();

    let offset = (pagination.page - 1) * pagination.per_page;
    let query = format!(
        "SELECT * FROM orders LIMIT {} OFFSET {}",
        pagination.per_page, offset
    );

    let mut orders = Vec::new();
    for row in client.query(query.as_str(), &[]).await.unwrap() {
        let order = OrderModel {
            id: Uuid::from_str(row.get("id")).unwrap(),
            created_at: DateTime::parse_from_rfc3339(row.get("date_created"))
                .unwrap()
                .to_utc(),
            customer_id: Uuid::from_str(row.get("customer_id")).unwrap(),
        };
        orders.push(order);
    }

    HttpResponse::Ok().json(orders)
}

async fn get_order_by_id(order_id: web::Path<i32>) -> impl Responder {
    let client = get_connection().await.unwrap();

    let query = format!("SELECT * FROM orders WHERE id = {}", order_id);
    let row = client.query_one(query.as_str(), &[]).await.unwrap();

    let order = OrderModel {
        id: Uuid::from_str(row.get("id")).unwrap(),
        created_at: DateTime::parse_from_rfc3339(row.get("date_created"))
            .unwrap()
            .to_utc(),
        customer_id: Uuid::from_str(row.get("customer_id")).unwrap(),
    };

    HttpResponse::Ok().json(order)
}

async fn create_order(new_order: web::Json<CreateOrderDTO>) -> impl Responder {
    let client = get_connection().await.unwrap();
    let new_uuid = Uuid::new_v4();
    let query = "INSERT INTO orders (id, created_at, customer_id) VALUES ($1, $2, $3) RETURNING id";
    let result = client
        .query_one(
            query,
            &[
                &new_uuid.to_string(),
                &new_order.created_at.to_rfc3339(),
                &new_order.customer_id.to_string(),
            ],
        )
        .await
        .unwrap();

    HttpResponse::Ok().body(format!(
        "Created order with id: {}",
        result.get::<usize, i32>(0)
    ))
}

async fn update_order(
    order_id: web::Path<i32>,
    updated_order: web::Json<UpdateOrderDTO>,
) -> impl Responder {
    let client = get_connection().await.unwrap();

    let query = "UPDATE orders SET created_at = $1, customer_id = $2 WHERE id = $3";
    client
        .execute(
            query,
            &[
                &updated_order.created_at.to_rfc3339(),
                &updated_order.customer_id.to_string(),
                &order_id.to_string(),
            ],
        )
        .await
        .unwrap();

    HttpResponse::Ok().body(format!("Updated order with id: {}", order_id))
}

async fn delete_order(order_id: web::Path<i32>) -> impl Responder {
    let mut client = get_connection().await.unwrap();

    let query = "DELETE FROM orders WHERE id = $1";
    client
        .execute(query, &[&order_id.to_string()])
        .await
        .unwrap();

    HttpResponse::Ok().body(format!("Deleted order with id: {}", order_id))
}

pub async fn init_database() -> Result<(), Error> {
    let client = get_connection().await.unwrap();
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS Order (
    id UUID PRIMARY KEY,
    created_at TIMESTAMP,
    customer_id UUID
    )",
            &[],
        )
        .await
        .unwrap();
    Ok(())
}
pub async fn get_connection() -> Result<Client, Error> {
    dotenv().ok();
    let url = std::env::var("DATABASE_URL").unwrap();
    let (client, connection) = tokio_postgres::connect(&url, NoTls).await.unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    Ok(client)
    /*  Client::connect(&url, NoTls) */
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_database().await.unwrap();
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
        App::new()
            .wrap(cors)
            .route("/orders", web::get().to(get_orders))
            .route("/orders/{id}", web::get().to(get_order_by_id))
            .route("/orders", web::post().to(create_order))
            .route("/orders/{id}", web::put().to(update_order))
            .route("/orders/{id}", web::delete().to(delete_order))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
