<script>
	import {
		addMinutes,
		eachHourOfInterval,
		format,
		getHours,
		isWithinInterval,
		setHours,
		setMinutes,
		setSeconds,
		subSeconds
	} from 'date-fns/fp';
	import Event from '$lib/components/event/index.js';
	import TaskBox from '$lib/components/task-box/task-box.svelte';
	import Card from 'flowbite-svelte/Card.svelte';
	import { EType } from '$lib/components/task-box/parser';

	/** @enum {string} */
	const EEventStyle = {
		[EType.BLOCK]: 'bg-amber-400 border-amber-600',
		[EType.EVENT]: 'bg-green-400 border-green-600',
		[EType.TASK]: 'bg-pink-400 border-pink-600',
		[EType.REMINDER]: 'bg-red-400 border-red-600',
	}

	/** @typedef {import('$lib/server/calendar').TEventSchema} TEventSchema */

	/** @type {import('./$types').PageData} */
	export let data;

	let now = setSeconds(0, new Date());
	let setZeroMin = setMinutes(0);
	let setStartHour = setHours(8);
	let setEndHour = setHours(23);
	let start = setZeroMin(setStartHour(now));
	let end = setZeroMin(setEndHour(now));
	let dates = eachHourOfInterval({ start, end }).map(d => [d, addMinutes(30, d)]).flat();

	/**
	 * @param {Date} startHour
	 * @param {TEventSchema} event
	 * @returns {boolean}
	 */
	let timeCheck = (startHour, event) => {
		if (!event.date) {
			return false;
		}
		return isWithinInterval(
			{
				start: subSeconds(1, startHour),
				end: subSeconds(1, addMinutes(30, startHour))
			},
			event.date
		);
	};

	/** @type {Array<[EType, TEventSchema[]]>} */
	let sortedEvents;

	$: {
		sortedEvents = 
			[
				[EType.BLOCK, data.events.filter((e) => e.type === EType.BLOCK)],
				[EType.EVENT, data.events.filter((e) => e.type === EType.EVENT)],
				[EType.TASK, data.events.filter((e) => e.type === EType.TASK)],
				[EType.REMINDER, data.events.filter((e) => e.type === EType.REMINDER)]
			]
	}

	/** @param {TEventSchema} e */
	function getScheduleSlot(e) {
		if (!e.date) return '';
		const endTime = e.endDate ?? addMinutes(30, e.date);
		return `time-${format('HHmm', e.date)} / time-${format('HHmm', endTime)}`
	}
</script>

<div>
	<Card size="lg">
		<TaskBox />
	</Card>

	<div class="schedule">
		<span class="track-slot" aria-hidden="true" style="grid-column: block; grid-row: tracks;">Block</span>
		<span class="track-slot" aria-hidden="true" style="grid-column: event; grid-row: tracks;">Events</span>
		<span class="track-slot" aria-hidden="true" style="grid-column: task; grid-row: tracks;">Tasks</span>
		<span class="track-slot" aria-hidden="true" style="grid-column: reminder; grid-row: tracks;">Reminder</span>
		{#each dates as time}
			<h2 class="time-slot" style:grid-row={`time-${format('HHmm', time)}`}>{format('HH:mm', time)}</h2>
			{#each sortedEvents as [type, events]}
				{#each events.filter(e => timeCheck(time, e)) as e}
					<div
						class={`${EEventStyle[type]} p-1 rounded-md shadow-2xl border bg-opacity-75`} 
						style:grid-column={type}
						style:grid-row={getScheduleSlot(e)}>
						{e.title}
					</div>	
				{/each}
			{/each}
		{/each}
	</div>
</div>



<style>
	/** Taken from https://css-tricks.com/building-a-conference-schedule-with-css-grid/ */
	.track-slot {
			display: block;
			padding: 10px 5px 5px;
			position: sticky;
			top: 0;
			z-index: 1000;
			background-color: rgba(255,255,255,.9);
		}
	.time-slot {
		grid-column: times;
	}
	.schedule {
		margin: 20px 0;
		display: grid;
		grid-gap: 1em;
		grid-template-columns:
			[times] 4em
			[block-start] 1fr
			[block-end event-start] 1fr
			[event-end task-start] 1fr
			[task-end reminder-start] 1fr
			[reminder-end];
		grid-template-rows:
			[tracks] auto
			[time-0800] 1fr
			[time-0830] 1fr
			[time-0900] 1fr
			[time-0930] 1fr
			[time-1000] 1fr
			[time-1030] 1fr
			[time-1100] 1fr
			[time-1130] 1fr
			[time-1200] 1fr
			[time-1230] 1fr
			[time-1300] 1fr
			[time-1330] 1fr
			[time-1400] 1fr
			[time-1430] 1fr
			[time-1500] 1fr
			[time-1530] 1fr
			[time-1600] 1fr
			[time-1630] 1fr
			[time-1700] 1fr
			[time-1730] 1fr
			[time-1800] 1fr
			[time-1830] 1fr
			[time-1900] 1fr
			[time-1930] 1fr
			[time-2000] 1fr
			[time-2030] 1fr
			[time-2100] 1fr
			[time-2130] 1fr
			[time-2200] 1fr
			[time-2230] 1fr
			[time-2300] 1fr
			[time-2330] 1fr
			[time-0000] 1fr;
	}
</style>