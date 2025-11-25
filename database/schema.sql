-- recall-cli Database Schema
-- Version: 1.0.0
-- Description: Stores command history, patterns, and suggestions

-- Main commands table
CREATE TABLE IF NOT EXISTS commands (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_path TEXT NOT NULL,
    command TEXT NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    is_fav INTEGER DEFAULT 0,
    usage_count INTEGER DEFAULT 1,
    execution_time_ms INTEGER,
    exit_code INTEGER,
    tags TEXT, -- JSON array of tags
    context TEXT, -- What was happening before this command
    UNIQUE(project_path, command) ON CONFLICT REPLACE
);

-- Command patterns (for auto-detection)
CREATE TABLE IF NOT EXISTS command_patterns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern_type TEXT NOT NULL, -- 'sequence', 'frequency', 'time_based', 'context_based'
    commands TEXT NOT NULL, -- JSON array of commands
    project_path TEXT,
    confidence_score REAL DEFAULT 0.0,
    occurrences INTEGER DEFAULT 1,
    last_seen DATETIME DEFAULT CURRENT_TIMESTAMP,
    metadata TEXT -- JSON for additional data
);

-- Suggestions table (pre-computed suggestions)
CREATE TABLE IF NOT EXISTS suggestions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_path TEXT NOT NULL,
    context TEXT, -- What triggers this suggestion
    suggested_command TEXT NOT NULL,
    reason TEXT, -- Why this is suggested
    confidence REAL DEFAULT 0.0,
    times_accepted INTEGER DEFAULT 0,
    times_rejected INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_suggested DATETIME
);

-- User preferences
CREATE TABLE IF NOT EXISTS preferences (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Command aliases (user-defined shortcuts)
CREATE TABLE IF NOT EXISTS aliases (
    alias TEXT PRIMARY KEY,
    command TEXT NOT NULL,
    project_path TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Command execution context (for better suggestions)
CREATE TABLE IF NOT EXISTS execution_context (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    command_id INTEGER,
    working_directory TEXT,
    previous_command TEXT,
    time_of_day TEXT, -- 'morning', 'afternoon', 'evening', 'night'
    day_of_week TEXT,
    git_branch TEXT,
    files_changed TEXT, -- JSON array
    FOREIGN KEY(command_id) REFERENCES commands(id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_commands_project ON commands(project_path);
CREATE INDEX IF NOT EXISTS idx_commands_timestamp ON commands(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_commands_usage ON commands(usage_count DESC);
CREATE INDEX IF NOT EXISTS idx_patterns_project ON command_patterns(project_path);
CREATE INDEX IF NOT EXISTS idx_suggestions_project ON suggestions(project_path);
CREATE INDEX IF NOT EXISTS idx_context_command ON execution_context(command_id);

-- Insert default preferences
INSERT OR IGNORE INTO preferences (key, value) VALUES
    ('max_history_size', '10000'),
    ('enable_suggestions', 'true'),
    ('enable_pattern_detection', 'true'),
    ('suggestion_threshold', '0.7'),
    ('auto_cleanup_days', '90');
