import { DAVClient, DAVNamespaceShort } from "tsdav";
import ICAL from 'ical.js'
import { v4 } from "uuid";
import { add, addMinutes, startOfDay } from "date-fns/fp";
import { endOfDay, isAfter, isWithinInterval } from "date-fns";
import yup from 'yup';
import { isValidRRule } from "$lib/utils/rrule";
import { isBlock, isDefined, isReminder, isTask } from "$lib/util";
import registerAllTz from "./timezones";
import { alarmSchema } from "./alarmSchema";
import { CalendarObjectModel, UserModel } from "../db";

/** @typedef {import('tsdav').DAVCalendar} DAVCalendar */
/** @typedef {import('tsdav').DAVObject} DAVObject */

/**
 * @template {import("yup").ISchema<any, any>} T
 * @typedef {import('yup').InferType<T>} InferType
 */

/** @typedef {import('./alarmSchema').TAlarm} TAlarm */

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
  date: yup.date(),
  endDate: yup.date(),
  description: yup.string(),
  tags: yup.array().of(yup.string().required()).required(),
  recur: yup.string().test(
    'is-recur',
    str => `${str.path} is not a valid RRule`,
    value => {
      // Do no validation on empty string
      if (!value || value.length === 0) return true
      return isValidRRule(value)
    },
  ), 
  lastDone: yup.date(),
  alarms: yup.array().of(alarmSchema).required(),
  externalName: yup.string(),
});

/**
 * Field status, block and events don't have it
 */
const statusField = yup.string().required().matches(statusRE, { excludeEmptyString: true });

/**
 * Fields related to ranking
 */
const rankingFields = yup.object({
  importance: yup.number().min(-3).max(3).required(),
  urgency: yup.number().min(0).max(3).required(),
  load: yup.number().min(0).max(3).required(),
});

/**
 * Blocks must have a start and end date
 * They have no importance nor status
 */
const blockSchema = baseSchema.shape({
  date: yup.date().required(),
  endDate: yup.date().required(),
});

/**
 * Reminders must have an start date
 * They also can have rank and status
 */
const reminderSchema = rankingFields
  .concat(baseSchema)
  .shape({
    date: yup.date().required(),
    status: statusField,
  })

const eventSchema = baseSchema.shape({
  date: yup.date().required(),
  endDate: yup.date().required(),
})

const taskSchema = rankingFields
  .concat(baseSchema)
  .shape({
    status: statusField,
  })


/** @typedef {InferType<typeof blockSchema>} TBlockSchema */
/** @typedef {InferType<typeof reminderSchema>} TReminderSchema */
/** @typedef {InferType<typeof eventSchema>} TEventSchema */
/** @typedef {InferType<typeof taskSchema>} TTaskSchema */
/** @typedef {TBlockSchema | TReminderSchema | TEventSchema | TTaskSchema} TAllTypes */

/**
 * @template {TAllTypes} T
 * @typedef {T & { eventId: string }} WithId
 */

/** @typedef {WithId<TReminderSchema> | WithId<TTaskSchema> | WithId<TBlockSchema> | WithId<TEventSchema> } TAllTypesWithId */

/**
 * @template T
 * @param {Array<T>} array 
 * @param {number} chunkSize 
 * @returns {Array<Array<T>>}
 */
function chunkArray(array, chunkSize) {
  return array.reduce((resultArray, item, index) => { 
      const chunkIndex = Math.floor(index/chunkSize)
      if(!resultArray[chunkIndex]) { resultArray[chunkIndex] = [] }
      resultArray[chunkIndex].push(item)
      return resultArray
    }, /** @type {Array<Array<T>>} */([]))
}

export class CalendarBackend {

  /**
   * @param {string} username
   * @param {import("../../../app").UserCalendar} auth
   * @param {boolean} [displayOnly=false] If the calendar is only for display
   */
  constructor(username, auth, displayOnly = false) {
    /** @private */
    this.username = username;
    /** @private */
    this.auth = auth;
    /** @private */
    this.displayOnly = displayOnly;

    // if (isBasicAuth(auth)) {
      /** @private */
      this.client = new DAVClient({
        serverUrl: auth.server,
        credentials: {
          username: auth.email,
          password: auth.password,
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
    // } else {
    //   /** @private */
    //   this.client = new DAVClient({
    //     serverUrl: 'https://apidata.googleusercontent.com/caldav/v2/',
    //     credentials: {
    //       ...auth,
    //       tokenUrl: 'https://accounts.google.com/o/oauth2/token',
    //       clientId: GOOGLE_CLIENT_ID,
    //       clientSecret: GOOGLE_CLIENT_SECRET,
    //     },
    //     authMethod: 'Oauth',
    //     defaultAccountType: 'caldav',
    //   });
    //   /** 
    //    * @private
    //    * @type {Promise<void>}
    //    * Used to only log to servers once 
    //    */
    //   this.logged = this.client.login();
    //   /**
    //    * @private
    //    * @type {DAVCalendar | undefined}
    //    */
    //   this.calendar = undefined;
    // }
  }

  async getCalendars() {
    if (this.calendars) return this.calendars
    await this.logged;

    this.calendars = await this.client.fetchCalendars();
    return this.calendars;
  }

  /**
   * 
   * @param {string} [calendarName] 
   * @param {boolean} [force]
   * @returns 
   */
  async getCalendar(calendarName, force) {
    if (this.calendar && !force) return this.calendar
    const calendars = await this.getCalendars();
    const auth = this.auth;
    let calendar;
    if (calendarName) {
      calendar = calendars.find(c => c.displayName === calendarName)
    } else {
      calendar = calendars.find(c => c.displayName === auth.calendar)
    }
    

    if (!calendar) throw new Error(`No Calendar found`);
    return calendar;
  }

  /**
   * Check the connection to the server and if the calendar exists
   * @param {string} [calendarName]
   */
  async check(calendarName) {
    await registerAllTz();
    await this.logged,
    await this.getCalendar(calendarName);
  }

  /**
   * @param {boolean} includeTodo
   * @param {string} [calendarName]
   */
  async initialSync(includeTodo, calendarName ) {
    const calendar = await this.getCalendar(calendarName, true);
    const [todoObjs, eventObjs] = await Promise.all([
      this.listTodosRaw(),
      this.client.fetchCalendarObjects({
        calendar,
      })
    ])
    const modelEvents = eventObjs.map(obj => {
      const comp = ICAL.Component.fromString(obj.data);
      const vevent = comp.getFirstSubcomponent('vevent');
      const event = new ICAL.Event(vevent);
      return {
        id: event.uid,
        user: this.username,
        calendarUrl: calendar.url,
        data: obj.data,
        date: event.startDate.toJSDate(),
        endDate: event.endDate.toJSDate(),
        etag: obj.etag,
        url: obj.url,
        icalType: 'vevent',
      }
    })
    const eventsRes = Promise.all(chunkArray(modelEvents, 4).map(chunk =>
      CalendarObjectModel.batchPut(chunk)
    ))

    if (!includeTodo) {
      return await eventsRes;
    }

    const modelTodos = todoObjs.map(obj => {
      const comp = ICAL.Component.fromString(obj.data);
      const vtodo = comp.getFirstSubcomponent('vtodo');
      return {
        id: vtodo?.getFirstPropertyValue('uid'),
        user: this.username,
        calendarUrl: calendar.url,
        data: obj.data,
        etag: obj.etag,
        url: obj.url,
        icalType: 'vtodo',
      } 
    })

    const todosRes = Promise.all(chunkArray(modelTodos, 4).map(chunk =>
      CalendarObjectModel.batchPut(chunk)
    ))

    await Promise.all([eventsRes, todosRes])
    const user = await UserModel.get({ username: this.username })
    user.main.url = calendar.url;
    user.main.ctag = calendar.ctag;
    user.main.syncToken = calendar.syncToken;
    await user.save();
  }

  /**
   * @param {TAllTypes} eventData
   */
  async createEvent(eventData) {
    const calendar = await this.getCalendar();

    var { id, component, meta } = this.toComponent(eventData);

    const model = await CalendarObjectModel.create({
      id,
      calendarUrl: calendar.url,
      url: await this.getEventUrl(id),
      user: this.username,
      data: component.toString(),
      date: eventData.date,
      endDate: eventData.endDate,
      icalType: meta.icalType,
    })

    const calendarPush = this.client.createCalendarObject({
      calendar: calendar,
      filename: `${id}.ics`,
      iCalString: component.toString(),
    });

    calendarPush.then(async () => {
      // TODO check result
      const newE = await this.getEventRaw(id);
      const etag = newE.raw.props?.getetag
      if (etag) {
        await CalendarObjectModel.update({ id, etag })
      }
    })

    return { id, model };
  }

  /** @private */
  async listTodosRaw() {
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

    return objects;
  }

  async listTodos() {
    const calendar = await this.getCalendar();
    // const objects = await this.listTodosRaw(); 
    const objects = await CalendarObjectModel.scan({
      calendarUrl: calendar.url,
      user: this.username,
      icalType: 'vtodo',
    }).exec()
  
    return objects.map(e => {
      const comp = ICAL.Component.fromString(e.data);
      const vtodo = comp.getFirstSubcomponent('vtodo');
      if (vtodo) {
        return this.fromVTodo(vtodo).event;
      }
    }).filter(isDefined)
  }

  /**
   * 
   * @param {Date} day 
   * @param {string} [calendar]
   */
  async listDayEvent(day, calendar) {
    const from = startOfDay(day)
    const to = addMinutes(1, endOfDay(day))
    return this.listEvents(from, to, calendar);
  }

  /**
   * 
   * List the events for an external event
   * This will be forced into an EType.EVENT and add a tag
   * @param {Date} day 
   * @param {string} calendar
   */
  async listExternalDayEvents(day, calendar) {
    const events = await this.listDayEvent(day, calendar);
    return events.map(e => ({
      ...e,
      type: EType.EVENT,
      externalName: calendar,
    }))

  }

  /**
   * @param {Date} from
   * @param {Date} to
   * @param {string} [calendarName]
   * @returns {Promise<TAllTypesWithId[]>}
   */
  async listEvents(from, to, calendarName) {
    const calendar = await this.getCalendar(calendarName);
    // const objects = await this.client.fetchCalendarObjects({
    //     calendar: calendar,
    //     timeRange: {
    //       start: formatISO(from),
    //       end: formatISO(to),
    //     }
    //   });
    const objects = await CalendarObjectModel.scan({
      calendarUrl: calendar.url,
      user: this.username,
      icalType: 'vevent',
    }).exec()
       
    return objects
      .map(o => o.data)
      .map(o => this.parseCalendarVEvent(o, from, to))
      .filter(isDefined)
  }

  /**
   * @param {string | DAVObject} obj 
   * @param {Date} from
   * @param {Date} to 
   */
  parseCalendarVEvent(obj, from, to) {
    const data = typeof obj === 'string' ? obj : obj.data
    const comp = ICAL.Component.fromString(data);
    const vevents = comp.getAllSubcomponents('vevent');

    if (vevents.length === 0) return;
    const parsed = vevents.map(e => this.fromVEvent(e))
    let occurrenceEvent;

    for (let index = 0; index < parsed.length; index++) {
      /** @type {ICAL.Time | undefined} */
      let currentOccurence;
      const element = parsed[index];
      const vevent = vevents[index];

      let iterator = new ICAL.RecurExpansion({
        component: vevent,
        dtstart: vevents[index].getFirstPropertyValue('dtstart')
      });
      // next is always an ICAL.Time or null
      /** @type {ICAL.Time | null} */
      let next = iterator.next()
      while (next) {
        const nextJS = next.toJSDate()

        if (isAfter(nextJS, to)) {
          break;
        }
        if (next && isWithinInterval(nextJS, { start: from, end: to })) {
          currentOccurence = next
          break;
        }
        next = iterator.next()
      }

      if (currentOccurence) {
        // @ts-expect-error add types
        const details = element.icalEvent.getOccurrenceDetails(currentOccurence);
        
        let startDate = details.startDate
        let endDate = details.endDate

        occurrenceEvent = {
          ...element.event,
          date: startDate.toJSDate(),
          endDate: endDate.toJSDate(),
        }
      }
    }

    return occurrenceEvent
  }

  /**
   * @private
   * @param {string} id 
   */
  async getEventRaw(id) {
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

    const raw = object;
    const comp = ICAL.Component.fromString(object?.props?.calendarData._cdata);
    return { raw, comp };
  }

  /** @param {string} id */
  async getEvent(id) {
    const dbEvent = await CalendarObjectModel.get({ id });
    const comp = ICAL.Component.fromString(dbEvent.data);

    /** @type {ReturnType<CalendarBackend['fromVTodo']> | ReturnType<CalendarBackend['fromVEvent']> | undefined} */
    let result;
    if (id.startsWith('vevent-')) {
      const vevent = comp.getFirstSubcomponent('vevent');
      if (vevent) (result = this.fromVEvent(vevent));
    } else if (id.startsWith('vtodo-')) {
      const vtodo = comp.getFirstSubcomponent('vtodo');
      if (vtodo) (result = this.fromVTodo(vtodo));
    } else {
      throw new Error('Event with old id: ' + id)
    }

    if (!result) {
      throw new Error("Can't parse event: :" + id);
    }
    return result;
  }

  /**
   * 
   * @param {TAllTypes} data 
   * @return {Promise<TAllTypes>}
   */
  async validateEventData(data) {
    switch (data.type) {
      case EType.BLOCK:
        return blockSchema.validate(data);
      case EType.EVENT:
        return eventSchema.validate(data)
      case EType.REMINDER:
        return reminderSchema.validate(data)
      default:
        return taskSchema.validate(data)  
    }
  }

  /**
   * @param {string} id
   * @param {TAllTypes} eventData
   */
  async editEvent(id, eventData) {
    const res  = await this.getEvent(id);
    if (!res) throw new Error('Event does not exists');
    const { meta } = res;

    const { component, id: newId } = this.toComponent(eventData, id);

    // If event changed type, destroy and recreate
    if (meta.icalType !== meta.icalType) {
      await this.deleteEvent(id)
      const result = await this.client.createCalendarObject({
        calendar: await this.getCalendar(),
        filename: `${newId}.ics`,
        iCalString: component.toString(),
      });
      return { id, result };
    }

    await CalendarObjectModel.update({ id }, { data: component.toString() })

    this.client.updateCalendarObject({
      calendarObject: {
        url: await this.getEventUrl(id),
        data: component.toString()
      }
    }).then(() => console.log('Updated calendar object'))
    return { id }
  }

  /**
   * @param {string} eventId
   * @param {EStatus} status
   */
  async updateStatus(eventId, status) {
    const res = await this.getEvent(eventId);
    if (!res) {
      throw new Error(`Could not find event with id: ${eventId}`)
    }
    const { event } = res; 

    if (isTask(event) || isReminder(event)) {
      event.status = status;
      return this.editEvent(eventId, event);
    }
  }

  /**
   * @param {string} id
   */
  async deleteEvent(id) {
    const event = await this.getEvent(id);
    if (!event) throw new Error('Event does not exists');

    await CalendarObjectModel.delete({ id })

    this.client.deleteCalendarObject({
      calendarObject: {
        url: await this.getEventUrl(id),
      }
    }).then(() => {
      console.log('Deleted calendar object')
    })
    return event
  }

  /**
   * @private
   * @param {string | TAllTypesWithId} eventOrId 
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
   * @return {{ event: WithId<TTaskSchema>, meta: TEventMeta }}
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
    const description = vtodo.getFirstPropertyValue('description');

    const tags = this.parseTags(vtodo);

    return {
      event: {
        eventId,
        description,
        alarms: [],
        title,
        type: vtodo.getFirstPropertyValue(CustomPropName.TYPE) ?? EType.TASK,
        tags: tags,
        status: vtodo.getFirstPropertyValue(CustomPropName.STATUS) ?? EStatus.TODO,
        originalText: vtodo.getFirstPropertyValue(CustomPropName.ORIGINAL_TEXT) ?? EStatus.TODO,
        urgency,
        importance,
        load,
      },
      meta: { icalType: 'vtodo' }
    }
  }

  /**
   * @param {ICAL.Component} vevent - vevent component from calendar
   * @return {{ event: TAllTypesWithId, icalEvent: ICAL.Event, meta: TEventMeta}}
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
    /** @type {TAllTypes['recur']} */
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
            isNegative: dur?.isNegative ?? true,
          }
        })
      
    }
    
    const tags = this.parseTags(vevent);

    /** @type {TEventMeta} */
    const meta = {
      icalType: 'vevent',
      recurrenceId: icalEvent.recurrenceId?.toJSDate(),
    }

    const event = {
      eventId: /** @type {string} */ (icalEvent.uid),
      title: /** @type {string} */ (icalEvent.summary),
      description: /** @type {string | undefined} */ (icalEvent.description),
      date,
      endDate,
      type: vevent.getFirstPropertyValue(CustomPropName.TYPE) ?? EStatus.TODO,
      tags, 
      status: vevent.getFirstPropertyValue(CustomPropName.STATUS) ?? EStatus.TODO,
      originalText: vevent.getFirstPropertyValue(CustomPropName.ORIGINAL_TEXT) ?? EStatus.TODO,
      urgency,
      importance,
      load,
      alarms,
      recur,
    }

    return { event, icalEvent, meta }
  }

  /**
   * @private
   * @param {ICAL.Component} comp
   */
  parseTags(comp) {
    const categories = comp
      .getAllProperties('categories')
      .map(v => v.getFirstValue().replace('\\:', ':'));
    
    if (categories.length > 0) return categories;

    const tagProp = comp.getFirstPropertyValue(CustomPropName.TAG)?.trim() ?? '';
    return tagProp.length > 0 ? tagProp.split(',') : []
  }

  /**
   * Transform an {@link TAllTypes} into a {@link ICAL.Component} for sending
   * If {@link TAllTypesWithId#eventId} is not defined, one will be created
   * @param {TAllTypes} eventData
   * @param {string} [eventId]
   * @returns {{ id: string, component: ICAL.Component, meta: TEventMeta }}
   */
  toComponent(eventData, eventId) {
    const {
      description,
      date,
      endDate,
      alarms,
      title,
      recur,
      originalText,
      type,
      tags,
    } = eventData;
    var component = new ICAL.Component(['vcalendar', [], []]);
    component.updatePropertyWithValue('prodid', '-//CyrusIMAP.org/Cyrus');

    /** @type {ICAL.Component} */
    let vcomponent
    // Remove type in ids in case it changes
    let id = eventId?.replace('vtodo-', '').replace('vevent-', '') ?? v4();
    /** @type {TEventMeta} */
    let meta;

    if (date) {
      meta = { icalType: 'vevent' };
      // Prefix id with vevent to reuse id when chaining to todo
      id = `vevent-${id}`;
      vcomponent = new ICAL.Component('vevent');
      const event = new ICAL.Event(vcomponent);
      // Set standard properties
      event.summary = title;
      event.uid = id
      if (description) {
        event.description = description;
      }
      event.startDate = ICAL.Time.fromJSDate(date, true);
      if (endDate) {
        event.endDate = ICAL.Time.fromJSDate(endDate, true);
      } else {
        event.duration = new ICAL.Duration({ minutes: 15 });
      }

      alarms.forEach(a => {
        const valarm = new ICAL.Component('valarm');
        valarm.addPropertyWithValue('action', 'DISPLAY');
        valarm.addPropertyWithValue('related', 'START');
        valarm.addPropertyWithValue('trigger', new ICAL.Duration({
          // Force before event
          ...a.duration, isNegative: true
        }))
        vcomponent.addSubcomponent(valarm)
      })

      if (recur) {
        const icalRecur = ICAL.Recur.fromString(recur.replace('RRULE:', ''))
        vcomponent.addPropertyWithValue('rrule', icalRecur);
      }
    } else {
      // Prefix id with vtodo to reuse id when chaining to event
      id = `vtodo-${id}`;
      meta = {icalType: 'vtodo'}
      vcomponent = new ICAL.Component('vtodo');
      vcomponent.addPropertyWithValue('uid', id);
      vcomponent.addPropertyWithValue('summary', title);
      if (description) {
        vcomponent.addPropertyWithValue('description', description)
      }
    } 

    vcomponent.addPropertyWithValue(CustomPropName.ORIGINAL_TEXT, originalText);
    vcomponent.addPropertyWithValue(CustomPropName.TYPE, type ?? EType.EVENT);

    if (tags.length > 0) {
      vcomponent.addPropertyWithValue('categories', tags.map(t => t.replace(':', '\\:')).join(',')),
      vcomponent.addPropertyWithValue(CustomPropName.TAG, tags.join(','));
    }
    
    if (!isBlock(eventData)) { 
      const { urgency, load, importance } = eventData;
      vcomponent.addPropertyWithValue(CustomPropName.URGENCY, urgency ?? 0);
      vcomponent.addPropertyWithValue(CustomPropName.LOAD, load ?? 0);
      vcomponent.addPropertyWithValue(CustomPropName.IMPORTANCE, importance ?? 0);
    }

    if (isTask(eventData) || isReminder(eventData)) {
      vcomponent.addPropertyWithValue(CustomPropName.STATUS, eventData.status ?? EStatus.TODO);
    }

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

/** @type {Record<string, CalendarBackend>} */
const backends = {};

/** @param {import('../db').User} user */
export async function getBackend(user) {

  if (!backends[user.username]) {
    // @ts-ignore
    const back = new CalendarBackend(user.username ,user.main);
    await back.check();
    backends[user.username] = back;
  }
  return backends[user.username]

}