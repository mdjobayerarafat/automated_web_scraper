import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './App.css';
import SplashScreen from './SplashScreen';
import HackingLoader from './HackingLoader';

// Import icons
import jobsIcon from './assets/jobs_icon.png';
import resultsIcon from './assets/results_icon.png';
import filesIcon from './assets/files_icon.png';
import settingsIcon from './assets/settings_icon.png';
import webScrapperLogo from './assets/web_scrapper.png';
import jobayerAvatar from './assets/jobayer.jpg';

// Types
interface ScrapingJob {
  id?: number;
  name: string;
  url: string;
  selector_type: 'CSS' | 'Regex';
  selector: string;
  data_type: 'Text' | 'Attribute';
  attribute_name?: string;
  schedule: string;
  is_active: boolean;
  user_agent?: string;
  proxy_url?: string;
  created_at?: string;
  updated_at?: string;
}

interface ScrapingResult {
  id: number;
  job_id: number;
  scraped_data: string;
  timestamp: string;
}

interface JobStats {
  total_jobs: number;
  active_jobs: number;
  total_results: number;
  last_run?: string;
}

interface EmailConfig {
  smtp_server: string;
  smtp_port: number;
  username: string;
  password: string;
  from_email: string;
  to_email: string;
  use_tls: boolean;
}

interface ExportRequest {
  job_id: number;
  format: 'CSV' | 'JSON' | 'HTML';
  start_date?: string;
  end_date?: string;
}

interface IndividualExportRequest {
  result_id: number;
  format: 'CSV' | 'JSON' | 'HTML';
}

interface ExportFileInfo {
  name: string;
  path: string;
  size: number;
  modified_timestamp: number;
  file_type: string;
}

function App() {
  const [showSplash, setShowSplash] = useState(true);
  const [splashStartTime] = useState(Date.now());
  const [activeTab, setActiveTab] = useState<'jobs' | 'results' | 'files' | 'documentation' | 'settings'>('jobs');
  const [jobs, setJobs] = useState<ScrapingJob[]>([]);
  const [selectedJob, setSelectedJob] = useState<ScrapingJob | null>(null);
  const [results, setResults] = useState<ScrapingResult[]>([]);
  const [stats, setStats] = useState<JobStats | null>(null);
  const [emailConfig, setEmailConfig] = useState<EmailConfig | null>(null);
  const [exportFiles, setExportFiles] = useState<ExportFileInfo[]>([]);
  const [selectedFile, setSelectedFile] = useState<ExportFileInfo | null>(null);
  const [fileContent, setFileContent] = useState<string>('');
  const [isLoading, setIsLoading] = useState(false);
  const [loadingProgress, setLoadingProgress] = useState(0);
  const [loadingMessage, setLoadingMessage] = useState('Processing...');
  const [message, setMessage] = useState<{ type: 'success' | 'error' | 'info'; text: string } | null>(null);
  const [showJobForm, setShowJobForm] = useState(false);
  const [editingJob, setEditingJob] = useState<ScrapingJob | null>(null);
  const [theme, setTheme] = useState<'dark' | 'light'>('light');

  const handleSplashComplete = () => {
    const elapsedTime = Date.now() - splashStartTime;
    const minSplashDuration = 3000; // 3 seconds minimum
    
    if (elapsedTime < minSplashDuration) {
      // Wait for the remaining time to ensure minimum splash duration
      setTimeout(() => {
        setShowSplash(false);
      }, minSplashDuration - elapsedTime);
    } else {
      setShowSplash(false);
    }
  };

  // Initialize app
  useEffect(() => {
    initializeApp();
    // Set initial theme
    document.documentElement.setAttribute('data-theme', theme);
  }, []);

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
  }, [theme]);

  const initializeApp = async () => {
    try {
      setIsLoading(true);
      await invoke('initialize_app');
      await loadJobs();
      await loadStats();
      await loadEmailConfig();
      showMessage('Application initialized successfully', 'success');
    } catch (error) {
      showMessage(`Failed to initialize app: ${error}`, 'error');
    } finally {
      setIsLoading(false);
    }
  };

  const loadJobs = async () => {
    try {
      const jobList = await invoke<ScrapingJob[]>('get_all_jobs');
      console.log('Loaded jobs:', jobList);
      console.log('Number of jobs:', jobList.length);
      setJobs(jobList);
    } catch (error) {
      console.error('Error loading jobs:', error);
      showMessage(`Failed to load jobs: ${error}`, 'error');
    }
  };

  const loadStats = async () => {
    try {
      const jobStats = await invoke<JobStats>('get_job_stats');
      setStats(jobStats);
    } catch (error) {
      console.error('Failed to load stats:', error);
    }
  };

  const loadEmailConfig = async () => {
    try {
      const config = await invoke<EmailConfig | null>('get_email_config');
      setEmailConfig(config);
    } catch (error) {
      console.error('Failed to load email config:', error);
    }
  };

  const loadJobResults = async (jobId: number) => {
    try {
      const jobResults = await invoke<ScrapingResult[]>('get_job_results', { jobId, limit: 100 });
      setResults(jobResults);
    } catch (error) {
      showMessage(`Failed to load results: ${error}`, 'error');
    }
  };

  const showMessage = (text: string, type: 'success' | 'error' | 'info') => {
    setMessage({ text, type });
    setTimeout(() => setMessage(null), 5000);
  };

  const handleCreateJob = async (job: Omit<ScrapingJob, 'id'>) => {
    try {
      setIsLoading(true);
      setLoadingProgress(0);
      setLoadingMessage('Creating new job...');
      
      const progressSteps = [
        { progress: 25, message: 'Validating job configuration...' },
        { progress: 50, message: 'Saving job to database...' },
        { progress: 75, message: 'Updating job statistics...' },
        { progress: 90, message: 'Refreshing job list...' }
      ];
      
      for (const step of progressSteps) {
        setLoadingProgress(step.progress);
        setLoadingMessage(step.message);
        await new Promise(resolve => setTimeout(resolve, 200));
      }
      
      setLoadingProgress(100);
      setLoadingMessage('Finalizing...');
      
      await invoke('create_job', { job });
      await loadJobs();
      await loadStats();
      setShowJobForm(false);
      showMessage('Job created successfully', 'success');
    } catch (error) {
      showMessage(`Failed to create job: ${error}`, 'error');
    } finally {
      setIsLoading(false);
      setLoadingProgress(0);
    }
  };

  const handleUpdateJob = async (job: ScrapingJob) => {
    try {
      setIsLoading(true);
      setLoadingProgress(0);
      setLoadingMessage('Updating job...');
      
      const progressSteps = [
        { progress: 30, message: 'Validating changes...' },
        { progress: 60, message: 'Updating job in database...' },
        { progress: 80, message: 'Refreshing job data...' },
        { progress: 95, message: 'Updating statistics...' }
      ];
      
      for (const step of progressSteps) {
        setLoadingProgress(step.progress);
        setLoadingMessage(step.message);
        await new Promise(resolve => setTimeout(resolve, 200));
      }
      
      setLoadingProgress(100);
      setLoadingMessage('Completing update...');
      
      await invoke('update_job', { job });
      await loadJobs();
      await loadStats();
      setEditingJob(null);
      setShowJobForm(false);
      showMessage('Job updated successfully', 'success');
    } catch (error) {
      showMessage(`Failed to update job: ${error}`, 'error');
    } finally {
      setIsLoading(false);
      setLoadingProgress(0);
    }
  };

  const handleDeleteJob = async (id: number) => {
    if (!confirm('Are you sure you want to delete this job?')) return;
    
    try {
      setIsLoading(true);
      setLoadingProgress(0);
      setLoadingMessage('Deleting job...');
      
      const progressSteps = [
        { progress: 25, message: 'Removing job from database...' },
        { progress: 50, message: 'Cleaning up job data...' },
        { progress: 75, message: 'Updating statistics...' },
        { progress: 90, message: 'Refreshing job list...' }
      ];
      
      for (const step of progressSteps) {
        setLoadingProgress(step.progress);
        setLoadingMessage(step.message);
        await new Promise(resolve => setTimeout(resolve, 200));
      }
      
      setLoadingProgress(100);
      setLoadingMessage('Finalizing deletion...');
      
      await invoke('delete_job', { id });
      await loadJobs();
      await loadStats();
      if (selectedJob?.id === id) {
        setSelectedJob(null);
        setResults([]);
      }
      showMessage('Job deleted successfully', 'success');
    } catch (error) {
      showMessage(`Failed to delete job: ${error}`, 'error');
    } finally {
      setIsLoading(false);
      setLoadingProgress(0);
    }
  };

  const handleRunJobNow = async (id: number) => {
    try {
      setIsLoading(true);
      setLoadingProgress(0);
      setLoadingMessage('Starting job execution...');
      
      // Simulate progress updates for job execution
      const progressSteps = [
        { progress: 15, message: 'Preparing scraping environment...' },
        { progress: 30, message: 'Loading job configuration...' },
        { progress: 45, message: 'Connecting to target website...' },
        { progress: 65, message: 'Scraping data from website...' },
        { progress: 85, message: 'Saving results to database...' },
        { progress: 95, message: 'Finalizing job execution...' }
      ];
      
      // Update progress gradually
      for (const step of progressSteps) {
        setLoadingProgress(step.progress);
        setLoadingMessage(step.message);
        await new Promise(resolve => setTimeout(resolve, 400)); // Slightly longer delay for job execution
      }
      
      setLoadingProgress(100);
      setLoadingMessage('Completing job...');
      
      const result = await invoke<string[]>('run_job_now', { id });
      await loadStats();
      if (selectedJob?.id === id) {
        await loadJobResults(id);
      }
      showMessage(`Job executed successfully. Found ${result.length} results.`, 'success');
    } catch (error) {
      showMessage(`Failed to run job: ${error}`, 'error');
    } finally {
      setIsLoading(false);
      setLoadingProgress(0);
    }
  };

  const handleTestJob = async (job: ScrapingJob) => {
    try {
      setIsLoading(true);
      setLoadingProgress(0);
      setLoadingMessage('Initializing test scrape...');
      
      // Simulate progress updates
      const progressSteps = [
        { progress: 10, message: 'Validating URL and selectors...' },
        { progress: 25, message: 'Establishing connection...' },
        { progress: 40, message: 'Fetching webpage content...' },
        { progress: 60, message: 'Parsing HTML structure...' },
        { progress: 80, message: 'Extracting data with selectors...' },
        { progress: 95, message: 'Processing results...' }
      ];
      
      // Update progress gradually
      for (const step of progressSteps) {
        setLoadingProgress(step.progress);
        setLoadingMessage(step.message);
        await new Promise(resolve => setTimeout(resolve, 300)); // Small delay for visual effect
      }
      
      setLoadingProgress(100);
      setLoadingMessage('Completing test...');
      
      const result = await invoke<string[]>('test_scrape_job', { job });
      showMessage(`Test successful! Found ${result.length} results: ${result.slice(0, 3).join(', ')}${result.length > 3 ? '...' : ''}`, 'success');
    } catch (error) {
      showMessage(`Test failed: ${error}`, 'error');
    } finally {
      setIsLoading(false);
      setLoadingProgress(0);
    }
  };

  const handleExportResults = async (jobId: number, format: 'CSV' | 'JSON' | 'HTML') => {
    try {
      setIsLoading(true);
      const request: ExportRequest = { job_id: jobId, format };
      const filePath = await invoke<string>('export_job_results', { request });
      showMessage(`Results exported to: ${filePath}`, 'success');
    } catch (error) {
      showMessage(`Export failed: ${error}`, 'error');
    } finally {
      setIsLoading(false);
    }
  };

  const handleExportIndividualResult = async (resultId: number, format: 'CSV' | 'JSON' | 'HTML') => {
    try {
      setIsLoading(true);
      const request: IndividualExportRequest = { result_id: resultId, format };
      const filePath = await invoke<string>('export_individual_result', { request });
      showMessage(`Individual result exported to: ${filePath}`, 'success');
      // Refresh the export files list
      if (activeTab === 'files') {
        loadExportFiles();
      }
    } catch (error) {
      showMessage(`Individual export failed: ${error}`, 'error');
    } finally {
      setIsLoading(false);
    }
  };

  const handleSaveEmailConfig = async (config: EmailConfig) => {
    try {
      setIsLoading(true);
      await invoke('save_email_config', { config });
      setEmailConfig(config);
      showMessage('Email configuration saved successfully', 'success');
    } catch (error) {
      showMessage(`Failed to save email config: ${error}`, 'error');
    } finally {
      setIsLoading(false);
    }
  };

  const handleTestEmailConnection = async () => {
    try {
      setIsLoading(true);
      await invoke('test_email_connection');
      showMessage('Email connection test successful', 'success');
    } catch (error) {
      showMessage(`Email connection test failed: ${error}`, 'error');
    } finally {
      setIsLoading(false);
    }
  };

  const handleThemeToggle = (newTheme: 'dark' | 'light') => {
    setTheme(newTheme);
    document.documentElement.setAttribute('data-theme', newTheme);
  };

  // File handling functions
  const loadExportFiles = async () => {
    try {
      setIsLoading(true);
      const files = await invoke<ExportFileInfo[]>('list_export_files');
      setExportFiles(files);
    } catch (error) {
      showMessage(`Failed to load export files: ${error}`, 'error');
    } finally {
      setIsLoading(false);
    }
  };

  const handleFileSelect = async (file: ExportFileInfo) => {
    try {
      setIsLoading(true);
      setSelectedFile(file);
      const content = await invoke<string>('read_export_file', { filePath: file.path });
      setFileContent(content);
    } catch (error) {
      showMessage(`Failed to read file: ${error}`, 'error');
      setFileContent('');
    } finally {
      setIsLoading(false);
    }
  };

  const handleOpenExportDirectory = async () => {
    try {
      await invoke('open_export_directory');
      showMessage('Export directory opened', 'info');
    } catch (error) {
      showMessage(`Failed to open directory: ${error}`, 'error');
    }
  };

  const handleDeleteFile = async (file: ExportFileInfo) => {
    if (!confirm(`Are you sure you want to delete ${file.name}?`)) {
      return;
    }
    
    try {
      setIsLoading(true);
      await invoke('delete_export_file', { filePath: file.path });
      showMessage('File deleted successfully', 'success');
      await loadExportFiles(); // Refresh the file list
      if (selectedFile?.path === file.path) {
        setSelectedFile(null);
        setFileContent('');
      }
    } catch (error) {
      showMessage(`Failed to delete file: ${error}`, 'error');
    } finally {
      setIsLoading(false);
    }
  };

  // Load export files when Files tab is selected
  useEffect(() => {
    if (activeTab === 'files') {
      loadExportFiles();
    }
  }, [activeTab]);

  // Show splash screen first
  if (showSplash) {
    return <SplashScreen onComplete={handleSplashComplete} />;
  }

  return (
    <>
      {isLoading && <HackingLoader isVisible={isLoading} progress={loadingProgress} message={loadingMessage} />}
      <div className="desktop-app">
      {/* Sidebar Navigation */}
      <aside className="desktop-sidebar">
        <div className="sidebar-header">
          <div className="app-logo">
            <img src={webScrapperLogo} alt="Web Scraper" className="logo-icon" />
          </div>
          <div className="app-title-text">
            Automate Web Scrapper
          </div>
          {stats && (
            <div className="app-stats">
              <div className="stat-item">
                <span className="stat-value">{stats.total_jobs}</span>
                <span className="stat-label">Jobs</span>
              </div>
              <div className="stat-item">
                <span className="stat-value">{stats.active_jobs}</span>
                <span className="stat-label">Active</span>
              </div>
              <div className="stat-item">
                <span className="stat-value">{stats.total_results}</span>
                <span className="stat-label">Results</span>
              </div>
            </div>
          )}
        </div>

        <nav className="sidebar-nav">
          <button 
                className={`nav-item ${activeTab === 'jobs' ? 'active' : ''}`}
                onClick={() => setActiveTab('jobs')}
              >
                <img src={jobsIcon} alt="Jobs" className="nav-icon" />
                <span className="nav-label">Jobs</span>
              </button>
              <button 
                className={`nav-item ${activeTab === 'results' ? 'active' : ''}`}
                onClick={() => setActiveTab('results')}
              >
                <img src={resultsIcon} alt="Results" className="nav-icon" />
                <span className="nav-label">Results</span>
              </button>
              <button 
                className={`nav-item ${activeTab === 'files' ? 'active' : ''}`}
                onClick={() => setActiveTab('files')}
              >
                <img src={filesIcon} alt="Files" className="nav-icon" />
                <span className="nav-label">Files</span>
              </button>
              <button 
                className={`nav-item ${activeTab === 'documentation' ? 'active' : ''}`}
                onClick={() => setActiveTab('documentation')}
              >
                <span className="nav-icon">📚</span>
                <span className="nav-label">Documentation</span>
              </button>
              <button 
                className={`nav-item ${activeTab === 'settings' ? 'active' : ''}`}
                onClick={() => setActiveTab('settings')}
              >
                <img src={settingsIcon} alt="Settings" className="nav-icon" />
                <span className="nav-label">Settings</span>
              </button>
        </nav>
      </aside>

      {/* Main Content Area */}
      <div className="desktop-main">
        {/* Top Bar with Messages */}
        {message && (
          <div className={`desktop-message message-${message.type}`}>
            <span>{message.text}</span>
            <button onClick={() => setMessage(null)} className="message-close">×</button>
          </div>
        )}

        <main className="desktop-content">
        {activeTab === 'jobs' && (
          <JobsTab
            jobs={jobs}
            selectedJob={selectedJob}
            setSelectedJob={setSelectedJob}
            showJobForm={showJobForm}
            setShowJobForm={setShowJobForm}
            editingJob={editingJob}
            setEditingJob={setEditingJob}
            onCreateJob={handleCreateJob}
            onUpdateJob={handleUpdateJob}
            onDeleteJob={handleDeleteJob}
            onRunJob={handleRunJobNow}
            onTestJob={handleTestJob}
            isLoading={isLoading}
          />
        )}

        {activeTab === 'results' && (
          <ResultsTab
            jobs={jobs}
            selectedJob={selectedJob}
            setSelectedJob={setSelectedJob}
            results={results}
            onLoadResults={loadJobResults}
            onExportResults={handleExportResults}
            onExportIndividualResult={handleExportIndividualResult}
            isLoading={isLoading}
          />
        )}

        {activeTab === 'files' && (
          <FilesTab
            exportFiles={exportFiles}
            selectedFile={selectedFile}
            fileContent={fileContent}
            onFileSelect={handleFileSelect}
            onOpenDirectory={handleOpenExportDirectory}
            onDeleteFile={handleDeleteFile}
            onRefresh={loadExportFiles}
            isLoading={isLoading}
          />
        )}

        {activeTab === 'documentation' && (
          <DocumentationTab />
        )}

        {activeTab === 'settings' && (
          <SettingsTab
            emailConfig={emailConfig}
            onSaveEmailConfig={handleSaveEmailConfig}
            onTestEmailConnection={handleTestEmailConnection}
            theme={theme}
            onThemeToggle={handleThemeToggle}
            isLoading={isLoading}
          />
        )}
        </main>

        {isLoading && (
          <div className="loading-overlay">
            <div className="loading-spinner">Loading...</div>
          </div>
        )}
      </div>
    </div>
    </>
  );
}

// Jobs Tab Component
interface JobsTabProps {
  jobs: ScrapingJob[];
  selectedJob: ScrapingJob | null;
  setSelectedJob: (job: ScrapingJob | null) => void;
  showJobForm: boolean;
  setShowJobForm: (show: boolean) => void;
  editingJob: ScrapingJob | null;
  setEditingJob: (job: ScrapingJob | null) => void;
  onCreateJob: (job: Omit<ScrapingJob, 'id'>) => void;
  onUpdateJob: (job: ScrapingJob) => void;
  onDeleteJob: (id: number) => void;
  onRunJob: (id: number) => void;
  onTestJob: (job: ScrapingJob) => void;
  isLoading: boolean;
}

function JobsTab({ 
  jobs, 
  selectedJob, 
  setSelectedJob, 
  showJobForm, 
  setShowJobForm, 
  editingJob, 
  setEditingJob,
  onCreateJob, 
  onUpdateJob, 
  onDeleteJob, 
  onRunJob, 
  onTestJob, 
  isLoading 
}: JobsTabProps) {
  console.log('JobsTab received jobs:', jobs);
  console.log('JobsTab jobs length:', jobs.length);
  
  const activeJobs = jobs.filter(job => job.is_active).length;
  const totalJobs = jobs.length;
  
  return (
    <div className="jobs-tab">
      <div className="jobs-sidebar">
        <div className="jobs-header">
          <div className="header-content">
            <h2>Scraping Jobs</h2>
            <div className="jobs-stats">
              <span className="stat-item">
                <span className="stat-number">{totalJobs}</span>
                <span className="stat-label">Total</span>
              </span>
              <span className="stat-divider">|</span>
              <span className="stat-item">
                <span className="stat-number active">{activeJobs}</span>
                <span className="stat-label">Active</span>
              </span>
            </div>
          </div>
          <button 
            className="btn btn-primary add-job-btn"
            onClick={() => {
              setEditingJob(null);
              setShowJobForm(true);
            }}
            disabled={isLoading}
          >
            <span className="btn-icon">+</span>
            <span className="btn-text">Add Job</span>
          </button>
        </div>
        
        <div className="jobs-list">
          {jobs.length === 0 ? (
            <div className="empty-jobs-list">
              <div className="empty-icon">📋</div>
              <p className="empty-title">No Jobs Yet</p>
              <p className="empty-subtitle">Create your first scraping job to get started</p>
            </div>
          ) : (
            jobs.map(job => (
              <div 
                key={job.id} 
                className={`job-item ${selectedJob?.id === job.id ? 'selected' : ''}`}
                onClick={() => setSelectedJob(job)}
              >
                <div className="job-header">
                  <div className="job-name">{job.name}</div>
                  <div className="job-type-badge">
                    {job.selector_type}
                  </div>
                </div>
                <div className="job-url">{job.url}</div>
                <div className="job-meta">
                  <div className="job-status">
                    <span className={`status ${job.is_active ? 'active' : 'inactive'}`}>
                      <span className="status-dot"></span>
                      {job.is_active ? 'Active' : 'Inactive'}
                    </span>
                    <span className="schedule">{job.schedule}</span>
                  </div>
                  <div className="job-data-type">
                    <span className="data-type-label">{job.data_type}</span>
                  </div>
                </div>
                {job.created_at && (
                  <div className="job-timestamp">
                    Created: {new Date(job.created_at).toLocaleDateString()}
                  </div>
                )}
              </div>
            ))
          )}
        </div>
      </div>

      <div className="jobs-content">
        {showJobForm ? (
          <JobForm
            job={editingJob}
            onSave={editingJob ? onUpdateJob : onCreateJob}
            onCancel={() => {
              setShowJobForm(false);
              setEditingJob(null);
            }}
            onTest={onTestJob}
            isLoading={isLoading}
          />
        ) : selectedJob ? (
          <JobDetails
            job={selectedJob}
            onEdit={() => {
              setEditingJob(selectedJob);
              setShowJobForm(true);
            }}
            onDelete={() => selectedJob.id && onDeleteJob(selectedJob.id)}
            onRun={() => selectedJob.id && onRunJob(selectedJob.id)}
            isLoading={isLoading}
          />
        ) : (
          <div className="empty-state">
            <div className="empty-state-icon">🎯</div>
            <h3 className="empty-state-title">Ready to Start Scraping</h3>
            <p className="empty-state-description">
              Select a job from the sidebar to view details and manage your scraping tasks,<br/>
              or create a new job to begin extracting data from websites.
            </p>
            <button 
              className="btn btn-primary"
              onClick={() => {
                setEditingJob(null);
                setShowJobForm(true);
              }}
            >
              Create Your First Job
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

// Job Form Component
interface JobFormProps {
  job: ScrapingJob | null;
  onSave: (job: ScrapingJob | Omit<ScrapingJob, 'id'>) => void;
  onCancel: () => void;
  onTest: (job: ScrapingJob) => void;
  isLoading: boolean;
}

function JobForm({ job, onSave, onCancel, onTest, isLoading }: JobFormProps) {
  // Transform backend data_type to frontend format
  const transformJobForForm = (job: ScrapingJob | null) => {
    if (!job) {
      return {
        name: '',
        url: '',
        selector_type: 'CSS' as const,
        selector: '',
        data_type: 'Text' as const,
        schedule: 'daily',
        is_active: true,
      };
    }
    
    let data_type: 'Text' | 'Attribute' = 'Text';
    let attribute_name: string | undefined;
    
    if (typeof job.data_type === 'string' && job.data_type.startsWith('Attribute(')) {
      data_type = 'Attribute';
      const match = job.data_type.match(/Attribute\((.+)\)/);
      attribute_name = match ? match[1] : '';
    } else if (job.data_type === 'Text') {
      data_type = 'Text';
    }
    
    return {
      ...job,
      data_type,
      attribute_name
    };
  };
  
  const [formData, setFormData] = useState<Omit<ScrapingJob, 'id'>>(
    transformJobForForm(job)
  );

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    // Transform data_type for backend compatibility
    const transformedData = {
      ...formData,
      data_type: formData.data_type === 'Attribute' && formData.attribute_name 
        ? `Attribute(${formData.attribute_name})` as any
        : formData.data_type
    };
    
    if (job) {
      onSave({ ...transformedData, id: job.id });
    } else {
      onSave(transformedData);
    }
  };

  const handleTest = () => {
    // Transform data_type for backend compatibility
    const transformedData = {
      ...formData,
      data_type: formData.data_type === 'Attribute' && formData.attribute_name 
        ? `Attribute(${formData.attribute_name})` as any
        : formData.data_type
    };
    onTest(transformedData as ScrapingJob);
  };

  return (
    <form className="job-form" onSubmit={handleSubmit}>
      <h3>{job ? 'Edit Job' : 'Create New Job'}</h3>
      
      <div className="form-group">
        <label>Job Name</label>
        <input
          type="text"
          value={formData.name}
          onChange={(e) => setFormData({ ...formData, name: e.target.value })}
          required
        />
      </div>

      <div className="form-group">
        <label>Target URL</label>
        <input
          type="url"
          value={formData.url}
          onChange={(e) => setFormData({ ...formData, url: e.target.value })}
          required
        />
      </div>

      <div className="form-row">
        <div className="form-group">
          <label>Selector Type</label>
          <select
            value={formData.selector_type}
            onChange={(e) => setFormData({ ...formData, selector_type: e.target.value as 'CSS' | 'Regex' })}
          >
            <option value="CSS">CSS Selector</option>
            <option value="Regex">Regex Pattern</option>
          </select>
        </div>

        <div className="form-group">
          <label>Data Type</label>
          <select
            value={formData.data_type}
            onChange={(e) => setFormData({ ...formData, data_type: e.target.value as 'Text' | 'Attribute' })}
          >
            <option value="Text">Text Content</option>
            <option value="Attribute">Attribute Value</option>
          </select>
        </div>
      </div>

      <div className="form-group">
        <label>{formData.selector_type === 'CSS' ? 'CSS Selector' : 'Regex Pattern'}</label>
        <input
          type="text"
          value={formData.selector}
          onChange={(e) => setFormData({ ...formData, selector: e.target.value })}
          placeholder={formData.selector_type === 'CSS' ? 'e.g., .title, #content h2' : 'e.g., <title>(.*?)</title>'}
          required
        />
      </div>

      {formData.data_type === 'Attribute' && (
        <div className="form-group">
          <label>Attribute Name</label>
          <input
            type="text"
            value={formData.attribute_name || ''}
            onChange={(e) => setFormData({ ...formData, attribute_name: e.target.value })}
            placeholder="e.g., href, src, alt"
          />
        </div>
      )}

      <div className="form-group">
        <label>Schedule</label>
        <select
          value={formData.schedule}
          onChange={(e) => setFormData({ ...formData, schedule: e.target.value })}
        >
          <option value="daily">Daily</option>
          <option value="hourly">Hourly</option>
          <option value="weekly">Weekly</option>
          <option value="monthly">Monthly</option>
        </select>
      </div>

      <div className="form-group">
        <label>User Agent (Optional)</label>
        <input
          type="text"
          value={formData.user_agent || ''}
          onChange={(e) => setFormData({ ...formData, user_agent: e.target.value })}
          placeholder="Custom user agent string"
        />
      </div>

      <div className="form-group">
        <label>Proxy URL (Optional)</label>
        <input
          type="text"
          value={formData.proxy_url || ''}
          onChange={(e) => setFormData({ ...formData, proxy_url: e.target.value })}
          placeholder="http://proxy:port"
        />
      </div>

      <div className="form-group">
        <label className="checkbox-label">
          <input
            type="checkbox"
            checked={formData.is_active}
            onChange={(e) => setFormData({ ...formData, is_active: e.target.checked })}
          />
          Active
        </label>
      </div>

      <div className="form-actions">
        <button type="button" onClick={onCancel} className="btn btn-secondary">
          Cancel
        </button>
        <button type="button" onClick={handleTest} className="btn btn-info" disabled={isLoading}>
          Test
        </button>
        <button type="submit" className="btn btn-primary" disabled={isLoading}>
          {job ? 'Update' : 'Create'}
        </button>
      </div>
    </form>
  );
}

// Job Details Component
interface JobDetailsProps {
  job: ScrapingJob;
  onEdit: () => void;
  onDelete: () => void;
  onRun: () => void;
  isLoading: boolean;
}

function JobDetails({ job, onEdit, onDelete, onRun, isLoading }: JobDetailsProps) {
  return (
    <div className="job-details">
      <div className="job-details-header">
        <h3>{job.name}</h3>
        <div className="job-actions">
          <button onClick={onEdit} className="btn btn-secondary">Edit</button>
          <button onClick={onRun} className="btn btn-primary" disabled={isLoading}>Run Now</button>
          <button onClick={onDelete} className="btn btn-danger">Delete</button>
        </div>
      </div>

      <div className="job-details-content">
        <div className="detail-group">
          <label>URL:</label>
          <span>{job.url}</span>
        </div>
        
        <div className="detail-group">
          <label>Selector Type:</label>
          <span>{job.selector_type}</span>
        </div>
        
        <div className="detail-group">
          <label>Selector:</label>
          <span>{job.selector}</span>
        </div>
        
        <div className="detail-group">
          <label>Data Type:</label>
          <span>{job.data_type}</span>
        </div>
        
        {job.attribute_name && (
          <div className="detail-group">
            <label>Attribute:</label>
            <span>{job.attribute_name}</span>
          </div>
        )}
        
        <div className="detail-group">
          <label>Schedule:</label>
          <span>{job.schedule}</span>
        </div>
        
        <div className="detail-group">
          <label>Status:</label>
          <span className={`status ${job.is_active ? 'active' : 'inactive'}`}>
            {job.is_active ? 'Active' : 'Inactive'}
          </span>
        </div>
        
        {job.user_agent && (
          <div className="detail-group">
            <label>User Agent:</label>
            <span>{job.user_agent}</span>
          </div>
        )}
        
        {job.proxy_url && (
          <div className="detail-group">
            <label>Proxy:</label>
            <span>{job.proxy_url}</span>
          </div>
        )}
        
        {job.created_at && (
          <div className="detail-group">
            <label>Created:</label>
            <span>{new Date(job.created_at).toLocaleString()}</span>
          </div>
        )}
      </div>
    </div>
  );
}

// Results Tab Component
interface ResultsTabProps {
  jobs: ScrapingJob[];
  selectedJob: ScrapingJob | null;
  setSelectedJob: (job: ScrapingJob | null) => void;
  results: ScrapingResult[];
  onLoadResults: (jobId: number) => void;
  onExportResults: (jobId: number, format: 'CSV' | 'JSON' | 'HTML') => void;
  onExportIndividualResult: (resultId: number, format: 'CSV' | 'JSON' | 'HTML') => void;
  isLoading: boolean;
}

function ResultsTab({ 
  jobs, 
  selectedJob, 
  setSelectedJob, 
  results, 
  onLoadResults, 
  onExportResults, 
  onExportIndividualResult,
  isLoading 
}: ResultsTabProps) {
  useEffect(() => {
    if (selectedJob?.id) {
      onLoadResults(selectedJob.id);
    }
  }, [selectedJob]);

  return (
    <div className="results-tab">
      <div className="results-header">
        <h2>Scraping Results</h2>
        <div className="job-selector">
          <select
            value={selectedJob?.id || ''}
            onChange={(e) => {
              const job = jobs.find(j => j.id === parseInt(e.target.value));
              setSelectedJob(job || null);
            }}
          >
            <option value="">Select a job...</option>
            {jobs.map(job => (
              <option key={job.id} value={job.id}>
                {job.name}
              </option>
            ))}
          </select>
        </div>
      </div>

      {selectedJob && (
        <div className="export-actions">
          <button 
            onClick={() => selectedJob.id && onExportResults(selectedJob.id, 'CSV')}
            className="btn btn-secondary"
            disabled={isLoading}
          >
            Export CSV
          </button>
          <button 
            onClick={() => selectedJob.id && onExportResults(selectedJob.id, 'JSON')}
            className="btn btn-secondary"
            disabled={isLoading}
          >
            Export JSON
          </button>
          <button 
            onClick={() => selectedJob.id && onExportResults(selectedJob.id, 'HTML')}
            className="btn btn-secondary"
            disabled={isLoading}
          >
            Export HTML
          </button>
        </div>
      )}

      <div className="results-content">
        {results.length > 0 ? (
          <div className="results-table">
            <table>
              <thead>
                <tr>
                  <th>ID</th>
                  <th>Data</th>
                  <th>Timestamp</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                {results.map(result => (
                  <tr key={result.id}>
                    <td>{result.id}</td>
                    <td className="result-data">{result.scraped_data}</td>
                    <td>{new Date(result.timestamp).toLocaleString()}</td>
                    <td className="result-actions">
                      <div className="individual-export-buttons">
                        <button 
                          onClick={() => onExportIndividualResult(result.id, 'CSV')}
                          className="btn btn-mini"
                          disabled={isLoading}
                          title="Export as CSV"
                        >
                          CSV
                        </button>
                        <button 
                          onClick={() => onExportIndividualResult(result.id, 'JSON')}
                          className="btn btn-mini"
                          disabled={isLoading}
                          title="Export as JSON"
                        >
                          JSON
                        </button>
                        <button 
                          onClick={() => onExportIndividualResult(result.id, 'HTML')}
                          className="btn btn-mini"
                          disabled={isLoading}
                          title="Export as HTML"
                        >
                          HTML
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : selectedJob ? (
          <div className="empty-state">
            <p>No results found for this job</p>
          </div>
        ) : (
          <div className="empty-state">
            <p>Select a job to view its results</p>
          </div>
        )}
      </div>
    </div>
  );
}

// Settings Tab Component
interface SettingsTabProps {
  emailConfig: EmailConfig | null;
  onSaveEmailConfig: (config: EmailConfig) => void;
  onTestEmailConnection: () => void;
  theme: 'dark' | 'light';
  onThemeToggle: (theme: 'dark' | 'light') => void;
  isLoading: boolean;
}

function SettingsTab({ emailConfig, onSaveEmailConfig, onTestEmailConnection, theme, onThemeToggle, isLoading }: SettingsTabProps) {
  const [formData, setFormData] = useState<EmailConfig>(
    emailConfig || {
      smtp_server: '',
      smtp_port: 587,
      username: '',
      password: '',
      from_email: '',
      to_email: '',
      use_tls: true,
    }
  );
  const [activeSection, setActiveSection] = useState<string>('appearance');

  useEffect(() => {
    if (emailConfig) {
      setFormData(emailConfig);
    }
  }, [emailConfig]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSaveEmailConfig(formData);
  };

  const openLinkedIn = () => {
    window.open('https://www.linkedin.com/in/md-jobayer-arafat-a14b61284/', '_blank');
  };

  const openEmail = () => {
    window.open('mailto:mdjobayerarafat@gmail.com', '_blank');
  };

  return (
    <div className="settings-container">
      <div className="settings-header">
        <h1 className="settings-title">Settings</h1>
        <p className="settings-subtitle">Manage your application preferences and configurations</p>
      </div>

      <div className="settings-navigation">
        <button 
          className={`nav-item ${activeSection === 'appearance' ? 'active' : ''}`}
          onClick={() => setActiveSection('appearance')}
        >
          <span className="nav-icon">🎨</span>
          <span>Appearance</span>
        </button>
        <button 
          className={`nav-item ${activeSection === 'email' ? 'active' : ''}`}
          onClick={() => setActiveSection('email')}
        >
          <span className="nav-icon">📧</span>
          <span>Email Configuration</span>
        </button>
        <button 
          className={`nav-item ${activeSection === 'about' ? 'active' : ''}`}
          onClick={() => setActiveSection('about')}
        >
          <span className="nav-icon">👨‍💻</span>
          <span>About Creator</span>
        </button>
      </div>

      <div className="settings-content">
        {activeSection === 'appearance' && (
          <div className="settings-panel fade-in">
            <div className="panel-header">
              <h2>Appearance Settings</h2>
              <p>Customize the look and feel of your application</p>
            </div>
            
            <div className="setting-group">
              <label className="setting-label">Theme Preference</label>
              <div className="theme-selector-modern">
                <div 
                  className={`theme-card ${theme === 'dark' ? 'selected' : ''}`}
                  onClick={() => onThemeToggle('dark')}
                >
                  <div className="theme-preview-modern dark-theme">
                    <div className="preview-header"></div>
                    <div className="preview-content">
                      <div className="preview-line"></div>
                      <div className="preview-line short"></div>
                    </div>
                  </div>
                  <div className="theme-info">
                    <h3>Dark Mode</h3>
                    <p>Easy on the eyes</p>
                  </div>
                  <div className="theme-radio">
                    <input type="radio" checked={theme === 'dark'} readOnly />
                  </div>
                </div>
                
                <div 
                  className={`theme-card ${theme === 'light' ? 'selected' : ''}`}
                  onClick={() => onThemeToggle('light')}
                >
                  <div className="theme-preview-modern light-theme">
                    <div className="preview-header"></div>
                    <div className="preview-content">
                      <div className="preview-line"></div>
                      <div className="preview-line short"></div>
                    </div>
                  </div>
                  <div className="theme-info">
                    <h3>Light Mode</h3>
                    <p>Clean and bright</p>
                  </div>
                  <div className="theme-radio">
                    <input type="radio" checked={theme === 'light'} readOnly />
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        {activeSection === 'email' && (
          <div className="settings-panel fade-in">
            <div className="panel-header">
              <h2>Email Configuration</h2>
              <p>Configure SMTP settings for email notifications</p>
            </div>
            
            <form className="modern-form" onSubmit={handleSubmit}>
              <div className="form-grid">
                <div className="input-group">
                  <label className="input-label">SMTP Server</label>
                  <input
                    type="text"
                    className="modern-input"
                    value={formData.smtp_server}
                    onChange={(e) => setFormData({ ...formData, smtp_server: e.target.value })}
                    placeholder="smtp.gmail.com"
                    required
                  />
                </div>
                
                <div className="input-group">
                  <label className="input-label">SMTP Port</label>
                  <input
                    type="number"
                    className="modern-input"
                    value={formData.smtp_port}
                    onChange={(e) => setFormData({ ...formData, smtp_port: parseInt(e.target.value) })}
                    required
                  />
                </div>
              </div>

              <div className="input-group">
                <label className="input-label">Username</label>
                <input
                  type="text"
                  className="modern-input"
                  value={formData.username}
                  onChange={(e) => setFormData({ ...formData, username: e.target.value })}
                  required
                />
              </div>

              <div className="input-group">
                <label className="input-label">Password</label>
                <input
                  type="password"
                  className="modern-input"
                  value={formData.password}
                  onChange={(e) => setFormData({ ...formData, password: e.target.value })}
                  required
                />
              </div>

              <div className="form-grid">
                <div className="input-group">
                  <label className="input-label">From Email</label>
                  <input
                    type="email"
                    className="modern-input"
                    value={formData.from_email}
                    onChange={(e) => setFormData({ ...formData, from_email: e.target.value })}
                    required
                  />
                </div>

                <div className="input-group">
                  <label className="input-label">To Email</label>
                  <input
                    type="email"
                    className="modern-input"
                    value={formData.to_email}
                    onChange={(e) => setFormData({ ...formData, to_email: e.target.value })}
                    required
                  />
                </div>
              </div>

              <div className="checkbox-group">
                <label className="modern-checkbox">
                  <input
                    type="checkbox"
                    checked={formData.use_tls}
                    onChange={(e) => setFormData({ ...formData, use_tls: e.target.checked })}
                  />
                  <span className="checkmark"></span>
                  <span className="checkbox-text">Use TLS encryption</span>
                </label>
              </div>

              <div className="form-actions-modern">
                <button type="button" onClick={onTestEmailConnection} className="btn-secondary" disabled={isLoading}>
                  <span className="btn-icon">🔍</span>
                  Test Connection
                </button>
                <button type="submit" className="btn-primary" disabled={isLoading}>
                  <span className="btn-icon">💾</span>
                  Save Configuration
                </button>
              </div>
            </form>
          </div>
        )}

        {activeSection === 'about' && (
          <div className="settings-panel fade-in">
            <div className="panel-header">
              <h2>About Creator</h2>
              <p>Meet the developer behind this application</p>
            </div>
            
            <div className="creator-card">
              <div className="creator-avatar">
                <img src={jobayerAvatar} alt="Jobayer" className="avatar-image" />
              </div>
              
              <div className="creator-info">
                <h3 className="creator-name">Md Jobayer Arafat</h3>
                <p className="creator-title">Full Stack Developer</p>
                <p className="creator-description">
                  Passionate developer focused on creating efficient and user-friendly applications. 
                  Specialized in web scraping, automation, and modern web technologies.
                </p>
                
                <div className="creator-links">
                  <button className="social-btn linkedin" onClick={openLinkedIn}>
                    <span className="social-icon">💼</span>
                    <span>LinkedIn Profile</span>
                  </button>
                  
                  <button className="social-btn email" onClick={openEmail}>
                    <span className="social-icon">📧</span>
                    <span>Send Email</span>
                  </button>
                </div>
                
                <div className="creator-contact">
                  <div className="contact-item">
                    <span className="contact-label">Email:</span>
                    <span className="contact-value">mdjobayerarafat@gmail.com</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

// Files Tab Component
interface FilesTabProps {
  exportFiles: ExportFileInfo[];
  selectedFile: ExportFileInfo | null;
  fileContent: string;
  onFileSelect: (file: ExportFileInfo) => void;
  onOpenDirectory: () => void;
  onDeleteFile: (file: ExportFileInfo) => void;
  onRefresh: () => void;
  isLoading: boolean;
}

function FilesTab({ 
  exportFiles, 
  selectedFile, 
  fileContent, 
  onFileSelect, 
  onOpenDirectory, 
  onDeleteFile, 
  onRefresh, 
  isLoading 
}: FilesTabProps) {
  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatDate = (timestamp: number): string => {
    return new Date(timestamp * 1000).toLocaleString();
  };

  const getFileIcon = (fileType: string): string => {
    switch (fileType.toLowerCase()) {
      case 'csv': return '📊';
      case 'json': return '📄';
      case 'html': return '🌐';
      default: return '📁';
    }
  };

  const renderFileContent = () => {
    if (!selectedFile || !fileContent) return null;

    const isJson = selectedFile.file_type.toLowerCase() === 'json';
    const isHtml = selectedFile.file_type.toLowerCase() === 'html';
    const isCsv = selectedFile.file_type.toLowerCase() === 'csv';

    if (isHtml) {
      return (
        <div className="file-content-html">
          <div dangerouslySetInnerHTML={{ __html: fileContent }} />
        </div>
      );
    }

    if (isJson) {
      try {
        const parsed = JSON.parse(fileContent);
        return (
          <pre className="file-content-json">
            {JSON.stringify(parsed, null, 2)}
          </pre>
        );
      } catch {
        return (
          <pre className="file-content-text">
            {fileContent}
          </pre>
        );
      }
    }

    if (isCsv) {
      const lines = fileContent.split('\n');
      const headers = lines[0]?.split(',') || [];
      const rows = lines.slice(1).filter(line => line.trim());

      return (
        <div className="file-content-csv">
          <table>
            <thead>
              <tr>
                {headers.map((header, index) => (
                  <th key={index}>{header.trim()}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {rows.map((row, rowIndex) => {
                const cells = row.split(',');
                return (
                  <tr key={rowIndex}>
                    {cells.map((cell, cellIndex) => (
                      <td key={cellIndex}>{cell.trim()}</td>
                    ))}
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      );
    }

    return (
      <pre className="file-content-text">
        {fileContent}
      </pre>
    );
  };

  return (
    <div className="files-tab">
      <div className="files-header">
        <h2>📁 Export Files</h2>
        <div className="files-actions">
          <button onClick={onRefresh} className="btn btn-secondary" disabled={isLoading}>
            🔄 Refresh
          </button>
          <button onClick={onOpenDirectory} className="btn btn-info" disabled={isLoading}>
            📂 Open Directory
          </button>
        </div>
      </div>

      <div className="files-content">
        <div className="files-list">
          <h3>Files ({exportFiles.length})</h3>
          {exportFiles.length === 0 ? (
            <div className="no-files">
              <p>No export files found.</p>
              <p>Export some job results to see files here.</p>
            </div>
          ) : (
            <div className="file-items">
              {exportFiles.map((file, index) => (
                <div 
                  key={index} 
                  className={`file-item ${selectedFile?.path === file.path ? 'selected' : ''}`}
                  onClick={() => onFileSelect(file)}
                >
                  <div className="file-info">
                    <div className="file-name">
                      <span className="file-icon">{getFileIcon(file.file_type)}</span>
                      <span className="file-title">{file.name}</span>
                    </div>
                    <div className="file-details">
                      <span className="file-size">{formatFileSize(file.size)}</span>
                      <span className="file-date">{formatDate(file.modified_timestamp)}</span>
                    </div>
                  </div>
                  <button 
                    className="file-delete"
                    onClick={(e) => {
                      e.stopPropagation();
                      onDeleteFile(file);
                    }}
                    title="Delete file"
                  >
                    🗑️
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="file-viewer">
          {selectedFile ? (
            <>
              <div className="file-viewer-header">
                <h3>📄 {selectedFile.name}</h3>
                <div className="file-meta">
                  <span>Type: {selectedFile.file_type.toUpperCase()}</span>
                  <span>Size: {formatFileSize(selectedFile.size)}</span>
                  <span>Modified: {formatDate(selectedFile.modified_timestamp)}</span>
                </div>
              </div>
              <div className="file-viewer-content">
                {isLoading ? (
                  <div className="loading-content">Loading file content...</div>
                ) : (
                  renderFileContent()
                )}
              </div>
            </>
          ) : (
            <div className="no-file-selected">
              <p>Select a file to view its content</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

// Documentation Tab Component
function DocumentationTab() {
  const [activeSection, setActiveSection] = React.useState('getting-started');

  const sections = [
    { id: 'getting-started', title: 'Getting Started', icon: '🚀' },
    { id: 'quick-start', title: 'Quick Start', icon: '⚡' },
    { id: 'selectors', title: 'Selectors', icon: '🎯' },
    { id: 'data-types', title: 'Data Types', icon: '📊' },
    { id: 'scheduling', title: 'Scheduling', icon: '⏰' },
    { id: 'exporting', title: 'Exporting', icon: '📤' },
    { id: 'email', title: 'Email Setup', icon: '📧' },
    { id: 'best-practices', title: 'Best Practices', icon: '🛡' },
    { id: 'troubleshooting', title: 'Troubleshooting', icon: '🔧' },
    { id: 'ui-features', title: 'UI Features', icon: '🎨' }
  ];

  const renderContent = () => {
    switch (activeSection) {
      case 'getting-started':
        return (
          <div className="doc-content-section">
            <h1>Getting Started</h1>
            <p className="doc-subtitle">Web Scraper is compatible with <strong>almost every frontend stack</strong>. Select yours and get started!</p>
            
            <div className="feature-grid">
              <div className="feature-card">
                <div className="feature-icon">🔍</div>
                <h3>Multiple Selector Types</h3>
                <p>CSS selectors, XPath, and text-based extraction for maximum flexibility</p>
              </div>
              <div className="feature-card">
                <div className="feature-icon">📊</div>
                <h3>Rich Data Types</h3>
                <p>Extract text, links, images, tables, and custom attributes</p>
              </div>
              <div className="feature-card">
                <div className="feature-icon">⏰</div>
                <h3>Smart Scheduling</h3>
                <p>Run jobs at specific intervals, times, or trigger manually</p>
              </div>
              <div className="feature-card">
                <div className="feature-icon">📤</div>
                <h3>Export Options</h3>
                <p>CSV, JSON, and beautifully formatted HTML reports</p>
              </div>
            </div>

            <div className="getting-started-steps">
              <h2>What is Web Scraper?</h2>
              <p>Web Scraper is a powerful desktop application that allows you to extract data from websites automatically. Built with modern technologies, it provides an intuitive interface for creating, managing, and scheduling web scraping jobs.</p>
              
              <div className="code-example">
                <div className="code-header">
                  <span className="code-title">Create your first job</span>
                </div>
                <div className="code-content">
                  <p>The easiest way to scaffold a new project is the <code>create-tauri-app</code> utility. It provides opinionated templates for vanilla HTML/CSS/JavaScript and many frontend frameworks like React, Svelte, and Yew.</p>
                </div>
              </div>
            </div>
          </div>
        );
 
       case 'quick-start':
         return (
           <div className="doc-content-section">
             <h1>Quick Start</h1>
             <p className="doc-subtitle">Get up and running with your first web scraping job in minutes.</p>
             
             <div className="quick-start-grid">
               <div className="step-card">
                 <div className="step-number">1</div>
                 <h3>Create Job</h3>
                 <p>Navigate to the Jobs tab and click "Add Job" to start creating your first scraping job.</p>
               </div>
               <div className="step-card">
                 <div className="step-number">2</div>
                 <h3>Configure Target</h3>
                 <p>Enter the website URL and add a descriptive name for your job.</p>
               </div>
               <div className="step-card">
                 <div className="step-number">3</div>
                 <h3>Add Selectors</h3>
                 <p>Define CSS selectors, XPath, or text patterns to extract the data you need.</p>
               </div>
               <div className="step-card">
                 <div className="step-number">4</div>
                 <h3>Test & Run</h3>
                 <p>Test your configuration and run the job to start collecting data.</p>
               </div>
             </div>

             <div className="code-example">
               <div className="code-header">
                 <span className="code-title">Example: Basic Product Scraper</span>
               </div>
               <div className="code-content">
                 <pre><code>URL: https://example-store.com/products
Selector: .product-title
Data Type: Text
Schedule: Daily at 9:00 AM</code></pre>
               </div>
             </div>
           </div>
         );

       case 'selectors':
         return (
           <div className="doc-content-section">
             <h1>Selectors</h1>
             <p className="doc-subtitle">Learn how to target and extract data from web elements using different selector types.</p>
             
             <div className="selector-types">
               <div className="selector-type-card">
                 <h3>CSS Selectors</h3>
                 <p>Use CSS selectors to target specific elements:</p>
                 <div className="code-example">
                   <div className="code-content">
                     <pre><code>.class-name     # Select by class
#element-id     # Select by ID
h1, h2, h3      # Select headings
a[href]         # Select links with href
.product .price # Select nested elements</code></pre>
                   </div>
                 </div>
               </div>

               <div className="selector-type-card">
                 <h3>XPath Selectors</h3>
                 <p>More powerful selection using XPath:</p>
                 <div className="code-example">
                   <div className="code-content">
                     <pre><code>//div[@class='content']
//a[contains(@href, 'product')]
//span[text()='Price:']/../span[2]</code></pre>
                   </div>
                 </div>
               </div>

               <div className="selector-type-card">
                 <h3>Text Patterns</h3>
                 <p>Extract text using patterns:</p>
                 <div className="code-example">
                   <div className="code-content">
                     <pre><code>Price: ${'{number}'}
Email: {'{email}'}
Phone: {'{phone}'}</code></pre>
                   </div>
                 </div>
               </div>
             </div>
           </div>
         );

       case 'data-types':
         return (
           <div className="doc-content-section">
             <h1>Data Types</h1>
             <p className="doc-subtitle">Choose the right data type for your extraction needs.</p>
             
             <div className="data-types-grid">
               <div className="data-type-card">
                 <div className="data-type-icon">📝</div>
                 <h3>Text</h3>
                 <p>Extract plain text content from elements</p>
               </div>
               <div className="data-type-card">
                 <div className="data-type-icon">🔗</div>
                 <h3>Link</h3>
                 <p>Extract URLs from anchor tags and href attributes</p>
               </div>
               <div className="data-type-card">
                 <div className="data-type-icon">🖼️</div>
                 <h3>Image</h3>
                 <p>Extract image URLs from src attributes</p>
               </div>
               <div className="data-type-card">
                 <div className="data-type-icon">📊</div>
                 <h3>Table</h3>
                 <p>Extract structured data from HTML tables</p>
               </div>
               <div className="data-type-card">
                 <div className="data-type-icon">🏷️</div>
                 <h3>Attribute</h3>
                 <p>Extract specific HTML attributes (data-*, title, etc.)</p>
               </div>
             </div>
           </div>
         );

       case 'scheduling':
         return (
           <div className="doc-content-section">
             <h1>Scheduling</h1>
             <p className="doc-subtitle">Automate your scraping jobs with flexible scheduling options.</p>
             
             <div className="schedule-types">
               <div className="schedule-card">
                 <h3>⏰ Time-based Scheduling</h3>
                 <p>Run jobs at specific times and intervals:</p>
                 <div className="code-example">
                   <div className="code-content">
                     <pre><code>Every 5 minutes: */5 * * * *
Daily at 9 AM: 0 9 * * *
Weekly on Monday: 0 9 * * 1
Monthly on 1st: 0 9 1 * *</code></pre>
                   </div>
                 </div>
               </div>

               <div className="schedule-card">
                 <h3>🔄 Interval Options</h3>
                 <ul>
                   <li><strong>Manual:</strong> Run jobs on-demand only</li>
                   <li><strong>Minutes:</strong> Every 1-59 minutes</li>
                   <li><strong>Hourly:</strong> Every 1-23 hours</li>
                   <li><strong>Daily:</strong> Once per day at specified time</li>
                   <li><strong>Weekly:</strong> Specific days of the week</li>
                   <li><strong>Monthly:</strong> Specific day of the month</li>
                 </ul>
               </div>

               <div className="schedule-card">
                 <h3>⚙️ Advanced Settings</h3>
                 <ul>
                   <li><strong>Retry Logic:</strong> Automatic retries on failure</li>
                   <li><strong>Timeout:</strong> Maximum execution time</li>
                   <li><strong>Concurrent Jobs:</strong> Limit simultaneous executions</li>
                   <li><strong>Error Handling:</strong> Continue or stop on errors</li>
                 </ul>
               </div>
             </div>
           </div>
         );

       case 'exporting':
         return (
           <div className="doc-content-section">
             <h1>Exporting Data</h1>
             <p className="doc-subtitle">Export your scraped data in multiple formats for analysis and integration.</p>
             
             <div className="export-formats">
               <div className="export-card">
                 <div className="export-icon">📊</div>
                 <h3>CSV Format</h3>
                 <p>Perfect for spreadsheet applications and data analysis</p>
                 <div className="code-example">
                   <div className="code-content">
                     <pre><code>timestamp,job_name,scraped_data
2024-01-15 09:00:00,Product Prices,$29.99
2024-01-15 09:05:00,Product Prices,$31.50</code></pre>
                   </div>
                 </div>
               </div>

               <div className="export-card">
                 <div className="export-icon">🔧</div>
                 <h3>JSON Format</h3>
                 <p>Structured data for APIs and programming integration</p>
                 <div className="code-example">
                    <div className="code-content">
                      <pre><code>{`{
  "results": [
    {
      "timestamp": "2024-01-15T09:00:00Z",
      "job_name": "Product Prices",
      "data": "$29.99"
    }
  ]
}`}</code></pre>
                    </div>
                  </div>
               </div>

               <div className="export-card">
                 <div className="export-icon">🌐</div>
                 <h3>HTML Reports</h3>
                 <p>Beautiful, formatted reports for presentations</p>
                 <ul>
                   <li>Professional styling and layout</li>
                   <li>Charts and visualizations</li>
                   <li>Print-ready format</li>
                   <li>Embedded metadata</li>
                 </ul>
               </div>
             </div>

             <div className="export-options">
               <h2>Export Options</h2>
               <ul>
                 <li><strong>Date Range:</strong> Filter results by time period</li>
                 <li><strong>Batch Export:</strong> Export multiple jobs at once</li>
                 <li><strong>Auto Export:</strong> Automatically export after job completion</li>
                 <li><strong>File Naming:</strong> Custom naming patterns with timestamps</li>
               </ul>
             </div>
           </div>
         );

       case 'email':
         return (
           <div className="doc-content-section">
             <h1>Email Setup</h1>
             <p className="doc-subtitle">Configure email notifications for job results and alerts.</p>
             
             <div className="email-setup">
               <div className="setup-card">
                 <h3>📧 SMTP Configuration</h3>
                 <div className="config-grid">
                   <div className="config-item">
                     <strong>SMTP Server:</strong> smtp.gmail.com
                   </div>
                   <div className="config-item">
                     <strong>Port:</strong> 587 (TLS) or 465 (SSL)
                   </div>
                   <div className="config-item">
                     <strong>Authentication:</strong> Username/Password
                   </div>
                   <div className="config-item">
                     <strong>Security:</strong> TLS/SSL encryption
                   </div>
                 </div>
               </div>

               <div className="setup-card">
                 <h3>🔧 Popular Providers</h3>
                 <div className="provider-list">
                   <div className="provider-item">
                     <strong>Gmail:</strong> smtp.gmail.com:587 (App Password required)
                   </div>
                   <div className="provider-item">
                     <strong>Outlook:</strong> smtp-mail.outlook.com:587
                   </div>
                   <div className="provider-item">
                     <strong>Yahoo:</strong> smtp.mail.yahoo.com:587
                   </div>
                   <div className="provider-item">
                     <strong>Custom:</strong> Your organization's SMTP server
                   </div>
                 </div>
               </div>

               <div className="setup-card">
                 <h3>📬 Notification Types</h3>
                 <ul>
                   <li><strong>Job Completion:</strong> Results summary and data preview</li>
                   <li><strong>Error Alerts:</strong> Immediate notification of failures</li>
                   <li><strong>Daily Reports:</strong> Scheduled summary of all activities</li>
                   <li><strong>Data Changes:</strong> Alerts when scraped data changes significantly</li>
                 </ul>
               </div>
             </div>

             <div className="email-tips">
               <h2>💡 Setup Tips</h2>
               <ul>
                 <li>Use app-specific passwords for Gmail and other providers</li>
                 <li>Test your configuration before enabling notifications</li>
                 <li>Consider using a dedicated email account for scraping notifications</li>
                 <li>Check spam folders if emails aren't arriving</li>
               </ul>
             </div>
           </div>
          );

       case 'best-practices':
         return (
           <div className="doc-content-section">
             <h1>Best Practices</h1>
             <p className="doc-subtitle">Follow these guidelines for efficient and reliable web scraping.</p>
             
             <div className="best-practices">
               <div className="practice-card">
                 <h3>🛡️ Respectful Scraping</h3>
                 <ul>
                   <li><strong>Rate Limiting:</strong> Don't overwhelm servers with requests</li>
                   <li><strong>robots.txt:</strong> Check and respect website policies</li>
                   <li><strong>User Agents:</strong> Use realistic browser user agents</li>
                   <li><strong>Session Management:</strong> Handle cookies and sessions properly</li>
                 </ul>
               </div>

               <div className="practice-card">
                 <h3>⚡ Performance Optimization</h3>
                 <ul>
                   <li><strong>Efficient Selectors:</strong> Use specific, fast CSS selectors</li>
                   <li><strong>Minimal Data:</strong> Only scrape what you need</li>
                   <li><strong>Caching:</strong> Cache results to avoid duplicate requests</li>
                   <li><strong>Parallel Processing:</strong> Use concurrent jobs wisely</li>
                 </ul>
               </div>

               <div className="practice-card">
                 <h3>🔒 Security & Privacy</h3>
                 <ul>
                   <li><strong>Proxy Usage:</strong> Rotate IP addresses when necessary</li>
                   <li><strong>Data Protection:</strong> Secure sensitive scraped data</li>
                   <li><strong>Legal Compliance:</strong> Respect copyright and terms of service</li>
                   <li><strong>Personal Data:</strong> Handle PII according to regulations</li>
                 </ul>
               </div>

               <div className="practice-card">
                 <h3>📊 Data Quality</h3>
                 <ul>
                   <li><strong>Validation:</strong> Verify scraped data accuracy</li>
                   <li><strong>Cleaning:</strong> Remove unwanted characters and formatting</li>
                   <li><strong>Consistency:</strong> Maintain uniform data formats</li>
                   <li><strong>Monitoring:</strong> Track data quality over time</li>
                 </ul>
               </div>
             </div>

             <div className="practice-tips">
               <h2>💡 Pro Tips</h2>
               <ul>
                 <li>Start with small test runs before scaling up</li>
                 <li>Monitor website changes that might break your selectors</li>
                 <li>Keep backups of your job configurations</li>
                 <li>Document your scraping logic for future reference</li>
                 <li>Use version control for complex scraping projects</li>
               </ul>
             </div>
           </div>
         );

       case 'troubleshooting':
         return (
           <div className="doc-content-section">
             <h1>Troubleshooting</h1>
             <p className="doc-subtitle">Common issues and solutions for web scraping problems.</p>
             
             <div className="troubleshooting-sections">
               <div className="trouble-card">
                 <h3>🚫 Common Errors</h3>
                 <div className="error-item">
                   <strong>Selector Not Found:</strong>
                   <p>The CSS selector doesn't match any elements on the page.</p>
                   <ul>
                     <li>Verify the selector in browser dev tools</li>
                     <li>Check if content loads dynamically</li>
                     <li>Try more specific or alternative selectors</li>
                   </ul>
                 </div>
                 <div className="error-item">
                   <strong>Connection Timeout:</strong>
                   <p>The website is not responding or taking too long.</p>
                   <ul>
                     <li>Check your internet connection</li>
                     <li>Verify the website is accessible</li>
                     <li>Increase timeout settings</li>
                     <li>Try using a proxy server</li>
                   </ul>
                 </div>
               </div>

               <div className="trouble-card">
                 <h3>🔧 Performance Issues</h3>
                 <div className="error-item">
                   <strong>Slow Scraping:</strong>
                   <ul>
                     <li>Reduce request frequency</li>
                     <li>Optimize selectors for speed</li>
                     <li>Use more efficient data types</li>
                     <li>Check system resources</li>
                   </ul>
                 </div>
                 <div className="error-item">
                   <strong>Memory Usage:</strong>
                   <ul>
                     <li>Limit concurrent jobs</li>
                     <li>Clear old results regularly</li>
                     <li>Reduce data retention period</li>
                     <li>Export and archive large datasets</li>
                   </ul>
                 </div>
               </div>

               <div className="trouble-card">
                 <h3>🌐 Website-Specific Issues</h3>
                 <div className="error-item">
                   <strong>JavaScript-Heavy Sites:</strong>
                   <ul>
                     <li>Wait for content to load</li>
                     <li>Use more specific selectors</li>
                     <li>Check for AJAX-loaded content</li>
                     <li>Consider alternative scraping methods</li>
                   </ul>
                 </div>
                 <div className="error-item">
                   <strong>Anti-Bot Protection:</strong>
                   <ul>
                     <li>Use realistic user agents</li>
                     <li>Implement delays between requests</li>
                     <li>Rotate IP addresses with proxies</li>
                     <li>Respect rate limits</li>
                   </ul>
                 </div>
               </div>
             </div>

             <div className="debug-tips">
               <h2>🐛 Debugging Tips</h2>
               <ul>
                 <li>Use the "Test Job" feature to verify configurations</li>
                 <li>Check browser developer tools for selector validation</li>
                 <li>Monitor network requests for blocked content</li>
                 <li>Review application logs for detailed error messages</li>
                 <li>Test with different websites to isolate issues</li>
               </ul>
             </div>
           </div>
         );

       case 'ui-features':
         return (
           <div className="doc-content-section">
             <h1>UI Features</h1>
             <p className="doc-subtitle">Explore the powerful features of the Web Scraper interface.</p>
             
             <div className="ui-features">
               <div className="feature-section">
                 <h3>📋 Jobs Management</h3>
                 <div className="feature-grid">
                   <div className="feature-item">
                     <strong>Job Creation:</strong> Intuitive form with validation and testing
                   </div>
                   <div className="feature-item">
                     <strong>Bulk Operations:</strong> Select and manage multiple jobs at once
                   </div>
                   <div className="feature-item">
                     <strong>Job Templates:</strong> Save and reuse common configurations
                   </div>
                   <div className="feature-item">
                     <strong>Status Monitoring:</strong> Real-time job status and progress
                   </div>
                 </div>
               </div>

               <div className="feature-section">
                 <h3>📊 Results Visualization</h3>
                 <div className="feature-grid">
                   <div className="feature-item">
                     <strong>Data Preview:</strong> Quick view of scraped results
                   </div>
                   <div className="feature-item">
                     <strong>Filtering:</strong> Search and filter results by date, job, or content
                   </div>
                   <div className="feature-item">
                     <strong>Sorting:</strong> Order results by timestamp, job name, or data
                   </div>
                   <div className="feature-item">
                     <strong>Pagination:</strong> Efficient browsing of large result sets
                   </div>
                 </div>
               </div>

               <div className="feature-section">
                 <h3>📁 File Management</h3>
                 <div className="feature-grid">
                   <div className="feature-item">
                     <strong>Export Browser:</strong> View and manage exported files
                   </div>
                   <div className="feature-item">
                     <strong>File Preview:</strong> Quick preview of file contents
                   </div>
                   <div className="feature-item">
                     <strong>Bulk Actions:</strong> Delete or organize multiple files
                   </div>
                   <div className="feature-item">
                     <strong>Directory Access:</strong> Open export folder in file explorer
                   </div>
                 </div>
               </div>

               <div className="feature-section">
                 <h3>⚙️ Settings & Customization</h3>
                 <div className="feature-grid">
                   <div className="feature-item">
                     <strong>Theme Toggle:</strong> Switch between dark and light modes
                   </div>
                   <div className="feature-item">
                     <strong>Email Configuration:</strong> Set up notifications and alerts
                   </div>
                   <div className="feature-item">
                     <strong>Performance Tuning:</strong> Adjust timeouts and concurrency
                   </div>
                   <div className="feature-item">
                     <strong>Data Retention:</strong> Configure automatic cleanup policies
                   </div>
                 </div>
               </div>
             </div>

             <div className="keyboard-shortcuts">
               <h2>⌨️ Keyboard Shortcuts</h2>
               <div className="shortcuts-grid">
                 <div className="shortcut-item">
                   <kbd>Ctrl + N</kbd> <span>Create new job</span>
                 </div>
                 <div className="shortcut-item">
                   <kbd>Ctrl + R</kbd> <span>Refresh current view</span>
                 </div>
                 <div className="shortcut-item">
                   <kbd>Ctrl + E</kbd> <span>Export selected results</span>
                 </div>
                 <div className="shortcut-item">
                   <kbd>Ctrl + T</kbd> <span>Test current job</span>
                 </div>
                 <div className="shortcut-item">
                   <kbd>Ctrl + S</kbd> <span>Save current configuration</span>
                 </div>
                 <div className="shortcut-item">
                   <kbd>F5</kbd> <span>Run selected job</span>
                 </div>
               </div>
             </div>
           </div>
         );

       default:
         return (
           <div className="doc-content-section">
             <h1>Documentation</h1>
             <p>Select a topic from the sidebar to get started.</p>
           </div>
         );
     }
   };

   return (
     <div className="documentation-tab">
       <div className="doc-layout">
         <div className="doc-sidebar">
           <div className="doc-sidebar-header">
             <h2>📚 Documentation</h2>
           </div>
           <nav className="doc-nav">
             {sections.map(section => (
               <button
                 key={section.id}
                 className={`doc-nav-item ${activeSection === section.id ? 'active' : ''}`}
                 onClick={() => setActiveSection(section.id)}
               >
                 <span className="doc-nav-icon">{section.icon}</span>
                 <span className="doc-nav-title">{section.title}</span>
               </button>
             ))}
           </nav>
         </div>
         <div className="doc-main">
            <div className="doc-content">
              {renderContent()}
            </div>
          </div>
        </div>
      </div>
    );
 }




export default App;
