use template_api::entities::BuildInfo;
use template_api::env::load as load_env;
use template_api::env::var as env_var;
use template_api::env::var_or as env_var_or;
use template_api::graph::{Mutation, Query, Subscription};
use template_api::handlers::graphql_handler;
use template_api::handlers::graphql_playground_handler;
use template_api::handlers::GraphQLExtension;
use template_api::handlers::GraphQLPlaygroundExtension;
use template_api::services::Config as ServicesConfig;
use template_api::services::{Services, Settings};
use template_api::util::default;

use std::env::VarError as EnvVarError;
use std::net::SocketAddr;

use anyhow::Context as AnyhowContext;
use anyhow::Result;

use http::header::CONTENT_TYPE;
use http::header::{HeaderValue, InvalidHeaderValue};
use http::Method;

use tower::ServiceBuilder;
use tower_http::cors::any as cors_any;
use tower_http::cors::AnyOr as CorsAnyOr;
use tower_http::cors::CorsLayer;
use tower_http::cors::Origin as CorsOrigin;
use tower_http::trace::TraceLayer;

use axum::body::Body;
use axum::handler::Handler;
use axum::routing::on;
use axum::routing::MethodFilter;
use axum::{AddExtensionLayer, Router, Server};

use graphql::extensions::apollo_persisted_queries as graphql_apq;
use graphql::Schema as GraphQLSchema;

use graphql_apq::ApolloPersistedQueries as GraphQLAPQExtension;
use graphql_apq::LruCacheStorage as GraphQLAPQStorage;

use mongodb::options::ClientOptions as MongoClientOptions;
use mongodb::Client as MongoClient;

use tracing::{debug, info};
use tracing_subscriber::fmt::layer as fmt_tracing_layer;
use tracing_subscriber::layer::SubscriberExt as TracingSubscriberLayerExt;
use tracing_subscriber::registry as tracing_registry;
use tracing_subscriber::util::SubscriberInitExt as TracingSubscriberInitExt;
use tracing_subscriber::EnvFilter as TracingEnvFilter;

use sentry::init as init_sentry;
use sentry::ClientOptions as SentryOptions;
use sentry::IntoDsn as IntoSentryDsn;
use sentry_tracing::layer as sentry_tracing_layer;

use bson::doc;
use chrono::{DateTime, FixedOffset};
use tokio::main as tokio;

#[tokio]
async fn main() -> Result<()> {
    // Load environment variables
    load_env().context("failed to load environment variables")?;

    // Initialize tracer
    debug!("initializing tracer");
    tracing_registry()
        .with(TracingEnvFilter::from_default_env())
        .with(fmt_tracing_layer())
        .with(sentry_tracing_layer())
        .try_init()
        .context("failed to initialize tracer")?;

    // Read environment name
    let environment = match env_var("TEMPLATE_ENV") {
        Ok(environment) => Some(environment),
        Err(EnvVarError::NotPresent) => None,
        Err(error) => {
            return Err(error)
                .context("failed to read environment variable TEMPLATE_ENV")
        }
    };

    // Read build info
    let build = {
        let timestamp = DateTime::<FixedOffset>::parse_from_rfc3339(env!(
            "BUILD_TIMESTAMP"
        ))
        .context("failed to parse build timestamp")?;
        let version = env!("CARGO_PKG_VERSION").to_owned();
        BuildInfo { timestamp, version }
    };

    // Initialize Sentry (if SENTRY_DSN is set)
    let _guard = match env_var("SENTRY_DSN") {
        Ok(dsn) => {
            debug!("initializing Sentry");
            let dsn = dsn.into_dsn().context("failed to parse Sentry DSN")?;
            let release = format!("template-api@{}", &build.version);
            let options = SentryOptions {
                dsn,
                release: Some(release.into()),
                environment: environment.clone().map(Into::into),
                ..default()
            };
            let guard = init_sentry(options);
            Some(guard)
        }
        Err(EnvVarError::NotPresent) => None,
        Err(error) => {
            return Err(error)
                .context("failed to read environment variable SENTRY_DSN")
        }
    };

    // Connect to database
    let database_client = {
        let uri = env_var_or("MONGO_URI", "mongodb://localhost:27017")
            .context("failed to read environment variable MONGO_URI")?;
        let options = {
            let mut options = MongoClientOptions::parse(uri)
                .await
                .context("failed to parse MongoDB connection string")?;
            options.retry_writes = true.into();
            options
        };
        MongoClient::with_options(options)
            .context("failed to build MongoDB client")?
    };

    // Connect to MongoDB
    info!("connecting to database");
    let database = {
        let name = env_var_or("MONGO_DATABASE", "template")
            .context("failed to read environment variable MONGO_DATABASE")?;
        let database = database_client.database(&name);
        database
            .run_command(doc! { "ping": 1 }, None)
            .await
            .context("failed to connect to MongoDB")?;
        database
    };

    info!("initializing services");

    // Build settings
    let settings = Settings::builder()
        .web_url({
            let url = env_var("JUSTCHAT_WEB_URL").context(
                "failed to read environment variable JUSTCHAT_WEB_URL",
            )?;
            url.parse().context("failed to parse justchat-web URL")?
        })
        .web_public_url({
            let url = env_var("JUSTCHAT_WEB_PUBLIC_URL").context(
                "failed to read environment variable JUSTCHAT_WEB_PUBLIC_URL",
            )?;
            url.parse()
                .context("failed to parse justchat-web public URL")?
        })
        .api_url({
            let url = env_var("JUSTCHAT_API_URL").context(
                "failed to read environment variable JUSTCHAT_API_URL",
            )?;
            url.parse().context("failed to parse justchat-api URL")?
        })
        .api_public_url({
            let url = env_var("JUSTCHAT_API_PUBLIC_URL").context(
                "failed to read environment variable JUSTCHAT_API_PUBLIC_URL",
            )?;
            url.parse()
                .context("failed to parse justchat-api public URL")?
        })
        .build();

    // Build services
    let services = {
        let config = ServicesConfig::builder()
            .database_client(database_client)
            .database(database)
            .settings(settings.clone())
            .build();
        Services::new(config)
    };

    // Build GraphQL schema
    let graphql_schema = {
        let query = Query::default();
        let mutation = Mutation::default();
        let subscription = Subscription::default();
        GraphQLSchema::build(query, mutation, subscription)
            .extension({
                let storage = GraphQLAPQStorage::new(1024);
                GraphQLAPQExtension::new(storage)
            })
            .data(build)
            .data(services.clone())
            .finish()
    };

    // Build extensions and middleware layers
    let graphql_extension = GraphQLExtension::new(&graphql_schema);
    let graphql_playground_extension =
        GraphQLPlaygroundExtension::new(&services)
            .context("failed to initialize GraphQL playground")?;
    let graphql_layer = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_headers(vec![CONTENT_TYPE])
        .allow_origin({
            match env_var("JUSTCHAT_API_CORS_ALLOW_ORIGIN") {
                Ok(origin) => {
                    let origin: CorsAnyOr<CorsOrigin> = if origin == "*" {
                        cors_any().into()
                    } else {
                        let origins = origin
                            .split(',')
                            .map(HeaderValue::from_str)
                            .collect::<Result<Vec<_>, InvalidHeaderValue>>()
                            .context("failed to parse CORS origin")?;
                        let list = CorsOrigin::list(origins);
                        list.into()
                    };
                    origin
                }
                Err(EnvVarError::NotPresent) => {
                    let Settings {
                        web_url,
                        web_public_url,
                        api_url,
                        api_public_url,
                        ..
                    } = &settings;
                    let origins =
                        [web_url, web_public_url, api_url, api_public_url]
                            .into_iter()
                            .map(|url| {
                                let mut url = url.to_owned();
                                url.set_path("");
                                let mut url = url.to_string();
                                url.pop();
                                HeaderValue::from_str(&url)
                            })
                            .collect::<Result<Vec<_>, InvalidHeaderValue>>()
                            .context("failed to parse CORS origin")?;
                    CorsOrigin::list(origins).into()
                }
                Err(error) => {
                    return Err(error).context(
                        "invalid environment variable \
                            JUSTCHAT_API_CORS_ALLOW_ORIGIN",
                    )
                }
            }
        });

    // Build routes
    let routes = Router::<Body>::new()
        .route(
            "/",
            on(
                MethodFilter::HEAD | MethodFilter::OPTIONS | MethodFilter::GET,
                graphql_playground_handler,
            ),
        )
        .route(
            "/graphql",
            on(
                MethodFilter::HEAD
                    | MethodFilter::OPTIONS
                    | MethodFilter::GET
                    | MethodFilter::POST,
                graphql_handler.layer(graphql_layer),
            ),
        );

    // Build service
    let service = routes
        .layer({
            ServiceBuilder::new()
                .layer(AddExtensionLayer::new(graphql_extension))
                .layer(AddExtensionLayer::new(graphql_playground_extension))
                .layer(TraceLayer::new_for_http())
        })
        .into_make_service();

    let host = env_var_or("TEMPLATE_API_HOST", "0.0.0.0")
        .context("failed to get environment variable TEMPLATE_API_HOST")?;
    let port = env_var_or("TEMPLATE_API_PORT", "3000")
        .context("failed to get environment variable TEMPLATE_API_PORT")?;
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .context("failed to parse server address")?;

    info!("listening on http://{}", &addr);
    Server::bind(&addr)
        .serve(service)
        .await
        .context("failed to serve routes")?;
    Ok(())
}
