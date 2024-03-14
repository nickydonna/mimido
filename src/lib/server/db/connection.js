import { DynamoDBClient } from '@aws-sdk/client-dynamodb';

export const client = new DynamoDBClient({ endpoint: 'http://localhost:8000', region: 'us-east-1' });
