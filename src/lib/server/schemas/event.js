import { v4 } from "uuid";
import dynamoose from "./";
import {Item} from "dynamoose/dist/Item";

/** @enum {string} */
export const EType = {
	EVENT: 'event',
	BLOCK: 'block',
	REMINDER: 'reminder',
	TASK: 'task',
}

/** @enum {string} */
export const EStatus = {
  BACK: 'back', // backlog
  TODO: 'todo',
  DOING: 'doing',
  DONE: 'done',
};

/**
 * @typedef {Object} TEventSchema
 * @prop {string} eventId
 * @prop {string} originalText 			
 * @prop {string} title
 * @prop {EType} type
 * @prop {Date | undefined} date
 * @prop {string} [description]
 * @prop {Date | undefined} endDate
 * @prop {boolean} hasStartTime
 * @prop {boolean} hasEndTime
 * @prop {string[]} tag 
 * @prop {EStatus} status
 * @prop {number} importance
 * @prop {number} urgency
 * @prop {number} load
 */

export class TEvent extends Item { 
  /** @param {TEventSchema} arg */
  constructor({ eventId, originalText, title, type, date, description, endDate, hasStartTime, hasEndTime, tag, status, urgency, importance, load }) {
    // @ts-expect-error - Not used
		super();
    /** @type {TEventSchema['eventId']} */
		this.eventId = eventId;
		/** @type {TEventSchema['originalText']} */
		this.originalText = originalText;
		/** @type {TEventSchema['title']} */
		this.title = title;
		/** @type {TEventSchema['description']} */
		this.description = description;
		/** @type {TEventSchema['date']} */
		this.date = date;
		/** @type {TEventSchema['endDate']} */
		this.endDate = endDate;
    /** @type {TEventSchema['hasStartTime']} */
    this.hasStartTime = hasStartTime; 
    /** @type {TEventSchema['hasEndTime']} */
    this.hasEndTime = hasEndTime; 
		/** @type {TEventSchema['type']} */
		this.type = type;
		/** @type {TEventSchema['tag']} */
		this.tag = tag
		/** @type {TEventSchema['status']} */
		this.status = status
		/** @type {TEventSchema['importance']} */
		this.importance = importance
		/** @type {TEventSchema['urgency']} */
		this.urgency = urgency
		/** @type {TEventSchema['load']} */
		this.load = load
  }
}

const schema = new dynamoose.Schema({
  eventId: {
    type: String,
    required: true,
    hashKey: true,
    default: () => v4(),
  },
  originalText: { type: String, required: true },
  title: {
    type: String,
    required: true,
    index: {
      name: 'titleIndex',
      type: 'global',
    }
  },
  description: String,
  date: {
    type: Date,
    index: {
      name: 'dateIndex',
      type: 'global'
    }
  },
  endDate: {
    type: Date,
    index: {
      name: 'endDateIndex',
      type: 'global'
    }
  },
  // If the time in the date is the start time of the event
  hasStartTime: Boolean,
  // If the time in the endDate is the end time of the event
  hasEndTime: Boolean,
  type: {
    type: String,
    required: true,
    enum: Object.values(EType),
  },
  tag: {
    type: Array,
    schema: [String]
  },
  status: {
    type: String,
    enum: Object.values(EStatus),
  },
  importance: {
    type: Number,
    default: 0,
  },
  urgency: {
    type: Number,
    default: 0,
  },
  load: {
    type: Number,
    default: 0,
  },
}, {
  timestamps: true
});


/** @type {import('dynamoose/dist/Model').Model<TEvent>} */
const Event = dynamoose.model('Event', schema, { update: true });

export {
  Event,
  schema,
}