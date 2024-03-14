import { parseISO } from 'date-fns/fp';
import { InputItemBuilder } from '$lib/server/inputItemBuilder';
import BaseEntity from '$lib/server/db/base-entity';
import * as yup from 'yup';
import { v4 } from 'uuid';

const TABLE_NAME = 'Event';

/** @typedef {import('@aws-sdk/client-dynamodb').AttributeValue} AttributeValue */
/** @typedef {import('$lib/server/db/entity-manager').default} EntityManager */

export const eventSchema = yup.object({
	eventId: yup.string().uuid().required(),
	title: yup.string().required(),
	description: yup.string(),
	date: yup.date(),
	endDate: yup.date(),
	time: yup.string().matches(/[0-2][0-9]:[0-5][0-9]/),
	endTime: yup.string().matches(/[0-2][0-9]:[0-5][0-9]/)
});

/** @typedef {import('yup').InferType<typeof eventSchema>} TEventSchema */
export class EventEntity extends BaseEntity {
	

	/** @override */
	static sGetTableName() {
		return TABLE_NAME;
	}

	/**
	 * @override
	 * @returns {string}
	 */
	getTableName() {
		return TABLE_NAME;
	}

	/**
	 * @override
	 * @returns {import('@aws-sdk/client-dynamodb').CreateTableCommandInput}
	 */
	static getTableCreateInput() {
		return {
			TableName: this.sGetTableName(),
			// For more information about data types,
			// see https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/HowItWorks.NamingRulesDataTypes.html#HowItWorks.DataTypes and
			// https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Programming.LowLevelAPI.html#Programming.LowLevelAPI.DataTypeDescriptors
			AttributeDefinitions: [
				{
					AttributeName: 'EventId',
					AttributeType: 'S'
				}
			],
			KeySchema: [
				{
					AttributeName: 'EventId',
					KeyType: 'HASH'
				}
			],
			ProvisionedThroughput: {
				ReadCapacityUnits: 1,
				WriteCapacityUnits: 1
			}
		};
	}

	/**
	 *
	 * @param {EntityManager} manager
	 * @param {string} eventId
	 * @returns {Promise<*>}
	 */
	static async findById(manager, eventId) {
		return manager.findById(EventEntity, { 'EventId': { S: eventId } });
	}

	/**
	 *
	 * @param {Record<string, AttributeValue>} item
	 * @returns {EventEntity}
	 */
	static fromDBItem(item) {
		return new EventEntity(
			/** @type {string} */ (item['EventId'].S),
			/** @type {string} */ (item['Title'].S),
			item['Date'].S ? parseISO(item['Date'].S) : undefined,
			item['Description'].S,
			item['EndDate'].S ? parseISO(item['EndDate'].S) : undefined,
			item['Time'].S,
			item['EndTime'].S
		);
	}

	/**
	 * @override
	 * @param {FormData} data
	 * @returns {Promise<EventEntity>}
	 */
	static async newFromForm(data) {
		const e = await eventSchema.validate({
			eventId: v4(),
			title: data.get('title'),
			description: data.get('description'),
			date: data.get('date'),
			endDate: data.get('endDate'),
			time: data.get('time'),
			endTime: data.get('endTime')
		});

		return new EventEntity(e.eventId, e.title, e.date, e.description, e.endDate, e.time, e.endTime);
	}

	/**
	 * @param {string} eventId
	 * @param {string} title
	 * @param {Date | undefined} date
	 * @param {string | undefined} description
	 * @param {Date | undefined} endDate
	 * @param {string | undefined} time
	 * @param {string | undefined} endTime
	 */
	constructor(eventId, title, date, description, endDate, time, endTime) {
		super();

		/** @type {string} */
	this.eventId = eventId;

	/** @type {string} */
	this.title = title;

	/** @type {string | undefined} */
	this.description = description;

	/** @type {Date | undefined} */
	this.date = date;

	/** @type {Date | undefined} */
	this.endDate = endDate;

	/**
	 * Time encoded as a number so 16:21
	 * @type {string | undefined}
	 */
	this.time = time;

	/**
	 * Time encoded as a number so 16:21
	 * @type {string | undefined}
	 */
	this.endTime = endTime;
	}

	/**
	 * @override
	 */
	toDbItem() {
		return InputItemBuilder.create()
			.addS('EventId', this.eventId)
			.addS('Title', this.title)
			.addS('Description', this.description)
			.addSDate('Date', this.date)
			.addSDate('EndDate', this.endDate)
			.addS('Time', this.time)
			.addS('EndTime', this.endTime)
			.build();
	}

	/**
	 * @override
	 * @returns {TEventSchema}
	 */
	toPOJO() {
		return /** @type {TEventSchema} */ (Object.fromEntries(Object.entries(this)));
	}
}