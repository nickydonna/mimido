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
		ExclamationCircleOutline,
		TrashBinOutline
	} from 'flowbite-svelte-icons';
	import { EType } from '$lib/parser/index';
	import {
		endOfDay,
		formatISO,
		getMinutes,
		roundToNearestMinutes,
		startOfDay
	} from 'date-fns';
	import { enhance } from '$app/forms';
	import * as pkg from 'rrule';
	import EventCard from '$lib/components/event-card/event-card.svelte';
	import { getEventCardClass, timeStore } from '$lib/util';
	import DetailModal from '$lib/components/details-modal/detail-modal.svelte';
	import { invalidateAll } from '$app/navigation';
	import { Modal } from 'flowbite-svelte';
	// @ts-expect-error - see https://github.com/jkbrzt/rrule/issues/548
	const { RRule, rrulestr } = pkg.default || pkg;

	/** @enum {string} */
	const EDefaultEventStyle = {
		[EType.BLOCK]: 'bg-polka-indigo-600 border-indigo-600',
		[EType.EVENT]: 'bg-green-400 border-green-600',
		[EType.TASK]: 'bg-pink-400 border-pink-600',
		[EType.REMINDER]: 'bg-red-400 border-red-600'
	};

	/** @typedef {import('$lib/server/calendar').TEventSchema} TEventSchema */

	/** @type {import('./$types').PageData} */
	export let data;

	/** @type {TEventSchema | undefined} */
	let selectedEvent;
	/** @type {string | undefined} */
	let idOfDeleting;
	let showDelete = false;
	let loading = false;
	/** @type {Date} */
	let current;
	/** @type {Array<{ time: Date, check: (d: Date) => boolean }>} */
	let timeBlocks;
	
	/** @type {Date} */
	let currentTime;
	/**
	 * Row style for the time indicator
	 * @type {{ row: string, offset: number }}
	 */
	let timeIndicator
	timeStore.subscribe(storeTime => {
		currentTime = storeTime;
		const neareastSlot = roundToNearestMinutes(storeTime, { nearestTo: 30, roundingMethod: 'floor'})
		const minutes = getMinutes(storeTime) - getMinutes(neareastSlot);
		timeIndicator = {
			row: `time-${format('HHmm', neareastSlot)}`,
			offset: (minutes * 100) / 30
		}
	})

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
		showDelete = !!idOfDeleting;
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

	/** @type {import('./$types').SubmitFunction} */
	const onDelete = () => {
		loading = true;
		return async ({ update }) => {
			loading = false;
			idOfDeleting = undefined;
			selectedEvent = undefined;
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

	/** @typedef {import('$lib/parser').EStatus} EStatus */
	/** @param {CustomEvent<{ status: EStatus}>} event */
	const handleStatusChange = async (event) => {
		if (!selectedEvent) return;
		loading = true;
		const res = await fetch(`/event/${selectedEvent.eventId}/status`, {
			method: 'PUT',
			body: JSON.stringify({ status: event.detail.status })
		});

		selectedEvent = /** @type {TEventSchema} */ (await res.json());
		// TODO manage error
		loading = false;
		invalidateAll();
	};
</script>

<div>
	<div class="flex">
		<div class="flex-1">
			<p class="text-4xl dark:text-white">
				{format('do MMM yy ', data.date)}
			</p>
		</div>
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

	<DetailModal
		loading={loading}
		event={!idOfDeleting ? selectedEvent : undefined}
		on:close={(e) => (selectedEvent = undefined)}
		on:statuschange={handleStatusChange}
		on:delete={() => (idOfDeleting = selectedEvent?.eventId)}
	/>
	<Modal bind:open={showDelete} size="xs" on:close={() => (idOfDeleting = undefined)}>
		<div class="text-center">
			<ExclamationCircleOutline class="mx-auto mb-4 h-12 w-12 text-gray-400 dark:text-gray-200" />
			<h3 class="mb-5 text-lg font-normal text-gray-500 dark:text-gray-400">
				Are you sure you want to delete this event?
			</h3>
			<form class="inline-block" method="POST" action="?/delete" use:enhance={onDelete}>
				<input type="text" name="eventId" value={idOfDeleting} class="hidden" />
				<Button disabled={loading} color="red" class="me-2" type="submit">Yes, I'm sure</Button>
				<Button disabled={loading} on:click={() => (idOfDeleting = undefined)} color="alternative">
					No, cancel
				</Button>
			</form>
		</div>
	</Modal>

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
		<div
			style:z-index="10001"
			style:grid-column="times"
			style:grid-row={timeIndicator.row} 
		>
			<span
				style:top="{timeIndicator.offset}%"
				class="relative py-3 px-2 rounded-xl"
				style:box-shadow="0 4px 30px rgba(0, 0, 0, 0.1);"
				style:backdrop-filter="blur(1.5px)"
			>
				{format('HH:mm', currentTime)}
			</span>
		</div>
		<div
			style:z-index="10000"
			style:grid-column={'times / reminder'}
			style:grid-row={timeIndicator.row}
		>
			<div
				style:top="{timeIndicator.offset}%"
				class="relative border-b-2 border-dotted border-gray-700 w-full"
			/>
		</div>

		{#each timeBlocks as { time, check }, j (time)}
			<h2 class="time-slot text-center" style:grid-row={`time-${format('HHmm', time)}`}>
				{format('HH:mm', time)}
			</h2>
			{#each sortedEvents as [type, events], i}
				<div
					class="border-t border-dotted"
					class:border-gray-600={getMinutes(time) === 0}
					class:border-gray-300={getMinutes(time) === 30}
					style:grid-column={type}
					style:grid-row="time-{format('HHmm', time)}"
				></div>
				{#each events.filter((e) => timeCheck(e, check)) as e, k}
					<div
						tabindex={i + j + k}
						role="button"
						class="{getEventCardClass(e)} group relative rounded-lg border p-2 shadow-2xl"
						class:m-1={type === EType.BLOCK}
						class:m-2={type !== EType.BLOCK}
						class:glass={type !== EType.BLOCK}
						style:grid-column={type === EType.BLOCK ? 'event / reminder' : type}
						style:grid-row={getScheduleSlot(e)}
						style:z-index={type === EType.BLOCK ? 0 : i + k}
						on:click={() => (selectedEvent = e)}
						on:keypress={(event) => {
							if (event.code === 'Enter') selectedEvent = e;
						}}
					>
						<div class="absolute right-2 hidden group-hover:block"></div>
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
		backdrop-filter: blur(1.5px);
		-webkit-backdrop-filter: blur(1.5px);
		border: 1px solid rgba(255, 255, 255, 0.3);
	}

	.card__bg-work {
		background-position: center;
		background-image: url('$lib/assets/work.jpg');
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
