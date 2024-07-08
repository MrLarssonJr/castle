use chrono::Utc;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Level {
	Trace,
	Debug,
	Info,
	Warn,
	Error,
}

impl Display for Level {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Level::Trace => write!(f, "TRACE"),
			Level::Debug => write!(f, "DEBUG"),
			Level::Info => write!(f, "INFO"),
			Level::Warn => write!(f, "WARN"),
			Level::Error => write!(f, "ERROR"),
		}
	}
}

#[derive(Debug, Clone)]
pub struct Logger {
	pub name: Arc<str>,
}

impl Logger {
	pub fn new(name: &str) -> Logger {
		Logger {
			name: Arc::from(name),
		}
	}

	pub fn child(&self, name: &str) -> Logger {
		let Logger { name: parent_name } = self;
		Logger {
			name: Arc::from(format!("{parent_name}.{name}")),
		}
	}

	pub fn at(&self, level: Level, msg: &str) {
		let now = Utc::now();
		let Logger { name } = self;
		println!("{now} {level} {name} {msg}")
	}

	pub fn err_at(&self, err: impl Error, level: Level, msg: &str) {
		let mut msg = String::from(msg);

		let mut err: &dyn Error = &err;
		loop {
			msg.push_str(&format!("\n{err}"));

			let Some(source) = err.source() else {
				break;
			};
			err = source;
		}

		self.at(level, &msg);
	}

	pub fn at_trace(&self, msg: &str) {
		self.at(Level::Trace, msg)
	}

	pub fn at_debug(&self, msg: &str) {
		self.at(Level::Debug, msg)
	}

	pub fn at_info(&self, msg: &str) {
		self.at(Level::Info, msg)
	}

	pub fn at_warn(&self, msg: &str) {
		self.at(Level::Warn, msg)
	}

	pub fn at_error(&self, msg: &str) {
		self.at(Level::Error, msg)
	}

	pub fn err_at_trace(&self, err: impl Error, msg: &str) {
		self.err_at(err, Level::Trace, msg)
	}

	pub fn err_at_debug(&self, err: impl Error, msg: &str) {
		self.err_at(err, Level::Debug, msg)
	}

	pub fn err_at_info(&self, err: impl Error, msg: &str) {
		self.err_at(err, Level::Info, msg)
	}

	pub fn err_at_warn(&self, err: impl Error, msg: &str) {
		self.err_at(err, Level::Warn, msg)
	}

	pub fn err_at_error(&self, err: impl Error, msg: &str) {
		self.err_at(err, Level::Error, msg)
	}
}
