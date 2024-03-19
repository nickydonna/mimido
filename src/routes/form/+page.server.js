import { parseTaskText } from '$lib/parser';
import { redirect } from '@sveltejs/kit';
import { parseISO, formatISO } from 'date-fns';

/** @type {import('./$types').Actions} */
export const actions = {
  default: async ({ request, locals }) => {
    const data = await request.formData();
		const originalText = /** @type {string} */ (data.get('originalText'));
		const description = /** @type {string | undefined} */ (data.get('description'));
    const queryDate = /** @type {string | undefined} */ (data.get('date'))
    
    const date = queryDate ? parseISO(queryDate) : new Date();

    const eventData = parseTaskText(originalText, date); 
    await locals.backend.createEvent({ ...eventData, description });
   
    throw redirect(303, `/day?date=${formatISO(date)}`)
  }
}