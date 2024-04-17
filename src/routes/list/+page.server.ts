import { fail } from '@sveltejs/kit';
import type { Actions } from './$types';
import type { PageServerLoad } from './$types';
import type { TAllTypesWithId } from '$lib/server/calendar';

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

export const load: PageServerLoad<{ events: TAllTypesWithId[] }> = async ({ locals }) => {
	const { backend } = locals;
	const events = await backend.listTodos();

	return { events };
};
