//! These queries are simply copied out of the official docs. I could clean them up some so that they don't have overlapping names, but I like this mod pattern.
use graphql_client::GraphQLQuery;

pub const AIRSTACK_CLAIMS_URL: &str = "https://claims.airstack.xyz/moxie";
pub const AIRSTACK_PROTOCOL_SUBGRAPH_URL: &str = "https://airstack.xyz/api/protocol-subgraph";

/// From <https://developer.moxie.xyz/use-cases/everyday-rewards/check-users-everyday-rewards-amount>.
pub mod check_claim_transaction_status {
    use super::*;

    #[derive(GraphQLQuery)]
    #[graphql(
        schema_path = "graphql/airstack-claims/check_claim_transaction_status/schema.graphql",
        query_path = "graphql/airstack-claims/check_claim_transaction_status/query.graphql",
        response_derives = "Debug"
    )]
    pub struct FarcasterUserClaimTransactionDetails;

    /// prettier name for the FarcasterUserClaimTransactionDetails derived query object.
    pub type Query = FarcasterUserClaimTransactionDetails;

    /// prettier name for farcaster_user_claim_transaction_details::Variables.
    pub type Variables = farcaster_user_claim_transaction_details::Variables;
}

/// From <https://developer.moxie.xyz/use-cases/everyday-rewards/claim-everyday-rewards>.
pub mod claim_everyday_rewards {
    use super::*;

    #[derive(GraphQLQuery)]
    #[graphql(
        schema_path = "graphql/airstack-claims/claim_everyday_rewards/schema.graphql",
        query_path = "graphql/airstack-claims/claim_everyday_rewards/query.graphql",
        response_derives = "Debug"
    )]
    pub struct FarcasterUserClaimMoxie;

    /// prettier name for the FarcasterUserClaimMoxie derived query object.
    pub type Query = FarcasterUserClaimMoxie;

    /// prettier name for farcaster_user_claim_moxie::Variables.
    pub type Variables = farcaster_user_claim_moxie::Variables;
}

/// From <https://developer.moxie.xyz/use-cases/everyday-rewards/check-users-everyday-rewards-amount>.
pub mod check_user_everyday_rewards_amount {
    use super::*;

    /// this is the same name as check_claim_transaction_status's Query object
    #[derive(GraphQLQuery)]
    #[graphql(
        schema_path = "graphql/airstack-claims/check_user_everyday_rewards_amount/schema.graphql",
        query_path = "graphql/airstack-claims/check_user_everyday_rewards_amount/query.graphql",
        response_derives = "Debug"
    )]
    pub struct FarcasterUserClaimTransactionDetails;

    /// prettier name for the FarcasterUserClaimTransactionDetails derived query object
    pub type Query = FarcasterUserClaimTransactionDetails;

    /// prettier name for farcaster_user_claim_transaction_details::Variables.
    pub type Variables = farcaster_user_claim_transaction_details::Variables;
}

/// From <https://airstack.xyz/my-assets>.
pub mod portfolio_tokens {
    use super::*;

    /// TODO: better types for these?
    type BigDecimal = String;
    /// TODO: better types for these?
    type BigInt = String;

    #[derive(GraphQLQuery)]
    #[graphql(
        schema_path = "graphql/airstack-protocol-subgraph/portfolio_tokens/schema.json",
        query_path = "graphql/airstack-protocol-subgraph/portfolio_tokens/query.graphql",
        response_derives = "Debug"
    )]
    pub struct PortfolioTokens;

    /// prettier name for the PortfolioTokens derived query object
    pub type Query = PortfolioTokens;

    /// prettier name for portfolio_tokens::Variables.
    pub type Variables = portfolio_tokens::Variables;
}
