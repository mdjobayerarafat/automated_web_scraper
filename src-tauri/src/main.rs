// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod database;
mod scraper;
mod scheduler;
mod email;
mod export;

use std::sync::Arc;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;
use tauri::{Manager, State};
use log::{info, error};

use models::*;
use database::Database;
use scraper::WebScraper;
use scheduler::ScrapingScheduler;
use email::EmailService;
use export::ExportService;

// Global application state
struct AppState {
    db: Arc<Mutex<Database>>,
    scheduler: Arc<Mutex<ScrapingScheduler>>,
    scraper: Arc<WebScraper>,
    email_service: Arc<Mutex<EmailService>>,
    export_service: Arc<ExportService>,
}

static APP_STATE: Lazy<Arc<Mutex<Option<AppState>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

#[tauri::command]
async fn show_main_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    let window = app_handle.get_window("main").ok_or("Main window not found")?;
    window.show().map_err(|e| format!("Failed to show window: {}", e))?;
    window.set_focus().map_err(|e| format!("Failed to focus window: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn initialize_app(app_handle: tauri::AppHandle) -> Result<String, String> {
    info!("Initializing application...");
    
    let app_dir = app_handle.path_resolver()
        .app_data_dir()
        .ok_or("Failed to get app data directory")?;
    
    // Create app directory if it doesn't exist
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir)
            .map_err(|e| format!("Failed to create app directory: {}", e))?;
    }
    
    let db_path = app_dir.join("scraper.db");
    let export_dir = app_dir.join("exports");
    
    // Initialize database
    let db = Arc::new(Mutex::new(
        Database::new(&db_path)
            .map_err(|e| format!("Failed to initialize database: {}", e))?
    ));
    
    // Initialize scheduler
    let scheduler = Arc::new(Mutex::new(
        ScrapingScheduler::new(Arc::clone(&db))
            .await
            .map_err(|e| format!("Failed to initialize scheduler: {}", e))?
    ));
    
    // Initialize other services
    let scraper = Arc::new(WebScraper::new());
    let email_service = Arc::new(Mutex::new(EmailService::new()));
    let export_service = Arc::new(
        ExportService::new(&export_dir)
            .map_err(|e| format!("Failed to initialize export service: {}", e))?
    );
    
    // Start scheduler
    scheduler.lock().await.start()
        .await
        .map_err(|e| format!("Failed to start scheduler: {}", e))?;
    
    // Store global state
    let state = AppState {
        db,
        scheduler,
        scraper,
        email_service,
        export_service,
    };
    
    *APP_STATE.lock().await = Some(state);
    
    info!("Application initialized successfully");
    Ok("Application initialized successfully".to_string())
}

#[tauri::command]
async fn create_job(job: ScrapingJob) -> Result<i64, String> {
    let state_guard = APP_STATE.lock().await;
    let state = state_guard.as_ref().ok_or("Application not initialized")?;
    
    let db = state.db.lock().await;
    let job_id = db.create_job(&job)
        .map_err(|e| format!("Failed to create job: {}", e))?;
    drop(db);
    
    // Schedule the job if it's active
    if job.is_active {
        let mut job_with_id = job;
        job_with_id.id = Some(job_id);
        
        let scheduler = state.scheduler.lock().await;
        scheduler.schedule_job(job_with_id)
            .await
            .map_err(|e| format!("Failed to schedule job: {}", e))?;
    }
    
    Ok(job_id)
}

#[tauri::command]
async fn get_all_jobs() -> Result<Vec<ScrapingJob>, String> {
    let state_guard = APP_STATE.lock().await;
    let state = state_guard.as_ref().ok_or("Application not initialized")?;
    
    let db = state.db.lock().await;
    db.get_all_jobs()
        .map_err(|e| format!("Failed to get jobs: {}", e))
}

#[tauri::command]
async fn get_job(id: i64) -> Result<Option<ScrapingJob>, String> {
    let state_guard = APP_STATE.lock().await;
    let state = state_guard.as_ref().ok_or("Application not initialized")?;
    
    let db = state.db.lock().await;
    db.get_job(id)
        .map_err(|e| format!("Failed to get job: {}", e))
}

#[tauri::command]
async fn update_job(job: ScrapingJob) -> Result<(), String> {
    let state_guard = APP_STATE.lock().await;
    let state = state_guard.as_ref().ok_or("Application not initialized")?;
    
    let db = state.db.lock().await;
    db.update_job(&job)
        .map_err(|e| format!("Failed to update job: {}", e))?;
    drop(db);
    
    // Reschedule the job
    let scheduler = state.scheduler.lock().await;
    scheduler.reschedule_job(job)
        .await
        .map_err(|e| format!("Failed to reschedule job: {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn delete_job(id: i64) -> Result<(), String> {
    let state_guard = APP_STATE.lock().await;
    let state = state_guard.as_ref().ok_or("Application not initialized")?;
    
    let scheduler = state.scheduler.lock().await;
    scheduler.unschedule_job(id)
        .await
        .map_err(|e| format!("Failed to unschedule job: {}", e))?;
    drop(scheduler);
    
    let db = state.db.lock().await;
    db.delete_job(id)
        .map_err(|e| format!("Failed to delete job: {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn test_scrape_job(job: ScrapingJob) -> Result<Vec<String>, String> {
    let state_guard = APP_STATE.lock().await;
    let state = state_guard.as_ref().ok_or("Application not initialized")?;
    
    state.scraper.test_scrape(&job)
        .await
        .map_err(|e| format!("Failed to test scrape: {}", e))
}

#[tauri::command]
async fn run_job_now(id: i64) -> Result<Vec<String>, String> {
    let state_guard = APP_STATE.lock().await;
    let state = state_guard.as_ref().ok_or("Application not initialized")?;
    
    let db = state.db.lock().await;
    let job = db.get_job(id)
        .map_err(|e| format!("Failed to get job: {}", e))?
        .ok_or("Job not found")?;
    drop(db);
    
    let scheduler = state.scheduler.lock().await;
    scheduler.run_job_now(job)
        .await
        .map_err(|e| format!("Failed to run job: {}", e))
}

#[tauri::command]
async fn get_job_results(job_id: i64, limit: Option<i64>) -> Result<Vec<ScrapingResult>, String> {
    let state_guard = APP_STATE.lock().await;
    let state = state_guard.as_ref().ok_or("Application not initialized")?;
    
    let db = state.db.lock().await;
    db.get_results_for_job(job_id, limit)
        .map_err(|e| format!("Failed to get results: {}", e))
}

#[tauri::command]
async fn get_job_stats() -> Result<JobStats, String> {
    let state_guard = APP_STATE.lock().await;
    let state = state_guard.as_ref().ok_or("Application not initialized")?;
    
    let db = state.db.lock().await;
    db.get_job_stats()
        .map_err(|e| format!("Failed to get stats: {}", e))
}

#[tauri::command]
async fn export_job_results(request: ExportRequest) -> Result<String, String> {
    let state_guard = APP_STATE.lock().await;
    let state = state_guard.as_ref().ok_or("Application not initialized")?;
    
    let db = Arc::clone(&state.db);
    let export_service = Arc::clone(&state.export_service);
    drop(state_guard);
    
    let db_guard = db.lock().await;
    
    // Get job details
    let job = db_guard.get_job(request.job_id)
        .map_err(|e| format!("Failed to get job: {}", e))?
        .ok_or("Job not found")?;
    
    // Get results
    let results = db_guard.get_results_for_job(request.job_id, None)
        .map_err(|e| format!("Failed to get results: {}", e))?;
    
    drop(db_guard);
    
    let file_path = export_service.export_job_results(job, results, &request)
        .await
        .map_err(|e| format!("Failed to export results: {}", e))?;
    
    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn export_individual_result(request: IndividualExportRequest) -> Result<String, String> {
    let state_guard = APP_STATE.lock().await;
    let state = state_guard.as_ref().ok_or("Application not initialized")?;
    
    let db = Arc::clone(&state.db);
    let export_service = Arc::clone(&state.export_service);
    drop(state_guard);
    
    let db_guard = db.lock().await;
    
    // Get result details
    let result = db_guard.get_result(request.result_id)
        .map_err(|e| format!("Failed to get result: {}", e))?
        .ok_or("Result not found")?;
    
    // Get job details
    let job = db_guard.get_job(result.job_id)
        .map_err(|e| format!("Failed to get job: {}", e))?
        .ok_or("Job not found")?;
    
    drop(db_guard);
    
    let file_path = export_service.export_individual_result(job, result, &request)
        .await
        .map_err(|e| format!("Failed to export individual result: {}", e))?;
    
    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn save_email_config(config: EmailConfig) -> Result<(), String> {
    let state_guard = APP_STATE.lock().await;
    let state = state_guard.as_ref().ok_or("Application not initialized")?;
    
    // Validate config
    EmailService::validate_config(&config)
        .map_err(|e| format!("Invalid email config: {}", e))?;
    
    let db = state.db.lock().await;
    db.save_email_config(&config)
        .map_err(|e| format!("Failed to save email config: {}", e))?;
    drop(db);
    
    // Update email service
    let mut email_service = state.email_service.lock().await;
    email_service.set_config(config);
    
    Ok(())
}

#[tauri::command]
async fn get_email_config() -> Result<Option<EmailConfig>, String> {
    let state_guard = APP_STATE.lock().await;
    let state = state_guard.as_ref().ok_or("Application not initialized")?;
    
    let db = state.db.lock().await;
    db.get_email_config()
        .map_err(|e| format!("Failed to get email config: {}", e))
}

#[tauri::command]
async fn test_email_connection() -> Result<(), String> {
    let state_guard = APP_STATE.lock().await;
    let state = state_guard.as_ref().ok_or("Application not initialized")?;
    
    let email_service = state.email_service.lock().await;
    email_service.test_connection()
        .await
        .map_err(|e| format!("Email connection test failed: {}", e))
}

#[tauri::command]
async fn send_export_email(file_path: String, job_name: String, format: String) -> Result<(), String> {
    let state_guard = APP_STATE.lock().await;
    let state = state_guard.as_ref().ok_or("Application not initialized")?;
    
    let email_service = state.email_service.lock().await;
    email_service.send_export_file(&file_path, &job_name, &format)
        .await
        .map_err(|e| format!("Failed to send email: {}", e))
}

#[tauri::command]
async fn validate_url(url: String) -> Result<bool, String> {
    let state_guard = APP_STATE.lock().await;
    let state = state_guard.as_ref().ok_or("Application not initialized")?;
    
    state.scraper.validate_url(&url)
        .await
        .map_err(|e| format!("URL validation failed: {}", e))
}

#[tauri::command]
async fn validate_css_selector(selector: String) -> Result<bool, String> {
    let state_guard = APP_STATE.lock().await;
    let state = state_guard.as_ref().ok_or("Application not initialized")?;
    
    state.scraper.validate_css_selector(&selector)
        .map_err(|e| format!("CSS selector validation failed: {}", e))
}

#[tauri::command]
async fn validate_regex_pattern(pattern: String) -> Result<bool, String> {
    let state_guard = APP_STATE.lock().await;
    let state = state_guard.as_ref().ok_or("Application not initialized")?;
    
    state.scraper.validate_regex_pattern(&pattern)
        .map_err(|e| format!("Regex pattern validation failed: {}", e))
}

#[tauri::command]
async fn validate_cron_expression(expression: String) -> Result<bool, String> {
    let state_guard = APP_STATE.lock().await;
    let state = state_guard.as_ref().ok_or("Application not initialized")?;
    
    let scheduler = state.scheduler.lock().await;
    scheduler.validate_cron_expression(&expression)
        .map_err(|e| format!("Cron expression validation failed: {}", e))
}

#[tauri::command]
async fn list_export_files() -> Result<Vec<ExportFileInfo>, String> {
    let state_guard = APP_STATE.lock().await;
    let state = state_guard.as_ref().ok_or("Application not initialized")?;
    
    let export_service = Arc::clone(&state.export_service);
    drop(state_guard);
    
    let files = export_service.list_export_files()
        .map_err(|e| format!("Failed to list export files: {}", e))?;
    
    let mut file_infos = Vec::new();
    for file_path in files {
        if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
            let metadata = std::fs::metadata(&file_path)
                .map_err(|e| format!("Failed to get file metadata: {}", e))?;
            
            let size = metadata.len();
            let modified = metadata.modified()
                .map_err(|e| format!("Failed to get file modification time: {}", e))?
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| format!("Failed to convert time: {}", e))?
                .as_secs();
            
            let extension = file_path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_lowercase();
            
            file_infos.push(ExportFileInfo {
                name: file_name.to_string(),
                path: file_path.to_string_lossy().to_string(),
                size,
                modified_timestamp: modified,
                file_type: match extension.as_str() {
                    "csv" => "CSV".to_string(),
                    "json" => "JSON".to_string(),
                    "html" => "HTML".to_string(),
                    _ => "Unknown".to_string(),
                },
            });
        }
    }
    
    Ok(file_infos)
}

#[tauri::command]
async fn read_export_file(file_path: String) -> Result<String, String> {
    let state_guard = APP_STATE.lock().await;
    let state = state_guard.as_ref().ok_or("Application not initialized")?;
    
    let export_service = Arc::clone(&state.export_service);
    drop(state_guard);
    
    let export_dir = export_service.get_export_directory();
    let file_path = std::path::Path::new(&file_path);
    
    // Security check: ensure file is within export directory
    if !file_path.starts_with(export_dir) {
        return Err("File is not within the export directory".to_string());
    }
    
    std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
async fn open_export_directory() -> Result<(), String> {
    let state_guard = APP_STATE.lock().await;
    let state = state_guard.as_ref().ok_or("Application not initialized")?;
    
    let export_service = Arc::clone(&state.export_service);
    drop(state_guard);
    
    let export_dir = export_service.get_export_directory();
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(export_dir)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(export_dir)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(export_dir)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }
    
    Ok(())
}

#[tauri::command]
async fn delete_export_file(file_path: String) -> Result<(), String> {
    let state_guard = APP_STATE.lock().await;
    let state = state_guard.as_ref().ok_or("Application not initialized")?;
    
    let export_service = Arc::clone(&state.export_service);
    drop(state_guard);
    
    export_service.delete_export_file(&file_path)
        .map_err(|e| format!("Failed to delete file: {}", e))
}

fn main() {
    env_logger::init();
    
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = initialize_app(app_handle).await {
                    error!("Failed to initialize app: {}", e);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            show_main_window,
            initialize_app,
            create_job,
            get_all_jobs,
            get_job,
            update_job,
            delete_job,
            test_scrape_job,
            run_job_now,
            get_job_results,
            get_job_stats,
            export_job_results,
            export_individual_result,
            save_email_config,
            get_email_config,
            test_email_connection,
            send_export_email,
            validate_url,
            validate_css_selector,
            validate_regex_pattern,
            validate_cron_expression,
            list_export_files,
            read_export_file,
            open_export_directory,
            delete_export_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
