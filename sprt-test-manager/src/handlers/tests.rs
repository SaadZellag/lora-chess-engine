use actix_web::{web, HttpResponse, Result};
use serde_json::json;
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;
use parking_lot::Mutex;
use log::{info, error};

use crate::db::Database;
use crate::models::{Test, TestStatus, CreateTestRequest};
use crate::config::Config;
use crate::engine;
use crate::fastchess;
use crate::discord;

pub async fn list_tests(
    db: web::Data<Arc<Mutex<Database>>>,
) -> Result<HttpResponse> {
    let db = db.lock();
    match db.list_tests() {
        Ok(tests) => Ok(HttpResponse::Ok().json(tests)),
        Err(e) => {
            error!("Failed to list tests: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({"error": "Failed to list tests"})))
        }
    }
}

pub async fn create_test(
    db_data: web::Data<Arc<Mutex<Database>>>,
    config_data: web::Data<Arc<Config>>,
    req: web::Json<CreateTestRequest>,
) -> Result<HttpResponse> {
    let db = db_data.lock();
    let test_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    // Merge with default settings
    let settings = match db.get_settings() {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to get settings: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({"error": "Failed to get settings"})));
        }
    };

    let env_vars = req.env_vars.clone().unwrap_or_else(|| settings.default_env_vars.clone());
    let fastchess_params = req.fastchess_params.clone();

    let test = Test {
        id: test_id.clone(),
        engine1_ref: req.engine1_ref.clone(),
        engine1_name: req.engine1_name.clone(),
        engine2_ref: req.engine2_ref.clone(),
        engine2_name: req.engine2_name.clone(),
        env_vars,
        fastchess_params,
        status: TestStatus::Pending,
        created_at: now,
        started_at: None,
        finished_at: None,
        games_played: 0,
        games_total: 0,
        discord_webhook: req.discord_webhook.clone().or_else(|| settings.default_discord_webhook.clone()),
        engine1_bin: None,
        engine2_bin: None,
    };

    if let Err(e) = db.create_test(&test) {
        error!("Failed to create test: {}", e);
        return Ok(HttpResponse::InternalServerError().json(json!({"error": "Failed to create test"})));
    }

    // Start compilation in background - clone before dropping db lock
    let db_arc = db_data.get_ref().clone();
    let config_arc = config_data.get_ref().clone();
    let test_clone = test.clone();
    
    drop(db);
    
    tokio::spawn(async move {
        if let Err(e) = run_test(db_arc, config_arc, test_clone).await {
            error!("Test failed: {}", e);
        }
    });

    Ok(HttpResponse::Created().json(&test))
}

pub async fn get_test(
    db: web::Data<Arc<Mutex<Database>>>,
    test_id: web::Path<String>,
) -> Result<HttpResponse> {
    let db = db.lock();
    match db.get_test(&test_id) {
        Ok(Some(test)) => Ok(HttpResponse::Ok().json(test)),
        Ok(None) => Ok(HttpResponse::NotFound().json(json!({"error": "Test not found"}))),
        Err(e) => {
            error!("Failed to get test: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({"error": "Failed to get test"})))
        }
    }
}

pub async fn delete_test(
    db: web::Data<Arc<Mutex<Database>>>,
    test_id: web::Path<String>,
) -> Result<HttpResponse> {
    let db = db.lock();
    match db.update_test_status(&test_id, &TestStatus::Discarded) {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({"status": "discarded"}))),
        Err(e) => {
            error!("Failed to delete test: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({"error": "Failed to delete test"})))
        }
    }
}

pub async fn pause_test(
    db: web::Data<Arc<Mutex<Database>>>,
    test_id: web::Path<String>,
) -> Result<HttpResponse> {
    let db = db.lock();
    match db.update_test_status(&test_id, &TestStatus::Paused) {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({"status": "paused"}))),
        Err(e) => {
            error!("Failed to pause test: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({"error": "Failed to pause test"})))
        }
    }
}

pub async fn resume_test(
    db: web::Data<Arc<Mutex<Database>>>,
    test_id: web::Path<String>,
) -> Result<HttpResponse> {
    let db = db.lock();
    match db.update_test_status(&test_id, &TestStatus::Running) {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({"status": "running"}))),
        Err(e) => {
            error!("Failed to resume test: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({"error": "Failed to resume test"})))
        }
    }
}

pub async fn get_logs(
    db: web::Data<Arc<Mutex<Database>>>,
    test_id: web::Path<String>,
) -> Result<HttpResponse> {
    let db = db.lock();
    match db.get_logs(&test_id, 1000) {
        Ok(logs) => Ok(HttpResponse::Ok().json(logs)),
        Err(e) => {
            error!("Failed to get logs: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({"error": "Failed to get logs"})))
        }
    }
}

async fn run_test(
    db: Arc<Mutex<Database>>,
    _config: Arc<Config>,
    mut test: Test,
) -> anyhow::Result<()> {
    let test_id = test.id.clone();
    
    // Update status to compiling
    {
        let db = db.lock();
        test.status = TestStatus::Compiling;
        db.update_test(&test)?;
    }

    // Notify Discord - test started
    if let Some(webhook) = &test.discord_webhook {
        discord::notify(webhook, &format!("Test started: {} vs {}", test.engine1_name, test.engine2_name)).await.ok();
    }

    // Compile engines
    let settings = {
        let db = db.lock();
        db.get_settings()?
    };

    info!("Compiling engine 1: {}", test.engine1_ref);
    let engine1_bin = match engine::compile(
        &test.engine1_ref,
        &settings,
        &test.env_vars,
    ).await {
        Ok(bin) => bin,
        Err(e) => {
            error!("Failed to compile engine 1: {}", e);
            {
                let db = db.lock();
                test.status = TestStatus::Failed;
                db.update_test(&test)?;
            }
            if let Some(webhook) = &test.discord_webhook {
                discord::notify(webhook, &format!("Test failed: failed to compile engine 1")).await.ok();
            }
            return Err(e);
        }
    };

    info!("Compiling engine 2: {}", test.engine2_ref);
    let engine2_bin = match engine::compile(
        &test.engine2_ref,
        &settings,
        &test.env_vars,
    ).await {
        Ok(bin) => bin,
        Err(e) => {
            error!("Failed to compile engine 2: {}", e);
            {
                let db = db.lock();
                test.status = TestStatus::Failed;
                db.update_test(&test)?;
            }
            if let Some(webhook) = &test.discord_webhook {
                discord::notify(webhook, &format!("Test failed: failed to compile engine 2")).await.ok();
            }
            return Err(e);
        }
    };

    test.engine1_bin = Some(engine1_bin.clone());
    test.engine2_bin = Some(engine2_bin.clone());
    test.status = TestStatus::Running;
    test.started_at = Some(Utc::now().to_rfc3339());

    {
        let db = db.lock();
        db.update_test(&test)?;
    }

    // Run fastchess
    info!("Starting fastchess test: {}", test_id);
    if let Err(e) = fastchess::run_test(test_id.clone(), test.clone(), settings).await {
        error!("Fastchess test failed: {}", e);
        {
            let db = db.lock();
            test.status = TestStatus::Failed;
            db.update_test(&test)?;
        }
        if let Some(webhook) = &test.discord_webhook {
            discord::notify(webhook, &format!("Test failed: {}", e)).await.ok();
        }
        return Err(e);
    }

    test.status = TestStatus::Finished;
    test.finished_at = Some(Utc::now().to_rfc3339());

    {
        let db = db.lock();
        db.update_test(&test)?;
    }

    // Notify Discord - test finished
    if let Some(webhook) = &test.discord_webhook {
        discord::notify(webhook, &format!("Test finished: {} vs {}", test.engine1_name, test.engine2_name)).await.ok();
    }

    Ok(())
}
