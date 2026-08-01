use actix_web::{web, HttpResponse, Result};
use serde_json::json;
use std::sync::Arc;
use parking_lot::Mutex;
use log::error;

use crate::db::Database;
use crate::models::UpdateSettingsRequest;

pub async fn get_settings(
    db: web::Data<Arc<Mutex<Database>>>,
) -> Result<HttpResponse> {
    let db = db.lock();
    match db.get_settings() {
        Ok(settings) => Ok(HttpResponse::Ok().json(settings)),
        Err(e) => {
            error!("Failed to get settings: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({"error": "Failed to get settings"})))
        }
    }
}

pub async fn update_settings(
    db: web::Data<Arc<Mutex<Database>>>,
    req: web::Json<UpdateSettingsRequest>,
) -> Result<HttpResponse> {
    let db = db.lock();
    
    // Get current settings and update with new values
    let mut settings = match db.get_settings() {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to get settings: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({"error": "Failed to get settings"})));
        }
    };

    if let Some(env_vars) = &req.default_env_vars {
        settings.default_env_vars = env_vars.clone();
    }
    if let Some(params) = &req.default_fastchess_params {
        settings.default_fastchess_params = params.clone();
    }
    if let Some(path) = &req.compiled_engines_path {
        settings.compiled_engines_path = path.clone();
    }
    if let Some(path) = &req.lora_repo_path {
        settings.lora_repo_path = path.clone();
    }
    if let Some(path) = &req.fastchess_path {
        settings.fastchess_path = path.clone();
    }
    if let Some(webhook) = &req.default_discord_webhook {
        settings.default_discord_webhook = Some(webhook.clone());
    }

    if let Err(e) = db.set_settings(&settings) {
        error!("Failed to set settings: {}", e);
        return Ok(HttpResponse::InternalServerError().json(json!({"error": "Failed to set settings"})));
    }

    Ok(HttpResponse::Ok().json(settings))
}
