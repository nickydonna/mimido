import { env } from '$env/dynamic/private';
import dynamoose from 'dynamoose';
import type { Item } from 'dynamoose/dist/Item';
import type { ExtendCalendarAccess, UserCalendar } from '../../../app';

// // Create new DynamoDB instance
if (env.REAL_DYNAMO === 'true') {
	const ddb = new dynamoose.aws.ddb.DynamoDB({
		region: 'us-east-1'
	});
	// Set DynamoDB instance to the Dynamoose DDB instance
	dynamoose.aws.ddb.set(ddb);
} else {
	dynamoose.aws.ddb.local();
}

export interface User extends Item {
	username: string;
	auth: 'basic';
	main: UserCalendar;
	calendars: ExtendCalendarAccess[];
}

export const UserModel = dynamoose.model<User>(
	'User',
	new dynamoose.Schema(
		{
			username: { type: String, required: true, hashKey: true },
			auth: {
				required: true,
				type: String,
				enum: ['basic']
			},
			main: {
				type: Object,
				schema: {
					calendar: { type: String, required: true },
					server: { type: String, required: true },
					password: { type: String, required: true },
					email: { type: String, required: true },
					ctag: { type: String },
					url: { type: String },
					syncToken: { type: String }
				}
			},
			calendars: {
				type: Array,
				schema: [
					{
						type: Object,
						schema: {
							provider: { type: String, enum: ['parent'], required: true },
							type: { type: String, enum: ['extend'], required: true },
							name: { type: String, required: true },
							ctag: { type: String },
							syncToken: { type: String },
							url: { type: String }
						}
					}
				],
				required: true,
				default: []
			}
		},
		{ timestamps: true }
	),
	{ create: false, update: true, throughput: 'ON_DEMAND' }
);

export interface CalendarObject extends Item {
	id: string;
	calendarUrl: string;
	user: string;
	url: string;
	etag: string | undefined;
	date: Date | undefined;
	endDate: Date | undefined;
	data: string;
	icalType: 'vtodo' | 'vevent';
	recur: string | undefined;
	postponed: number;
}

export const CalendarObjectModel = dynamoose.model<CalendarObject>(
	'CalendarObject',
	new dynamoose.Schema(
		{
			id: { type: String, required: true, hashKey: true },
			user: { type: String, required: true, index: { type: 'global', name: 'user' } },
			url: { type: String, required: true },
			calendarUrl: { type: String, required: true, index: { type: 'global', name: 'calendarUrl' } },
			etag: { type: String },
			date: { type: Date },
			endDate: { type: Date },
			data: { type: String, required: true },
			recur: { type: String },
			icalType: { type: String, required: true, enum: ['vtodo', 'vevent'] },
			postponed: { type: Number, default: 0 }
		},
		{ timestamps: true }
	),
	{ create: false, update: true, throughput: 'ON_DEMAND' }
);
