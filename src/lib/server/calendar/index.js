import { DAVClient, DAVNamespaceShort } from "tsdav";
import { createHash } from 'node:crypto';
import ICAL from 'ical.js'
import { v4 } from "uuid";
import { add, addMinutes, formatISO, startOfDay } from "date-fns/fp";
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
 * @prop {Date} [date]
 * @prop {string} [description]
 * @prop {Date} [endDate]
 * @prop {string[]} tag 
 * @prop {EStatus} status
 * @prop {number} importance
 * @prop {number} urgency
 * @prop {number} load
 * @prop {TAlarm} [alarm]
 * @prop {string} [recur]
 */

/** 
 * @typedef {Omit<TEventSchema, 'eventId'>} ParsedEventSchema
 * Result of the parsed event from the parsing, without id
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
        username: user.email,
        password: user.password,
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

  /**
   * Check the connection to the server and if the calendar exists
   */
  async check() {
    await this.logged;
    await this.getCalendar();
  }

  /**
   * @param {Omit<TEventSchema, 'eventId'>} eventData
   */
  async createEvent(eventData) {
    const calendar = await this.getCalendar();

    var { id, component } = this.toComponent(eventData);

    const result = await this.client.createCalendarObject({
      calendar: calendar,
      filename: `${id}.ics`,
      iCalString: component.toString(),
    });
    console.log(result);
    return result;
  }

  async listTodos() {
    const calendar = await this.getCalendar();
    const objects = await this.client.fetchCalendarObjects({
      calendar: calendar,
      filters: {
        'comp-filter': {
          _attributes: {
            name: 'VCALENDAR',
          },
          'comp-filter': {
            _attributes: {
              name: 'VTODO',
            },
          }
        }
      }
    });

    return objects.map(e => {
      const comp = ICAL.Component.fromString(e.data);
      const vtodo = comp.getFirstSubcomponent('vtodo');
      if (vtodo) {
        return this.fromVTodo(vtodo);
      }
    }).filter(
      /**
       * @param {TEventSchema | undefined} e 
       * @returns {e is TEventSchema}
       */
      e => typeof e !== 'undefined'
    )
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
      const comp = ICAL.Component.fromString(e.data);
      const vevent = comp.getFirstSubcomponent('vevent');
      if (vevent) {
        return this.fromVEvent(vevent);
      }
    }).filter(
      /**
       * @param {TEventSchema | undefined} e 
       * @returns {e is TEventSchema}
       */
      e => typeof e !== 'undefined'
    )
  }

  /** @param {string} id */
  async getEvent(id) {
    const [object] = await this.client.calendarMultiGet({
      url: await this.getCalendarUrl(),
      objectUrls: [await this.getEventUrl(id)],
      depth: '1',
      props: {
        [`${DAVNamespaceShort.DAV}:getetag`]: {},
        [`${DAVNamespaceShort.CALDAV}:calendar-data`]: {},
      },
    });
    if (!object?.ok || !object?.props) {
      return;
    }
    const comp = ICAL.Component.fromString(object?.props.calendarData._cdata);
    const vevent = comp.getFirstSubcomponent('vevent');
    if (vevent) return this.fromVEvent(vevent);
  }

  /**
   * @param {string} id
   * @param {ParsedEventSchema} eventData
   */
  async editEvent(id, eventData) {
    const event = await this.getEvent(id);
    if (!event) throw new Error('Event does not exists');

    const { component } = this.toComponent(eventData, id)

    return this.client.updateCalendarObject({
      calendarObject: {
        url: await this.getEventUrl(id),
        data: component.toString()
      }
    })
  }

  /**
   * @private
   * @param {string | TEventSchema} eventOrId 
   * @returns {Promise<string>} 
   */
  async getEventUrl(eventOrId) {
    const calUrl = await this.getCalendarUrl();
    return `${calUrl}/${typeof eventOrId === 'string' ? eventOrId : eventOrId.eventId}.ics`

  }

  /** @private */
  async getCalendarUrl() {
    return (await this.getCalendar()).url
  }
 /**
   * @param {ICAL.Component} vtodo - vtodo component from calendar
   * @return {TEventSchema}
   */
  fromVTodo(vtodo) {
    let eventId = vtodo.getFirstPropertyValue('uid');
    let title = vtodo.getFirstPropertyValue('summary');
    let urgency = parseInt(vtodo.getFirstPropertyValue(CustomPropName.URGENCY), 10)
    urgency = Number.isFinite(urgency) ? urgency : 0;
    let load = parseInt(vtodo.getFirstPropertyValue(CustomPropName.LOAD), 10)
    load = Number.isFinite(load) ? load : 0;
    let importance = parseInt(vtodo.getFirstPropertyValue(CustomPropName.IMPORTANCE), 10)
    importance = Number.isFinite(importance) ? importance : 0;

    const tagProp = vtodo.getFirstPropertyValue(CustomPropName.TAG)?.trim() ?? '';
    const tag = tagProp.length > 0 ? tagProp.split(',') : []

    return {
      eventId,
      title,
      type: vtodo.getFirstPropertyValue(CustomPropName.TYPE) ?? EStatus.TODO,
      tag, 
      status: vtodo.getFirstPropertyValue(CustomPropName.STATUS) ?? EStatus.TODO,
      originalText: vtodo.getFirstPropertyValue(CustomPropName.ORIGINAL_TEXT) ?? EStatus.TODO,
      urgency,
      importance,
      load,
    }
  }

  /**
   * @param {ICAL.Component} vevent - vevent component from calendar
   * @return {TEventSchema}
   */
  fromVEvent(vevent) {
    const valarm = vevent.getFirstSubcomponent('valarm');
    const icalEvent = new ICAL.Event(vevent);
    const date = /** @type {Date | undefined} */ (icalEvent.startDate?.toJSDate());
    /** @type {Date | undefined} */
    let endDate;
    if (icalEvent.endDate) {
      endDate = /** @type {Date | undefined} */ (icalEvent.endDate?.toJSDate());
    } else if (date && icalEvent.duration) {
      endDate = add(icalEvent.duration, date);
    } else if (date) {
      endDate = addMinutes(30, date);
    }

    let rrule = vevent.getFirstPropertyValue('rrule');
    /** @type {TEventSchema['recur']} */
    let recur;

    if (rrule) {
      const icalRecur = new ICAL.Recur(rrule)
      console.log(icalRecur.toString());
      recur = icalRecur.toString()
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
    const tagProp = vevent.getFirstPropertyValue(CustomPropName.TAG)?.trim() ?? '';
    const tag = tagProp.length > 0 ? tagProp.split(',') : []

    return {
      eventId: /** @type {string} */ (icalEvent.uid),
      title: /** @type {string} */ (icalEvent.summary),
      date,
      endDate,
      type: vevent.getFirstPropertyValue(CustomPropName.TYPE) ?? EStatus.TODO,
      tag, 
      status: vevent.getFirstPropertyValue(CustomPropName.STATUS) ?? EStatus.TODO,
      originalText: vevent.getFirstPropertyValue(CustomPropName.ORIGINAL_TEXT) ?? EStatus.TODO,
      urgency,
      importance,
      load,
      alarm,
      recur,
    }
  }  

  /**
   * Transform an TEventSchema into a {@link ICAL.Component} for sending
   * If {@link TEventSchema#eventId} is not defined, one will be created
   * @param {ParsedEventSchema} eventData
   * @param {string} [eventId]
   * @returns {{ id: string, component: ICAL.Component, isTodo: boolean }}
   */
  toComponent(eventData, eventId) {
    var component = new ICAL.Component(['vcalendar', [], []]);
    component.updatePropertyWithValue('prodid', '-//CyrusIMAP.org/Cyrus');

    let vcomponent
    let id = eventId ?? v4();
    let isTodo = false;

    if (eventData.date) {
      vcomponent = new ICAL.Component('vevent');
      const event = new ICAL.Event(vcomponent);
      // Set standard properties
      event.summary = eventData.title;
      event.uid = id;
      if (eventData.description) {
        event.description = eventData.description;
      }
      event.startDate = ICAL.Time.fromJSDate(eventData.date, true);
      if (eventData.endDate) {
        event.endDate = ICAL.Time.fromJSDate(eventData.endDate, true);
      } else {
        event.duration = new ICAL.Duration({ minutes: 15 });
      }

      if (eventData.recur) {
        const icalRecur = ICAL.Recur.fromString(eventData.recur.replace('RRULE:', ''))
        vcomponent.addPropertyWithValue('rrule', icalRecur);
      }
    } else if (!eventId) {
      console.log('creating todo')
      isTodo = true;
      vcomponent = new ICAL.Component('vtodo');
      vcomponent.addPropertyWithValue('uid', id);
      vcomponent.addPropertyWithValue('summary', eventData.title);
      if (eventData.description) {
        vcomponent.addPropertyWithValue('description', eventData.description)
      }
    } else {
      throw new Error('Edit TODO not supported yet')
    }

    vcomponent.addPropertyWithValue(CustomPropName.TYPE, eventData.type ?? EType.EVENT);
    if (eventData.tag.length > 0) {
      vcomponent.addPropertyWithValue(CustomPropName.TAG, eventData.tag.join(','));
    }
    vcomponent.addPropertyWithValue(CustomPropName.URGENCY, eventData.urgency ?? 0);
    vcomponent.addPropertyWithValue(CustomPropName.LOAD, eventData.load ?? 0);
    vcomponent.addPropertyWithValue(CustomPropName.IMPORTANCE, eventData.importance ?? 0);
    vcomponent.addPropertyWithValue(CustomPropName.ORIGINAL_TEXT, eventData.originalText);
    vcomponent.addPropertyWithValue(CustomPropName.STATUS, eventData.status ?? EStatus.TODO);

    // Add the new component
    component.addSubcomponent(vcomponent);

    return { id, component, isTodo };
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
    await back.check();
    backends[key] = back;
  }
  return backends[key]

}
