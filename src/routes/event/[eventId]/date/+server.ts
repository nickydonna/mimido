import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { parseISO } from 'date-fns';

export const PUT: RequestHandler = async ({ request, params, locals }) => {
	const { eventId } = params;
	const { from, to, postponing }: { from: string; to?: string; postponing?: boolean } =
		await request.json();

	// Get new id in case the type changed
	const { id } = await locals.backend.updateDate(
		eventId,
		parseISO(from),
		to ? parseISO(to) : undefined,
		postponing
	);
	const event = (await locals.backend.getEvent(id)).event;

	return json(event, { status: 202 });
};
