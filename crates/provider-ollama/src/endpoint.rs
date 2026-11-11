//! The address of the Ollama server, restricted to this computer.

use reqwest::Url;

use crate::client::OllamaError;

/// Where Ollama listens unless configured otherwise.
pub const DEFAULT_ENDPOINT: &str = "http://localhost:11434";

/// Host names that always mean "this computer".
const LOOPBACK_HOSTS: [&str; 3] = ["localhost", "127.0.0.1", "[::1]"];

/// An HTTP address on this computer. It cannot be constructed for any
/// other host, which is what makes this provider local-only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalEndpoint {
    base: Url,
}

impl LocalEndpoint {
    /// Parses an address such as `http://localhost:11434`.
    ///
    /// # Errors
    ///
    /// Returns [`OllamaError::NotLocal`] unless the address is plain HTTP
    /// to a loopback host, with no path, credentials or query.
    pub fn parse(address: &str) -> Result<Self, OllamaError> {
        let base = Url::parse(address.trim()).map_err(|_| OllamaError::NotLocal)?;
        let is_local = base.scheme() == "http"
            && base
                .host_str()
                .is_some_and(|host| LOOPBACK_HOSTS.contains(&host))
            && base.username().is_empty()
            && base.password().is_none()
            && base.query().is_none()
            && base.fragment().is_none()
            && base.path() == "/";
        if is_local {
            Ok(Self { base })
        } else {
            Err(OllamaError::NotLocal)
        }
    }

    /// The full URL of an API path such as `/api/tags`.
    pub(crate) fn url(&self, path: &str) -> Url {
        let mut url = self.base.clone();
        url.set_path(path);
        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_addresses_are_accepted() {
        for address in [
            "http://localhost:11434",
            " http://127.0.0.1:11434/ ",
            "http://[::1]:11434",
            "http://localhost",
        ] {
            assert!(
                LocalEndpoint::parse(address).is_ok(),
                "{address} should be accepted"
            );
        }
    }

    #[test]
    fn anything_that_could_leave_this_computer_is_refused() {
        for address in [
            "http://example.com:11434",
            "http://192.168.1.20:11434",
            "http://0.0.0.0:11434",
            "https://localhost:11434",
            "http://localhost.example.com",
            "http://user:secret@localhost:11434",
            "http://localhost:11434/proxy/elsewhere",
            "http://localhost:11434?next=http://example.com",
            "ftp://localhost",
            "localhost:11434",
            "",
        ] {
            assert_eq!(
                LocalEndpoint::parse(address),
                Err(OllamaError::NotLocal),
                "{address}"
            );
        }
    }

    #[test]
    fn api_paths_are_resolved_against_the_endpoint() {
        let endpoint = LocalEndpoint::parse("http://localhost:11434").unwrap();
        assert_eq!(
            endpoint.url("/api/tags").as_str(),
            "http://localhost:11434/api/tags"
        );
    }
}
