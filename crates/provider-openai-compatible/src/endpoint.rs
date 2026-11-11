//! The address of an OpenAI-compatible service.

use reqwest::Url;

/// A base URL the settings start with, as an example of the format.
pub const DEFAULT_BASE_URL: &str = "https://openrouter.ai/api/v1";

/// Host names that always mean "this computer".
const LOOPBACK_HOSTS: [&str; 3] = ["localhost", "127.0.0.1", "[::1]"];

/// The address was not one this provider will send data to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndpointError;

/// The base URL of an OpenAI-compatible API, such as
/// `https://openrouter.ai/api/v1`.
///
/// A remote service must use HTTPS, so the API key and the text are never
/// sent in the clear. Plain HTTP is accepted only for a server on this
/// computer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    base: Url,
}

impl Endpoint {
    /// Parses a base URL.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is not `https` (or `http` on this
    /// computer), or carries credentials, a query or a fragment.
    pub fn parse(address: &str) -> Result<Self, EndpointError> {
        let base = Url::parse(address.trim()).map_err(|_| EndpointError)?;
        let is_loopback = base
            .host_str()
            .is_some_and(|host| LOOPBACK_HOSTS.contains(&host));
        let is_secure_enough = match base.scheme() {
            "https" => base.host_str().is_some(),
            "http" => is_loopback,
            _ => false,
        };
        let is_plain = base.username().is_empty()
            && base.password().is_none()
            && base.query().is_none()
            && base.fragment().is_none();
        if is_secure_enough && is_plain {
            Ok(Self { base })
        } else {
            Err(EndpointError)
        }
    }

    /// Whether the service runs on this computer, so that using it sends
    /// nothing over the network.
    #[must_use]
    pub fn is_local(&self) -> bool {
        self.base
            .host_str()
            .is_some_and(|host| LOOPBACK_HOSTS.contains(&host))
    }

    /// The host name, for telling the user where their data goes.
    #[must_use]
    pub fn host(&self) -> &str {
        self.base.host_str().unwrap_or_default()
    }

    /// The URL of the chat completions endpoint under this base.
    pub(crate) fn chat_completions_url(&self) -> String {
        format!(
            "{}/chat/completions",
            self.base.as_str().trim_end_matches('/')
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_services_must_use_https() {
        assert!(Endpoint::parse("https://openrouter.ai/api/v1").is_ok());
        assert!(Endpoint::parse(" https://api.openai.com/v1/ ").is_ok());
        assert_eq!(
            Endpoint::parse("http://openrouter.ai/api/v1"),
            Err(EndpointError)
        );
    }

    #[test]
    fn plain_http_is_accepted_only_on_this_computer() {
        let local = Endpoint::parse("http://localhost:1234/v1").unwrap();
        assert!(local.is_local());
        assert!(Endpoint::parse("http://127.0.0.1:8080/v1").is_ok());
        assert_eq!(
            Endpoint::parse("http://192.168.1.20:1234/v1"),
            Err(EndpointError)
        );
        assert!(
            !Endpoint::parse("https://openrouter.ai/api/v1")
                .unwrap()
                .is_local()
        );
    }

    #[test]
    fn addresses_with_credentials_or_extras_are_refused() {
        for address in [
            "https://user:secret@openrouter.ai/api/v1",
            "https://openrouter.ai/api/v1?key=abc",
            "https://openrouter.ai/api/v1#x",
            "ftp://openrouter.ai",
            "openrouter.ai/api/v1",
            "",
        ] {
            assert_eq!(Endpoint::parse(address), Err(EndpointError), "{address}");
        }
    }

    #[test]
    fn the_chat_url_is_built_under_the_base_path() {
        for base in [
            "https://openrouter.ai/api/v1",
            "https://openrouter.ai/api/v1/",
        ] {
            assert_eq!(
                Endpoint::parse(base).unwrap().chat_completions_url(),
                "https://openrouter.ai/api/v1/chat/completions"
            );
        }
    }

    #[test]
    fn the_host_names_where_data_goes() {
        assert_eq!(
            Endpoint::parse("https://openrouter.ai/api/v1")
                .unwrap()
                .host(),
            "openrouter.ai"
        );
    }
}
