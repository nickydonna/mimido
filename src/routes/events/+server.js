import { json } from '@sveltejs/kit';
import { Event } from '$lib/server/schemas/event';
import { parseISO } from 'date-fns';

/** @type {import('./$types').RequestHandler} */
export async function POST({ request }) {
  const data = await request.json();
  console.log(data);

  const eventData = {
    ...data,
    date: data.date ? parseISO(data.date) : undefined,
    endDate: data.endDate ? parseISO(data.endDate) : undefined,
  }

  const event = await Event.create(eventData);
  return json(event, { status: 201 });
}