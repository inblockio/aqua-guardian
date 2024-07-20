/// Verifies that the connecting client is on the approved list
pub mod cert_verifier {
    type SharedList = [webpki::types::TrustAnchor<'static>];
    type ArcSharedList = std::sync::Arc<SharedList>;
    #[derive(Debug)]
    pub struct CertVerifier {
        inner: parking_lot::RwLock<ArcSharedList>,
    }
    impl CertVerifier {
        /// Creates an instance of the verifier to keep context
        pub fn new() -> std::sync::Arc<Self> {
            std::sync::Arc::new(Self::default())
        }
        /// Returns what changed in the PKC
        pub fn change(&self, f: impl FnOnce(&SharedList) -> ArcSharedList) {
            eprintln!("Debug write: certverifier change");
            let mut w = self.inner.write();
            *w = f(&w);
        }
        /// Save these addresses as trusted peers
        pub fn set(&self, trusted: ArcSharedList) {
             eprintln!("Debug write: certverifier set");
            *self.inner.write() = trusted;
        }
        /// Return the selected witnessing network
        fn get_provider(&self) -> rustls::crypto::CryptoProvider {
            rustls::crypto::ring::default_provider()
        }
    }
    impl CertVerifier {
        fn default() -> Self {
            CertVerifier {
                inner: parking_lot::RwLock::new(std::sync::Arc::new([])),
            }
        }
    }
    impl rustls::server::danger::ClientCertVerifier for CertVerifier {
        fn root_hint_subjects(&self) -> &[rustls::DistinguishedName] {
            &[]
        }
        /// Ensure the clients certificate is valid
        fn verify_client_cert(
            &self,
            end_entity: &rustls::pki_types::CertificateDer<'_>,
            intermediates: &[rustls::pki_types::CertificateDer<'_>],
            now: rustls::pki_types::UnixTime,
        ) -> Result<rustls::server::danger::ClientCertVerified, rustls::Error> {
            let end_entity = webpki::EndEntityCert::try_from(end_entity)
                .map_err(|e| rustls::Error::Other(rustls::OtherError(std::sync::Arc::new(e))))?;
            eprintln!("Debug read: Verify Client Cert");
            let trust_anchors_read = self.inner.read();
            end_entity
                .verify_for_usage(
                    self.get_provider().signature_verification_algorithms.all,
                    &trust_anchors_read[..],
                    intermediates,
                    now,
                    webpki::KeyUsage::client_auth(),
                    None,
                    None,
                )
                .map_err(|e| rustls::Error::Other(rustls::OtherError(std::sync::Arc::new(e))))
                .map(|_| rustls::server::danger::ClientCertVerified::assertion())
        }

        fn verify_tls12_signature(
            &self,
            message: &[u8],
            cert: &rustls::pki_types::CertificateDer<'_>,
            dss: &rustls::DigitallySignedStruct,
        ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
            rustls::crypto::verify_tls12_signature(
                message,
                cert,
                dss,
                &self.get_provider().signature_verification_algorithms,
            )
        }

        fn verify_tls13_signature(
            &self,
            message: &[u8],
            cert: &rustls::pki_types::CertificateDer<'_>,
            dss: &rustls::DigitallySignedStruct,
        ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
            rustls::crypto::verify_tls13_signature(
                message,
                cert,
                dss,
                &self.get_provider().signature_verification_algorithms,
            )
        }

        fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
            self.get_provider()
                .signature_verification_algorithms
                .supported_schemes()
        }
    }
}

/// Stores the information needed for server hosting
pub struct ServerInfo {
    pub addr: std::net::SocketAddr,
    pub trusted: std::sync::Arc<cert_verifier::CertVerifier>,
    pub cert_chain: Vec<webpki::types::CertificateDer<'static>>,
    pub key_der: webpki::types::PrivateKeyDer<'static>,
}
/// Struct to store Server stuff
pub struct GuardianServer;

#[derive(thiserror::Error, Debug)]
/// Server error types
pub enum ServerError {
    /// Rustls is not happy
    #[error("tls: {0}")]
    Tls(#[from] rustls::Error),
    /// The std::io library would like to have a word with you
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

struct Service<H>(H);
impl<H: super::ApiHandler + Clone + 'static + Send, R> hyper::service::Service<hyper::Request<R>>
    for Service<H>
where
    H::Context: serde::Serialize,
{
    /// Contains the answer to the client's request
    type Response = http::Response<String>;

    type Error = http::Error;

    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<http::Response<String>, Self::Error>> + Send>,
    >;

    fn call(&self, req: hyper::Request<R>) -> Self::Future {
        macro_rules! ret {
            // ($($t:tt)*) => {
            //     return Box::pin(async {Ok($($t)*?)});
            // };
            ($status:ident: $($t:tt)*) => { {
                let resp = ::http::Response::builder()
                    .status(http::StatusCode::$status)
                    .body($($t)*);
                return Box::pin(async {resp});
            } }
        }
        #[derive(Clone, Copy)]
        enum Path {
            List,
            GetBranch,
            GetRevision,
        }
        let path_str = req.uri().path();
        let path = match path_str {
            "/list" => Path::List,
            "/get_branch" => Path::GetBranch,
            "/get_revision" => Path::GetRevision,
            _ => {
                ret!(NOT_FOUND: format!("endpoint does not exist: {path_str}"));
            }
        };
        match (path, req.method()) {
            (Path::List, &hyper::Method::GET) => (),
            (Path::GetBranch, &hyper::Method::GET) => (),
            (Path::GetRevision, &hyper::Method::GET) => (),
            (_, method) => {
                ret!(METHOD_NOT_ALLOWED: format!("endpoint {path_str} does not support method: {method}"));
            }
        };
        let handler = self.0.clone();
        let query = req.uri().query().unwrap_or_default().to_string();
        Box::pin(async move {
            macro_rules! query {
                ($query:ident) => {
                    match serde_urlencoded::from_str(&query) {
                        Ok(k) => k,
                        Err(e) => {
                            return http::Response::builder()
                                .status(http::StatusCode::BAD_REQUEST)
                                .body(format!("malformatted query: {e:?}"))
                        }
                    }
                };
            }

            let res = match path {
                // add get_rev_accessible & accessible_brach
                // replace with function out of state (accessible_latest)
                // add check if latest is accessible (???)
                Path::List => handler.list().await.map(|res| serde_json::to_string(&res)),
                Path::GetBranch => {
                    #[derive(serde::Deserialize)]
                    struct GetBranchArgs {
                        hash: guardian_common::custom_types::Hash,
                    }
                    let args: GetBranchArgs = query!(query);
                    // add check if branch is accessible (???)
                    handler
                        .get_branch(args.hash)
                        .await
                        .map(|res| serde_json::to_string(&res))
                }
                Path::GetRevision => {
                    #[derive(serde::Deserialize)]
                    struct GetRevisionArgs {
                        hash: guardian_common::custom_types::Hash,
                    }
                    let args: GetRevisionArgs = query!(query);
                    // add check if revision is accessible (???)
                    handler
                        .get_revision(args.hash)
                        .await
                        .map(|res| serde_json::to_string(&res))
                }
            };
            let (status, s) = match res {
                Ok(serde_res) => match serde_res {
                    Ok(s) => (http::StatusCode::OK, s),
                    Err(e) => (
                        http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!("error serializing result: {e:?}"),
                    ),
                },
                Err(e) => (http::StatusCode::INTERNAL_SERVER_ERROR, format!("{e:?}")),
            };
            http::Response::builder().status(status).body(s)
        })
    }
}

/// Serverside API of the guardian
impl<H: crate::ApiHandler + Send + Clone + 'static> super::ApiServer<H> for GuardianServer
where
    H::Context: serde::Serialize,
{
    /// Implements failures
    type Error = ServerError;
    /// Connection identification to deduce the client from
    type Info = [webpki::types::CertificateDer<'static>];
    /// Saves the info needed for a connection
    type Setup = ServerInfo;

    /// Start the hosting guardian
    fn run<
        G: for<'a> Fn(&'a Self::Info) -> F + Send + Clone + 'static,
        F: std::future::Future<Output = H> + Send,
    >(
        setup: Self::Setup,
        get_handler: G,
    ) -> Result<impl std::future::Future<Output = Result<(), Self::Error>>, Self::Error> {
        let config = std::sync::Arc::new(
            rustls::ServerConfig::builder()
                .with_client_cert_verifier(setup.trusted)
                .with_single_cert(setup.cert_chain, setup.key_der)?,
        );
        // let acceptor = tokio_rustls::TlsAcceptor::from(config);
        let listener = std::net::TcpListener::bind(setup.addr)?;
        listener.set_nonblocking(true)?;
        let listener = tokio::net::TcpListener::from_std(listener)?;
        Ok(async move {
            loop {
                let get_handler = get_handler.clone();
                // let acceptor = acceptor.clone();
                let config = config.clone();
                let (stream, _addr) = listener.accept().await?;
                tokio::spawn(async move {
                    // acceptor.accept_with(stream, |a| {
                    // });
                    let Ok(acceptor) = tokio_rustls::LazyConfigAcceptor::new(
                        rustls::server::Acceptor::default(),
                        stream,
                    )
                    .await
                    else {
                        println!("bad");
                        return;
                    };
                    acceptor.client_hello();

                    let Ok(conn) = acceptor.into_stream(config).await else {
                        println!("connection not okay");
                        return;
                    };
                    let certs = conn
                        .get_ref()
                        .1
                        .peer_certificates()
                        .expect("you needed a cert m8");
                    let handler = get_handler(certs).await;
                    eprintln!("got a handler for {:?}", conn.get_ref().0.peer_addr());
                    let io = hyper_util::rt::TokioIo::new(conn);
                    hyper_util::server::conn::auto::Builder::new(
                        hyper_util::rt::TokioExecutor::new(),
                    )
                    .serve_connection(io, Service(handler))
                    .await
                    .unwrap();
                });
            }
        })
    }
}
