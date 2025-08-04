use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub event_type: String,
    pub lambda_arn: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalCredentials {
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
}


#[derive(Debug, Clone)]
pub struct Schedule {
    pub id: String,  
    pub event_type: String,      // Unique identifier for the scheduled event (e.g., "daily.backup")
    pub lambda_arn: String,      // ARN of the Lambda function to trigger (truncated for display in handler)
    pub cron: String,            // Cron expression defining the schedule (e.g., "0 9 * * ?")
    pub next_trigger: String,    // Calculated next trigger time (e.g., "2025-08-05 09:00:00 UTC")
}