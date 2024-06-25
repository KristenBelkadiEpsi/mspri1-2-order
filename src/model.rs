use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize)]
pub struct OrderModel {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub customer_id: Uuid,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct ReadOrderDTO {
    pub created_at: DateTime<Utc>,
    pub customer_id: Uuid,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct CreateOrderDTO {
    pub created_at: DateTime<Utc>,
    pub customer_id: Uuid,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct DeleteOrderDTO {
    pub created_at: DateTime<Utc>,
    pub customer_id: Uuid,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct UpdateOrderDTO {
    pub created_at: DateTime<Utc>,
    pub customer_id: Uuid,
}