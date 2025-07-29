use tokio_cron_scheduler::{JobScheduler, Job};
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::Utc;
use uuid::Uuid;
use std::collections::HashMap;
use crate::models::*;
use crate::database::Database;
use crate::scraper::WebScraper;
use anyhow::Result;
use log::{info, error, warn};

pub struct ScrapingScheduler {
    scheduler: JobScheduler,
    db: Arc<Mutex<Database>>,
    scraper: Arc<WebScraper>,
    job_handles: Arc<Mutex<HashMap<i64, Uuid>>>,
}

impl ScrapingScheduler {
    pub async fn new(db: Arc<Mutex<Database>>) -> Result<Self> {
        let scheduler = JobScheduler::new().await?;
        let scraper = Arc::new(WebScraper::new());
        let job_handles = Arc::new(Mutex::new(HashMap::new()));
        
        Ok(ScrapingScheduler {
            scheduler,
            db,
            scraper,
            job_handles,
        })
    }
    
    pub async fn start(&self) -> Result<()> {
        info!("Starting job scheduler");
        self.scheduler.start().await?;
        
        // Load and schedule all active jobs from database
        self.load_active_jobs().await?;
        
        Ok(())
    }
    
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping job scheduler");
        self.scheduler.shutdown().await?;
        Ok(())
    }
    
    async fn load_active_jobs(&self) -> Result<()> {
        let db = self.db.lock().await;
        let active_jobs = db.get_active_jobs()?;
        drop(db);
        
        info!("Loading {} active jobs", active_jobs.len());
        
        for job in active_jobs {
            if let Some(job_id) = job.id {
                if let Err(e) = self.schedule_job_internal(job).await {
                    error!("Failed to schedule job {}: {}", job_id, e);
                }
            }
        }
        
        Ok(())
    }
    
    pub async fn schedule_job(&self, job: ScrapingJob) -> Result<()> {
        if !job.is_active {
            warn!("Attempting to schedule inactive job: {}", job.name);
            return Ok(());
        }
        
        self.schedule_job_internal(job).await
    }
    
    async fn schedule_job_internal(&self, job: ScrapingJob) -> Result<()> {
        let job_id = job.id.ok_or_else(|| anyhow::anyhow!("Job must have an ID to be scheduled"))?;
        
        info!("Scheduling job: {} with schedule: {}", job.name, job.schedule);
        
        // Parse the schedule to convert human-readable formats to cron expressions
        let cron_expression = parse_schedule(&job.schedule)
            .map_err(|e| anyhow::anyhow!("Failed to parse schedule '{}': {}", job.schedule, e))?;
        
        info!("Using cron expression: {}", cron_expression);
        
        // Remove existing job if it exists
        self.unschedule_job(job_id).await?;
        
        let db_clone = Arc::clone(&self.db);
        let scraper_clone = Arc::clone(&self.scraper);
        let job_clone = job.clone();
        
        let scheduled_job = Job::new_async(cron_expression.as_str(), move |_uuid, _l| {
            let db = Arc::clone(&db_clone);
            let scraper = Arc::clone(&scraper_clone);
            let job = job_clone.clone();
            
            Box::pin(async move {
                if let Err(e) = execute_scraping_job(db, scraper, job).await {
                    error!("Failed to execute scraping job: {}", e);
                }
            })
        })?;
        
        let job_uuid = self.scheduler.add(scheduled_job).await?;
        
        // Store the job handle for later removal
        let mut handles = self.job_handles.lock().await;
        handles.insert(job_id, job_uuid);
        
        info!("Successfully scheduled job: {} (ID: {})", job.name, job_id);
        Ok(())
    }
    
    pub async fn unschedule_job(&self, job_id: i64) -> Result<()> {
        let mut handles = self.job_handles.lock().await;
        
        if let Some(job_uuid) = handles.remove(&job_id) {
            self.scheduler.remove(&job_uuid).await?;
            info!("Unscheduled job with ID: {}", job_id);
        }
        
        Ok(())
    }
    
    pub async fn reschedule_job(&self, job: ScrapingJob) -> Result<()> {
        if let Some(job_id) = job.id {
            self.unschedule_job(job_id).await?;
        }
        
        if job.is_active {
            self.schedule_job(job).await?;
        }
        
        Ok(())
    }
    
    pub async fn run_job_now(&self, job: ScrapingJob) -> Result<Vec<String>> {
        info!("Running job immediately: {}", job.name);
        
        let results = self.scraper.scrape_job(&job).await?;
        
        // Save results to database
        let db = self.db.lock().await;
        let job_id = job.id.ok_or_else(|| anyhow::anyhow!("Job must have an ID"))?;
        
        let result = ScrapingResult {
            id: None,
            job_id,
            scraped_data: results.join("\n"),
            timestamp: Utc::now(),
            success: true,
            error_message: None,
        };
        
        db.save_result(&result)?;
        drop(db);
        
        info!("Job completed successfully: {}", job.name);
        Ok(results)
    }
    
    pub async fn get_scheduled_jobs(&self) -> Vec<i64> {
        let handles = self.job_handles.lock().await;
        handles.keys().cloned().collect()
    }
    
    pub fn validate_cron_expression(&self, expression: &str) -> Result<bool> {
        // Parse the schedule first to handle human-readable formats
        let cron_expression = parse_schedule(expression)?;
        
        // Try to create a dummy job with the parsed expression to validate it
        match Job::new(cron_expression.as_str(), |_uuid, _l| {}) {
            Ok(_) => Ok(true),
            Err(e) => Err(anyhow::anyhow!("Invalid cron expression: {}", e)),
        }
    }
}

async fn execute_scraping_job(
    db: Arc<Mutex<Database>>,
    scraper: Arc<WebScraper>,
    job: ScrapingJob,
) -> Result<()> {
    let job_id = job.id.ok_or_else(|| anyhow::anyhow!("Job must have an ID"))?;
    
    info!("Executing scheduled job: {} (ID: {})", job.name, job_id);
    
    let result = match scraper.scrape_job(&job).await {
        Ok(data) => {
            info!("Job {} completed successfully with {} items", job.name, data.len());
            ScrapingResult {
                id: None,
                job_id,
                scraped_data: data.join("\n"),
                timestamp: Utc::now(),
                success: true,
                error_message: None,
            }
        }
        Err(e) => {
            error!("Job {} failed: {}", job.name, e);
            ScrapingResult {
                id: None,
                job_id,
                scraped_data: String::new(),
                timestamp: Utc::now(),
                success: false,
                error_message: Some(e.to_string()),
            }
        }
    };
    
    // Save result to database
    let db = db.lock().await;
    db.save_result(&result)?;
    drop(db);
    
    Ok(())
}

// Helper function to convert common schedule formats to cron expressions
pub fn parse_schedule(schedule: &str) -> Result<String> {
    match schedule.to_lowercase().as_str() {
        "daily" => Ok("0 0 0 * * *".to_string()), // Every day at midnight
        "hourly" => Ok("0 0 * * * *".to_string()), // Every hour
        "weekly" => Ok("0 0 0 * * 0".to_string()), // Every Sunday at midnight
        "monthly" => Ok("0 0 0 1 * *".to_string()), // First day of every month
        _ => {
            // Assume it's already a cron expression
            // Validate it by trying to create a job
            match Job::new(schedule, |_uuid, _l| {}) {
                Ok(_) => Ok(schedule.to_string()),
                Err(e) => Err(anyhow::anyhow!("Invalid schedule format: {}", e)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_schedule() {
        assert_eq!(parse_schedule("daily").unwrap(), "0 0 0 * * *");
        assert_eq!(parse_schedule("hourly").unwrap(), "0 0 * * * *");
        assert_eq!(parse_schedule("weekly").unwrap(), "0 0 0 * * 0");
        assert_eq!(parse_schedule("monthly").unwrap(), "0 0 0 1 * *");
        
        // Test custom cron expression
        assert_eq!(parse_schedule("0 30 14 * * *").unwrap(), "0 30 14 * * *");
        
        // Test invalid expression
        assert!(parse_schedule("invalid").is_err());
    }
    
    #[tokio::test]
    async fn test_scheduler_creation() {
        // This test requires a database, so we'll just test that we can create the scheduler
        // In a real test environment, you'd set up a test database
        
        // For now, just test the parse_schedule function
        assert!(parse_schedule("0 0 12 * * *").is_ok());
    }
}