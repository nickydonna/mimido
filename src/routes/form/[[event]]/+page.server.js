import { parseTaskText } from '$lib/parser';
import { redirect, fail } from '@sveltejs/kit';
import yup from 'yup';

/** @typedef {import('$lib/server/calendar').TAllTypesWithId} TAllTypesWithId */

/** @type {import('./$types').PageServerLoad<{event?: TAllTypesWithId }>} */
export const load = async ({ locals, params }) => {
  const eventId = params.event;

  if (!eventId) {
    return { }
  }

  const event = (await locals.backend.getEvent(eventId)).event;
  return { event }
}

/** @type {import('./$types').Actions} */
export const actions = {
  default: async ({ request, locals, params }) => {
    const eventId = params.event; 
    const data = await request.formData();
		const originalText = /** @type {string} */ (data.get('originalText'));
		const description = /** @type {string | undefined} */ (data.get('description'));
    
    const eventData = {
      ...parseTaskText(originalText),
      description,
    };
    
    try {
      const valid = await locals.backend.validateEventData(eventData)
      if (valid) {
        if (eventId) {
          await locals.backend.editEvent(eventId, valid)
        } else {
          await locals.backend.createEvent(valid);
        }
      } else {
        throw new Error('No valid object created');
      }
    } catch (e) {
      if (e instanceof yup.ValidationError) {
        return fail(400, { originalText, description, errors: e.errors })
      }
      console.log(e);
      throw e;
    }
   
    throw redirect(303, `/day`)
  }
}