<script>
	import {
		addMinutes,
		eachHourOfInterval,
		format,
		getHours,
		isWithinInterval,
		parse,
		setHours,
		setMinutes,
		setSeconds,
		subSeconds
	} from 'date-fns/fp';
	import Event from '$lib/components/event/index.js';
	import TaskBox from '$lib/components/task-box/task-box.svelte';
	import Card from 'flowbite-svelte/Card.svelte'


	/** @typedef {import('$lib/server/schemas/event.js').TEventSchema} TEventSchema */

	/** @type {import('./$types').PageData} */
	export let data;

	let now = setSeconds(0, new Date());
	let setZeroMin = setMinutes(0);
	let setStartHour = setHours(8);
	let setEndHour = setHours(24);
	let start = setZeroMin(setStartHour(now));
	let end = setZeroMin(setEndHour(now));
	let dates = eachHourOfInterval({start, end})

	/**
	 * @param {number} startHour
	 * @param {number} minOffset
	 * @param {TEventSchema} event
	 * @returns {boolean}
	 */
	let timeCheck = (startHour, minOffset, event) => {
		if (!event.date || !event.hasStartTime) {
			return false;
		}
		const low = subSeconds(1, setMinutes(minOffset, setHours(startHour, now)));
		const high = subSeconds(1, setMinutes(minOffset + 30, setHours(startHour, now)))
		return isWithinInterval({
			start: low,
			end: high
		}, event.date);
	}

	let events = data.events
</script>
<div>
	<Card size='lg'>
		<TaskBox />
	</Card>

	<table class="w-full my-5">
		<colgroup>
			<col class="w-16 text-right border-r" >
		</colgroup>
		<caption class="caption-top">{format('LLL yyyy', now)}</caption>
		<tbody>
		<tr>
			<td></td>
			<td>
					{format('do', now)}
			</td>
		</tr>
		{#each dates as time}
			<tr class="border-b h-8 p-0">
				<td>{format('HH:mm', time)}</td>
				<td class="p-0">
					{#each events.filter(e => timeCheck(getHours(time), 0, e)) as e}
						<Event event={e} />
					{/each}
				</td>
			</tr>
			<tr class="border-b border-gray-400 h-8">
				<td class="text-white">{format('HH:mm', addMinutes(30, time))}</td>
				<td class="p-0">
					{#each events.filter(e => timeCheck(getHours(time), 30, e)) as e}
						<Event event={e} />
					{/each}
				</td>
			</tr>
		{/each}
		</tbody>
	</table>

</div>

