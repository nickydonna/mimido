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
	import { AngleLeftOutline, AngleRightOutline, EditOutline, TrashBinOutline } from 'flowbite-svelte-icons';
	import { EType } from '$lib/parser/index';
	import { formatDuration, formatISO, getMinutes, isAfter, roundToNearestMinutes, startOfDay } from 'date-fns';
	import { enhance } from '$app/forms';

	/** @enum {string} */
	const EEventStyle = {
		[EType.BLOCK]: 'bg-amber-400 border-amber-600 bg-opacity-50',
		[EType.EVENT]: 'bg-green-400 border-green-600 bg-opacity-75',
		[EType.TASK]: 'bg-pink-400 border-pink-600 bg-opacity-75',
		[EType.REMINDER]: 'bg-red-400 border-red-600 bg-opacity-75',
	}

	/** @typedef {import('$lib/server/calendar').TEventSchema} TEventSchema */

	/** @type {import('./$types').PageData} */
	export let data;

	/** @type {Date} */
	let current;
	/** @type {Date[]} */
	let dates;
	$: {
		current = startOfDay(data.date);
		let start = setHours(8, current);
		let end = setHours(23, current);
		dates = eachHourOfInterval({ start, end }).map(d => [d, addMinutes(30, d)]).flat();
	}

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

	let loading = false;

	  /** @type {import('./$types').SubmitFunction} */
  const onDelete = () => {
    loading = true;
    return async ({ update }) => {
      loading = false;
      update();
    }
  }

	/** @param {TEventSchema} e */
	function getScheduleSlot(e) {
		if (!e.date) return '';
		let endTime = e.endDate ?? addMinutes(30, e.date);
		// 
		endTime = roundToNearestMinutes(endTime, { nearestTo: 30 })
		return `time-${format('HHmm', e.date)} / time-${format('HHmm', endTime)}`
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
		console.log('drag')
		dragData = {type, start: time, end: addMinutes(30, time)}
	}

	/**
	 * @param {Date} time
	 * @param {EType} type
	 */
	const handleDragEnd = (time, type) => /** @param {MouseEvent} event */ (event) => {
		dragData = undefined;
	}

		/**
	 * @param {Date} time
	 * @param {EType} type
	 */
	const handleDragEnter = (time, type) => /** @param {MouseEvent} event */ (event) => {
		if (!dragData) return;
		if (type != dragData.type || isAfter(dragData.start, time)) return;

		dragData = {...dragData, end: time}
	}

		/**
	 * @param {Date} time
	 * @param {EType} type
	 */
	const handleDrop = (time, type) => /** @param {MouseEvent} event */ (event) => {
		// Create Evente
	}
</script>

<div>
	<div class="flex">
		<div class="flex-1"></div>
		<ButtonGroup>
			<Button href="/day?date={formatISO(subDays(1, startOfDay(data.date)))}">
				<AngleLeftOutline />
			</Button>
			<Button href="/day">
				Today
			</Button>
			<Button href="/day?date={formatISO(addDays(1, startOfDay(data.date)))}">
				<AngleRightOutline />
			</Button>
		</ButtonGroup>
	</div>


	<div class="schedule">
		<span class="track-slot text-center" aria-hidden="true" style="grid-column: event; grid-row: tracks;">Events</span>
		<span class="track-slot text-center" aria-hidden="true" style="grid-column: task; grid-row: tracks;">Tasks</span>
		<span class="track-slot text-center" aria-hidden="true" style="grid-column: reminder; grid-row: tracks;">Reminder</span>
		{#if dragData}
			<div
				class="bg-slate-800" 
				style:grid-column={dragData.type} 
				style:grid-row="time-{format('HHmm', dragData.start)} / time-{format('HHmm', dragData.end)}"
				draggable="true"
			></div>	
		{/if}
		{#each dates as time, j}
			<h2 class="time-slot" style:grid-row={`time-${format('HHmm', time)}`}>{format('HH:mm', time)}</h2>
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
				{#each events.filter(e => timeCheck(time, e)) as e}
					<div
						class="{EEventStyle[type]} relative p-2 rounded-md shadow-2xl border group" 
						style:grid-column={type === EType.BLOCK ? "event / reminder" : type}
						style:grid-row={getScheduleSlot(e)}>
						<div class="absolute right-2 hidden group-hover:block">
							 <Button href="/form/{e.eventId}" color="none" pill={true} outline={true} class="!p-1" size="xl">
									<EditOutline />
  						</Button>
							<form class="inline-block" method="POST" action="?/delete" use:enhance={onDelete}>
      			    <input type="text" name="eventId" value={e.eventId} class="hidden">
			          <Button disabled={loading} class="!p-1" size="xs" color="red" type="submit">
									<TrashBinOutline />
								</Button>
        			</form>
						</div>
						{#if e.type === EType.BLOCK}
							<div class="h-full flex flex-col justify-center items-center">
								<p class="inline-block text-amber-900 font-medium text-2xl opacity-65">
								{e.title.toUpperCase()}
								</p>
							</div>
						{:else}
							<div>
								{e.title}
							</div>
							{#if e.alarm}
								<div>
									Alarm:
									{formatDuration({...e.alarm.duration}, { format: ['days', 'hours', 'minutes']})}
									{e.alarm.isNegative ? 'before' : 'after'}
								</div>
							{/if}
						{/if}
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
