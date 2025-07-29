use std::path::{Path, PathBuf};
use std::fs;
use csv::Writer;
use serde_json;
use chrono::{DateTime, Utc};
use crate::models::*;
use crate::database::Database;
use anyhow::{Result, anyhow};
use log::{info, error};

pub struct ExportService {
    export_dir: PathBuf,
}

impl ExportService {
    pub fn new<P: AsRef<Path>>(export_dir: P) -> Result<Self> {
        let export_dir = export_dir.as_ref().to_path_buf();
        
        // Create export directory if it doesn't exist
        if !export_dir.exists() {
            fs::create_dir_all(&export_dir)
                .map_err(|e| anyhow!("Failed to create export directory: {}", e))?;
        }
        
        Ok(ExportService { export_dir })
    }
    
    pub async fn export_job_results(
        &self,
        job: ScrapingJob,
        mut results: Vec<ScrapingResult>,
        request: &ExportRequest,
    ) -> Result<PathBuf> {
        info!("Exporting results for job ID: {} in format: {:?}", request.job_id, request.format);
        
        // Apply date filtering if specified
        if let (Some(start_date), Some(end_date)) = (&request.start_date, &request.end_date) {
            results.retain(|result| {
                result.timestamp >= *start_date && result.timestamp <= *end_date
            });
        } else if let Some(start_date) = &request.start_date {
            results.retain(|result| result.timestamp >= *start_date);
        } else if let Some(end_date) = &request.end_date {
            results.retain(|result| result.timestamp <= *end_date);
        }
        
        if results.is_empty() {
            return Err(anyhow!("No results found for the specified criteria"));
        }
        
        let file_path = match request.format {
            ExportFormat::CSV => self.export_to_csv(&job, &results).await?,
            ExportFormat::JSON => self.export_to_json(&job, &results).await?,
            ExportFormat::HTML => self.export_to_pdf(&job, &results).await?,
        };
        
        info!("Export completed: {:?}", file_path);
        Ok(file_path)
    }

    pub async fn export_individual_result(
        &self,
        job: ScrapingJob,
        result: ScrapingResult,
        request: &IndividualExportRequest,
    ) -> Result<PathBuf> {
        info!("Exporting individual result ID: {} in format: {:?}", request.result_id, request.format);
        
        let results = vec![result];
        
        let file_path = match request.format {
            ExportFormat::CSV => self.export_individual_to_csv(&job, &results[0]).await?,
            ExportFormat::JSON => self.export_individual_to_json(&job, &results[0]).await?,
            ExportFormat::HTML => self.export_individual_to_html(&job, &results[0]).await?,
        };
        
        info!("Individual export completed: {:?}", file_path);
        Ok(file_path)
    }
    
    async fn export_to_csv(
        &self,
        job: &ScrapingJob,
        results: &[ScrapingResult],
    ) -> Result<PathBuf> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}.csv", sanitize_filename(&job.name), timestamp);
        let file_path = self.export_dir.join(filename);
        
        let mut writer = Writer::from_path(&file_path)
            .map_err(|e| anyhow!("Failed to create CSV file: {}", e))?;
        
        // Write header
        writer.write_record(&["ID", "Job Name", "Scraped Data", "Timestamp", "Success", "Error Message"])
            .map_err(|e| anyhow!("Failed to write CSV header: {}", e))?;
        
        // Write data rows
        for result in results {
            writer.write_record(&[
                result.id.map(|id| id.to_string()).unwrap_or_default(),
                job.name.clone(),
                result.scraped_data.clone(),
                result.timestamp.to_rfc3339(),
                result.success.to_string(),
                result.error_message.clone().unwrap_or_default(),
            ])
            .map_err(|e| anyhow!("Failed to write CSV row: {}", e))?;
        }
        
        writer.flush()
            .map_err(|e| anyhow!("Failed to flush CSV file: {}", e))?;
        
        Ok(file_path)
    }
    
    async fn export_to_json(
        &self,
        job: &ScrapingJob,
        results: &[ScrapingResult],
    ) -> Result<PathBuf> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}.json", sanitize_filename(&job.name), timestamp);
        let file_path = self.export_dir.join(filename);
        
        #[derive(serde::Serialize)]
        struct ExportData {
            job: ScrapingJob,
            results: Vec<ScrapingResult>,
            export_info: ExportInfo,
        }
        
        #[derive(serde::Serialize)]
        struct ExportInfo {
            exported_at: DateTime<Utc>,
            total_results: usize,
            successful_results: usize,
            failed_results: usize,
        }
        
        let successful_results = results.iter().filter(|r| r.success).count();
        let failed_results = results.len() - successful_results;
        
        let export_data = ExportData {
            job: job.clone(),
            results: results.to_vec(),
            export_info: ExportInfo {
                exported_at: Utc::now(),
                total_results: results.len(),
                successful_results,
                failed_results,
            },
        };
        
        let json_content = serde_json::to_string_pretty(&export_data)
            .map_err(|e| anyhow!("Failed to serialize to JSON: {}", e))?;
        
        fs::write(&file_path, json_content)
            .map_err(|e| anyhow!("Failed to write JSON file: {}", e))?;
        
        Ok(file_path)
    }
    
    async fn export_to_pdf(
        &self,
        job: &ScrapingJob,
        results: &[ScrapingResult],
    ) -> Result<PathBuf> {
        // For PDF export, we'll create an HTML file and note that PDF generation
        // would require additional dependencies like wkhtmltopdf or headless Chrome
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}.html", sanitize_filename(&job.name), timestamp);
        let file_path = self.export_dir.join(filename);
        
        let successful_results = results.iter().filter(|r| r.success).count();
        let failed_results = results.len() - successful_results;
        
        let html_content = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Web Scraping Results - {}</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
            line-height: 1.6;
        }}
        .container {{
            max-width: 1400px;
            margin: 0 auto;
            background: rgba(255, 255, 255, 0.95);
            border-radius: 20px;
            box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
            backdrop-filter: blur(10px);
            overflow: hidden;
        }}
        .header {{
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f3460 100%);
            color: white;
            padding: 40px;
            text-align: center;
            position: relative;
        }}
        .header::before {{
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: url('data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><defs><pattern id="grain" width="100" height="100" patternUnits="userSpaceOnUse"><circle cx="50" cy="50" r="1" fill="%23ffffff" opacity="0.1"/></pattern></defs><rect width="100" height="100" fill="url(%23grain)"/></svg>') repeat;
            opacity: 0.3;
        }}
        .header h1 {{
            font-size: 3em;
            font-weight: 700;
            margin-bottom: 10px;
            text-shadow: 0 0 20px rgba(255, 255, 255, 0.3);
            position: relative;
            z-index: 1;
        }}
        .header h2 {{
            font-size: 1.5em;
            margin-bottom: 20px;
            opacity: 0.9;
            position: relative;
            z-index: 1;
        }}
        .job-details {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 15px;
            margin-top: 20px;
            position: relative;
            z-index: 1;
        }}
        .job-detail {{
            background: rgba(255, 255, 255, 0.1);
            padding: 15px;
            border-radius: 10px;
            border: 1px solid rgba(255, 255, 255, 0.2);
        }}
        .job-detail strong {{
            color: #63b3ed;
            display: block;
            margin-bottom: 5px;
        }}
        .stats {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            padding: 30px;
            background: #f8f9fa;
        }}
        .stat-box {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 25px;
            border-radius: 15px;
            text-align: center;
            box-shadow: 0 10px 25px rgba(0, 0, 0, 0.1);
            transform: translateY(0);
            transition: transform 0.3s ease;
        }}
        .stat-box:hover {{
            transform: translateY(-5px);
        }}
        .stat-number {{
            font-size: 2.5em;
            font-weight: 700;
            margin-bottom: 10px;
            text-shadow: 0 0 10px rgba(255, 255, 255, 0.3);
        }}
        .stat-label {{
            font-size: 1.1em;
            opacity: 0.9;
            text-transform: uppercase;
            letter-spacing: 1px;
        }}
        .table-container {{
            padding: 30px;
            overflow-x: auto;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            background: white;
            border-radius: 15px;
            overflow: hidden;
            box-shadow: 0 10px 25px rgba(0, 0, 0, 0.1);
        }}
        th {{
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            color: white;
            padding: 20px 15px;
            text-align: left;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 1px;
            font-size: 0.9em;
        }}
        td {{
            padding: 15px;
            border-bottom: 1px solid #e9ecef;
            vertical-align: top;
        }}
        tr:nth-child(even) {{
            background-color: #f8f9fa;
        }}
        tr:hover {{
            background-color: #e3f2fd;
            transform: scale(1.01);
            transition: all 0.2s ease;
        }}
        .success {{
            color: #28a745;
            font-weight: 600;
        }}
        .error {{
            color: #dc3545;
            font-weight: 600;
        }}
        .timestamp {{
            font-size: 0.9em;
            color: #6c757d;
            font-family: 'Courier New', monospace;
        }}
        .data-cell {{
            max-width: 400px;
            word-wrap: break-word;
            overflow-wrap: break-word;
            background: #f8f9fa;
            padding: 10px;
            border-radius: 8px;
            border-left: 4px solid #007bff;
            font-family: 'Courier New', monospace;
            font-size: 0.9em;
        }}
        .footer {{
            background: #1a1a2e;
            color: white;
            text-align: center;
            padding: 20px;
            font-size: 0.9em;
        }}
        @media (max-width: 768px) {{
            .container {{
                margin: 10px;
                border-radius: 10px;
            }}
            .header {{
                padding: 20px;
            }}
            .header h1 {{
                font-size: 2em;
            }}
            .stats {{
                grid-template-columns: 1fr;
                padding: 20px;
            }}
            .table-container {{
                padding: 15px;
            }}
            th, td {{
                padding: 10px 8px;
                font-size: 0.9em;
            }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🕷️ Web Scraping Results Report</h1>
            <h2>Job: {}</h2>
            <div class="job-details">
                <div class="job-detail">
                    <strong>🌐 Target URL:</strong>
                    <a href="{}" target="_blank" style="color: #63b3ed; text-decoration: none;">{}</a>
                </div>
                <div class="job-detail">
                    <strong>🎯 Selector:</strong>
                    {} ({})
                </div>
                <div class="job-detail">
                    <strong>⏰ Schedule:</strong>
                    {}
                </div>
                <div class="job-detail">
                    <strong>📅 Generated:</strong>
                    {}
                </div>
            </div>
        </div>
        
        <div class="stats">
            <div class="stat-box">
                <div class="stat-number">{}</div>
                <div class="stat-label">Total Results</div>
            </div>
            <div class="stat-box">
                <div class="stat-number">{}</div>
                <div class="stat-label">Successful</div>
            </div>
            <div class="stat-box">
                <div class="stat-number">{}</div>
                <div class="stat-label">Failed</div>
            </div>
        </div>
        
        <div class="table-container">
            <table>
                <thead>
                    <tr>
                        <th>🕐 Timestamp</th>
                        <th>📊 Status</th>
                        <th>📄 Scraped Data</th>
                        <th>⚠️ Error Message</th>
                    </tr>
                </thead>
                <tbody>
{}
                </tbody>
            </table>
        </div>
        
        <div class="footer">
            <p>Generated by Automated Web Scraper | {} | 🚀 Powered by Rust & Tauri</p>
        </div>
    </div>
</body>
</html>"#,
            job.name,
            job.name,
            job.url,
            job.url,
            job.selector,
            job.selector_type,
            job.schedule,
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            results.len(),
            successful_results,
            failed_results,
            generate_table_rows(results),
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );
        
        fs::write(&file_path, html_content)
            .map_err(|e| anyhow!("Failed to write HTML file: {}", e))?;
        
        Ok(file_path)
    }

    async fn export_individual_to_csv(
        &self,
        job: &ScrapingJob,
        result: &ScrapingResult,
    ) -> Result<PathBuf> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let result_id = result.id.unwrap_or(0);
        let filename = format!("{}_{}_result_{}.csv", sanitize_filename(&job.name), timestamp, result_id);
        let file_path = self.export_dir.join(filename);
        
        let mut writer = Writer::from_path(&file_path)
            .map_err(|e| anyhow!("Failed to create CSV file: {}", e))?;
        
        // Write header
        writer.write_record(&["ID", "Job Name", "Scraped Data", "Timestamp", "Success", "Error Message"])
            .map_err(|e| anyhow!("Failed to write CSV header: {}", e))?;
        
        // Write data row
        writer.write_record(&[
            result.id.map(|id| id.to_string()).unwrap_or_default(),
            job.name.clone(),
            result.scraped_data.clone(),
            result.timestamp.to_rfc3339(),
            result.success.to_string(),
            result.error_message.clone().unwrap_or_default(),
        ])
        .map_err(|e| anyhow!("Failed to write CSV row: {}", e))?;
        
        writer.flush()
            .map_err(|e| anyhow!("Failed to flush CSV file: {}", e))?;
        
        Ok(file_path)
    }

    async fn export_individual_to_json(
        &self,
        job: &ScrapingJob,
        result: &ScrapingResult,
    ) -> Result<PathBuf> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let result_id = result.id.unwrap_or(0);
        let filename = format!("{}_{}_result_{}.json", sanitize_filename(&job.name), timestamp, result_id);
        let file_path = self.export_dir.join(filename);
        
        #[derive(serde::Serialize)]
        struct IndividualExportData {
            job: ScrapingJob,
            result: ScrapingResult,
            export_info: IndividualExportInfo,
        }
        
        #[derive(serde::Serialize)]
        struct IndividualExportInfo {
            exported_at: DateTime<Utc>,
            result_id: Option<i64>,
        }
        
        let export_data = IndividualExportData {
            job: job.clone(),
            result: result.clone(),
            export_info: IndividualExportInfo {
                exported_at: Utc::now(),
                result_id: result.id,
            },
        };
        
        let json_content = serde_json::to_string_pretty(&export_data)
            .map_err(|e| anyhow!("Failed to serialize to JSON: {}", e))?;
        
        fs::write(&file_path, json_content)
            .map_err(|e| anyhow!("Failed to write JSON file: {}", e))?;
        
        Ok(file_path)
    }

    async fn export_individual_to_html(
        &self,
        job: &ScrapingJob,
        result: &ScrapingResult,
    ) -> Result<PathBuf> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let result_id = result.id.unwrap_or(0);
        let filename = format!("{}_{}_result_{}.html", sanitize_filename(&job.name), timestamp, result_id);
        let file_path = self.export_dir.join(filename);
        
        let status_class = if result.success { "success" } else { "error" };
        let status_text = if result.success { "✓ Success" } else { "✗ Failed" };
        let error_message = result.error_message.as_deref().unwrap_or("-");
        
        let html_content = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Individual Scraping Result - {}</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f8f9fa;
            line-height: 1.6;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background-color: white;
            border-radius: 10px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
            overflow: hidden;
        }}
        .header {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 30px;
            text-align: center;
        }}
        .header h1 {{
            margin: 0;
            font-size: 2.5em;
            font-weight: 300;
        }}
        .header h2 {{
            margin: 10px 0 0 0;
            font-size: 1.2em;
            opacity: 0.9;
        }}
        .content {{
            padding: 30px;
        }}
        .info-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }}
        .info-card {{
            background-color: #f8f9fa;
            border-radius: 8px;
            padding: 20px;
            border-left: 4px solid #007bff;
        }}
        .info-card h3 {{
            margin: 0 0 10px 0;
            color: #495057;
            font-size: 1.1em;
        }}
        .info-card p {{
            margin: 5px 0;
            color: #6c757d;
        }}
        .status {{
            display: inline-block;
            padding: 8px 16px;
            border-radius: 20px;
            font-weight: bold;
            font-size: 0.9em;
        }}
        .status.success {{
            background-color: #d4edda;
            color: #155724;
            border: 1px solid #c3e6cb;
        }}
        .status.error {{
            background-color: #f8d7da;
            color: #721c24;
            border: 1px solid #f5c6cb;
        }}
        .data-section {{
            background-color: #f8f9fa;
            border-radius: 8px;
            padding: 20px;
            margin-top: 20px;
        }}
        .data-section h3 {{
            margin: 0 0 15px 0;
            color: #495057;
        }}
        .data-content {{
            background-color: white;
            border: 1px solid #dee2e6;
            border-radius: 4px;
            padding: 15px;
            font-family: 'Courier New', monospace;
            white-space: pre-wrap;
            word-wrap: break-word;
            max-height: 400px;
            overflow-y: auto;
        }}
        .footer {{
            background-color: #f8f9fa;
            padding: 20px;
            text-align: center;
            color: #6c757d;
            font-size: 0.9em;
        }}
        @media (max-width: 768px) {{
            .container {{
                margin: 10px;
                border-radius: 0;
            }}
            .header {{
                padding: 20px;
            }}
            .header h1 {{
                font-size: 2em;
            }}
            .content {{
                padding: 20px;
            }}
            .info-grid {{
                grid-template-columns: 1fr;
            }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🔍 Individual Scraping Result</h1>
            <h2>Result ID: {}</h2>
        </div>
        
        <div class="content">
            <div class="info-grid">
                <div class="info-card">
                    <h3>📋 Job Information</h3>
                    <p><strong>Name:</strong> {}</p>
                    <p><strong>URL:</strong> <a href="{}" target="_blank">{}</a></p>
                    <p><strong>Selector:</strong> {} ({})</p>
                    <p><strong>Schedule:</strong> {}</p>
                </div>
                
                <div class="info-card">
                    <h3>📊 Result Details</h3>
                    <p><strong>Status:</strong> <span class="status {}">{}</span></p>
                    <p><strong>Timestamp:</strong> {}</p>
                    <p><strong>Result ID:</strong> {}</p>
                    {}
                </div>
            </div>
            
            <div class="data-section">
                <h3>📄 Scraped Data</h3>
                <div class="data-content">{}</div>
            </div>
        </div>
        
        <div class="footer">
            <p>Generated on {} | Automated Web Scraper</p>
        </div>
    </div>
</body>
</html>"#,
            job.name,
            result_id,
            job.name,
            job.url,
            job.url,
            job.selector,
            job.selector_type,
            job.schedule,
            status_class,
            status_text,
            result.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            result_id,
            if !result.success {
                format!("<p><strong>Error:</strong> <span style=\"color: #dc3545;\">{}</span></p>", html_escape(error_message))
            } else {
                String::new()
            },
            html_escape(&result.scraped_data),
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );
        
        fs::write(&file_path, html_content)
            .map_err(|e| anyhow!("Failed to write HTML file: {}", e))?;
        
        Ok(file_path)
    }
    
    pub fn get_export_directory(&self) -> &Path {
        &self.export_dir
    }
    
    pub fn list_export_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        if self.export_dir.exists() {
            for entry in fs::read_dir(&self.export_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            {
                files.push(entry.path());
            }
        }
        
        // Sort by modification time (newest first)
        files.sort_by(|a, b| {
            let a_time = a.metadata().and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH);
            let b_time = b.metadata().and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH);
            b_time.cmp(&a_time)
        });
        
        Ok(files)
    }
    
    pub fn delete_export_file<P: AsRef<Path>>(&self, file_path: P) -> Result<()> {
        let file_path = file_path.as_ref();
        
        // Ensure the file is within our export directory for security
        if !file_path.starts_with(&self.export_dir) {
            return Err(anyhow!("File is not within the export directory"));
        }
        
        fs::remove_file(file_path)
            .map_err(|e| anyhow!("Failed to delete export file: {}", e))?;
        
        Ok(())
    }
    
    pub fn cleanup_old_exports(&self, days_old: u64) -> Result<usize> {
        let cutoff_time = std::time::SystemTime::now() - std::time::Duration::from_secs(days_old * 24 * 60 * 60);
        let mut deleted_count = 0;
        
        if self.export_dir.exists() {
            for entry in fs::read_dir(&self.export_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified_time) = metadata.modified() {
                        if modified_time < cutoff_time {
                            if fs::remove_file(entry.path()).is_ok() {
                                deleted_count += 1;
                            }
                        }
                    }
                }
            }
        }
        
        info!("Cleaned up {} old export files", deleted_count);
        Ok(deleted_count)
    }
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

fn generate_table_rows(results: &[ScrapingResult]) -> String {
    results
        .iter()
        .map(|result| {
            let status_class = if result.success { "success" } else { "error" };
            let status_text = if result.success { "✓ Success" } else { "✗ Failed" };
            let error_message = result.error_message.as_deref().unwrap_or("-");
            
            format!(
                "            <tr>\n                <td class=\"timestamp\">{}</td>\n                <td class=\"{}\"><strong>{}</strong></td>\n                <td class=\"data-cell\">{}</td>\n                <td class=\"data-cell\">{}</td>\n            </tr>",
                result.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                status_class,
                status_text,
                html_escape(&result.scraped_data),
                html_escape(error_message)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("normal_name"), "normal_name");
        assert_eq!(sanitize_filename("name/with\\bad:chars*"), "name_with_bad_chars_");
        assert_eq!(sanitize_filename("name<with>quotes\"and|pipes"), "name_with_quotes_and_pipes");
    }
    
    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("normal text"), "normal text");
        assert_eq!(html_escape("<script>alert('xss')</script>"), "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;");
        assert_eq!(html_escape("A & B > C"), "A &amp; B &gt; C");
    }
    
    #[tokio::test]
    async fn test_export_service_creation() {
        let temp_dir = TempDir::new().unwrap();
        let export_service = ExportService::new(temp_dir.path()).unwrap();
        
        assert_eq!(export_service.get_export_directory(), temp_dir.path());
        assert!(temp_dir.path().exists());
    }
}