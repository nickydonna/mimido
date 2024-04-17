import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const PUT: RequestHandler = async ({ params, locals }) => {
	const { eventId } = params;

	// Get new id in case the type changed
	const { id } = await locals.backend.removeDate(eventId);
	const event = (await locals.backend.getEvent(id)).event;

	return json(event, { status: 202 });
};
