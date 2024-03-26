<script>
	import {
		addMinutes,
		eachHourOfInterval,
		format,
		isWithinInterval,
		setHours,
		subDays,
		addDays,
		subSeconds
	} from 'date-fns/fp';
	import Button from 'flowbite-svelte/Button.svelte';
	import ButtonGroup from 'flowbite-svelte/ButtonGroup.svelte';
	import {
		AngleLeftOutline,
		AngleRightOutline,
		EditOutline,
		TrashBinOutline
	} from 'flowbite-svelte-icons';
	import { EType } from '$lib/parser/index';
	import {
		endOfDay,
		formatISO,
		getMinutes,
		isAfter,
		roundToNearestMinutes,
		startOfDay
	} from 'date-fns';
	import { enhance } from '$app/forms';
	import workImg from '$lib/assets/work.jpg';
	import * as pkg from 'rrule';
	import EventCard from '$lib/components/event-card/event-card.svelte';
	// @ts-expect-error - see https://github.com/jkbrzt/rrule/issues/548
	const { RRule, rrulestr } = pkg.default || pkg;

	/** @enum {string} */
	const EEventStyle = {
		[EType.BLOCK]: 'bg-blue-400 border-blue-600',
		[EType.EVENT]: 'bg-green-400 border-green-600 bg-opacity-45',
		[EType.TASK]: 'bg-pink-400 border-pink-600 bg-opacity-45',
		[EType.REMINDER]: 'bg-red-400 border-red-600 bg-opacity-45',
	}

	/** @typedef {import('$lib/server/calendar').TEventSchema} TEventSchema */

	/** @type {import('./$types').PageData} */
	export let data;

	/** @type {string | undefined} */
	let deleting;
	/** @type {Date} */
	let current;
	/** @type {Array<{ time: Date, check: (d: Date) => boolean }>} */
	let timeBlocks;
	$: {
		current = startOfDay(data.date);
		let start = setHours(8, current);
		let end = setHours(23, current);
		let eachHour = eachHourOfInterval({ start, end })
			.map((d) => [d, addMinutes(30, d)])
			.flat();
		timeBlocks = eachHour.map((h) => ({
			time: h,
			// TODO maybe memo this?
			check:
				/** @type {(d: Date) => boolean} */
				(isWithinInterval({ start: subSeconds(1, h), end: subSeconds(1, addMinutes(30, h)) }))
		}));
	}

	/**
	 * @param {TEventSchema} event
	 * @param {(d: Date) => boolean} timeCheck
	 * @returns {boolean}
	 */
	let timeCheck = (event, timeCheck) => {
		if (!event.date) return false;

		let recurRule = event.recur ? rrulestr(event.recur) : undefined;
		let checkDates = recurRule?.between(current, endOfDay(current)) ?? [];
		checkDates = [...checkDates, event.date];
		return checkDates.some(timeCheck);
	};

	/** @type {Array<[EType, TEventSchema[]]>} */
	let sortedEvents;

	$: {
		sortedEvents = [
			[EType.BLOCK, data.events.filter((e) => e.type === EType.BLOCK)],
			[EType.EVENT, data.events.filter((e) => e.type === EType.EVENT)],
			[EType.TASK, data.events.filter((e) => e.type === EType.TASK)],
			[EType.REMINDER, data.events.filter((e) => e.type === EType.REMINDER)]
		];
	}

	let loading = false;

	/** @type {import('./$types').SubmitFunction} */
	const onDelete = () => {
		loading = true;
		return async ({ update }) => {
			loading = false;
			update();
		};
	};

	/** @param {TEventSchema} e */
	function getScheduleSlot(e) {
		if (!e.date) return '';
		let endTime = e.endDate ?? addMinutes(30, e.date);
		//
		endTime = roundToNearestMinutes(endTime, { nearestTo: 30 });
		return `time-${format('HHmm', e.date)} / time-${format('HHmm', endTime)}`;
	}

	/**
	 * @param {TEventSchema} event
	 * @return 
	 */
	function getImageBg(event) {
		const { tag } = event;
		if (event.type !== EType.BLOCK) return '';
		const lcTags = tag.map(t => t.toLowerCase())
		if (lcTags.includes('bgwork')) {
			console.log( `background-image: ${workImg};`)
			return `url(${workImg})`;
		}
	}

	/**
	 * Tuple of type, start time, end time
	 * @type {{ type: EType, start: Date, end: Date} | undefined}
	 */
	let dragData;

	/**
	 * @param {Date} time
	 * @param {EType} type
	 */
	const handleDragStart = (time, type) => /** @param {MouseEvent} event */ (event) => {
		dragData = { type, start: time, end: addMinutes(30, time) };
	};

	/**
	 * @param {Date} time
	 * @param {EType} type
	 */
	const handleDragEnd = (time, type) => /** @param {MouseEvent} event */ (event) => {
		dragData = undefined;
	};

	/**
	 * @param {Date} time
	 * @param {EType} type
	 */
	const handleDragEnter = (time, type) => /** @param {MouseEvent} event */ (event) => {
		if (!dragData) return;
		if (type != dragData.type || isAfter(dragData.start, time)) return;

		dragData = { ...dragData, end: time };
	};

	/**
	 * @param {Date} time
	 * @param {EType} type
	 */
	const handleDrop = (time, type) => /** @param {MouseEvent} event */ (event) => {
		// Create Event
	};
</script>

<div>
	<div class="flex">
		<div class="flex-1"></div>
		<ButtonGroup>
			<Button href="/day?date={formatISO(subDays(1, startOfDay(data.date)))}">
				<AngleLeftOutline />
			</Button>
			<Button href="/day">Today</Button>
			<Button href="/day?date={formatISO(addDays(1, startOfDay(data.date)))}">
				<AngleRightOutline />
			</Button>
		</ButtonGroup>
	</div>

	<div class="schedule">
		<span
			class="track-slot text-center"
			aria-hidden="true"
			style="grid-column: event; grid-row: tracks;">Events</span
		>
		<span
			class="track-slot text-center"
			aria-hidden="true"
			style="grid-column: task; grid-row: tracks;">Tasks</span
		>
		<span
			class="track-slot text-center"
			aria-hidden="true"
			style="grid-column: reminder; grid-row: tracks;">Reminder</span
		>
		{#if dragData}
			<div
				class="bg-slate-800"
				style:grid-column={dragData.type}
				style:grid-row="time-{format('HHmm', dragData.start)} / time-{format('HHmm', dragData.end)}"
				draggable="true"
			></div>
		{/if}
		{#each timeBlocks as { time, check }, j (time)}
			<h2 class="time-slot" style:grid-row={`time-${format('HHmm', time)}`}>
				{format('HH:mm', time)}
			</h2>
			{#each sortedEvents as [type, events], i}
				<div
					tabindex={i + j}
					role="cell"
					class="border-t border-dotted"
					class:border-gray-600={getMinutes(time) === 0}
					class:border-gray-300={getMinutes(time) === 30}
					style:grid-column={type}
					style:grid-row="time-{format('HHmm', time)}"
					draggable="true"
					on:dragstart={handleDragStart(time, type)}
					on:dragend={handleDragEnd(time, type)}
					on:dragenter={handleDragEnter(time, type)}
					on:drop={handleDrop(time, type)}
				></div>
				{#each events.filter((e) => timeCheck(e, check)) as e, k}
					<div
						class="{EEventStyle[type]} group relative rounded-lg border p-2 shadow-2xl"
						class:m-1={type === EType.BLOCK}
						class:m-2={type !== EType.BLOCK}
						class:glass={type !== EType.BLOCK}
						style:grid-column={type === EType.BLOCK ? 'event / reminder' : type}
						style:grid-row={getScheduleSlot(e)}
						style:z-index={type === EType.BLOCK ? 0 : i + k}
						style:background-image={getImageBg(e)}
						style:background-position="center"
					>
						<div class="absolute right-2 hidden group-hover:block">
							<Button
								href="/form/{e.eventId}"
								color="none"
								pill={true}
								outline={true}
								class="!p-1"
								size="xs"
							>
								<EditOutline />
							</Button>
							<form class="inline-block" method="POST" action="?/delete" use:enhance={onDelete}>
								<input type="text" name="eventId" value={e.eventId} class="hidden" />
								{#if deleting !== e.eventId}
									<Button
										disabled={loading}
										class="!p-1"
										size="xs"
										color="red"
										type="button"
										on:click={() => (deleting = e.eventId)}
									>
										<TrashBinOutline />
									</Button>
								{:else}
									<div class="flex">
										<Button
											disabled={loading}
											class="mr-1 !p-1"
											size="xs"
											type="button"
											on:click={() => (deleting = undefined)}
										>
											Cancel
										</Button>
										<Button disabled={loading} class="!p-1" size="xs" color="red" type="submit">
											<TrashBinOutline />
											Delete
										</Button>
									</div>
								{/if}
							</form>
						</div>
						{#if e.type === EType.BLOCK}
							<div class="flex h-full flex-col items-center justify-center">
								<p class="inline-block text-2xl font-medium text-blue-900 opacity-65">
									{e.title.toUpperCase()}
								</p>
							</div>
						{:else}
							<EventCard event={e} />
						{/if}
					</div>
				{/each}
			{/each}
		{/each}
	</div>
</div>

<style>
	.glass {
		/* From https://css.glass */
		/* background: rgba(255, 255, 255, 0.47); */

		box-shadow: 0 4px 30px rgba(0, 0, 0, 0.1);
		backdrop-filter: blur(5.4px);
		-webkit-backdrop-filter: blur(5.4px);
		border: 1px solid rgba(255, 255, 255, 0.3);
	}
	/** Taken from https://css-tricks.com/building-a-conference-schedule-with-css-grid/ */
	.track-slot {
		display: block;
		padding: 10px 5px 5px;
		position: sticky;
		top: 0;
		z-index: 1000;
		background-color: rgba(255, 255, 255, 0.9);
	}
	.time-slot {
		grid-column: times;
		padding: 0.4em 0.2em 0.4em 0;
		margin-right: 0.5em;
		border-right: 1px solid gray;
	}
	.schedule {
		margin: 20px 0;
		display: grid;
		grid-template-columns:
			[times] 4em
			[event-start] 1fr
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
