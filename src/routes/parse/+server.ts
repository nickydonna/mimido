import { OPENAI_KEY } from '$env/static/private';
import { alarmFromString } from '$lib/parser/index.js';
import { json } from '@sveltejs/kit';
import {
	subMinutes,
	differenceInDays,
	parseISO,
	startOfDay,
	addDays,
	setMinutes,
	setHours,
	getMinutes,
	getHours
} from 'date-fns/fp';
import OpenAi from 'openai';

const openai = new OpenAi({
	apiKey: OPENAI_KEY
});

/** @type {import('./$types').RequestHandler} */
export async function POST({ request }) {
	const { prompt } = /** @type {{ prompt: string, offset: number }} */ await request.json();

	const thread = await openai.beta.threads.create();

	await openai.beta.threads.messages.create(thread.id, { role: 'user', content: prompt });
	const run = openai.beta.threads.runs.stream(thread.id, {
		assistant_id: 'asst_wWo8F5NQrkyUqP4SXow79tgF'
	});
	return new Promise((resolve) => {
		run.on('messageDone', (message) => {
			const content = message.content[0] as OpenAi.Beta.Threads.TextContentBlock;
			if (!content) resolve(json({}, { status: 204 }));
			const event = JSON.parse(content.text.value);
			const aiCtxDay = parseISO(event.now);
			if (event.date) {
				// AI run on a different date, so we ask for their context and get the offset from the current date
				const diff = differenceInDays(startOfDay(aiCtxDay), startOfDay(new Date()));
				// We add the days to the date, and offset by the timezone
				const aiDate = parseISO(event.date);
				let date = addDays(diff, aiDate);
				date = setHours(getHours(date), date);
				date = setMinutes(getMinutes(date), date);
				event.date = subMinutes(0, date);
				if (event.endDate) {
					const aiDate = parseISO(event.endDate);
					let date = addDays(diff, aiDate);
					date = setHours(getHours(date), date);
					date = setMinutes(getMinutes(date), date);
					event.endDate = subMinutes(0, date);
				}
			}

			event.alarms = event.alarms.map(alarmFromString);

			return resolve(json(event, { status: 202 }));
		});
	});
}
