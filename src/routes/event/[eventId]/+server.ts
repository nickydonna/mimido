import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const DELETE: RequestHandler = async ({ params, locals }) => {
	const { eventId } = params;
	await locals.backend.deleteEvent(eventId);
	return json({ result: true}, { status: 202 });
};
