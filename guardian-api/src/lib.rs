use std::collections::HashSet;

use guardian_common::prelude::*;

/// Handles Clientside tasks of the guardian
pub trait ApiClient: Sized {
    /// Stores client connection data
    type ConnInfo;
    /// Failure return type
    type Error: std::error::Error;
    type Context;
    fn new(conn: ConnInfo) -> impl std::future::Future<Output = Result<Self, Self::Error>> + Send;
    fn list(&self) -> impl std::future::Future<Output = Result<HashSet<Hash>, Self::Error>> + Send;
    fn get_branch(
        &self,
        hash: Hash,
    ) -> impl std::future::Future<Output = Result<Branch<Self::Context>, Self::Error>> + Send;
    fn get_revision(
        &self,
        hash: Hash,
    ) -> impl std::future::Future<Output = Result<Revision, Self::Error>> + Send;
}

/// Defines the functions the guardians can use
pub trait ApiHandler {
    type Error: std::error::Error;
    type Context;
    /// Lists all available Hash chains by the latest revision hash
    fn list(
        &self,
    ) -> impl std::future::Future<Output = Result<HashSet<Hash>, Self::Error>> + std::marker::Send;
    /// Returns chain from requested revision back to the genesis
    fn get_branch(
        &self,
        hash: Hash,
    ) -> impl std::future::Future<Output = Result<Branch<Self::Context>, Self::Error>> + std::marker::Send;
    /// Returns only the requested revision
    fn get_revision(
        &self,
        hash: Hash,
    ) -> impl std::future::Future<Output = Result<Revision, Self::Error>> + std::marker::Send;
}

/// Handles Serverside tasks of the guardian
pub trait ApiServer<H> {
    type Error: std::error::Error;
    type Info: ?Sized;
    type Setup;
    fn run<
        G: for<'a> Fn(&'a Self::Info) -> F + Send + Clone + 'static,
        F: std::future::Future<Output = H> + Send,
    >(
        conn: Self::Setup,
        get_handler: G,
    ) -> Result<impl std::future::Future<Output = Result<(), Self::Error>> + Send, Self::Error>;
}

/// Handles tasks of the receiving guardian
pub mod client;

/// Stores the information needed for a connection between two guardians
pub struct ConnInfo {
    /// Hosting guardian's address
    pub url: reqwest::Url,
    /// Accepted client certificates
    pub cert: reqwest::Certificate,
    /// Connected  client's certificate
    pub identity: reqwest::Identity,
}

/// Handles tasks of the sharing guardian
pub mod server;
