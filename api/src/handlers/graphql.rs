use super::*;

use graph::{Mutation, Query, Subscription};

use axum::extract::ws::WebSocketUpgrade;

use ::graphql::http::ALL_WEBSOCKET_PROTOCOLS as GRAPHQL_WEBSOCKET_PROTOCOLS;
use ::graphql::Schema as GraphQLSchema;
use ::graphql::ServerError as GraphQLError;

use graphql_axum::graphql_subscription;
use graphql_axum::GraphQLRequest;
use graphql_axum::GraphQLResponse;
use graphql_axum::SecWebsocketProtocol as WebSocketProtocol;

#[derive(Clone)]
pub struct GraphQLExtension {
    schema: GraphQLSchema<Query, Mutation, Subscription>,
}

impl GraphQLExtension {
    pub fn new(schema: &GraphQLSchema<Query, Mutation, Subscription>) -> Self {
        GraphQLExtension {
            schema: schema.to_owned(),
        }
    }
}

pub async fn graphql_handler(
    Extension(extension): Extension<GraphQLExtension>,
    request: Option<GraphQLRequest>,
    websocket: Option<WebSocketUpgrade>,
    websocket_protocol: Option<HeaderExtractor<WebSocketProtocol>>,
) -> Response<BoxBody> {
    let GraphQLExtension { schema } = extension;
    if let (Some(websocket), Some(HeaderExtractor(protocol))) =
        (websocket, websocket_protocol)
    {
        let response = websocket
            .protocols(GRAPHQL_WEBSOCKET_PROTOCOLS)
            .on_upgrade(move |websocket| async move {
                trace!("received WebSocket connection");
                graphql_subscription(websocket, schema, protocol).await
            })
            .into_response();
        let (head, body) = response.into_parts();
        return Response::from_parts(head, box_body(body));
    }
    if let Some(GraphQLRequest(request)) = request {
        let response = schema.execute(request).await;
        response
            .errors
            .iter()
            .for_each(|error| match error.message.as_str() {
                "PersistedQueryNotFound" => (),
                _ => {
                    let GraphQLError {
                        message,
                        locations,
                        path,
                        ..
                    } = error;
                    let locations = {
                        let locations = locations
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>();
                        to_json_string(&locations).unwrap()
                    };
                    let path = to_json_string(path).unwrap();
                    error!(
                        target: "template_api::graphql",
                        %locations,
                        %path,
                        "{}", message,
                    );
                }
            });
        let response = GraphQLResponse::from(response).into_response();
        let (head, body) = response.into_parts();
        return Response::from_parts(head, box_body(body));
    }
    {
        let response = StatusCode::BAD_REQUEST.into_response();
        let (head, body) = response.into_parts();
        Response::from_parts(head, box_body(body))
    }
}
