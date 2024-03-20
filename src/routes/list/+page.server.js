/** @typedef {import('../$types').Actions} Actions */
/** @typedef {import('$lib/server/calendar').TEventSchema} TEventSchema */


/** @type {import('./$types').PageServerLoad<{ events: TEventSchema[] }>} */
export const load = async ({ locals }) => {

	const { backend } = locals;
	const events = await backend.listTodos();

	return { events };
};
