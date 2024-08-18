use graphql_client::GraphQLQuery;

/// TODO: why doesn't this work? i think the schema is the problem. i think one schema is shared by both queries?
/// <https://developer.moxie.xyz/use-cases/everyday-rewards/check-users-everyday-rewards-amount>
pub mod check_claim_transaction_status {
    // use super::*;
    //
    // #[derive(GraphQLQuery)]
    // #[graphql(
    //     schema_path = "graphql/check_claim_transaction_status/schema.graphql",
    //     query_path = "graphql/check_claim_transaction_status/query.graphql"
    // )]
    // pub struct FarcasterUserClaimTransactionDetails;
}

/// <https://developer.moxie.xyz/use-cases/everyday-rewards/claim-everyday-rewards>
pub mod claim_everyday_rewards {
    use super::*;

    #[derive(GraphQLQuery)]
    #[graphql(
        schema_path = "graphql/claim_everyday_rewards/schema.graphql",
        query_path = "graphql/claim_everyday_rewards/query.graphql"
    )]
    pub struct FarcasterUserClaimMoxie;
}

/// TODO: why doesn't this work? i think the schema is the problem. i think one schema is shared by both queries?
pub mod check_user_everyday_rewards_amount {
    // use super::*;
    //
    // #[derive(GraphQLQuery)]
    // #[graphql(
    //     schema_path = "graphql/check_user_everyday_rewards_amount/schema.graphql",
    //     query_path = "graphql/check_user_everyday_rewards_amount/query.graphql"
    // )]
    // pub struct FarcasterUserClaimTransactionDetails;
}
