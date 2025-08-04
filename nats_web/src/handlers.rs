// src/handlers.rs (Updated to set behavior version in code)
use actix_web::{web, HttpResponse, Responder};
use askama::Template;
use sled::Db;
use std::{str::FromStr, sync::Arc};
use aws_credential_types::Credentials as AwsCredentials; // Alias to avoid conflict
use aws_types::region::Region;
use aws_config::BehaviorVersion; // Added import for BehaviorVersion

use crate::models::{LocalCredentials, Rule, Schedule};

use croner::Cron;
use chrono::Utc;


#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub rules: Vec<Rule>,
    pub lambda_arns: Vec<(String, String)>,
}

#[derive(Template)]
#[template(path = "credentials.html")]
pub struct CredentialsTemplate;

pub async fn index(db: web::Data<Arc<Db>>) -> impl Responder {
    let mut rules = Vec::new();
    for result in db.iter() {
        if let Ok((key, value)) = result {
            let event_type = String::from_utf8(key.to_vec()).unwrap_or_default();
            let lambda_arn = String::from_utf8(value.to_vec()).unwrap_or_default();
            if !event_type.starts_with("aws_") {
                // Truncate lambda_arn to 40 characters for display
                let truncated_lambda_arn = lambda_arn.chars().take(40).collect();
                rules.push(Rule {
                    event_type,
                    lambda_arn: truncated_lambda_arn,
                });
            }
        }
    }

    let lambda_arns = if let (Some(ak), Some(sk), Some(rg)) = (
        db.get("aws_access_key").ok().flatten(),
        db.get("aws_secret_key").ok().flatten(),
        db.get("aws_region").ok().flatten(),
    ) {
        let access_key = String::from_utf8(ak.to_vec()).unwrap_or_default();
        let secret_key = String::from_utf8(sk.to_vec()).unwrap_or_default();
        let region = String::from_utf8(rg.to_vec()).unwrap_or_default();
        if !access_key.is_empty() && !secret_key.is_empty() && !region.is_empty() {
            let credentials = AwsCredentials::new(access_key, secret_key, None, None, "static");
            let config = aws_config::ConfigLoader::default()
                .credentials_provider(credentials)
                .region(Region::new(region))
                .behavior_version(BehaviorVersion::latest())
                .load()
                .await;
            let client = aws_sdk_lambda::Client::new(&config);
            match client.list_functions().send().await {
                Ok(output) => output
                    .functions
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|f| {
                        f.function_arn.map(|arn| {
                            let truncated = arn.chars().take(50).collect::<String>();
                            (arn, truncated)
                        })
                    })
                    .collect(),
                Err(e) => {
                    eprintln!("Error fetching Lambdas: {:?}", e);
                    vec![]
                }
            }
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let template = IndexTemplate { rules, lambda_arns };
    HttpResponse::Ok()
        .content_type("text/html")
        .body(template.render().unwrap())
}
pub async fn create_rule(db: web::Data<Arc<Db>>, form: web::Form<Rule>) -> impl Responder {
    let rule = form.into_inner();
    db.insert(rule.event_type.as_bytes(), rule.lambda_arn.as_bytes()).unwrap();
    HttpResponse::SeeOther().append_header(("Location", "/")).finish()
}

pub async fn credentials_page() -> impl Responder {
    let template = CredentialsTemplate;
    HttpResponse::Ok().content_type("text/html").body(template.render().unwrap())
}

pub async fn set_credentials(db: web::Data<Arc<Db>>, form: web::Form<LocalCredentials>) -> impl Responder {
    let creds = form.into_inner();
    db.insert("aws_access_key", creds.access_key.as_bytes()).unwrap();
    db.insert("aws_secret_key", creds.secret_key.as_bytes()).unwrap();
    db.insert("aws_region", creds.region.as_bytes()).unwrap();
    HttpResponse::SeeOther().append_header(("Location", "/")).finish()
}





#[derive(Template)]
#[template(path = "scheduler.html")]
pub struct SchedulerTemplate {
    pub schedules: Vec<Schedule>,
    pub lambda_arns: Vec<(String, String)>,
}


pub async fn scheduler(db: web::Data<Arc<Db>>) -> impl Responder {
    let mut schedules = Vec::new();
    for result in db.iter() {
        if let Ok((key, value)) = result {
            let key_str = String::from_utf8(key.to_vec()).unwrap_or_default();
            if key_str.starts_with("schedule:") {
                let event_type = key_str.strip_prefix("schedule:").unwrap_or("").to_string();
                let value_str = String::from_utf8(value.to_vec()).unwrap_or_default();
                let parts: Vec<String> = value_str.split(':').map(|s| s.to_string()).collect();
                if parts.len() >= 2 {
                    let id = "1".to_string();
                    let lambda_arn = parts[0].clone();
                    let cron = parts[1].clone();
                    let truncated_lambda_arn = lambda_arn.chars().take(40).collect::<String>();
                    let next_trigger = Cron::from_str(&cron).expect("Couldn't parse cron string");
                    schedules.push(Schedule {
                        id,
                        event_type,
                        lambda_arn: truncated_lambda_arn,
                        cron,
                        next_trigger: next_trigger.to_string(),
                    });
                }
            }
        }
    }

    let lambda_arns = if let (Some(ak), Some(sk), Some(rg)) = (
        db.get("aws_access_key").ok().flatten(),
        db.get("aws_secret_key").ok().flatten(),
        db.get("aws_region").ok().flatten(),
    ) {
        let access_key = String::from_utf8(ak.to_vec()).unwrap_or_default();
        let secret_key = String::from_utf8(sk.to_vec()).unwrap_or_default();
        let region = String::from_utf8(rg.to_vec()).unwrap_or_default();
        if !access_key.is_empty() && !secret_key.is_empty() && !region.is_empty() {
            let credentials = AwsCredentials::new(access_key, secret_key, None, None, "static");
            let config = aws_config::ConfigLoader::default()
                .credentials_provider(credentials)
                .region(Region::new(region))
                .behavior_version(BehaviorVersion::latest())
                .load()
                .await;
            let client = aws_sdk_lambda::Client::new(&config);
            match client.list_functions().send().await {
                Ok(output) => output
                    .functions
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|f| {
                        f.function_arn.map(|arn| {
                            let truncated = arn.chars().take(50).collect::<String>();
                            (arn, truncated)
                        })
                    })
                    .collect(),
                Err(e) => {
                    eprintln!("Error fetching Lambdas: {:?}", e);
                    vec![]
                }
            }
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let template = SchedulerTemplate { schedules, lambda_arns };
    HttpResponse::Ok()
        .content_type("text/html")
        .body(template.render().unwrap())
}