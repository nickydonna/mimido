import { REAL_DYNAMO } from '$env/static/private';
import { EStatus, EType } from '$lib/parser';
import dynamoose from 'dynamoose';
import { Item } from 'dynamoose/dist/Item';

// // Create new DynamoDB instance
if (REAL_DYNAMO === 'true') {
  const ddb = new dynamoose.aws.ddb.DynamoDB({
    "credentials": {
      "accessKeyId": "AKID",
      "secretAccessKey": "SECRET"
    },
    "region": "us-east-1"
  });
  // Set DynamoDB instance to the Dynamoose DDB instance
  dynamoose.aws.ddb.set(ddb);
} else {
  dynamoose.aws.ddb.local();
}


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
        ctag: { type: String },
        url: { type: String},
        syncToken: { type: String },
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
  }, { timestamps: true })));
  
export class CalendarObject extends Item {
  /** @type {string} */
  id = '';
  /** @type {string} */
  calendarUrl = ''; 
  /** @type {string} */
  user = '';
  /** @type {string} */
  url = '';
  /** @type {string | undefined} */
  etag = '';
  /** @type {Date | undefined} */
  date = undefined; // recommend to have this field for easy filtering/sorting
  /** @type {Date | undefined} */
  endDate = undefined; // recommend to have this field for easy filtering/sorting
  /** @type {string} */
  data = '';
  /** @type {'vtodo' | 'vevent'} */
  icalType = 'vtodo';
};

export const CalendarObjectModel =
  /** @type {import('dynamoose/dist/General').ModelType<CalendarObject>} */
  (dynamoose.model("CalendarObject", new dynamoose.Schema({
    id: { type: String, required: true, hashKey: true },
    user: { type: String, required: true, index: { type: 'local', name: 'user'} },
    url: { type: String, required: true },
    calendarUrl: { type: String, required: true, index: { type: 'local', name: 'calendarUrl'} },
    etag: { type: String },
    date: { type: Date },
    endDate: { type: Date },
    data: { type: String, required: true },
    icalType: { type: String, required: true, enum: ['vtodo', 'vevent'] },
  })))

/** @typedef {import('../calendar/alarmSchema').TAlarm} TAlarm */
    
// class Event extends Item {
  /** @type {string | undefined} */
//   eventId = undefined;
//   /** @type {string | undefined} */
//   title = undefined;
//   /** @type {Date | undefined} */
//   date = undefined;
//   /** @type {Date | undefined} */
//   endDate = undefined;
//   /** @type {string | undefined} */
//   description = undefined;
//   /** @type {string[]} */
//   tags = [];
//   /** @type {string | undefined} */
//   recur = undefined
//   /** @type {EStatus} */
//   status = EStatus.BACK;
//   /** @type {EType} */
//   type = EType.TASK;
//   /** @type {TAlarm[]} */
//   alarms = []
//   /** @type {number | undefined} */
//   importance = undefined
//   /** @type {number | undefined} */
//   load = undefined
//   /** @type {number | undefined} */
//   urgency = undefined
// }

// export const EventModel =
//   /** @type {import('dynamoose/dist/General').ModelType<Event>} */
//   (dynamoose.model("Event", new dynamoose.Schema({
//     eventId: { type: String, required: true, hashKey: true },
//     title: { type: String, required: true },
//     date: { type: Date },
//     endDate: { type: Date },
//     description: { type: String },
//     recur: { type: String },
//     tags: { type: Array, schema: String },
//     status: {type: String, enum: Object.values(EStatus) },
//     type: { type: String, enum: Object.values(EType) },
//     importance: { type: Number },
//     urgency: { type: Number },
//     load: { type: Number },
//     alarms: {
//       type: Array,
//       schema: [{
//         type: Object,
//         schema: {
//           isNegative: Boolean,
//           related: { type: String, enum: ['START'] },
//           duration: {
//             type: Object,
//             schema: {
//               years: Number,
//               months: Number,
//               weeks: Number,
//               days: Number,
//               hours: Number,
//               minutes: Number,
//               seconds: Number,
//             }
//           }
//         }
//       }]
//     }
//   })))