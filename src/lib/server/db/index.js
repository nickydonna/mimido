import dynamoose from 'dynamoose';
import { Item } from 'dynamoose/dist/Item';

// // Create new DynamoDB instance
// const ddb = new dynamoose.aws.ddb.DynamoDB({
//     "credentials": {
//         "accessKeyId": "AKID",
//         "secretAccessKey": "SECRET"
//     },
//     "region": "us-east-1"
// });

// // Set DynamoDB instance to the Dynamoose DDB instance
// dynamoose.aws.ddb.set(ddb);
dynamoose.aws.ddb.local();


export class User extends Item {
  /** @type {string} */
  username = ''; 
  /** @type {'basic'} */
  auth = 'basic'
  /** @type {import('../../../app').UserCalendar} */
  main = { email: '', password: '', server: '', calendar: ''}
  /** @type {Array<import('../../../app').ExtendCalendarAccess>} */
  calendars = [];
}

export const UserModel =
  /** @type {import('dynamoose/dist/General').ModelType<User>} */
  (dynamoose.model("User", new dynamoose.Schema({
    "username": { type: String, required: true, hashKey: true },
    "auth": {
      required: true,
      "type": String,
      "enum": ['basic']
    },
    main: {
      type: Object,
      schema: {
        calendar: { type: String, required: true },
        server: { type: String, required: true },
        password: { type: String, required: true },
        email: { type: String, required: true },
      }
    },
    calendars: {
      type: Array,
      schema: [{
        type: Object,
        schema: {
          provider: { type: String, enum: ['parent'], required: true },
          type: { type: String, enum: ['extend'], required: true },
          name: { type: String, required: true },
        }
      }],
      required: true,
      default: [],
    }
  })));