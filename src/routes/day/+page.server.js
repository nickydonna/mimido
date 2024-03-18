import { Event } from '$lib/server/schemas/event';
import { endOfDay, parseISO, startOfDay } from 'date-fns';

/** @typedef {import('../$types').Actions} Actions */
/** @typedef {import('$lib/server/calendar').TEventSchema} TEventSchema */

/** @type {import('../$types').PageServerLoad<{ events: TEventSchema[] }>} */
export const load = async ({ url, locals }) => {
  const queryDate = url.searchParams.get('date');
	const date = queryDate ? parseISO(queryDate) : new Date();
	const events = await locals.backend.listDayEvent(date);

	return { events };
};
