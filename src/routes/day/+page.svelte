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
		ExclamationCircleOutline
	} from 'flowbite-svelte-icons';
	import { EType } from '$lib/parser/index';
	import {
		endOfDay,
		formatISO,
		getMinutes,
		isSameDay,
		roundToNearestMinutes,
		startOfDay
	} from 'date-fns';
	import { enhance } from '$app/forms';
	import EventCard from '$lib/components/event-card/event-card.svelte';
	import { getEventCardClass, isBlock, isEvent, isReminder, isTask, timeStore } from '$lib/util';
	import DetailModal from '$lib/components/details-modal/detail-modal.svelte';
	import { invalidateAll } from '$app/navigation';
	import { Modal } from 'flowbite-svelte';
	import { inview } from 'svelte-inview';

	/** @typedef {import('$lib/server/calendar').TAllTypesWithId} TAllTypesWithId */

	/** @type {import('./$types').PageData} */
	export let data;

	/** @type {TAllTypesWithId | undefined} */
	let selectedEvent;
	/** @type {string | undefined} */
	let idOfDeleting;
	let showDelete = false;
	let loading = false;
	/** @type {Date} */
	let current;
	/** @type {Array<{ time: Date, check: (d: Date) => boolean }>} */
	let timeBlocks;
	let showingToday = false;
	/** @type {Date} */
	let currentTime;
	/**
	 * Row style for the time indicator
	 * @type {{ neareastSlot: Date, offset: number }}
	 */
	let timeIndicator;
	timeStore.subscribe((storeTime) => {
		currentTime = storeTime;
		const neareastSlot = roundToNearestMinutes(storeTime, {
			nearestTo: 15,
			roundingMethod: 'floor'
		});
		const minutes = getMinutes(storeTime) - getMinutes(neareastSlot);
		timeIndicator = {
			neareastSlot,
			offset: (minutes * 100) / 15
		};
	});

	$: {
		showingToday = isSameDay(new Date(), data.date);
		current = startOfDay(data.date);
		let start = setHours(8, current);
		let end = setHours(23, current);
		let eachHour = eachHourOfInterval({ start, end })
			.map((d) => [d, addMinutes(15, d), addMinutes(30, d), addMinutes(45, d)])
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
	 * @param {TAllTypesWithId} event
	 * @param {(d: Date) => boolean} slotCheck
	 * @returns {boolean}
	 */
	let timeCheck = (event, slotCheck) => {
		if (!event.date) return false;

		return slotCheck(event.date)
	};

	/** 
	 * @template {import('$lib/server/calendar').TAllTypes} T
	 * @typedef {import('$lib/server/calendar').WithId<T>} WithId
	 */
	/** @typedef {import('$lib/server/calendar').TBlockSchema} TBlockSchema */
	/** @typedef {import('$lib/server/calendar').TTaskSchema} TTaskSchema */
	/** @typedef {import('$lib/server/calendar').TEventSchema} TEventSchema */
	/** @typedef {import('$lib/server/calendar').TReminderSchema} TReminderSchema */

	/** 
	 * @type {Array<
	 * 	[EType, Array<TAllTypesWithId>]
	 * >}
	 */
	let sortedEvents;

	$: {
		sortedEvents = [
			[EType.BLOCK, data.events.filter(isBlock)],
			[EType.EVENT, data.events.filter(isEvent)],
			[EType.TASK, data.events.filter(isTask)],
			[EType.REMINDER, data.events.filter(isReminder)],
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

	/** @param {TAllTypesWithId} e */
	function getScheduleSlot(e) {
		if (!e.date) return '';
		let endTime = e.endDate ?? addMinutes(15, e.date);
		//
		endTime = roundToNearestMinutes(endTime, { nearestTo: 15, roundingMethod: 'floor' });
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

		selectedEvent = /** @type {TAllTypesWithId} */ (await res.json());
		// TODO manage error
		loading = false;
		invalidateAll();
	};
	const modalZIndex = 40;


	let currentTimeInView = false;
	/** @type {import('svelte-inview').Options}*/
	const inviewOption = {
		rootMargin: "-50px"
	}
	
	/**
	 * @param {CustomEvent<import('svelte-inview').ObserverEventDetails>} event
	 */
  const handleViewChange = ({ detail }) => {
    currentTimeInView = detail.inView;
  };
	const scrollCurrentIntoView = () => {
		const el = document.getElementById('current-time');
		if (!el) return;

		el.scrollIntoView({
			block: 'center',
      behavior: 'smooth',
    });
	}
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
		{loading}
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
			style:z-index={modalZIndex - 2}
			class="track-slot text-center"
			aria-hidden="true"
			style="grid-column: event; grid-row: tracks;">Events</span
		>
		<span
			style:z-index={modalZIndex - 2}
			class="track-slot text-center"
			aria-hidden="true"
			style="grid-column: task; grid-row: tracks;">Tasks</span
		>
		<span
			style:z-index={modalZIndex - 2}
			class="track-slot text-center"
			aria-hidden="true"
			style="grid-column: reminder; grid-row: tracks;">Reminder</span
		>
		{#if showingToday}
			<!-- Blur time before current slot -->
			<div
				class="blurred-time pointer-events-none"
				style:z-index={modalZIndex - 4}
				style:grid-column="times / reminder"
				style:grid-row="time-{format('HHmm', timeBlocks[0].time)} / time-{format(
					'HHmm',
					timeIndicator.neareastSlot
				)}"
			/>
			<!-- Blur percentage time of current slot -->
			<div
				class="pointer-events-none"
				style:z-index={modalZIndex - 4}
				style:grid-column="times / reminder"
				style:grid-row="time-{format('HHmm', timeIndicator.neareastSlot)}"
			>
				<div class="blurred-time relative w-full" style:height="{timeIndicator.offset}%" />
			</div>
			{/if}
			{#if !currentTimeInView}
				<Button class="z-40 fixed end-6 bottom-[5rem]" on:click={scrollCurrentIntoView}>
					Go to Current Time
				</Button>
			{/if}
			<!-- Time indicator -->
			<div
				class="pointer-events-none"
				style:z-index={modalZIndex - 3}
				style:grid-column="times / reminder"
				style:grid-row="time-{format('HHmm', timeIndicator.neareastSlot)}"
			>
				<div class="relative w-full" style:top="calc({timeIndicator.offset}% - 25px)">
					<span class="relative px-2">
						{format('HH:mm', currentTime)}
					</span>
				</div>
			</div>
			<!-- Dotted line for current time -->
			<div
				use:inview={inviewOption} on:change={handleViewChange}
				id="current-time"
				class="pointer-events-none"
				style:z-index={modalZIndex - 3}
				style:grid-column="times / reminder"
				style:grid-row="time-{format('HHmm', timeIndicator.neareastSlot)}"
			>
				<div
					style:top="{timeIndicator.offset}%"
					class="relative w-full border-b-2 border-dotted border-gray-700"
				/>
			</div>

		{#each timeBlocks as { time, check } (time)}
			<h2 class="time-slot m-0.5 text-center text-xs" style:grid-row={`time-${format('HHmm', time)}`}>
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
						tabindex={i * 10 + k}
						role="button"
						class="{getEventCardClass(e)} group relative rounded-lg border p-0.5 shadow-2xl"
						class:m-px={type === EType.BLOCK}
						class:m-0.5={type !== EType.BLOCK}
						class:glass={type !== EType.BLOCK}
						style:grid-column={type === EType.BLOCK ? 'event / reminder' : type}
						style:grid-row={getScheduleSlot(e)}
						style:z-index={type === EType.BLOCK ? 0 : k}
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

	.blurred-time {
		background-color: rgba(255, 255, 255, 0.4);
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
		background-color: rgba(255, 255, 255, 0.9);
	}

	.time-slot {
		grid-column: times;
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
			[time-0815] 1fr
			[time-0830] 1fr
			[time-0845] 1fr
			[time-0900] 1fr
			[time-0915] 1fr
			[time-0930] 1fr
			[time-0945] 1fr
			[time-1000] 1fr
			[time-1015] 1fr
			[time-1030] 1fr
			[time-1045] 1fr
			[time-1100] 1fr
			[time-1115] 1fr
			[time-1130] 1fr
			[time-1145] 1fr
			[time-1200] 1fr
			[time-1215] 1fr
			[time-1230] 1fr
			[time-1245] 1fr
			[time-1300] 1fr
			[time-1315] 1fr
			[time-1330] 1fr
			[time-1345] 1fr
			[time-1400] 1fr
			[time-1415] 1fr
			[time-1430] 1fr
			[time-1445] 1fr
			[time-1500] 1fr
			[time-1515] 1fr
			[time-1530] 1fr
			[time-1545] 1fr
			[time-1600] 1fr
			[time-1615] 1fr
			[time-1630] 1fr
			[time-1645] 1fr
			[time-1700] 1fr
			[time-1715] 1fr
			[time-1730] 1fr
			[time-1745] 1fr
			[time-1800] 1fr
			[time-1815] 1fr
			[time-1830] 1fr
			[time-1845] 1fr
			[time-1900] 1fr
			[time-1915] 1fr
			[time-1930] 1fr
			[time-1945] 1fr
			[time-2000] 1fr
			[time-2015] 1fr
			[time-2030] 1fr
			[time-2045] 1fr
			[time-2100] 1fr
			[time-2115] 1fr
			[time-2130] 1fr
			[time-2145] 1fr
			[time-2200] 1fr
			[time-2215] 1fr
			[time-2230] 1fr
			[time-2245] 1fr
			[time-2300] 1fr
			[time-2315] 1fr
			[time-2330] 1fr
			[time-2345] 1fr
			[time-0000] 1fr;
	}
</style>
