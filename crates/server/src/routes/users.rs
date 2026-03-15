use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

pub fn router() -> Router {
    Router::new()
        .route("/user", get(get_user))
        .route("/user/extended", get(get_user))
        .route("/users/extended", get(get_user))
        .route("/user/payment", get(get_payment))
        .route("/user/privacy-settings", get(get_privacy_settings))
        .route("/org", get(get_org))
        .route("/user/shortlinks", get(get_shortlinks))
}

async fn get_user() -> Json<Value> {
    Json(json!({
        "id": "00000000-0000-0000-0000-000000000001",
        "name": "Local User",
        "email": "local@localhost",
        "image": "",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "company": "",
        "discord": "",
        "github": "",
        "phone": "",
        "first_name": "Local",
        "last_name": "User",
        "can_design": true,
        "can_train_on_designs": false,
        "is_service_account": false,
        "block": null,
        "mailchimp_newsletter_consent": "unsubscribed",
    }))
}

async fn get_payment() -> Json<Value> {
    Json(json!({
        "name": "",
        "subscription_id": "",
        "subscription_details": null,
        "plan": "team",
        "features": {
            "can_export": true,
            "can_import": true,
            "max_projects": 999,
        }
    }))
}

async fn get_privacy_settings() -> Json<Value> {
    Json(json!({
        "can_train_on_designs": false,
    }))
}

async fn get_org() -> Json<Value> {
    Json(json!({
        "id": "00000000-0000-0000-0000-000000000002",
        "name": "Local",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "role": "admin",
        "allow_users_in_domain_to_auto_join": false,
        "billing_email": null,
        "block": null,
        "domain": null,
        "image": null,
        "phone": "",
        "stripe_id": null,
        "plan": "team",
    }))
}

async fn get_shortlinks() -> Json<Value> {
    Json(json!([]))
}
