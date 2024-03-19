import { DAVClient } from "tsdav";
import { createHash } from 'node:crypto';
// @ts-ignore
import ICAL from 'ical.js'
import { v4 } from "uuid";
import { add, formatISO, startOfDay } from "date-fns/fp";
import { endOfDay } from "date-fns";

/** @typedef {import('tsdav').DAVCalendar} DAVCalendar */

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
 * @typedef {Object} TAlarm
 * @prop {import("date-fns").Duration} duration - The time before the related date
 * @prop {'START'} related - The date to use for the duraction, for now just related to start date
 * @prop {boolean} [isNegative=true] - if the duration is before or after related 
 */

/**
 * @typedef {Object} TEventSchema
 * @prop {string} eventId
 * @prop {string} originalText 			
 * @prop {string} title
 * @prop {EType} type
 * @prop {Date | undefined} date
 * @prop {string} [description]
 * @prop {Date | undefined} endDate
 * @prop {string[]} tag 
 * @prop {EStatus} status
 * @prop {number} importance
 * @prop {number} urgency
 * @prop {number} load
 * @prop {TAlarm} [alarm]
 */

export class Backend {
  /**
   * @param { App.Locals['user']} user - User data to auth to the server
   */
  constructor(user) {
    /** @private */
    this.user = user;
    /** @private */
    this.client = new DAVClient({
      serverUrl: user.server,
      credentials: {
        username: user.email, // 'nickydonna@fastmail.com',
        password: user.password, // 'kra9gqhecya42pkw',
      },
      authMethod: 'Basic',
      defaultAccountType: 'caldav',
    });
    /** 
     * @private
     * @type {Promise<void>}
     * Used to only log to servers once 
     */
    this.logged = this.client.login();
    /**
     * @private
     * @type {DAVCalendar | undefined}
     */
    this.calendar = undefined;
  }

  async getCalendar() {
    if (this.calendar) return this.calendar
    await this.logged;
    const calendars = await this.client.fetchCalendars();

    this.calendar = calendars.find(c => c.displayName === this.user.calendar)
    if (!this.calendar) throw new Error(`No Calendar with name ${this.user.calendar}`);
    return this.calendar;
  }

  async init() {
    await this.logged;
  }

  async test() {
    await this.init();
    await this.getCalendar();
  }

  /**
   * @param {Omit<TEventSchema, 'eventId'>} eventData
   */
  async createEvent(eventData) {
    const calendar = await this.getCalendar();

    var comp = new ICAL.Component(['vcalendar', [], []]);
    comp.updatePropertyWithValue('prodid', '-//CyrusIMAP.org/Cyrus');

    const vevent = new ICAL.Component('vevent');
    const event = new ICAL.Event(vevent);

    // Set standard properties
    event.summary = eventData.title;
    event.uid = v4();
    event.startDate = ICAL.Time.fromJSDate(eventData.date);
    if (eventData.endDate) {
      event.endDate = ICAL.Time.fromJSDate(eventData.endDate);
    } else {
      event.duration = new ICAL.Duration({ minutes: 15 })
    }

    // Set custom property
    vevent.addPropertyWithValue(CustomPropName.TYPE, eventData.type);
    vevent.addPropertyWithValue(CustomPropName.TAG, eventData.tag.join(','));
    vevent.addPropertyWithValue(CustomPropName.URGENCY, eventData.urgency);
    vevent.addPropertyWithValue(CustomPropName.LOAD, eventData.load);
    vevent.addPropertyWithValue(CustomPropName.IMPORTANCE, eventData.importance);
    vevent.addPropertyWithValue(CustomPropName.ORIGINAL_TEXT, eventData.originalText);
    vevent.addPropertyWithValue(CustomPropName.STATUS, eventData.status);

    // Add the new component
    comp.addSubcomponent(vevent);

    return await this.client.createCalendarObject({
      calendar: calendar,
      filename: `${event.uid}.ics`,
      iCalString: comp.toString(),
    });

  }

  /**
   * 
   * @param {Date} [day] 
   */
  async listDayEvent(day) {
    const from = startOfDay(day ?? (new Date()))
    const to = endOfDay(from)
    return this.listEvents(from, to);
  }

  /**
   * @param {Date} from
   * @param {Date} to
   */
  async listEvents(from, to) {
    const calendar = await this.getCalendar();
    const objects = await this.client.fetchCalendarObjects({
      calendar: calendar,
      timeRange: {
        start: formatISO(from),
        end: formatISO(to), 
      }
    });

    return objects.map(e => {
      const comp = ICAL.Component.fromString(e.data)
      const vevent = comp.getFirstSubcomponent('vevent');
      const valarm = vevent.getFirstSubcomponent('valarm');
      return this.fromComponent(vevent, valarm); 
    })
  }

  /**
   * @param {Record<string, any>} vevent - vevent component from calendar
   * @param {Record<string, any>} [valarm] - valarm component from calendar
   * @return {TEventSchema}
   */
  fromComponent(vevent, valarm) {
    const icalEvent = new ICAL.Event(vevent);
    const date = /** @type {Date | undefined} */ (icalEvent.startDate?.toJSDate());
    /** @type {Date | undefined} */
    let endDate;
    if (icalEvent.endDate) {
      endDate = /** @type {Date | undefined} */ (icalEvent.endDate?.toJSDate());
    } else if (date && icalEvent.duration) {
      endDate = add(icalEvent.duration, date);
    }

    let urgency = parseInt(vevent.getFirstPropertyValue(CustomPropName.URGENCY), 10)
    urgency = Number.isFinite(urgency) ? urgency : 0;
    let load = parseInt(vevent.getFirstPropertyValue(CustomPropName.LOAD), 10)
    load = Number.isFinite(load) ? load : 0;
    let importance = parseInt(vevent.getFirstPropertyValue(CustomPropName.IMPORTANCE), 10)
    importance = Number.isFinite(importance) ? importance : 0;

    /** @type {TAlarm | undefined} */
    let alarm;

    if (valarm && valarm.getFirstPropertyValue('action') === 'DISPLAY') {
      // The ICAL duraction is not good for formating
      const dur = valarm.getFirstPropertyValue('trigger')      
      alarm = {
        related: 'START', // TODO check actual related
        duration: {
          hours: Math.abs(dur.hours),
          minutes: Math.abs(dur.minutes),
          days: Math.abs(dur.days),
          weeks: Math.abs(dur.weeks),
        },
        isNegative: dur?.isNegative,
      }
    }

    return {
      eventId: /** @type {string} */ (icalEvent.uid),
      title: /** @type {string} */ (icalEvent.summary),
      date,
      endDate,
      type: vevent.getFirstPropertyValue(CustomPropName.TYPE) ?? EStatus.TODO,
      tag: vevent.getFirstPropertyValue(CustomPropName.TAG)?.split(',') ?? [],
      status: vevent.getFirstPropertyValue(CustomPropName.STATUS) ?? EStatus.TODO,
      originalText: vevent.getFirstPropertyValue(CustomPropName.ORIGINAL_TEXT) ?? EStatus.TODO,
      urgency,
      importance,
      load,
      alarm,
    }
  }  

}

const CustomPropName = {
  TYPE: 'x-type',
  TAG: 'x-tag',
  URGENCY: 'x-urgency',
  LOAD: 'x-load',
  IMPORTANCE: 'x-importance',
  ORIGINAL_TEXT: 'x-original-text',
  STATUS: 'x-status',
}

/** @type {Record<string, Backend>} */
const backends = {};

/** @param {App.Locals['user']} user */
function hashUser(user) {
  const hash = createHash('sha256');
  hash.update(user.email);
  hash.update(user.calendar);
  hash.update(user.server);
  return hash.digest('hex');

}

/** @param {App.Locals['user']} user */
export async function getBackend(user) {
  const key = hashUser(user);

  if (!backends[key]) {
    const back = new Backend(user);
    await back.init();
    backends[key] = back;
  }
  return backends[key]

}



// const logging = client.login();

// export async function fetchCalendars() {
//   await logging;
//   return client.fetchCalendars();
// }

// export async function fetchCalendar() {
//   const calendars = await fetchCalendars();

//   // Replace with user info
//   return calendars.find(c => c.displayName === 'mimido')
// }

// export async function listEvents() {
//   const calendar = await fetchCalendar();
//   if (!calendar) return;
//   const objects = await client.fetchCalendarObjects({
//     calendar: calendar,
//     timeRange: {
//       start: formatISO(startOfDay(new Date())),
//       end: formatISO(endOfDay(new Date())),
//     }
//   });

//   const e = objects.map(e => ICAL.Component.fromString(e.data))[0]
//   var vevent = e.getFirstSubcomponent('vevent');
//   var event = new ICAL.Event(vevent); 
//   const o = event.getOccurrenceDetails(ICAL.Time.fromJSDate(startOfDay(new Date())))
//   console.log(event.isRecurrenceException(), e.startDate, e.exceptions)

// }

// export async function createEvent() {
//   const calendar = await fetchCalendar();
//   if (!calendar) return;

//   var comp = new ICAL.Component(['vcalendar', [], []]);
//   comp.updatePropertyWithValue('prodid', '-//CyrusIMAP.org/Cyrus');

//   const vevent = new ICAL.Component('vevent');
//   const event = new ICAL.Event(vevent);
//   const recur = new ICAL.Recur({
//     freq: 'WEEKLY',
//     byday: ['MO', 'TH', 'FR']
//   });

//   // Set standard properties
//   event.summary = 'Hells';
//   event.uid = v4();
//   event.startDate = ICAL.Time.fromJSDate(addHours(2, new Date()))
//   event.duration = new ICAL.Duration({ hours: 2 })

//   // Set custom property
//   vevent.addPropertyWithValue('x-my-custom-property', 'custom');
//   vevent.addPropertyWithValue('RRULE', recur.toString())

//   // Add the new component
//   comp.addSubcomponent(vevent);

//   // const result = await client.createCalendarObject({
//   //   calendar: calendar,
//   //   filename: `${event.uid}.ics`,
//   //   iCalString: comp.toString(),
//   // });

//   console.log(result);
// }