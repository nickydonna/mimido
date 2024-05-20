import type { DAVCalendar } from 'tsdav';
import { DAVClient, DAVNamespaceShort, type DAVObject } from 'tsdav';
import ICAL from 'ical.js';
import { v4 } from 'uuid';
import { add, addMinutes, addDays, isWithinInterval } from 'date-fns/fp';
import { isAfter } from 'date-fns';
import yup, { type InferType } from 'yup';
import { isValidRRule } from '$lib/utils/rrule';
import { isBlock, isDefined, isDone, isReminder, isTask } from '$lib/util';
import registerAllTz from './timezones';
import { alarmSchema, type TAlarmSchema } from './alarmSchema';
import type { Calendar } from '@prisma/client';
import { prisma } from '../prisma';

export enum EType {
	EVENT = 'event',
	BLOCK = 'block',
	REMINDER = 'reminder',
	TASK = 'task'
}

export enum EStatus {
	BACK = 'back', // backlog
	TODO = 'todo',
	DOING = 'doing',
	DONE = 'done'
}

export type TEventMeta = {
	icalType: 'vtodo' | 'vevent';
	recurrenceId?: Date;
};

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
	postponed: yup.number().required(),
	recur: yup.string().test(
		'is-recur',
		(str) => `${str.path} is not a valid RRule`,
		(value) => {
			// Do no validation on empty string
			if (!value || value.length === 0) return true;
			return isValidRRule(value);
		}
	),
	lastDone: yup.date(),
	alarms: yup.array().of(alarmSchema).required(),
	externalName: yup.string()
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
	load: yup.number().min(0).max(3).required()
});

/**
 * Blocks must have a start and end date
 * They have no importance nor status
 */
const blockSchema = baseSchema.shape({
	date: yup.date().required(),
	endDate: yup.date().required()
});

/**
 * Reminders must have a start date
 * They also can have rank and status
 */
const reminderSchema = rankingFields.concat(baseSchema).shape({
	date: yup.date().required(),
	status: statusField
});

const eventSchema = baseSchema.shape({
	date: yup.date().required(),
	endDate: yup.date().required()
});

const taskSchema = rankingFields.concat(baseSchema).shape({
	status: statusField
});

export type TBlockSchema = InferType<typeof blockSchema>;
export type TReminderSchema = InferType<typeof reminderSchema>;
export type TEventSchema = InferType<typeof eventSchema>;
export type TTaskSchema = InferType<typeof taskSchema>;
export type TAllTypes = TBlockSchema | TReminderSchema | TEventSchema | TTaskSchema;

export type WithId<T extends TAllTypes> = Omit<T, 'eventId'> & { eventId: NonNullable<string> };
export type TAllTypesWithId =
	| WithId<TBlockSchema>
	| WithId<TReminderSchema>
	| WithId<TEventSchema>
	| WithId<TTaskSchema>;


export class CalendarBackend {
	private client: DAVClient;
	logged: Promise<void>;
	calendar: DAVCalendar | undefined;
	calendars: DAVCalendar[] | undefined;

	constructor(
		readonly calendarInfo: Omit<Calendar, 'createdAt' | 'updatedAt'>
	) {
		this.client = new DAVClient({
			serverUrl: calendarInfo.server,
			credentials: {
				username: calendarInfo.email,
				password: calendarInfo.password
			},
			authMethod: 'Basic',
			defaultAccountType: 'caldav'
		});
		this.logged = this.client.login();
		this.calendar = undefined;
	}

	async getCalendars() {
		if (this.calendars) return this.calendars;
		await this.logged;

		this.calendars = await this.client.fetchCalendars();
		return this.calendars;
	}

	async getCalendar(calendarName?: string | undefined, force: boolean = false) {
		if (this.calendar && !force) return this.calendar;
		const calendars = await this.getCalendars();
		const auth = this.calendarInfo;
		let calendar;
		if (calendarName) {
			calendar = calendars.find((c) => c.displayName === calendarName);
		} else {
			calendar = calendars.find((c) => c.displayName === auth.calendar);
		}

		if (!calendar) throw new Error(`No Calendar found`);
		return calendar;
	}

	/**
	 * Check the connection to the server and if the calendar exists
	 */
	async check(calendarName?: string): Promise<boolean> {
		await registerAllTz();
		await this.logged;
		await this.getCalendar(calendarName, true);
		return true
	}

	async initialSync(includeTodo: boolean, syncCalendarInfo: Omit<Calendar, 'createdAt' | 'updatedAt'> = this.calendarInfo) {
		const calendar = await this.getCalendar(syncCalendarInfo.calendar, true);

		if (includeTodo) {
			await prisma.calendarObject.deleteMany({
				where: {
					icalType: 'vtodo',
					calendarId: syncCalendarInfo.id,
				}
			})
		}

		await prisma.calendarObject.deleteMany({ where: { calendarId: syncCalendarInfo.id } })
		const [todoObjs, eventObjs] = await Promise.all([
			this.listTodosRaw(),
			this.client.fetchCalendarObjects({ calendar })
		]);
		const modelEvents = eventObjs.map((obj) => {
			const comp = ICAL.Component.fromString(obj.data);
			const vevent = comp.getFirstSubcomponent('vevent');
			const event = new ICAL.Event(vevent);
			const rrule = vevent?.getFirstPropertyValue('rrule');
			let recur: TAllTypes['recur'];

			if (rrule) {
				const icalRecur = new ICAL.Recur(rrule);
				recur = icalRecur.toString();
			}
			return {
				eventId: event.uid,
				data: obj.data,
				date: event.startDate.toJSDate(),
				endDate: event.endDate.toJSDate(),
				etag: obj.etag,
				url: obj.url,
				recur: recur,
				icalType: 'vevent',
				calendarId: syncCalendarInfo.id,
			};
		});

		const eventsRes = prisma.calendarObject.createMany({ data: modelEvents })

		if (!includeTodo) {
			await eventsRes;
			return {
				url: calendar.url,
				ctag: calendar.ctag,
				syncToken: calendar.syncToken
			};
		}

		const modelTodos = todoObjs.map((obj) => {
			const comp = ICAL.Component.fromString(obj.data);
			const vtodo = comp.getFirstSubcomponent('vtodo');
			return {
				eventId: vtodo?.getFirstPropertyValue<string>('uid')!,
				data: obj.data,
				etag: obj.etag,
				url: obj.url,
				icalType: 'vtodo',
				calendarId: syncCalendarInfo.id,
			};
		});

		const todosRes = prisma.calendarObject.createMany({ data: modelTodos })

		await Promise.all([eventsRes, todosRes]);
		return {
			url: calendar.url,
			ctag: calendar.ctag,
			syncToken: calendar.syncToken
		};
	}

	/**
	 * @param {TAllTypes} eventData
	 */
	async createEvent(eventData: TAllTypes) {
		const calendar = await this.getCalendar();

		const { id, component, meta } = this.toComponent(eventData);

		const model = await prisma.calendarObject.create({
			data: {
				eventId: id,
				url: await this.getEventUrl(id),
				data: component.toString(),
				date: eventData.date,
				endDate: eventData.endDate,
				icalType: meta.icalType,
				calendar: {
					connect: { id: this.calendarInfo.id }
				}
			}
		});

		const calendarPush = this.client.createCalendarObject({
			calendar: calendar,
			filename: `${id}.ics`,
			iCalString: component.toString()
		});

		calendarPush.then(async () => {
			// TODO check result
			const newE = await this.getEventRaw(id);
			const etag = newE.raw.props?.getetag;
			if (etag) {
				await prisma.calendarObject.update({ where: { id: model.id }, data: { etag } })
			}
			console.log('Create calendar object');
		});

		return { id, model };
	}

	/** @private */
	async listTodosRaw() {
		const calendar = await this.getCalendar();
		return await this.client.fetchCalendarObjects({
			calendar: calendar,
			filters: {
				'comp-filter': {
					_attributes: {
						name: 'VCALENDAR'
					},
					'comp-filter': {
						_attributes: {
							name: 'VTODO'
						}
					}
				}
			}
		});
	}

	async listTodos({ excludeDone }: { excludeDone?: boolean } = {}) {
		const objects = await prisma.calendarObject.findMany({
			where: {
				calendarId: this.calendarInfo.id,
				icalType: 'vtodo'
			}
		})

		const todos = objects
			.map((e) => {
				const comp = ICAL.Component.fromString(e.data);
				const vtodo = comp.getFirstSubcomponent('vtodo');
				if (vtodo) {
					return this.fromVTodo(vtodo).event;
				}
			})
			.filter(isDefined);
		return excludeDone ? todos.filter((t) => !isDone(t)) : todos;
	}

	async listDayEvent(startTime: Date, calendarId?: number) {
		const from = startTime;
		const to = addDays(1, startTime)
		return this.listEvents(from, to, calendarId);
	}

	async listExternalDayEvents(day: Date, calendarId: number) {
		const events = await this.listDayEvent(day, calendarId);
		return events.map((e) => ({
			...e,
			type: EType.EVENT,
			externalId: calendarId,
		}));
	}

	async listEvents(from: Date, to: Date, calendarId?: number) {
		const objects = await prisma.calendarObject.findMany({
			where: {
				calendarId: calendarId ?? this.calendarInfo.id,
				icalType: 'vevent',
				OR: [
					// Check end time for recur
					{ NOT: { recur: null } },
					{ date: { gte: from }, endDate: { lte: to } },
					{ date: { gte: from }, endDate: null },

				]
			}
		})

		return objects
			.map((o) => o.data)
			.map((o) => this.parseCalendarVEvent(o, from, to))
			.filter(isDefined);
	}

	parseCalendarVEvent(obj: string | DAVObject, from: Date, to: Date) {
		const data = typeof obj === 'string' ? obj : obj.data;
		const comp = ICAL.Component.fromString(data);
		const vevents = comp.getAllSubcomponents('vevent');

		if (vevents.length === 0) return;
		const parsed = vevents.map((e) => this.fromVEvent(e));
		let occurrenceEvent;
		const isBetween = isWithinInterval({ start: from, end: to });

		for (let index = 0; index < parsed.length; index++) {
			let currentOccurrence: ICAL.Time | undefined;
			const element = parsed[index];
			const vevent = vevents[index];

			if (
				element.icalEvent.isRecurrenceException() &&
				element.event.date &&
				isBetween(element.event.date)
			) {
				occurrenceEvent = element.event;
				break;
			}

			const iterator = new ICAL.RecurExpansion({
				component: vevent,
				dtstart: vevent.getFirstPropertyValue('dtstart')
			});

			// next is always an ICAL.Time or null
			let next: ICAL.Time | null = iterator.next();
			while (next) {
				const nextJS = next.toJSDate();

				if (isAfter(nextJS, to)) {
					break;
				}
				if (next && isBetween(nextJS)) {
					currentOccurrence = next;
					break;
				}
				next = iterator.next();
			}

			if (currentOccurrence) {
				// @ts-expect-error add types
				const details = element.icalEvent.getOccurrenceDetails(currentOccurrence);

				const startDate = details.startDate;
				const endDate = details.endDate;

				occurrenceEvent = {
					...element.event,
					date: startDate.toJSDate(),
					endDate: endDate.toJSDate()
				};
			}
		}

		return occurrenceEvent;
	}

	async getEventRaw(id: string) {
		const [object] = await this.client.calendarMultiGet({
			url: await this.getCalendarUrl(),
			objectUrls: [await this.getEventUrl(id)],
			depth: '1',
			props: {
				[`${DAVNamespaceShort.DAV}:getetag`]: {},
				[`${DAVNamespaceShort.CALDAV}:calendar-data`]: {}
			}
		});
		if (!object?.ok || !object?.props || object.status % 200 > 100) {
			throw new Error('Event not found: ' + id);
		}

		const raw = object;
		const comp = ICAL.Component.fromString(object?.props?.calendarData._cdata);
		return { raw, comp };
	}

	async getEvent(id: string) {
		const dbEvent = await prisma.calendarObject.findFirstOrThrow({ where: { eventId: id } })
		const comp = ICAL.Component.fromString(dbEvent.data);

		let result:
			| ReturnType<CalendarBackend['fromVTodo']>
			| ReturnType<CalendarBackend['fromVEvent']>
			| undefined;
		if (id.startsWith('vevent-')) {
			const vevent = comp.getFirstSubcomponent('vevent');
			if (vevent) result = this.fromVEvent(vevent);
		} else if (id.startsWith('vtodo-')) {
			const vtodo = comp.getFirstSubcomponent('vtodo');
			if (vtodo) result = this.fromVTodo(vtodo);
		} else {
			throw new Error('Event with old id: ' + id);
		}

		if (!result) {
			throw new Error("Can't parse event: :" + id);
		}
		return result;
	}

	async validateEventData(data: TAllTypes) {
		switch (data.type) {
			case EType.BLOCK:
				return blockSchema.validate(data);
			case EType.EVENT:
				return eventSchema.validate(data);
			case EType.REMINDER:
				return reminderSchema.validate(data);
			default:
				return taskSchema.validate(data);
		}
	}

	async editEvent(id: string, eventData: TAllTypes) {
		const res = await this.getEvent(id);
		if (!res) throw new Error('Event does not exists');
		const { meta } = res;

		const { component, meta: newMeta } = this.toComponent(eventData, id);

		// If event changed type, destroy and recreate
		if (meta.icalType !== newMeta.icalType) {
			await this.deleteEvent(id);
			const { id: newId } = await this.createEvent(eventData);
			console.log('recreate', newId);
			return { id: newId };
		}

		// const isRecur = !eventData.recur
		//
		// const isDiffDate = eventData.date && res.event.date && !isSameSecond(eventData.date, res.event.date)
		// const isDiffEndDate = eventData.endDate && res.event.endDate && !isSameSecond(eventData.endDate, res.event.endDate)
		//
		// if (isRecur && (isDiffDate || isDiffEndDate)) {
		// 	const newRrule = addUntilDate(eventData.recur!, new Date());
		// 	res.event.recur = newRrule;
		//
		//
		// }

		await prisma.calendarObject.updateMany({
			where: { eventId: id },
			data: {
				data: component.toString(),
				date: eventData.date,
				endDate: eventData.endDate,
				recur: eventData.recur,
				postponed: eventData.postponed ?? 0
			}
		}
		);

		this.client
			.updateCalendarObject({
				calendarObject: {
					url: await this.getEventUrl(id),
					data: component.toString()
				}
			})
			.then(() => console.log('Updated calendar object'));
		return { id };
	}

	async updateStatus(eventId: string, status: EStatus) {
		const res = await this.getEvent(eventId);
		if (!res) {
			throw new Error(`Could not find event with id: ${eventId}`);
		}
		const { event } = res;

		if (isTask(event) || isReminder(event)) {
			event.status = status;
			return this.editEvent(eventId, event);
		}
	}

	async removeDate(eventId: string) {
		const res = await this.getEvent(eventId);
		if (!res) {
			throw new Error(`Could not find event with id: ${eventId}`);
		}
		const { event } = res;

		event.date = undefined;
		event.endDate = undefined;
		return this.editEvent(eventId, event);
	}

	async updateDate(eventId: string, from: Date, to: Date | undefined, postponing?: boolean) {
		const res = await this.getEvent(eventId);
		if (!res) {
			throw new Error(`Could not find event with id: ${eventId}`);
		}
		const { event } = res;

		event.date = from;
		event.endDate = to;
		if (postponing) {
			event.postponed = (event.postponed ?? 0) + 1;
		}
		return this.editEvent(eventId, event);
	}

	async deleteEvent(id: string) {
		const event = await this.getEvent(id);
		if (!event) throw new Error('Event does not exists');

		await prisma.calendarObject.deleteMany({ where: { eventId: id } });

		this.client
			.deleteCalendarObject({
				calendarObject: {
					url: await this.getEventUrl(id)
				}
			})
			.then(() => {
				console.log('Deleted calendar object');
			});
		return event;
	}

	private async getEventUrl(eventOrId: string | TAllTypesWithId) {
		const calUrl = await this.getCalendarUrl();
		return `${calUrl}${typeof eventOrId === 'string' ? eventOrId : eventOrId.eventId}.ics`;
	}

	async getCalendarUrl(calendarName?: string): Promise<string> {
		return (await this.getCalendar(calendarName)).url;
	}

	private parseRankProp(comp: ICAL.Component): {
		importance: number;
		load: number;
		urgency: number;
	} {
		let urgency = parseInt(comp.getFirstPropertyValue<string>(CustomPropName.URGENCY), 10);
		urgency = Number.isFinite(urgency) ? urgency : 0;
		let load = parseInt(comp.getFirstPropertyValue<string>(CustomPropName.LOAD), 10);
		load = Number.isFinite(load) ? load : 0;
		let importance = parseInt(comp.getFirstPropertyValue<string>(CustomPropName.IMPORTANCE), 10);
		importance = Number.isFinite(importance) ? importance : 0;
		return { importance, load, urgency };
	}

	/**
	 * @param {ICAL.Component} vtodo - vtodo component from calendar
	 * @return {{ event: WithId<TTaskSchema>, meta: TEventMeta }}
	 */
	fromVTodo(vtodo: ICAL.Component): { event: WithId<TTaskSchema>; meta: TEventMeta } {
		const eventId = vtodo.getFirstPropertyValue<string>('uid');
		const title = vtodo.getFirstPropertyValue<string>('summary');
		const { urgency, load, importance } = this.parseRankProp(vtodo);
		const description = vtodo.getFirstPropertyValue<string>('description');
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
				postponed: parseInt(vtodo.getFirstPropertyValue<string>(CustomPropName.POSTPONED) ?? '0')
			},
			meta: { icalType: 'vtodo' }
		};
	}

	/**
	 * @param {ICAL.Component} vevent - vevent component from calendar
	 * @return {{ event: TAllTypesWithId, icalEvent: ICAL.Event, meta: TEventMeta}}
	 */
	fromVEvent(vevent: ICAL.Component): {
		event: TAllTypesWithId;
		icalEvent: ICAL.Event;
		meta: TEventMeta;
	} {
		const icalEvent = new ICAL.Event(vevent);
		const date = icalEvent.startDate?.toJSDate();
		let endDate: Date | undefined;
		if (icalEvent.endDate) {
			endDate = icalEvent.endDate?.toJSDate();
		} else if (date && icalEvent.duration) {
			endDate = add(icalEvent.duration, date);
		} else if (date) {
			endDate = addMinutes(30, date);
		}

		const rrule = vevent.getFirstPropertyValue('rrule');
		let recur: TAllTypes['recur'];

		if (rrule) {
			const icalRecur = new ICAL.Recur(rrule);
			recur = icalRecur.toString();
		}

		const { importance, urgency, load } = this.parseRankProp(vevent);

		let alarms: TAlarmSchema[] = [];

		const valarms = vevent.getAllSubcomponents('valarm');
		if (valarms.length !== 0) {
			alarms = valarms
				// Only support display, no email
				.filter((comp) => comp.getFirstPropertyValue('action') === 'DISPLAY')
				.map((comp) => {
					// The ICAL duration is not good for formating
					const dur = comp.getFirstPropertyValue<ICAL.Duration>('trigger');
					return {
						related: 'START', // TODO check actual related
						duration: {
							hours: Math.abs(dur.hours),
							minutes: Math.abs(dur.minutes),
							days: Math.abs(dur.days),
							weeks: Math.abs(dur.weeks)
						},
						isNegative: dur?.isNegative ?? true
					};
				});
		}

		const tags = this.parseTags(vevent);

		const meta: TEventMeta = {
			icalType: 'vevent',
			recurrenceId: icalEvent.recurrenceId?.toJSDate()
		};

		const event = {
			eventId: /** @type {string} */ icalEvent.uid,
			title: /** @type {string} */ icalEvent.summary,
			description: /** @type {string | undefined} */ icalEvent.description,
			date,
			endDate,
			type: vevent.getFirstPropertyValue<EType>(CustomPropName.TYPE) ?? EType.TASK,
			tags,
			status: vevent.getFirstPropertyValue<EStatus>(CustomPropName.STATUS) ?? EStatus.TODO,
			originalText: vevent.getFirstPropertyValue<string>(CustomPropName.ORIGINAL_TEXT) ?? '',
			postponed: parseInt(vevent.getFirstPropertyValue<string>(CustomPropName.POSTPONED) ?? '0'),
			urgency,
			importance,
			load,
			alarms,
			recur
		};

		return { event, icalEvent, meta };
	}

	/**
	 * @private
	 * @param {ICAL.Component} comp
	 */
	parseTags(comp: ICAL.Component) {
		const categories = comp
			.getAllProperties('categories')
			.map((v) => v.getFirstValue<string>().replace('\\:', ':'));

		if (categories.length > 0) return categories;

		const tagProp = comp.getFirstPropertyValue<string>(CustomPropName.TAG)?.trim() ?? '';
		return tagProp.length > 0 ? tagProp.split(',') : [];
	}

	/**
	 * Transform an {@link TAllTypes} into a {@link ICAL.Component} for sending
	 * If {@link TAllTypesWithId#eventId} is not defined, one will be created
	 */
	toComponent(
		eventData: TAllTypes,
		eventId?: string,
	): { id: string; component: ICAL.Component; meta: TEventMeta } {
		const {
			postponed,
			description,
			date,
			endDate,
			alarms,
			title,
			recur,
			originalText,
			type,
			tags
		} = eventData;
		const component = new ICAL.Component(['vcalendar', [], []]);
		component.updatePropertyWithValue('prodid', '-//CyrusIMAP.org/Cyrus');

		let vcomponent: ICAL.Component;
		// Remove type in ids in case it changes
		let id = eventId?.replace('vtodo-', '').replace('vevent-', '') ?? v4();
		let meta: TEventMeta;

		if (date) {
			meta = { icalType: 'vevent' };
			// Prefix id with vevent to reuse id when chaining to todo
			id = `vevent-${id}`;
			vcomponent = new ICAL.Component('vevent');
			const event = new ICAL.Event(vcomponent);
			// Set standard properties
			event.summary = title;
			event.uid = id;
			if (description) {
				event.description = description;
			}
			event.startDate = ICAL.Time.fromJSDate(date, true);
			if (endDate) {
				event.endDate = ICAL.Time.fromJSDate(endDate, true);
			} else {
				event.duration = new ICAL.Duration({ minutes: 15 });
			}

			alarms.forEach((a) => {
				const valarm = new ICAL.Component('valarm');
				valarm.addPropertyWithValue('action', 'DISPLAY');
				valarm.addPropertyWithValue('related', 'START');
				valarm.addPropertyWithValue(
					'trigger',
					new ICAL.Duration({
						// Force before event
						...a.duration,
						isNegative: true
					})
				);
				vcomponent.addSubcomponent(valarm);
			});

			if (recur) {
				const icalRecur = ICAL.Recur.fromString(recur.replace('RRULE:', ''));
				vcomponent.addPropertyWithValue('rrule', icalRecur);
			}
		} else {
			// Prefix id with vtodo to reuse id when chaining to event
			id = `vtodo-${id}`;
			meta = { icalType: 'vtodo' };
			vcomponent = new ICAL.Component('vtodo');
			vcomponent.addPropertyWithValue('uid', id);
			vcomponent.addPropertyWithValue('summary', title);
			if (description) {
				vcomponent.addPropertyWithValue('description', description);
			}
		}

		vcomponent.addPropertyWithValue(CustomPropName.ORIGINAL_TEXT, originalText);
		vcomponent.addPropertyWithValue(CustomPropName.TYPE, type ?? EType.EVENT);
		vcomponent.addPropertyWithValue(CustomPropName.POSTPONED, postponed?.toString() ?? '0');

		if (tags.length > 0) {
			vcomponent.addPropertyWithValue(
				'categories',
				tags.map((t) => t.replace(':', '\\:')).join(',')
			);
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

	// TODO fix this
	// async smartSync(otherCalendars: ExtendCalendarAccess[]) {
	// 	// @ts-expect-error
	// 	const oldCalendars: DAVCalendar[] = [
	// 		{
	// 			name: this.auth.calendar,
	// 			syncToken: this.auth.syncToken,
	// 			ctag: this.auth.ctag,
	// 			url: this.auth.url
	// 		},
	// 		...otherCalendars
	// 	].filter((c) => isDefined(c.url));
	//
	// 	const { updated } = (await this.client.syncCalendars({
	// 		oldCalendars: oldCalendars,
	// 		// @ts-expect-error bad lib types
	// 		detailedResult: true
	// 	})) as unknown as { updated: DAVCalendar[] };
	//
	// 	// console.log(updated);
	// 	if (updated.length === 0) {
	// 		return;
	// 	}
	// 	const lc = updated[0];
	// 	const objects = await CalendarObjectModel.scan({
	// 		calendarUrl: lc.url,
	// 		user: this.username,
	// 		icalType: 'vevent',
	// 	}).exec();
	// 	const localObjects = objects.map((o): DAVObject => ({ etag: o.etag, url: o.url, data: o.data }))
	// 	const res =
	// 		await this.client.smartCollectionSync({
	// 			collection: {
	// 				url: oldCalendars[0].url,
	// 				ctag: oldCalendars[0].ctag,
	// 				syncToken: oldCalendars[0].syncToken,
	// 				objects: localObjects,
	// 				objectMultiGet: this.client.calendarMultiGet,
	// 			},
	// 			method: 'basic',
	// 			// @ts-expect-error bad lib types
	// 			detailedResult: true,
	// 		})
	//
	// 	const {
	// 		created: createdObjects,
	// 			updated: updatedObjects,
	// 			deleted: deletedObjects,
	// 	} = res.objects;
	//
	//
	// }
}

const CustomPropName = {
	TYPE: 'x-type',
	TAG: 'x-tag',
	URGENCY: 'x-urgency',
	LOAD: 'x-load',
	IMPORTANCE: 'x-importance',
	ORIGINAL_TEXT: 'x-original-text',
	STATUS: 'x-status',
	POSTPONED: 'x-postponed'
};

const backends: Record<string, CalendarBackend> = {};

export async function getBackend(calendarId: number, info: Calendar) {
	if (!backends[calendarId]) {
		const back = new CalendarBackend(info)
		await back.check();
		backends[calendarId] = back;
	}
	return backends[calendarId]
}
