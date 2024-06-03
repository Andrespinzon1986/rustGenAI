use std::fmt;

pub struct ClientConfig {
	/// The api key to be used. Will take precendence over key_from_env is set
	pub key: Option<String>,

	/// Will take the key from env. (if .key is None)
	/// NOT IMPLEMENTED YET
	pub key_from_env: Option<EnvName>,

	/// Eventual endpoint
	pub endpoint: Option<EndPoint>,
}

/// Convenient Constructors
/// Note: Those constructor(s) will call `default()` and sent the given property
///       They are just for convenience, the builder setter function can be used.
impl ClientConfig {
	pub fn from_key(key: impl Into<String>) -> Self {
		Self::default().key(key)
	}
}

/// Builder setters
impl ClientConfig {
	pub fn key(mut self, key: impl Into<String>) -> Self {
		self.key = Some(key.into());
		self
	}
	pub fn key_from_env(mut self, key_from_env: impl Into<EnvName>) -> Self {
		self.key_from_env = Some(key_from_env.into());
		self
	}
	pub fn endpoint(mut self, endpoint: impl Into<EndPoint>) -> Self {
		self.endpoint = Some(endpoint.into());
		self
	}
}

impl fmt::Debug for ClientConfig {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("ClientConfig")
			.field("key_from_env", &self.key_from_env)
			.field("key", &self.key.as_ref().map(|_| "REDACTED"))
			.field("endpoint", &self.endpoint)
			.finish()
	}
}

/// Create the default config.
impl Default for ClientConfig {
	fn default() -> Self {
		Self {
			key_from_env: Some(EnvName::ProviderDefault),
			key: None,
			endpoint: None,
		}
	}
}

// region:    --- KeyFrom
#[derive(Debug)]
pub enum EnvName {
	ProviderDefault,
	Named(String),
}

impl From<String> for EnvName {
	fn from(name: String) -> Self {
		Self::Named(name)
	}
}

impl From<&str> for EnvName {
	fn from(name: &str) -> Self {
		Self::Named(name.to_string())
	}
}

impl From<&String> for EnvName {
	fn from(name: &String) -> Self {
		Self::Named(name.to_string())
	}
}
// endregion: --- KeyFrom

// region:    --- EndPoint
#[derive(Debug)]
pub struct EndPoint {
	pub host: Option<String>,
	pub port: Option<u16>,
}

impl From<(String, u16)> for EndPoint {
	fn from((host, port): (String, u16)) -> Self {
		Self {
			host: Some(host),
			port: Some(port),
		}
	}
}

impl From<(&str, u16)> for EndPoint {
	fn from((host, port): (&str, u16)) -> Self {
		Self {
			host: Some(host.to_string()),
			port: Some(port),
		}
	}
}

// endregion: --- EndPoint
