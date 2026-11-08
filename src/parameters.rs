use std::time::Duration;

use redis::aio::ConnectionManager;

pub const SYSTEM_PARAMETER_CACHE_KEY: &str = "tmj:parameters:system";
pub const ERROR_CODE_CACHE_KEY: &str = "tmj:parameters:errors";
const LOOKUP_TIMEOUT: Duration = Duration::from_millis(250);

pub mod keys {
    pub const PAGINATION_DEFAULT_LIMIT: &str = "PAGINATION_DEFAULT_LIMIT";
    pub const PAGINATION_MAX_LIMIT: &str = "PAGINATION_MAX_LIMIT";
    pub const JWT_ACCESS_EXPIRATION_SECS: &str = "JWT_ACCESS_EXPIRATION_SECS";
    pub const JWT_REFRESH_EXPIRATION_SECS: &str = "JWT_REFRESH_EXPIRATION_SECS";
    pub const RECOVERY_OTP_TTL_SECS: &str = "RECOVERY_OTP_TTL_SECS";
    pub const RECOVERY_RESEND_WAIT_SECS: &str = "RECOVERY_RESEND_WAIT_SECS";
    pub const RECOVERY_RESET_GRANT_TTL_SECS: &str = "RECOVERY_RESET_GRANT_TTL_SECS";
    pub const RECOVERY_IP_LIMIT: &str = "RECOVERY_IP_LIMIT";
    pub const RECOVERY_IP_WINDOW_SECS: &str = "RECOVERY_IP_WINDOW_SECS";
    pub const RECOVERY_EMAIL_SEND_LIMIT: &str = "RECOVERY_EMAIL_SEND_LIMIT";
    pub const RECOVERY_EMAIL_SEND_WINDOW_SECS: &str = "RECOVERY_EMAIL_SEND_WINDOW_SECS";
    pub const RECOVERY_OTP_MAX_ATTEMPTS: &str = "RECOVERY_OTP_MAX_ATTEMPTS";
}

pub const DEFAULT_ERROR_MESSAGES: &[(&str, &str)] = &[
    ("01", "Invalid username/email or password"),
    ("02", "Invalid or expired authentication token"),
    ("03", "Invalid request data"),
    ("04", "Resource not found"),
    ("05", "Access forbidden"),
    ("09", "Resource conflict"),
    ("99", "An internal error occurred"),
];

pub fn default_error_message(status_code: &str) -> &'static str {
    DEFAULT_ERROR_MESSAGES
        .iter()
        .find_map(|(code, message)| (*code == status_code).then_some(*message))
        .unwrap_or("An error occurred")
}

#[derive(Clone, Copy)]
pub struct NumericParameter {
    pub key: &'static str,
    pub default: i64,
    pub min: i64,
    pub max: i64,
}

impl NumericParameter {
    fn parse(self, value: Option<&str>) -> i64 {
        value
            .and_then(|value| value.parse::<i64>().ok())
            .filter(|value| (self.min..=self.max).contains(value))
            .unwrap_or(self.default)
    }
}

#[derive(Clone)]
pub struct ParameterCache {
    connection: ConnectionManager,
}

impl ParameterCache {
    pub fn new(connection: ConnectionManager) -> Self {
        Self { connection }
    }

    async fn field(&self, hash: &str, key: &str) -> Option<String> {
        let mut connection = self.connection.clone();
        match tokio::time::timeout(
            LOOKUP_TIMEOUT,
            redis::cmd("HGET")
                .arg(hash)
                .arg(key)
                .query_async::<Option<String>>(&mut connection),
        )
        .await
        {
            Ok(Ok(value)) => value,
            Ok(Err(_)) => {
                tracing::warn!(target: "tmj_parameters", hash, key, "Cache lookup failed; using constant default");
                None
            }
            Err(_) => {
                tracing::warn!(target: "tmj_parameters", hash, key, "Cache lookup timed out; using constant default");
                None
            }
        }
    }

    pub async fn string(&self, key: &str, default: &str) -> String {
        self.field(SYSTEM_PARAMETER_CACHE_KEY, key)
            .await
            .unwrap_or_else(|| default.to_owned())
    }

    pub async fn message_error(&self, status_code: &str) -> String {
        self.field(ERROR_CODE_CACHE_KEY, status_code)
            .await
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| default_error_message(status_code).to_owned())
    }

    /// Read a related group in a single Redis command, with per-field consumer defaults.
    pub async fn numbers<const N: usize>(&self, specs: &[NumericParameter; N]) -> [i64; N] {
        if N == 0 {
            return [0; N];
        }
        let mut command = redis::cmd("HMGET");
        command.arg(SYSTEM_PARAMETER_CACHE_KEY);
        for spec in specs {
            command.arg(spec.key);
        }
        let mut connection = self.connection.clone();
        let values = match tokio::time::timeout(
            LOOKUP_TIMEOUT,
            command.query_async::<Vec<Option<String>>>(&mut connection),
        )
        .await
        {
            Ok(Ok(values)) => Some(values),
            Ok(Err(_)) => {
                tracing::warn!(target: "tmj_parameters", "Parameter group lookup failed; using constant defaults");
                None
            }
            Err(_) => {
                tracing::warn!(target: "tmj_parameters", "Parameter group lookup timed out; using constant defaults");
                None
            }
        };
        std::array::from_fn(|index| {
            let value = values
                .as_ref()
                .and_then(|values| values.get(index))
                .and_then(|value| value.as_deref());
            specs[index].parse(value)
        })
    }
}
