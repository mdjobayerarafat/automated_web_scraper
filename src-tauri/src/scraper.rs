use reqwest::Client;
use scraper::{Html, Selector};
use regex::Regex;
use std::time::Duration;
use crate::models::*;
use anyhow::{Result, anyhow};
use log::{info, error, warn};

pub struct WebScraper {
    client: Client,
}

impl WebScraper {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300)) // 5 minutes timeout
            .build()
            .expect("Failed to create HTTP client");
        
        WebScraper { client }
    }

    pub async fn scrape_job(&self, job: &ScrapingJob) -> Result<Vec<String>> {
        info!("Starting scrape for job: {} ({})", job.name, job.url);
        
        let mut request = self.client.get(&job.url);
        
        // Set custom user agent if provided
        if let Some(user_agent) = &job.user_agent {
            request = request.header("User-Agent", user_agent);
        } else {
            request = request.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36");
        }
        
        // Set proxy if provided
        if let Some(proxy_url) = &job.proxy_url {
            let proxy = reqwest::Proxy::all(proxy_url)
                .map_err(|e| anyhow!("Invalid proxy URL: {}", e))?;
            let client_with_proxy = Client::builder()
                .proxy(proxy)
                .timeout(Duration::from_secs(300)) // 5 minutes timeout
                .build()
                .map_err(|e| anyhow!("Failed to create client with proxy: {}", e))?;
            request = client_with_proxy.get(&job.url);
            if let Some(user_agent) = &job.user_agent {
                request = request.header("User-Agent", user_agent);
            }
        }
        
        let response = request.send().await
            .map_err(|e| anyhow!("Failed to fetch URL: {}", e))?;
        
        if !response.status().is_success() {
            return Err(anyhow!("HTTP error: {}", response.status()));
        }
        
        let html_content = response.text().await
            .map_err(|e| anyhow!("Failed to read response body: {}", e))?;
        
        match job.selector_type {
            SelectorType::CSS => self.scrape_with_css(&html_content, &job.selector, &job.data_type),
            SelectorType::Regex => self.scrape_with_regex(&html_content, &job.selector),
        }
    }
    
    fn scrape_with_css(&self, html: &str, selector_str: &str, data_type: &DataType) -> Result<Vec<String>> {
        let document = Html::parse_document(html);
        let selector = Selector::parse(selector_str)
            .map_err(|e| anyhow!("Invalid CSS selector '{}': {:?}", selector_str, e))?;
        
        let mut results = Vec::new();
        
        for element in document.select(&selector) {
            let data = match data_type {
                DataType::Text => {
                    element.text().collect::<Vec<_>>().join(" ").trim().to_string()
                }
                DataType::Attribute(attr_name) => {
                    element.value().attr(attr_name)
                        .unwrap_or_default()
                        .to_string()
                }
            };
            
            if !data.is_empty() {
                results.push(data);
            }
        }
        
        if results.is_empty() {
            warn!("No data found with CSS selector: {}", selector_str);
        } else {
            info!("Found {} items with CSS selector", results.len());
        }
        
        Ok(results)
    }
    
    fn scrape_with_regex(&self, text: &str, pattern: &str) -> Result<Vec<String>> {
        let regex = Regex::new(pattern)
            .map_err(|e| anyhow!("Invalid regex pattern '{}': {}", pattern, e))?;
        
        let mut results = Vec::new();
        
        for captures in regex.captures_iter(text) {
            // If there are capture groups, use the first one; otherwise use the full match
            let matched_text = if captures.len() > 1 {
                captures.get(1).map(|m| m.as_str()).unwrap_or_default()
            } else {
                captures.get(0).map(|m| m.as_str()).unwrap_or_default()
            };
            
            if !matched_text.is_empty() {
                results.push(matched_text.to_string());
            }
        }
        
        if results.is_empty() {
            warn!("No data found with regex pattern: {}", pattern);
        } else {
            info!("Found {} items with regex pattern", results.len());
        }
        
        Ok(results)
    }
    
    pub async fn test_scrape(&self, job: &ScrapingJob) -> Result<Vec<String>> {
        info!("Testing scrape for job: {}", job.name);
        
        let results = self.scrape_job(job).await?;
        
        // Limit test results to first 5 items to avoid overwhelming the UI
        let limited_results = results.into_iter().take(5).collect();
        
        Ok(limited_results)
    }
    
    pub async fn validate_url(&self, url: &str) -> Result<bool> {
        let response = self.client.head(url).send().await
            .map_err(|e| anyhow!("Failed to validate URL: {}", e))?;
        
        Ok(response.status().is_success())
    }
    
    pub fn validate_css_selector(&self, selector: &str) -> Result<bool> {
        match Selector::parse(selector) {
            Ok(_) => Ok(true),
            Err(e) => Err(anyhow!("Invalid CSS selector: {:?}", e)),
        }
    }
    
    pub fn validate_regex_pattern(&self, pattern: &str) -> Result<bool> {
        match Regex::new(pattern) {
            Ok(_) => Ok(true),
            Err(e) => Err(anyhow!("Invalid regex pattern: {}", e)),
        }
    }
}

impl Default for WebScraper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_css_selector_validation() {
        let scraper = WebScraper::new();
        
        assert!(scraper.validate_css_selector("div.class").is_ok());
        assert!(scraper.validate_css_selector("#id").is_ok());
        assert!(scraper.validate_css_selector("p > a").is_ok());
        assert!(scraper.validate_css_selector("[data-test]").is_ok());
        
        // Invalid selectors
        assert!(scraper.validate_css_selector(">>>").is_err());
        assert!(scraper.validate_css_selector("[[[").is_err());
    }
    
    #[test]
    fn test_regex_validation() {
        let scraper = WebScraper::new();
        
        assert!(scraper.validate_regex_pattern(r"\d+").is_ok());
        assert!(scraper.validate_regex_pattern(r"[a-zA-Z]+").is_ok());
        assert!(scraper.validate_regex_pattern(r"(\w+)@(\w+\.\w+)").is_ok());
        
        // Invalid regex
        assert!(scraper.validate_regex_pattern(r"[").is_err());
        assert!(scraper.validate_regex_pattern(r"*").is_err());
    }
    
    #[test]
    fn test_css_scraping() {
        let scraper = WebScraper::new();
        let html = r#"
            <html>
                <body>
                    <div class="content">
                        <p>First paragraph</p>
                        <p>Second paragraph</p>
                        <a href="https://example.com">Link</a>
                    </div>
                </body>
            </html>
        "#;
        
        let results = scraper.scrape_with_css(html, "p", &DataType::Text).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], "First paragraph");
        assert_eq!(results[1], "Second paragraph");
        
        let link_results = scraper.scrape_with_css(html, "a", &DataType::Attribute("href".to_string())).unwrap();
        assert_eq!(link_results.len(), 1);
        assert_eq!(link_results[0], "https://example.com");
    }
    
    #[test]
    fn test_regex_scraping() {
        let scraper = WebScraper::new();
        let text = "Contact us at john@example.com or jane@test.org for more info";
        
        let results = scraper.scrape_with_regex(text, r"(\w+)@(\w+\.\w+)").unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], "john"); // First capture group
        assert_eq!(results[1], "jane");
        
        // Test without capture groups
        let email_results = scraper.scrape_with_regex(text, r"\w+@\w+\.\w+").unwrap();
        assert_eq!(email_results.len(), 2);
        assert_eq!(email_results[0], "john@example.com");
        assert_eq!(email_results[1], "jane@test.org");
    }
}