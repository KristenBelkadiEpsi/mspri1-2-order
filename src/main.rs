use model::{CreateOrderDTO, OrderModel, UpdateOrderDTO};

use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use chrono::DateTime;
use dotenv::dotenv;
use orders::model;
use serde::Deserialize;
use serde_json::json;
use std::str::FromStr;
use tokio_postgres::{Client, Error, NoTls};
use uuid::Uuid;

#[derive(Deserialize)]
struct Pagination {
    page: i32,
    per_page: i32,
}

async fn get_orders(pagination: web::Query<Pagination>) -> impl Responder {
    let client = get_connection().await.unwrap();

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
            id: Uuid::from_str(row.get("id")).unwrap(),
            created_at: DateTime::parse_from_rfc3339(row.get("created_at"))
                .unwrap()
                .to_utc(),
            customer_id: Uuid::from_str(row.get("customer_id")).unwrap(),
        };
        orders.push(order);
    }

    HttpResponse::Ok().json(json!({"value":orders,"total_count":total_count}))
}

async fn get_order_by_id(order_id: web::Path<Uuid>) -> impl Responder {
    let client = get_connection().await.unwrap();

    let row = client
        .query_one(r#"SELECT * FROM "orders" WHERE id = $1"#, &[&*order_id])
        .await
        .unwrap();

    let order = OrderModel {
        id: row.get("id"),
        created_at: row.get("created_at"),
        customer_id: row.get("customer_id"),
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
            &[&new_uuid, &new_order.created_at, &new_order.customer_id],
        )
        .await
        .unwrap();

    HttpResponse::Ok().body(format!(
        "Created order with id: {}",
        result.get::<usize, i32>(0)
    ))
}

async fn update_order(
    order_id: web::Path<Uuid>,
    updated_order: web::Json<UpdateOrderDTO>,
) -> impl Responder {
    let client = get_connection().await.unwrap();

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

async fn delete_order(order_id: web::Path<Uuid>) -> impl Responder {
    let client = get_connection().await.unwrap();

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
            r#"CREATE TABLE IF NOT EXISTS orders (
    id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ,
    customer_id UUID
    )"#,
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
    .bind(("localhost", 8080))?
    .run()
    .await
}
#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use chrono::Utc;
    use uuid::Uuid;

    async fn setup_test_app() -> impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    > {
        test::init_service(
            App::new()
                .route("/orders", web::get().to(get_orders))
                .route("/orders/{id}", web::get().to(get_order_by_id))
                .route("/orders", web::post().to(create_order))
                .route("/orders/{id}", web::put().to(update_order))
                .route("/orders/{id}", web::delete().to(delete_order)),
        )
        .await
    }

    #[actix_rt::test]
    async fn test_get_orders() {
        let app = setup_test_app().await;
        let req = test::TestRequest::get()
            .uri("/orders?page=1&per_page=10")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_get_order_by_id() {
        let app = setup_test_app().await;
        let req = test::TestRequest::get().uri("/orders/1").to_request();
        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp.status());
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_create_order() {
        let app = setup_test_app().await;
        let new_order = CreateOrderDTO {
            created_at: Utc::now(),
            customer_id: Uuid::new_v4(),
        };
        let req = test::TestRequest::post()
            .uri("/orders")
            .set_json(&new_order)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_update_order() {
        let app = setup_test_app().await;
        let updated_order = UpdateOrderDTO {
            created_at: Utc::now(),
            customer_id: Uuid::new_v4(),
        };
        let req = test::TestRequest::put()
            .uri("/orders/1")
            .set_json(&updated_order)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_delete_order() {
        let app = setup_test_app().await;
        let req = test::TestRequest::delete().uri("/orders/1").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[tokio::test]
    async fn test_init_database() {
        let result = init_database().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_connection() {
        let result = get_connection().await;
        assert!(result.is_ok());
    }
}
