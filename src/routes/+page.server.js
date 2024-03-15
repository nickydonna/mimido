import { Event } from '$lib/server/schemas/event';
import { endOfDay, parseISO, startOfDay } from 'date-fns';

/** @typedef {import('./$types').Actions} Actions */
/** @typedef {import('$lib/server/schemas/event.js').TEventSchema} TEventSchema */
/** @typedef {import('$lib/server/schemas/event.js').TEvent} TEvent */

/** @type {import('./$types').PageServerLoad<{ events: TEventSchema[] }>} */
export const load = async ({ url }) => {
  const queryDate = url.searchParams.get('date');
	const date = queryDate ? parseISO(queryDate) : new Date();
	const start = startOfDay(date);
	const end = endOfDay(date);
	const result = await Event.scan('date').between(start.valueOf(), end.valueOf()).exec();

	return {
		events: /** @type {TEventSchema[]} */ (result.map(e => e.toJSON())),
	};
};
