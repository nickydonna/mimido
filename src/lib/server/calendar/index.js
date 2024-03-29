import { DAVClient, DAVNamespaceShort } from "tsdav";
import { createHash } from 'node:crypto';
import ICAL from 'ical.js'
import { v4 } from "uuid";
import { add, addMinutes, formatISO, startOfDay } from "date-fns/fp";
import { endOfDay } from "date-fns";
import yup from 'yup';

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
 * @typedef {Object} TEventMeta
 * @prop {'vtodo' | 'vevent'} icalType
 * @prop {Date} [recurrenceId]
 */

/** 
 * @typedef {Object} TAlarm
 * @prop {import("date-fns").Duration} duration - The time before the related date
 * @prop {string} related - The date to use for the duraction, for now just related to start date
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
 * @prop {Array<TAlarm>} alarms
 * @prop {string} [recur]
 * @prop {TEventMeta} meta
 */

// eslint-disable-next-line no-useless-escape
const typeRE = new RegExp(Object.values(EType).join('|'));
// eslint-disable-next-line no-useless-escape
const statusRE = new RegExp(Object.values(EStatus).join('|'));

/**
 * Base schema with optional dates
 * Right now only tasks can have blank dates
 */
const baseSchema = yup.object({
  originalText: yup.string().required(),
  title: yup.string().required().min(3),
  type: yup.string().required().matches(typeRE, { excludeEmptyString: true }),
  status: yup.string().required().matches(statusRE, { excludeEmptyString: true }),
  date: yup.date(),
  endDate: yup.date(),
  description: yup.string(),
  tag: yup.array().of(yup.string().required()).required(),
  importance: yup.number().min(-3).max(3).required(),
  urgency: yup.number().min(0).max(3).required(),
  load: yup.number().min(0).max(3).required(),
  recur: yup.string(), // TODO Add RRULE validation - .test('rrule', 'Recur is not a valid RRULE', (v => RRule.fromString(v).))
  alarm: yup.object({
    related: yup.string().matches(/START/).required(),
    isNegative: yup.boolean().required(),
    duration: yup.object({
      years: yup.number(),
      months: yup.number(),
      weeks: yup.number(),
      days: yup.number(),
      hours: yup.number(),
      minutes: yup.number(),
      seconds: yup.number(),
    })
  }).default(undefined)
});

/**
 * Block and event need start and end date
 */
const blockOrEventSchema = baseSchema.shape({
  date: yup.date().required(),
  endDate: yup.date().required(),
});

/**
 * Reminders must have an start date
 */
const reminderSchema = baseSchema.shape({
  date: yup.date().required(),
})



/** 
 * @typedef {Omit<TEventSchema, 'eventId' | 'meta'> & { meta?: TEventMeta }} ParsedEventSchema
 * Result of the parsed event from the parsing, without id and optional meta
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

    // TODO Review intervalcheck
    // const intervalCheck =
    //   /** @param {TEventSchema} e  */
    //   (e) => e.date && isWithinInterval({ start: from, end: to }, e.date)

    // Calendar components can have many event components
    // Map all to a TEventSchema, filter them for in range and flat()
    return objects.map(e => {
      const comp = ICAL.Component.fromString(e.data);
      const vevents = comp.getAllSubcomponents('vevent');
      if (vevents.length === 0) return;
      const parsed = vevents.map(e => this.fromVEvent(e))
      return parsed.find(p => !!p.meta?.recurrenceId) ?? parsed[parsed.length - 1];
    })
      .filter(
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
    if (!object?.ok || !object?.props || object.status % 200 > 100) {
      throw new Error('Event not found: ' + id);
    }

    const comp = ICAL.Component.fromString(object?.props.calendarData._cdata);
    /** @type {TEventSchema | undefined} */
    let event; 
    /** @type {TEventMeta} */
    let meta;
    if (id.startsWith('vevent-')) {
      meta = { icalType: 'vevent' }
      const vevent = comp.getFirstSubcomponent('vevent');
      if (vevent) (event = this.fromVEvent(vevent));
    } else if (id.startsWith('vtodo-')) {
      meta = { icalType: 'vtodo' }
      const vtodo = comp.getFirstSubcomponent('vtodo');
      if (vtodo) return this.fromVTodo(vtodo);
    } else {
      throw new Error('Event with old id: ' + id)
    }

    if (!event) throw new Error("Can't parse event: :" + id);
    return {...event, meta }

  }

  /**
   * 
   * @param {ParsedEventSchema} data 
   */
  async validateEventData(data) {
    switch (data.type) {
      case EType.BLOCK:
      case EType.EVENT:
        return blockOrEventSchema.validate(data)
      case EType.REMINDER:
        return reminderSchema.validate(data)
      default:
        return baseSchema.validate(data)  
    }
  }

  /**
   * @param {string} id
   * @param {ParsedEventSchema} eventData
   */
  async editEvent(id, eventData) {
    const event = await this.getEvent(id);
    if (!event) throw new Error('Event does not exists');

    const { component, meta, id: newId } = this.toComponent(eventData, id);

    // If event changed type, destroy and recreate
    if (event.meta?.icalType != meta.icalType) {
      await this.deleteEvent(id)
      const result = await this.client.createCalendarObject({
        calendar: await this.getCalendar(),
        filename: `${newId}.ics`,
        iCalString: component.toString(),
      });
      return result;
    }

    const result = await this.client.updateCalendarObject({
      calendarObject: {
        url: await this.getEventUrl(id),
        data: component.toString()
      }
    })
    return result
  }

  /**
   * @param {string} eventId
   * @param {EStatus} status
   */
  async updateStatus(eventId, status) {
    const event = await this.getEvent(eventId);
    event.status = status;
    return this.editEvent(eventId, event);
  }

  /**
   * @param {string} id
   */
  async deleteEvent(id) {
    const event = await this.getEvent(id);
    if (!event) throw new Error('Event does not exists');

    await this.client.deleteCalendarObject({
      calendarObject: {
        url: await this.getEventUrl(id),
      }
    })
    return event
  }

  /**
   * @private
   * @param {string | TEventSchema} eventOrId 
   * @returns {Promise<string>} 
   */
  async getEventUrl(eventOrId) {
    const calUrl = await this.getCalendarUrl();
    return `${calUrl}${typeof eventOrId === 'string' ? eventOrId : eventOrId.eventId}.ics`

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
      alarms: [],
      title,
      type: vtodo.getFirstPropertyValue(CustomPropName.TYPE) ?? EStatus.TODO,
      tag, 
      status: vtodo.getFirstPropertyValue(CustomPropName.STATUS) ?? EStatus.TODO,
      originalText: vtodo.getFirstPropertyValue(CustomPropName.ORIGINAL_TEXT) ?? EStatus.TODO,
      urgency,
      importance,
      load,
      meta: { icalType: 'vtodo' }
    }
  }

  /**
   * @param {ICAL.Component} vevent - vevent component from calendar
   * @return {TEventSchema}
   */
  fromVEvent(vevent) {
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
      recur = icalRecur.toString()
    }

    let urgency = parseInt(vevent.getFirstPropertyValue(CustomPropName.URGENCY), 10)
    urgency = Number.isFinite(urgency) ? urgency : 0;
    let load = parseInt(vevent.getFirstPropertyValue(CustomPropName.LOAD), 10)
    load = Number.isFinite(load) ? load : 0;
    let importance = parseInt(vevent.getFirstPropertyValue(CustomPropName.IMPORTANCE), 10)
    importance = Number.isFinite(importance) ? importance : 0;

    /** @type {Array<TAlarm>} */
    let alarms = [];

    const valarms = vevent.getAllSubcomponents('valarm');
    if (valarms.length !== 0) {
      alarms = valarms
        // Only support display, no email
        .filter(comp => comp.getFirstPropertyValue('action') === 'DISPLAY')
        .map(comp => {
          // The ICAL duraction is not good for formating
          const dur = comp.getFirstPropertyValue('trigger') 
          return {
            related: 'START', // TODO check actual related
            duration: {
              hours: Math.abs(dur.hours),
              minutes: Math.abs(dur.minutes),
              days: Math.abs(dur.days),
              weeks: Math.abs(dur.weeks),
            },
            isNegative: dur?.isNegative,
          }
        })
      
    }
    const tagProp = vevent.getFirstPropertyValue(CustomPropName.TAG)?.trim() ?? '';
    const tag = tagProp.length > 0 ? tagProp.split(',') : []

    /** @type {TEventMeta} */
    const meta = {
      icalType: 'vevent',
      recurrenceId: icalEvent.recurrenceId?.toJSDate(),
    }

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
      alarms,
      recur,
      meta,
    }
  }  

  /**
   * Transform an TEventSchema into a {@link ICAL.Component} for sending
   * If {@link TEventSchema#eventId} is not defined, one will be created
   * @param {ParsedEventSchema} eventData
   * @param {string} [eventId]
   * @returns {{ id: string, component: ICAL.Component, meta: TEventMeta }}
   */
  toComponent(eventData, eventId) {
    var component = new ICAL.Component(['vcalendar', [], []]);
    component.updatePropertyWithValue('prodid', '-//CyrusIMAP.org/Cyrus');

    /** @type {ICAL.Component} */
    let vcomponent
    // Remove type in ids in case it changes
    let id = eventId?.replace('vtodo-', '').replace('vevent-', '') ?? v4();
    /** @type {TEventMeta} */
    let meta;

    if (eventData.date) {
      meta = { icalType: 'vevent' };
      // Prefix id with vevent to reuse id when chaining to todo
      id = `vevent-${id}`;
      vcomponent = new ICAL.Component('vevent');
      const event = new ICAL.Event(vcomponent);
      // Set standard properties
      event.summary = eventData.title;
      event.uid = id
      if (eventData.description) {
        event.description = eventData.description;
      }
      event.startDate = ICAL.Time.fromJSDate(eventData.date, true);
      if (eventData.endDate) {
        event.endDate = ICAL.Time.fromJSDate(eventData.endDate, true);
      } else {
        event.duration = new ICAL.Duration({ minutes: 15 });
      }

      eventData.alarms.forEach(a => {
        const valarm = new ICAL.Component('valarm');
        valarm.addPropertyWithValue('action', 'DISPLAY');
        valarm.addPropertyWithValue('related', 'START');
        valarm.addPropertyWithValue('trigger', new ICAL.Duration({
          // Force before event
          ...a.duration, isNegative: true
        }))
        vcomponent.addSubcomponent(valarm)
      })

      if (eventData.recur) {
        const icalRecur = ICAL.Recur.fromString(eventData.recur.replace('RRULE:', ''))
        vcomponent.addPropertyWithValue('rrule', icalRecur);
      }
    } else {
      // Prefix id with vtodo to reuse id when chaining to event
      id = `vtodo-${id}`;
      meta = {icalType: 'vtodo'}
      vcomponent = new ICAL.Component('vtodo');
      vcomponent.addPropertyWithValue('uid', id);
      vcomponent.addPropertyWithValue('summary', eventData.title);
      if (eventData.description) {
        vcomponent.addPropertyWithValue('description', eventData.description)
      }
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

    return { id, component, meta };
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
