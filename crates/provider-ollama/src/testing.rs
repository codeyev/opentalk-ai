//! The loopback mock server, set up as a stand-in for Ollama.

pub(crate) use opentalk_test_support::{MockServer, Route};

use crate::client::OllamaClient;
use crate::endpoint::LocalEndpoint;

/// A client that talks to `server`.
pub(crate) fn client_for(server: &MockServer) -> OllamaClient {
    OllamaClient::new(LocalEndpoint::parse(server.base_url()).expect("a local endpoint"))
}
