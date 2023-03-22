#[macro_use] extern crate rocket;

pub mod model;

use async_graphql::{EmptyMutation, EmptySubscription, Schema, http::GraphiQLSource};
use async_graphql_rocket::{GraphQLQuery, GraphQLRequest, GraphQLResponse};
use rocket::{response::content, routes, State};
use crate::model::{QueryRoot, SubscriptionRoot};

pub type GraphQLSchema = Schema<QueryRoot, EmptyMutation, SubscriptionRoot>;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/playground")]
fn graphiql() -> content::RawHtml<String> {
    content::RawHtml(GraphiQLSource::build().endpoint("/graphql").subscription_endpoint("/graphql").finish())
}

#[get("/graphql?<query..>")]
async fn graphql_query(schema: &State<GraphQLSchema>, query: GraphQLQuery) -> GraphQLResponse {
    query.execute(schema.inner()).await
}

#[post("/graphql", data = "<request>", format = "application/json")]
async fn graphql_request(
    schema: &State<GraphQLSchema>,
    request: GraphQLRequest,
) -> GraphQLResponse {
    request.execute(schema.inner()).await
}

#[launch]
fn rocket() -> _ {
    GraphQlSub
    let schema = Schema::build(QueryRoot, EmptyMutation, SubscriptionRoot).finish();

    rocket::build().manage(schema).mount("/", routes![index, graphql_query, graphql_request, graphiql])
}
