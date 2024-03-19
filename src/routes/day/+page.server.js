/** @typedef {import('../$types').Actions} Actions */
/** @typedef {import('$lib/server/calendar').TEventSchema} TEventSchema */

import { parseISO } from 'date-fns/fp';

/** @type {import('./$types').PageServerLoad<{ date: Date, events: TEventSchema[] }>} */
export const load = async ({ locals, url }) => {
	const queryDate = url.searchParams.get('date');
	const date = queryDate ? parseISO(queryDate) : new Date();

	const { backend } = locals;
	const events = await backend.listDayEvent(date);

	return { date, events };
};
