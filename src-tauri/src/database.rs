use rusqlite::{Connection, Result as SqliteResult, params};
use chrono::{DateTime, Utc};
use std::path::Path;
use crate::models::*;
use anyhow::Result;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = Database { conn };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        // Create jobs table
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS jobs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                url TEXT NOT NULL,
                selector_type TEXT NOT NULL,
                selector TEXT NOT NULL,
                data_type TEXT NOT NULL,
                schedule TEXT NOT NULL,
                user_agent TEXT,
                proxy_url TEXT,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
            [],
        )?;

        // Create results table
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                job_id INTEGER NOT NULL,
                scraped_data TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                success BOOLEAN NOT NULL,
                error_message TEXT,
                FOREIGN KEY (job_id) REFERENCES jobs (id) ON DELETE CASCADE
            )
            "#,
            [],
        )?;

        // Create email_config table
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS email_config (
                id INTEGER PRIMARY KEY,
                smtp_server TEXT NOT NULL,
                smtp_port INTEGER NOT NULL,
                username TEXT NOT NULL,
                password TEXT NOT NULL,
                sender_email TEXT NOT NULL,
                receiver_email TEXT NOT NULL,
                use_tls BOOLEAN NOT NULL DEFAULT 1
            )
            "#,
            [],
        )?;

        Ok(())
    }

    pub fn create_job(&self, job: &ScrapingJob) -> Result<i64> {
        let now = Utc::now().to_rfc3339();
        let _id = self.conn.execute(
            r#"
            INSERT INTO jobs (name, url, selector_type, selector, data_type, schedule, 
                            user_agent, proxy_url, is_active, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                job.name,
                job.url,
                job.selector_type.to_string(),
                job.selector,
                job.data_type.to_string(),
                job.schedule,
                job.user_agent,
                job.proxy_url,
                job.is_active,
                now,
                now
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_job(&self, id: i64) -> Result<Option<ScrapingJob>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, url, selector_type, selector, data_type, schedule, 
                    user_agent, proxy_url, is_active, created_at, updated_at 
             FROM jobs WHERE id = ?1"
        )?;

        let job_iter = stmt.query_map([id], |row| {
            Ok(ScrapingJob {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                url: row.get(2)?,
                selector_type: row.get::<_, String>(3)?.parse().unwrap(),
                selector: row.get(4)?,
                data_type: row.get::<_, String>(5)?.parse().unwrap(),
                schedule: row.get(6)?,
                user_agent: row.get(7)?,
                proxy_url: row.get(8)?,
                is_active: row.get(9)?,
                created_at: Some(DateTime::parse_from_rfc3339(&row.get::<_, String>(10)?).unwrap().with_timezone(&Utc)),
                updated_at: Some(DateTime::parse_from_rfc3339(&row.get::<_, String>(11)?).unwrap().with_timezone(&Utc)),
            })
        })?;

        for job in job_iter {
            return Ok(Some(job?));
        }
        Ok(None)
    }

    pub fn get_all_jobs(&self) -> Result<Vec<ScrapingJob>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, url, selector_type, selector, data_type, schedule, 
                    user_agent, proxy_url, is_active, created_at, updated_at 
             FROM jobs ORDER BY created_at DESC"
        )?;

        let job_iter = stmt.query_map([], |row| {
            Ok(ScrapingJob {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                url: row.get(2)?,
                selector_type: row.get::<_, String>(3)?.parse().unwrap(),
                selector: row.get(4)?,
                data_type: row.get::<_, String>(5)?.parse().unwrap(),
                schedule: row.get(6)?,
                user_agent: row.get(7)?,
                proxy_url: row.get(8)?,
                is_active: row.get(9)?,
                created_at: Some(DateTime::parse_from_rfc3339(&row.get::<_, String>(10)?).unwrap().with_timezone(&Utc)),
                updated_at: Some(DateTime::parse_from_rfc3339(&row.get::<_, String>(11)?).unwrap().with_timezone(&Utc)),
            })
        })?;

        let mut jobs = Vec::new();
        for job in job_iter {
            jobs.push(job?);
        }
        Ok(jobs)
    }

    pub fn get_active_jobs(&self) -> Result<Vec<ScrapingJob>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, url, selector_type, selector, data_type, schedule, 
                    user_agent, proxy_url, is_active, created_at, updated_at 
             FROM jobs WHERE is_active = 1"
        )?;

        let job_iter = stmt.query_map([], |row| {
            Ok(ScrapingJob {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                url: row.get(2)?,
                selector_type: row.get::<_, String>(3)?.parse().unwrap(),
                selector: row.get(4)?,
                data_type: row.get::<_, String>(5)?.parse().unwrap(),
                schedule: row.get(6)?,
                user_agent: row.get(7)?,
                proxy_url: row.get(8)?,
                is_active: row.get(9)?,
                created_at: Some(DateTime::parse_from_rfc3339(&row.get::<_, String>(10)?).unwrap().with_timezone(&Utc)),
                updated_at: Some(DateTime::parse_from_rfc3339(&row.get::<_, String>(11)?).unwrap().with_timezone(&Utc)),
            })
        })?;

        let mut jobs = Vec::new();
        for job in job_iter {
            jobs.push(job?);
        }
        Ok(jobs)
    }

    pub fn update_job(&self, job: &ScrapingJob) -> Result<()> {
        let job_id = job.id.ok_or_else(|| anyhow::anyhow!("Job ID is required for update"))?;
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            r#"
            UPDATE jobs SET name = ?1, url = ?2, selector_type = ?3, selector = ?4, 
                          data_type = ?5, schedule = ?6, user_agent = ?7, proxy_url = ?8, 
                          is_active = ?9, updated_at = ?10
            WHERE id = ?11
            "#,
            params![
                job.name,
                job.url,
                job.selector_type.to_string(),
                job.selector,
                job.data_type.to_string(),
                job.schedule,
                job.user_agent,
                job.proxy_url,
                job.is_active,
                now,
                job_id
            ],
        )?;
        Ok(())
    }

    pub fn delete_job(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM jobs WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn save_result(&self, result: &ScrapingResult) -> Result<i64> {
        let _id = self.conn.execute(
            "INSERT INTO results (job_id, scraped_data, timestamp, success, error_message) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                result.job_id,
                result.scraped_data,
                result.timestamp.to_rfc3339(),
                result.success,
                result.error_message
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_result(&self, id: i64) -> Result<Option<ScrapingResult>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, job_id, scraped_data, timestamp, success, error_message 
             FROM results WHERE id = ?1"
        )?;

        let result_iter = stmt.query_map([id], |row| {
            Ok(ScrapingResult {
                id: Some(row.get(0)?),
                job_id: row.get(1)?,
                scraped_data: row.get(2)?,
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?).unwrap().with_timezone(&Utc),
                success: row.get(4)?,
                error_message: row.get(5)?,
            })
        })?;

        for result in result_iter {
            return Ok(Some(result?));
        }
        Ok(None)
    }

    pub fn get_results_for_job(&self, job_id: i64, limit: Option<i64>) -> Result<Vec<ScrapingResult>> {
        let query = if let Some(limit) = limit {
            format!(
                "SELECT id, job_id, scraped_data, timestamp, success, error_message 
                 FROM results WHERE job_id = ?1 ORDER BY timestamp DESC LIMIT {}",
                limit
            )
        } else {
            "SELECT id, job_id, scraped_data, timestamp, success, error_message 
             FROM results WHERE job_id = ?1 ORDER BY timestamp DESC".to_string()
        };

        let mut stmt = self.conn.prepare(&query)?;
        let result_iter = stmt.query_map([job_id], |row| {
            Ok(ScrapingResult {
                id: Some(row.get(0)?),
                job_id: row.get(1)?,
                scraped_data: row.get(2)?,
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?).unwrap().with_timezone(&Utc),
                success: row.get(4)?,
                error_message: row.get(5)?,
            })
        })?;

        let mut results = Vec::new();
        for result in result_iter {
            results.push(result?);
        }
        Ok(results)
    }

    pub fn get_job_stats(&self) -> Result<JobStats> {
        let total_jobs: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM jobs",
            [],
            |row| row.get(0)
        )?;

        let active_jobs: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM jobs WHERE is_active = 1",
            [],
            |row| row.get(0)
        )?;

        let total_results: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM results",
            [],
            |row| row.get(0)
        )?;

        let last_run: Option<DateTime<Utc>> = self.conn.query_row(
            "SELECT timestamp FROM results ORDER BY timestamp DESC LIMIT 1",
            [],
            |row| {
                let timestamp_str: String = row.get(0)?;
                Ok(DateTime::parse_from_rfc3339(&timestamp_str).unwrap().with_timezone(&Utc))
            }
        ).ok();

        Ok(JobStats {
            total_jobs,
            active_jobs,
            total_results,
            last_run,
        })
    }

    pub fn save_email_config(&self, config: &EmailConfig) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO email_config 
            (id, smtp_server, smtp_port, username, password, sender_email, receiver_email, use_tls)
            VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                config.smtp_server,
                config.smtp_port,
                config.username,
                config.password,
                config.sender_email,
                config.receiver_email,
                config.use_tls
            ],
        )?;
        Ok(())
    }

    pub fn get_email_config(&self) -> Result<Option<EmailConfig>> {
        let mut stmt = self.conn.prepare(
            "SELECT smtp_server, smtp_port, username, password, sender_email, receiver_email, use_tls 
             FROM email_config WHERE id = 1"
        )?;

        let config_iter = stmt.query_map([], |row| {
            Ok(EmailConfig {
                smtp_server: row.get(0)?,
                smtp_port: row.get(1)?,
                username: row.get(2)?,
                password: row.get(3)?,
                sender_email: row.get(4)?,
                receiver_email: row.get(5)?,
                use_tls: row.get(6)?,
            })
        })?;

        for config in config_iter {
            return Ok(Some(config?));
        }
        Ok(None)
    }
}