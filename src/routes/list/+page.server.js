/** @typedef {import('../$types').Actions} Actions */
/** @typedef {import('$lib/server/calendar').TAllTypesWithId} TAllTypesWithId */

import { fail } from '@sveltejs/kit';


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

/** @type {import('./$types').PageServerLoad<{ events: TAllTypesWithId[] }>} */
export const load = async ({ locals }) => {

	const { backend } = locals;
	const events = await backend.listTodos();

	return { events };
};
