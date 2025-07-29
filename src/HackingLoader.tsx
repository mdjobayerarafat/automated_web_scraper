import React, { useEffect, useState } from 'react';
import './HackingLoader.css';

interface HackingLoaderProps {
  isVisible: boolean;
  message?: string;
  progress?: number;
}

const HackingLoader: React.FC<HackingLoaderProps> = ({ 
  isVisible, 
  message = 'Executing job...', 
  progress = 0 
}) => {
  const [displayText, setDisplayText] = useState('');
  const [currentIndex, setCurrentIndex] = useState(0);
  const [glitchText, setGlitchText] = useState('');
  const [terminalLines, setTerminalLines] = useState<string[]>([]);
  const [internalVisible, setInternalVisible] = useState(false);
  const [startTime, setStartTime] = useState<number | null>(null);

  const hackingMessages = [
    '> Initializing web scraper...',
    '> Establishing secure connection...',
    '> Bypassing anti-bot measures...',
    '> Parsing DOM structure...',
    '> Extracting target data...',
    '> Processing results...',
    '> Finalizing extraction...'
  ];

  const glitchChars = '!@#$%^&*()_+-=[]{}|;:,.<>?';
  const matrixChars = '01';

  // Handle minimum display duration
  useEffect(() => {
    if (isVisible && !internalVisible) {
      // Starting to show loader
      setInternalVisible(true);
      setStartTime(Date.now());
      setDisplayText('');
      setCurrentIndex(0);
      setTerminalLines([]);
    } else if (!isVisible && internalVisible && startTime) {
      // Request to hide loader - check if minimum time has passed
      const elapsed = Date.now() - startTime;
      const minDisplayTime = 5000; // 5 seconds
      
      if (elapsed >= minDisplayTime) {
        // Minimum time has passed, hide immediately
        setInternalVisible(false);
        setStartTime(null);
      } else {
        // Wait for remaining time
        const remainingTime = minDisplayTime - elapsed;
        setTimeout(() => {
          setInternalVisible(false);
          setStartTime(null);
        }, remainingTime);
      }
    }
  }, [isVisible, internalVisible, startTime]);

  useEffect(() => {
    if (!internalVisible) {
      return;
    }

    // Typing effect for main message
    const typingInterval = setInterval(() => {
      if (currentIndex < message.length) {
        setDisplayText(message.slice(0, currentIndex + 1));
        setCurrentIndex(prev => prev + 1);
      } else {
        clearInterval(typingInterval);
      }
    }, 50);

    return () => clearInterval(typingInterval);
  }, [message, currentIndex, internalVisible]);

  useEffect(() => {
    if (!internalVisible) return;

    // Glitch effect
    const glitchInterval = setInterval(() => {
      const randomText = Array.from({ length: 20 }, () => 
        glitchChars[Math.floor(Math.random() * glitchChars.length)]
      ).join('');
      setGlitchText(randomText);
    }, 100);

    return () => clearInterval(glitchInterval);
  }, [internalVisible]);

  useEffect(() => {
    if (!internalVisible) return;

    // Terminal lines effect
    const terminalInterval = setInterval(() => {
      const randomMessage = hackingMessages[Math.floor(Math.random() * hackingMessages.length)];
      const timestamp = new Date().toLocaleTimeString();
      const newLine = `[${timestamp}] ${randomMessage}`;
      
      setTerminalLines(prev => {
        const updated = [...prev, newLine];
        return updated.slice(-6); // Keep only last 6 lines
      });
    }, 800);

    return () => clearInterval(terminalInterval);
  }, [internalVisible]);

  if (!internalVisible) return null;

  return (
    <div className="hacking-loader-overlay">
      <div className="hacking-loader">
        {/* Matrix background */}
        <div className="matrix-background">
          {Array.from({ length: 50 }, (_, i) => (
            <div key={i} className={`matrix-column matrix-column-${i}`}>
              {Array.from({ length: 20 }, (_, j) => (
                <span key={j} className="matrix-char">
                  {matrixChars[Math.floor(Math.random() * matrixChars.length)]}
                </span>
              ))}
            </div>
          ))}
        </div>

        {/* Main content */}
        <div className="hacking-content">
          {/* Header */}
          <div className="hacking-header">
            <div className="terminal-bar">
              <div className="terminal-buttons">
                <span className="terminal-button red"></span>
                <span className="terminal-button yellow"></span>
                <span className="terminal-button green"></span>
              </div>
              <div className="terminal-title">AUTOMATED_WEB_SCRAPER.exe</div>
            </div>
          </div>

          {/* Main display */}
          <div className="hacking-display">
            <div className="main-message">
              <span className="prompt">root@scraper:~$ </span>
              <span className="typing-text">{displayText}</span>
              <span className="cursor">█</span>
            </div>

            {/* Progress bar */}
            <div className="progress-container">
              <div className="progress-label">PROGRESS</div>
              <div className="progress-bar-hacking">
                <div 
                  className="progress-fill-hacking" 
                  style={{ width: `${progress}%` }}
                ></div>
                <div className="progress-text">{Math.round(progress)}%</div>
              </div>
            </div>

            {/* Terminal output */}
            <div className="terminal-output">
              {terminalLines.map((line, index) => (
                <div key={index} className="terminal-line">
                  <span className="line-prefix"></span>
                  <span className="line-text">{line}</span>
                </div>
              ))}
            </div>

            {/* Glitch text */}
            <div className="glitch-container">
              <div className="glitch-text">{glitchText}</div>
            </div>

            {/* Status indicators */}
            <div className="status-indicators">
              <div className="status-item">
                <span className="status-label">CONNECTION</span>
                <span className="status-value secure">SECURE</span>
              </div>
              <div className="status-item">
                <span className="status-label">STEALTH</span>
                <span className="status-value active">ACTIVE</span>
              </div>
              <div className="status-item">
                <span className="status-label">EXTRACTION</span>
                <span className="status-value running">RUNNING</span>
              </div>
            </div>
          </div>

          {/* Footer */}
          <div className="hacking-footer">
            <div className="system-info">
              <span>SYSTEM: ONLINE</span>
              <span>•</span>
              <span>ENCRYPTION: AES-256</span>
              <span>•</span>
              <span>PROXY: ENABLED</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default HackingLoader;