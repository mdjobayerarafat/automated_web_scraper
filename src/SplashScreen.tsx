import React, { useState, useEffect } from 'react';
import './SplashScreen.css';

interface SplashScreenProps {
  onComplete: () => void;
}

const SplashScreen: React.FC<SplashScreenProps> = ({ onComplete }) => {
  const [progress, setProgress] = useState(0);
  const [currentStep, setCurrentStep] = useState('Initializing...');
  const [isVisible, setIsVisible] = useState(true);

  const steps = [
    'Initializing application...',
    'Loading components...',
    'Setting up workspace...',
    'Preparing interface...',
    'Ready to scrape!'
  ];

  useEffect(() => {
    const duration = 3000; // 3 seconds total
    let currentStepIndex = 0;

    const progressInterval = setInterval(() => {
      setProgress(prev => {
        const newProgress = prev + (100 / (duration / 50)); // Update every 50ms
        
        // Update step text based on progress
        const stepIndex = Math.floor((newProgress / 100) * steps.length);
        if (stepIndex !== currentStepIndex && stepIndex < steps.length) {
          currentStepIndex = stepIndex;
          setCurrentStep(steps[stepIndex]);
        }

        if (newProgress >= 100) {
          clearInterval(progressInterval);
          // Fade out animation
          setTimeout(() => {
            setIsVisible(false);
            setTimeout(() => {
              onComplete();
            }, 500); // Wait for fade out animation
          }, 500); // Show complete state for 500ms
          return 100;
        }
        
        return newProgress;
      });
    }, 50);

    return () => clearInterval(progressInterval);
  }, [onComplete]);

  if (!isVisible) {
    return null;
  }

  return (
    <div className={`splash-screen ${!isVisible ? 'fade-out' : ''}`}>
      <div className="splash-background">
        <div className="splash-particles">
          {[...Array(20)].map((_, i) => (
            <div key={i} className={`particle particle-${i + 1}`}></div>
          ))}
        </div>
      </div>
      
      <div className="splash-content">
        <div className="splash-logo">
          <div className="logo-container">
            <div className="logo-icon">
              <div className="icon-layers">
                <div className="layer layer-1">🌐</div>
                <div className="layer layer-2">🔍</div>
                <div className="layer layer-3">⚡</div>
              </div>
            </div>
            <div className="logo-text">
              <h1 className="app-title">Automated Web Scraper</h1>
              <p className="app-subtitle">Professional Data Extraction Tool</p>
            </div>
          </div>
        </div>
        
        <div className="splash-progress">
          <div className="progress-container">
            <div className="progress-bar">
              <div 
                className="progress-fill" 
                style={{ width: `${progress}%` }}
              ></div>
              <div className="progress-glow"></div>
            </div>
            <div className="progress-text">
              <span className="progress-percentage">{Math.round(progress)}%</span>
            </div>
          </div>
          
          <div className="status-text">
            <span className="status-message">{currentStep}</span>
            <div className="loading-dots">
              <span className="dot"></span>
              <span className="dot"></span>
              <span className="dot"></span>
            </div>
          </div>
        </div>
        
        <div className="splash-footer">
          <p className="creator-info">Created by Md Jobayer Arafat</p>
          <div className="version-info">
            <span className="version">v1.0.0</span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default SplashScreen;