use template_api::entities::BuildInfo;
use template_api::env::load as load_env;
use template_api::env::var as env_var;
use template_api::env::var_or as env_var_or;
use template_api::graph::Query;
use template_api::services::Config as ServicesConfig;
use template_api::services::{Services, Settings};

use std::borrow::Cow;
use std::convert::Infallible;
use std::net::SocketAddr;

use anyhow::Context as AnyhowContext;
use anyhow::Result;

use http::header::CONTENT_TYPE;
use http::{Response, StatusCode};

use warp::path::end as path_end;
use warp::reject::custom as rejection;
use warp::reject::Reject;
use warp::reply::json as reply_json;
use warp::reply::with_status as reply_with_status;
use warp::{get, head, path, serve};
use warp::{Filter, Rejection, Reply};

use graphql::extensions::apollo_persisted_queries as graphql_apq;
use graphql::http::playground_source as graphql_playground_source;
use graphql::http::GraphQLPlaygroundConfig;
use graphql::Request as GraphQLRequest;
use graphql::Response as GraphQLResponse;
use graphql::{EmptyMutation, EmptySubscription, Schema};

use graphql_apq::ApolloPersistedQueries as GraphQLAPQExtension;
use graphql_apq::LruCacheStorage as GraphQLAPQStorage;

use graphql_warp::graphql as warp_graphql;
use graphql_warp::graphql_subscription as warp_graphql_subscription;
use graphql_warp::BadRequest as WarpGraphQLBadRequest;
use graphql_warp::Response as WarpGraphQLResponse;

use mongodb::options::ClientOptions as MongoClientOptions;
use mongodb::Client as MongoClient;

use tracing::{error, info};
use tracing_subscriber::fmt::init as init_tracer;

use anyhow::Error;
use bson::doc;
use chrono::{DateTime, FixedOffset};
use serde::Serialize;
use tokio::main as tokio;

#[tokio]
async fn main() -> Result<()> {
    // Load environment variables and initialize tracer
    load_env().context("failed to load environment variables")?;
    init_tracer();

    // Read build info
    let build = {
        let timestamp = DateTime::<FixedOffset>::parse_from_rfc3339(env!(
            "BUILD_TIMESTAMP"
        ))
        .context("failed to parse build timestamp")?;
        let version = env!("CARGO_PKG_VERSION").to_owned();
        BuildInfo { timestamp, version }
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
    info!(target: "template-api", "connecting to database");
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

    info!(target: "template-api", "initializing services");

    // Build settings
    let settings = Settings::builder()
        .web_public_url({
            let url = env_var("TEMPLATE_WEB_PUBLIC_URL").context(
                "failed to read environment variable TEMPLATE_WEB_PUBLIC_URL",
            )?;
            url.parse()
                .context("failed to parse template-web public URL")?
        })
        .api_public_url({
            let url = env_var("TEMPLATE_API_PUBLIC_URL").context(
                "failed to read environment variable TEMPLATE_API_PUBLIC_URL",
            )?;
            url.parse()
                .context("failed to parse template-api public URL")?
        })
        .build();

    // Build services
    let services = {
        let config = ServicesConfig::builder()
            .database_client(database_client)
            .database(database)
            .settings(settings)
            .build();
        Services::new(config)
    };

    // Build GraphQL schema
    let graphql_schema = {
        let query = Query::new();
        let mutation = EmptyMutation;
        let subscription = EmptySubscription;
        Schema::build(query, mutation, subscription)
            .extension({
                let storage = GraphQLAPQStorage::new(1024);
                GraphQLAPQExtension::new(storage)
            })
            .data(build)
            .data(services.clone())
            .finish()
    };

    // Build GraphQL filter
    let graphql_filter = {
        let graphql = {
            warp_graphql(graphql_schema.clone()).untuple_one().and_then(
                |schema: Schema<_, _, _>, request: GraphQLRequest| async move {
                    let response = schema.execute(request).await;
                    trace_graphql_response(&response);
                    let response = WarpGraphQLResponse::from(response);
                    Ok::<_, Infallible>(response)
                },
            )
        };
        let graphql_subscription = warp_graphql_subscription(graphql_schema);
        path("graphql")
            .and(path_end())
            .and(graphql_subscription.or(graphql))
    };

    // Build GraphQL playground filter
    let graphql_playground_filter = (get().or(head()))
        .map({
            let services = services.clone();
            move |_| services.clone()
        })
        .and_then(|services: Services| async move {
            let endpoint = {
                let mut endpoint = services.settings().api_public_url.clone();
                if !matches!(endpoint.scheme(), "http" | "https") {
                    let error = ErrorRejection::new(
                        "invalid GraphQL playground endpoint scheme",
                    );
                    return Err(rejection(error));
                }
                let path = endpoint.path();
                if !path.ends_with('/') {
                    let path = path.to_owned() + "/";
                    endpoint.set_path(&path);
                }
                endpoint.join("graphql").unwrap()
            };

            let subscription_endpoint = {
                let mut endpoint = endpoint.clone();
                let scheme = match endpoint.scheme() {
                    "http" => "ws",
                    "https" => "wss",
                    _ => {
                        panic!("invalid GraphQL playground endpoint scheme")
                    }
                };
                endpoint.set_scheme(scheme).unwrap();
                endpoint
            };

            let config = GraphQLPlaygroundConfig::new(endpoint.as_str())
                .subscription_endpoint(subscription_endpoint.as_str());
            let source = graphql_playground_source(config);
            Ok(source)
        })
        .map(|source: String| {
            Response::builder()
                .header(CONTENT_TYPE, "text/html")
                .body(source)
        });

    // Build root filter
    let filter = (path_end().and(graphql_playground_filter))
        .or(graphql_filter)
        .recover(recover);

    let host = env_var_or("TEMPLATE_API_HOST", "0.0.0.0")
        .context("failed to get environment variable TEMPLATE_API_HOST")?;
    let port = env_var_or("TEMPLATE_API_PORT", "3000")
        .context("failed to get environment variable TEMPLATE_API_PORT")?;
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .context("failed to parse server address")?;

    info!(target: "template-api", "listening on http://{}", &addr);
    serve(filter).run(addr).await;
    Ok(())
}

async fn recover(rejection: Rejection) -> Result<impl Reply, Infallible> {
    let (error, status_code) = if rejection.is_not_found() {
        let error = ErrorRejection::new("not found");
        (error, StatusCode::NOT_FOUND)
    } else if let Some(error) = rejection.find::<ErrorRejection>() {
        let error = error.to_owned();
        (error, StatusCode::INTERNAL_SERVER_ERROR)
    } else if let Some(error) = rejection.find::<WarpGraphQLBadRequest>() {
        let WarpGraphQLBadRequest(error) = error;
        let error = ErrorRejection::new(error.to_string());
        (error, StatusCode::BAD_REQUEST)
    } else if let Some(error) = rejection.find::<Error>() {
        let error = ErrorRejection::from(error);
        (error, StatusCode::INTERNAL_SERVER_ERROR)
    } else {
        error!(target: "template-api", "unhandled rejection: {:?}", &rejection);
        let error = ErrorRejection::new("internal server error");
        (error, StatusCode::INTERNAL_SERVER_ERROR)
    };

    let reply = ErrorReply {
        errors: vec![error],
        status_code: status_code.as_u16(),
    };
    let reply = reply_json(&reply);
    let reply = reply_with_status(reply, status_code);
    Ok::<_, Infallible>(reply)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorReply {
    errors: Vec<ErrorRejection>,
    status_code: u16,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorRejection {
    message: Cow<'static, str>,
}

impl ErrorRejection {
    pub fn new(msg: impl Into<Cow<'static, str>>) -> Self {
        ErrorRejection {
            message: msg.into(),
        }
    }
}

impl Reject for ErrorRejection {}

impl From<&Error> for ErrorRejection {
    fn from(error: &Error) -> Self {
        let msg = format!("{:#}", error);
        Self::new(msg)
    }
}

impl From<Error> for ErrorRejection {
    fn from(error: Error) -> Self {
        Self::from(&error)
    }
}

fn trace_graphql_response(response: &GraphQLResponse) {
    response
        .errors
        .iter()
        .for_each(|error| match error.message.as_str() {
            "PersistedQueryNotFound" => (),
            _ => {
                error!(target: "template-api", "GraphQL error: {:#}", error)
            }
        })
}
