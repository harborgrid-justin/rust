-- Teams and Organizations
CREATE TABLE IF NOT EXISTS teams (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS team_members (
    id TEXT PRIMARY KEY,
    team_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('owner', 'admin', 'member', 'viewer')) DEFAULT 'member',
    added_by TEXT NOT NULL,
    added_at TEXT NOT NULL,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (added_by) REFERENCES users(id),
    UNIQUE(team_id, user_id)
);

-- Document versions and collaboration
CREATE TABLE IF NOT EXISTS document_versions (
    id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL,
    version_number INTEGER NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    file_path TEXT,
    file_name TEXT,
    file_size INTEGER,
    mime_type TEXT,
    changes_summary TEXT,
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users(id),
    UNIQUE(document_id, version_number)
);

CREATE TABLE IF NOT EXISTS document_permissions (
    id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL,
    user_id TEXT,
    team_id TEXT,
    permission TEXT NOT NULL CHECK (permission IN ('read', 'write', 'admin')) DEFAULT 'read',
    granted_by TEXT NOT NULL,
    granted_at TEXT NOT NULL,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    FOREIGN KEY (granted_by) REFERENCES users(id),
    CHECK ((user_id IS NOT NULL AND team_id IS NULL) OR (user_id IS NULL AND team_id IS NOT NULL))
);

CREATE TABLE IF NOT EXISTS document_comments (
    id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL,
    parent_id TEXT,
    content TEXT NOT NULL,
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES document_comments(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

-- Case templates and workflows
CREATE TABLE IF NOT EXISTS case_templates (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    default_priority TEXT NOT NULL CHECK (default_priority IN ('low', 'medium', 'high', 'critical')) DEFAULT 'medium',
    default_assignee TEXT,
    workflow_steps TEXT, -- JSON array of workflow steps
    custom_fields TEXT, -- JSON object for custom fields schema
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (default_assignee) REFERENCES users(id),
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS case_workflows (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL,
    step_name TEXT NOT NULL,
    step_order INTEGER NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'in_progress', 'completed', 'skipped')) DEFAULT 'pending',
    assigned_to TEXT,
    completed_by TEXT,
    completed_at TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (case_id) REFERENCES cases(id) ON DELETE CASCADE,
    FOREIGN KEY (assigned_to) REFERENCES users(id),
    FOREIGN KEY (completed_by) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS case_custom_fields (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL,
    field_name TEXT NOT NULL,
    field_type TEXT NOT NULL CHECK (field_type IN ('text', 'number', 'date', 'boolean', 'select')),
    field_value TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (case_id) REFERENCES cases(id) ON DELETE CASCADE,
    UNIQUE(case_id, field_name)
);

-- Notifications and activities
CREATE TABLE IF NOT EXISTS notifications (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    title TEXT NOT NULL,
    message TEXT NOT NULL,
    notification_type TEXT NOT NULL CHECK (notification_type IN ('info', 'success', 'warning', 'error')) DEFAULT 'info',
    entity_type TEXT,
    entity_id TEXT,
    read_at TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS activities (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    action TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    details TEXT, -- JSON object with additional details
    ip_address TEXT,
    user_agent TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Settings and configurations
CREATE TABLE IF NOT EXISTS user_settings (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    setting_key TEXT NOT NULL,
    setting_value TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, setting_key)
);

CREATE TABLE IF NOT EXISTS system_settings (
    id TEXT PRIMARY KEY,
    setting_key TEXT NOT NULL UNIQUE,
    setting_value TEXT NOT NULL,
    setting_type TEXT NOT NULL CHECK (setting_type IN ('string', 'number', 'boolean', 'json')) DEFAULT 'string',
    description TEXT,
    updated_by TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (updated_by) REFERENCES users(id)
);

-- Create additional indexes
CREATE INDEX IF NOT EXISTS idx_teams_created_by ON teams(created_by);
CREATE INDEX IF NOT EXISTS idx_team_members_team_id ON team_members(team_id);
CREATE INDEX IF NOT EXISTS idx_team_members_user_id ON team_members(user_id);

CREATE INDEX IF NOT EXISTS idx_document_versions_document_id ON document_versions(document_id);
CREATE INDEX IF NOT EXISTS idx_document_permissions_document_id ON document_permissions(document_id);
CREATE INDEX IF NOT EXISTS idx_document_comments_document_id ON document_comments(document_id);

CREATE INDEX IF NOT EXISTS idx_case_templates_created_by ON case_templates(created_by);
CREATE INDEX IF NOT EXISTS idx_case_workflows_case_id ON case_workflows(case_id);
CREATE INDEX IF NOT EXISTS idx_case_custom_fields_case_id ON case_custom_fields(case_id);

CREATE INDEX IF NOT EXISTS idx_notifications_user_id ON notifications(user_id);
CREATE INDEX IF NOT EXISTS idx_notifications_read_at ON notifications(read_at);
CREATE INDEX IF NOT EXISTS idx_activities_user_id ON activities(user_id);
CREATE INDEX IF NOT EXISTS idx_activities_entity ON activities(entity_type, entity_id);

CREATE INDEX IF NOT EXISTS idx_user_settings_user_id ON user_settings(user_id);