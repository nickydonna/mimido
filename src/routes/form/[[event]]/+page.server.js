import { parseTaskText } from '$lib/parser';
import { redirect } from '@sveltejs/kit';
import { formatISO } from 'date-fns';

/** @typedef {import('$lib/server/calendar').TEventSchema} TEventSchema */

/** @type {import('./$types').PageServerLoad<{event?: TEventSchema }>} */
export const load = async ({ locals, params }) => {
  const eventId = params.event;

  if (!eventId) {
    return { }
  }

  const event = await locals.backend.getEvent(eventId);
  return { event }
}

/** @type {import('./$types').Actions} */
export const actions = {
  default: async ({ request, locals, params }) => {
    const eventId = params.event; 
    const data = await request.formData();
		const originalText = /** @type {string} */ (data.get('originalText'));
		const description = /** @type {string | undefined} */ (data.get('description'));
    
    const date = new Date();

    const eventData = parseTaskText(originalText, date); 
    if (eventId) {
      await locals.backend.editEvent(eventId, eventData)
    } else {
      await locals.backend.createEvent({ ...eventData, description });
    }
   
    throw redirect(303, `/day?date=${formatISO(date)}`)
  }
}