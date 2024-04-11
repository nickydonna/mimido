import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const PUT: RequestHandler = async ({ request, params, locals }) => {
	const { eventId } = params;
	const { status } = /** @type {{ status: EStatus }} */ await request.json();

	await locals.backend.updateStatus(eventId, status);
	const event = (await locals.backend.getEvent(eventId)).event;

	return json(event, { status: 202 });
};
