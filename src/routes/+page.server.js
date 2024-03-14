import { EventEntity } from '$lib/server/db/event.entity';
import { client } from '$lib/server/db/connection';
import { v4 } from 'uuid';
import { setHours, setMinutes, setSeconds } from 'date-fns/fp';
import EntityManager from '$lib/server/db/entity-manager';
import { ValidationError } from 'yup';
import { error } from '@sveltejs/kit';

/** @typedef {import('./$types').Actions} Actions */
/** @typedef {import('$lib/server/db/event.entity.js').TEventSchema} TEventSchema */

/** @type {import('./$types').PageServerLoad<{ event: TEventSchema }>} */
export const load = async () => {

	const manager = new EntityManager(client);
	await manager.createTable(EventEntity, true);
	let event = await EventEntity.findById(manager, "Hllo");
	if (typeof event === 'undefined') {
		const nine = setSeconds(0, setMinutes(30, setHours(9, new Date())));
		const sixteen = setSeconds(0, setMinutes(0, setHours(16, new Date())));
		const nEvent = new EventEntity(v4(), "Hllo", nine, "something", sixteen, "09:30", "");
		await manager.create(nEvent);
		event = nEvent;
	}

	return {
		event: event.toPOJO(),
	};
};

export const actions = {
	create: async ({ request }) => {
		const manager = new EntityManager(client);
		try {
			const data = await request.formData();

			const event = await EventEntity.newFromForm(data);
			await manager.create(event);

			return { event };
		} catch(e) {
			if (e instanceof ValidationError) {
				error(400, e.errors.join(', '));
			}
		}
	}
}
