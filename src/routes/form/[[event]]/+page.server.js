import { parseTaskText } from '$lib/parser';
import { redirect, fail } from '@sveltejs/kit';
import { formatISO } from 'date-fns';
import yup from 'yup';

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
    
    const eventData = parseTaskText(originalText); 
    
    try {
      const valid = await locals.backend.validateEventData(eventData)
      if (!valid)
        if (eventId) {
          await locals.backend.editEvent(eventId, eventData)
        } else {
          await locals.backend.createEvent({ ...eventData, description });
        }
    } catch (e) {
      if (e instanceof yup.ValidationError) {
        return fail(400, { originalText, description, errors: e.errors })
      }
      throw e;
    }
   
    throw redirect(303, `/day`)
  }
}