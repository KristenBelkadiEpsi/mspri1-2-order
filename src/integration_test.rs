use super::*;
use actix_http::StatusCode;
use actix_web::{test, web, App};
use chrono::{Days, Utc};
use uuid::Uuid;

async fn refresh_table_test() {
    let pool_test = get_pool_test().unwrap();
    let conn = pool_test.get().await.unwrap();
    conn.execute("DROP TABLE IF EXISTS orders", &[])
        .await
        .unwrap();
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
}

#[actix_rt::test]
async fn get_order_by_id_test() {
    //GIVEN
    refresh_table_test().await;
    let pool_test = get_pool_test().unwrap();
    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool_test.clone()))
            .route("/orders", web::get().to(get_orders))
            .route("/orders/{id}", web::get().to(get_order_by_id))
            .route("/orders", web::post().to(create_order))
            .route("/orders/{id}", web::put().to(update_order))
            .route("/orders/{id}", web::delete().to(delete_order)),
    )
    .await;
    let conn = pool_test.get().await.unwrap();
    let order = OrderModel {
        id: Uuid::new_v4(),
        created_at: Utc::now(),
        customer_id: Uuid::new_v4(),
    };
    conn.execute(
        "INSERT INTO orders VALUES ($1, $2, $3)",
        &[&order.id, &order.created_at, &order.customer_id],
    )
    .await
    .unwrap();
    //WHEN
    let req = test::TestRequest::get()
        .uri(&format!("/orders/{}", order.id))
        .to_request();
    let rep = test::call_service(&app, req).await;
    //ASSERT
    assert_eq!(rep.status(), StatusCode::OK);
}
#[actix_rt::test]
async fn get_orders_test() {
    //GIVEN
    refresh_table_test().await;
    let pool_test = get_pool_test().unwrap();
    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool_test.clone()))
            .route("/orders", web::get().to(get_orders))
            .route("/orders/{id}", web::get().to(get_order_by_id))
            .route("/orders", web::post().to(create_order))
            .route("/orders/{id}", web::put().to(update_order))
            .route("/orders/{id}", web::delete().to(delete_order)),
    )
    .await;
    let conn = pool_test.get().await.unwrap();

    for _ in 1..=100 {
        let order = OrderModel {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            customer_id: Uuid::new_v4(),
        };
        conn.execute(
            "INSERT INTO orders VALUES ($1, $2, $3)",
            &[&order.id, &order.created_at, &order.customer_id],
        )
        .await
        .unwrap();
    }

    //WHEN
    let req = test::TestRequest::get()
        .uri("/orders?page=1&per_page=10")
        .to_request();
    let rep = test::call_service(&app, req).await;
    //ASSERT
    assert_eq!(rep.status(), StatusCode::OK);
}
#[actix_rt::test]
async fn delete_order_test() {
    //GIVEN
    refresh_table_test().await;
    let pool_test = get_pool_test().unwrap();
    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool_test.clone()))
            .route("/orders", web::get().to(get_orders))
            .route("/orders/{id}", web::get().to(get_order_by_id))
            .route("/orders", web::post().to(create_order))
            .route("/orders/{id}", web::put().to(update_order))
            .route("/orders/{id}", web::delete().to(delete_order)),
    )
    .await;
    let conn = pool_test.get().await.unwrap();

    let order = OrderModel {
        id: Uuid::new_v4(),
        created_at: Utc::now(),
        customer_id: Uuid::new_v4(),
    };
    conn.execute(
        "INSERT INTO orders VALUES ($1, $2, $3)",
        &[&order.id, &order.created_at, &order.customer_id],
    )
    .await
    .unwrap();

    //WHEN
    let req = test::TestRequest::delete()
        .uri(&format!("/orders/{}", order.id))
        .to_request();
    let rep = test::call_service(&app, req).await;
    //ASSERT
    assert_eq!(rep.status(), StatusCode::OK);
}
#[actix_rt::test]
async fn update_order_test() {
    //GIVEN
    refresh_table_test().await;
    let pool_test = get_pool_test().unwrap();
    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool_test.clone()))
            .route("/orders", web::get().to(get_orders))
            .route("/orders/{id}", web::get().to(get_order_by_id))
            .route("/orders", web::post().to(create_order))
            .route("/orders/{id}", web::put().to(update_order))
            .route("/orders/{id}", web::delete().to(delete_order)),
    )
    .await;
    let conn = pool_test.get().await.unwrap();

    let order = OrderModel {
        id: Uuid::new_v4(),
        created_at: Utc::now(),
        customer_id: Uuid::new_v4(),
    };
    conn.execute(
        "INSERT INTO orders VALUES ($1, $2, $3)",
        &[&order.id, &order.created_at, &order.customer_id],
    )
    .await
    .unwrap();
    //WHEN
    let updated_order = UpdateOrderDTO {
        created_at: Utc::now().checked_add_days(Days::new(10)).unwrap(),
        customer_id: order.customer_id,
    };
    let req = test::TestRequest::put()
        .uri(&format!("/orders/{}", order.id))
        .set_json(updated_order)
        .to_request();
    let rep = test::call_service(&app, req).await;
    //ASSERT
    assert_eq!(rep.status(), StatusCode::OK);
}
#[actix_rt::test]
async fn create_order_test() {
    //GIVEN
    refresh_table_test().await;
    let pool_test = get_pool_test().unwrap();
    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool_test.clone()))
            .route("/orders", web::get().to(get_orders))
            .route("/orders/{id}", web::get().to(get_order_by_id))
            .route("/orders", web::post().to(create_order))
            .route("/orders/{id}", web::put().to(update_order))
            .route("/orders/{id}", web::delete().to(delete_order)),
    )
    .await;
    //WHEN
    let create_order_dto = CreateOrderDTO {
        created_at: Utc::now(),
        customer_id: Uuid::new_v4(),
    };
    let req = test::TestRequest::post()
        .uri("/orders")
        .set_json(create_order_dto)
        .to_request();
    let rep = test::call_service(&app, req).await;
    //ASSERT
    assert_eq!(rep.status(), StatusCode::OK);
}
