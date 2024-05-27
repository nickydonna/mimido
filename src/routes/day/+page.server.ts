import type { PageServerLoad, Actions } from './$types';
import { fail } from '@sveltejs/kit';
import { parseISO } from 'date-fns/fp';
import type { TAllTypesWithId } from '$lib/server/calendar';

export const load: PageServerLoad<{
	date: Date;
	events: TAllTypesWithId[];
	externalEvents: TAllTypesWithId[];
	tasks: TAllTypesWithId[];
}> = async ({ locals, url }) => {
	const queryDate = url.searchParams.get('date');
	const date = queryDate ? parseISO(queryDate) : new Date();

	const { backend, user } = locals;

	// await backend.smartSync();
	// Hardcoded timezoneOffset, move to user model
	const events = backend.listDayEvent(date, 180);

	const externalEvents = Promise.all(
		user.calendars.filter(c => c.type === 'extend-basic').map(async (c) => {
			// Hardcoded timezoneOffset, move to user model
			return backend.listExternalDayEvents(date, 180, c.id);
		})
	);

	return {
		date,
		externalEvents: (await externalEvents).flat(),
		events: await events,
		tasks: await backend.listTodos({ excludeDone: true })
	};
};

export const actions: Actions = {
	delete: async ({ request, locals }) => {
		const eventId = (await request.formData()).get('eventId');
		if (!eventId || typeof eventId !== 'string') {
			return fail(404, { error: 'Not Found' });
		}
		const { event } = await locals.backend.deleteEvent(eventId);
		return { success: true, event };
	}
};
