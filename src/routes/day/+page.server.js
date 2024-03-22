/** @typedef {import('../$types').Actions} Actions */
/** @typedef {import('$lib/server/calendar').TEventSchema} TEventSchema */

import { fail } from '@sveltejs/kit';
import { parseISO } from 'date-fns/fp';

/** @type {import('./$types').PageServerLoad<{ date: Date, events: TEventSchema[] }>} */
export const load = async ({ locals, url }) => {
	const queryDate = url.searchParams.get('date');
	const date = queryDate ? parseISO(queryDate) : new Date();

	const { backend } = locals;
	const events = await backend.listDayEvent(date);

	return { date, events };
};

/** @type {import('./$types').Actions} */
export const actions = {
	delete: async ({request, locals}) => {
		const eventId = (await request.formData()).get('eventId');
		if (!eventId || typeof eventId !== 'string') {
			return fail(404, { error: 'Not Found' });
		}
		const event = await locals.backend.deleteEvent(eventId);
		return { success: true, event }
	}
}