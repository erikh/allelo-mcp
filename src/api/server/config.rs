use crate::api::llm::{LLMClientParams, LLMClientType};

use serde::Deserialize;
use std::{net::SocketAddr, path::PathBuf};
use tracing::info;
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Clone, Deserialize)]
pub enum LogLevel {
	#[serde(rename = "warn")]
	Warn,
	#[serde(rename = "info")]
	Info,
	#[serde(rename = "error")]
	Error,
	#[serde(rename = "debug")]
	Debug,
	#[serde(rename = "trace")]
	Trace,
}

impl From<LogLevel> for tracing::Level {
	fn from(value: LogLevel) -> Self {
		match value {
			LogLevel::Info => tracing::Level::INFO,
			LogLevel::Warn => tracing::Level::WARN,
			LogLevel::Error => tracing::Level::ERROR,
			LogLevel::Debug => tracing::Level::DEBUG,
			LogLevel::Trace => tracing::Level::TRACE,
		}
	}
}

impl From<tracing::Level> for LogLevel {
	fn from(value: tracing::Level) -> Self {
		match value {
			tracing::Level::INFO => LogLevel::Info,
			tracing::Level::WARN => LogLevel::Warn,
			tracing::Level::ERROR => LogLevel::Error,
			tracing::Level::DEBUG => LogLevel::Debug,
			tracing::Level::TRACE => LogLevel::Trace,
		}
	}
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
	pub listen: SocketAddr,
	pub log_level: LogLevel,
	pub client_type: Option<LLMClientType>,
	pub client_params: Option<LLMClientParams>,
}

impl Default for Config {
	fn default() -> Self {
		Config {
			listen: "127.0.0.1:8999".parse().unwrap(),
			log_level: LogLevel::Info,
			client_params: None,
			client_type: None,
		}
	}
}

impl Config {
	pub fn from_file(filename: PathBuf) -> anyhow::Result<Self> {
		let r =
			std::fs::OpenOptions::new().read(true).open(filename)?;
		let this: Self = serde_yaml_ng::from_reader(r)?;
		let subscriber = FmtSubscriber::builder()
			.with_max_level(Into::<tracing::Level>::into(
				this.log_level.clone(),
			))
			.finish();
		tracing::subscriber::set_global_default(subscriber)?;
		info!("Configuration parsed successfully.");
		Ok(this)
	}
}
