CREATE TABLE IF NOT EXISTS jobs(
  job_id INTEGER PRIMARY KEY, job_type INTEGER NOT NULL, state INTEGER NOT NULL,
  attempt_count INTEGER NOT NULL DEFAULT 0, next_attempt_at_ms INTEGER,
  last_error_code INTEGER, created_at_ms INTEGER NOT NULL, updated_at_ms INTEGER,
  payload_codec INTEGER NOT NULL, payload_version INTEGER NOT NULL, payload BLOB NOT NULL
);

CREATE INDEX IF NOT EXISTS jobs_ready ON jobs(state,next_attempt_at_ms,created_at_ms);
