// import { gql, GraphQLClient } from "graphql-request";
const { gql, GraphQLClient } = require('graphql-request');
import { config } from "dotenv";

config();

const graphQLClient = new GraphQLClient(
  "https://claims.airstack.xyz/moxie"
);

const query = gql`
query FarcasterUserClaimTransactionDetails($fid: Int!) {
  FarcasterUserClaimTransactionDetails(input: { fid: $fid }) {
    fid
    availableClaimAmount
    minimumClaimableAmountInWei
    availableClaimAmountInWei
    claimedAmount
    claimedAmountInWei
    processingAmount
    processingAmountInWei
  }
}
`;

const variable = {
  fid: process.env.FARCASTER_ID as string,
};

const headers = {
  "x-airstack-claims": process.env.AIRSTACK_API_KEY as string,
};

(async () => {
  try {
    const data = await graphQLClient.request(query, variable, headers);
    console.log(data);
  } catch (e) {
    throw new Error(String(e));
  }
})();
