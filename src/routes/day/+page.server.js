/** @typedef {import('../$types').Actions} Actions */
/** @typedef {import('$lib/server/calendar').TAllTypesWithId} TAllTypesWithId */

import { CalendarBackend } from '$lib/server/calendar';
import { fail } from '@sveltejs/kit';
import { parseISO } from 'date-fns/fp';


/** @type {import('./$types').PageServerLoad<{ date: Date, events: TAllTypesWithId[], otherEvents: TAllTypesWithId[] }>} */
export const load = async ({ locals, url }) => {
	const queryDate = url.searchParams.get('date');
	const date = queryDate ? parseISO(queryDate) : new Date();

	const { backend } = locals;
	const events = backend.listDayEvent(date);
	const otherEvents = Promise.all(locals.session.data.calendars.map(c => {
		const back = new CalendarBackend({ type: 'oauth', ...c });
		return back.listDayEvent(date);
	}))

	return {
		date,
		otherEvents: (await otherEvents).flat(),
		events: await events,
	};
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