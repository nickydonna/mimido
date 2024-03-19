import { json } from '@sveltejs/kit';
import { parseISO } from 'date-fns';

/** @type {import('./$types').RequestHandler} */
export async function POST({ request, locals }) {
  const data = await request.json();

  const eventData = {
    ...data,
    date: data.date ? parseISO(data.date) : undefined,
    endDate: data.endDate ? parseISO(data.endDate) : undefined,
  }

  const event = await locals.backend.createEvent(eventData);

  return json(event, { status: 201 });
}