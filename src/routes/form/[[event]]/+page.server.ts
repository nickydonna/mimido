import { parseTaskText } from '$lib/parser/index.js';
import { fail } from '@sveltejs/kit';
import yup from 'yup';
import type { PageServerLoad, Actions } from './$types';
import type { TAllTypesWithId } from '$lib/server/calendar';

export const load: PageServerLoad<{ event?: TAllTypesWithId }> = async ({ locals, params }) => {
	const eventId = params.event;

	if (!eventId) {
		return {};
	}

	const event = (await locals.backend.getEvent(eventId)).event;
	return { event };
};

export const actions: Actions = {
	// TOD return new event
	default: async ({ request, locals, params }) => {
		const eventId = params.event;
		const data = await request.formData();
		const originalText = data.get('originalText') as string;
		const description = data.get('description') as string | undefined;

		const eventData = {
			...parseTaskText(originalText),
			description
		};

		/** @type {{ id: string }} */
		let result;

		try {
			const valid = await locals.backend.validateEventData(eventData);
			if (valid) {
				if (eventId) {
					result = await locals.backend.editEvent(eventId, valid);
				} else {
					result = await locals.backend.createEvent(valid);
				}
			} else {
				throw new Error('No valid object created');
			}
		} catch (e) {
			if (e instanceof yup.ValidationError) {
				return fail(400, { originalText, description, errors: e.errors });
			}
			throw e;
		}
		const fetchRes = await locals.backend.getEvent(result.id);

		return { success: true, event: fetchRes.event };
	}
};
