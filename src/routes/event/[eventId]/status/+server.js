import { json } from '@sveltejs/kit';

/** @typedef {import('$lib/server/calendar').EStatus} EStatus */

export async function PUT({ request, params, locals }) {
  const { eventId } = params;
	const { status } = /** @type {{ status: EStatus }} */ (await request.json());

	await locals.backend.updateStatus(eventId, status);
	const event = await locals.backend.getEvent(eventId);

	return json(event, { status: 202 });
}