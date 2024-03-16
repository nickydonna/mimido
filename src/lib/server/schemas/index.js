import dynamoose from 'dynamoose';

const appEnv = process.env.APP_ENV ?? 'dev'; 

// On prod it uses IAM profile
if (appEnv === 'dev') {
  dynamoose.aws.ddb.local();
}

export default dynamoose;