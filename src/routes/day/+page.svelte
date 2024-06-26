<script lang="ts">
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
	import { AngleLeftOutline, AngleRightOutline, CloseOutline } from 'flowbite-svelte-icons';
	import { EType } from '$lib/parser/index.js';
	import {
		formatISO,
		getMinutes,
		isSameDay,
		isSameMinute,
		roundToNearestMinutes,
		startOfDay
	} from 'date-fns';
	import EventCard from '$lib/components/event-card/event-card.svelte';
	import TaskList from '$lib/components/task-list';
	import {
		getEventCardClass,
		isBlock,
		isDefined,
		isDone,
		isEvent,
		isReminder,
		isTask,
		timeStore
	} from '$lib/util.js';
	import { goto, invalidateAll } from '$app/navigation';
	import { Modal, Spinner } from 'flowbite-svelte';
	import { inview } from 'svelte-inview';
	import type { Options, ObserverEventDetails } from 'svelte-inview';
	import type { PageData } from './$types';
	import type { TAllTypesWithId } from '$lib/server/calendar';

	import { isLoading, loading, upsert, selectedEvent } from '$lib/stores';
	import { page } from '$app/stores';

	export let data: PageData;

	let dragging: TAllTypesWithId | undefined;
	let hoverTime: Date | undefined;
	let current: Date;
	let timeBlocks: Array<{ time: Date; check: (d: Date) => boolean }>;
	let showingToday = false;
	let currentTime: Date;
	/**
	 * Row style for the time indicator
	 */
	let timeIndicator: { nearestSlot: Date; offset: number };
	timeStore.subscribe((storeTime) => {
		currentTime = storeTime;
		const nearestSlot = roundToNearestMinutes(storeTime, {
			nearestTo: 15,
			roundingMethod: 'floor'
		});
		const minutes = getMinutes(storeTime) - getMinutes(nearestSlot);
		timeIndicator = {
			nearestSlot: nearestSlot,
			offset: (minutes * 100) / 15
		};
	});
	let tags: string[] = [];
	let tagFilter: string | undefined;
	let tasks = data.tasks;
	$: {
		tags = [...new Set(data.tasks.map((e) => e.tags).flat())];
		tagFilter = $page.url.searchParams.get('tag') ?? undefined;
		if (isDefined(tagFilter)) {
			tasks = data.tasks.filter((e) => e.tags.includes(tagFilter as string));
		} else {
			tasks = data.tasks;
		}

		showingToday = isSameDay(new Date(), data.date);
		current = startOfDay(data.date);
		let start = setHours(8, current);
		let end = setHours(23, current);
		let eachHour = eachHourOfInterval({ start, end })
			.map((d) => [d, addMinutes(15, d), addMinutes(30, d), addMinutes(45, d)])
			.flat();
		timeBlocks = eachHour.map((h) => ({
			time: h,
			check: isWithinInterval({ start: subSeconds(1, h), end: subSeconds(1, addMinutes(30, h)) })
		}));
	}

	let timeCheck = (event: TAllTypesWithId, slotCheck: (d: Date) => boolean) =>
		!!event.date && slotCheck(event.date);

	let sortedEvents: Array<[EType, Array<TAllTypesWithId>]>;
	$: {
		sortedEvents = [
			[EType.BLOCK, data.events.filter(isBlock)],
			[EType.EVENT, [...data.events.filter(isEvent), ...data.externalEvents]],
			[EType.TASK, data.events.filter(isTask)],
			[EType.REMINDER, data.events.filter(isReminder)]
		];

		// Update reference in events change
		$selectedEvent = [...data.events, ...data.tasks].find(
			(c) => c.eventId === $selectedEvent?.eventId
		);
	}

	/**
	 * Give a time slot find the start and end time grid slot
	 * if endTime is null, add 15 min to the start time
	 * if the time is not in a 15 min slot, move it to the nearest before
	 * if when moving start and end are the same, move the end 15 min later
	 */
	function getScheduleSlot(e: TAllTypesWithId) {
		if (!e.date) return '';
		let startDate = roundToNearestMinutes(e.date, { nearestTo: 15, roundingMethod: 'floor' });
		let endTime = e.endDate ?? addMinutes(15, e.date);
		endTime = roundToNearestMinutes(endTime, { nearestTo: 15, roundingMethod: 'floor' });
		if (isSameMinute(startDate, endTime)) {
			endTime = addMinutes(15, startDate);
		}
		return `time-${format('HHmm', startDate)} / time-${format('HHmm', endTime)}`;
	}

	const modalZIndex = 40;

	let currentTimeInView = false;
	const inviewOption: Options = {
		rootMargin: '-50px'
	};

	const handleViewChange = ({ detail }: CustomEvent<ObserverEventDetails>) => {
		currentTimeInView = detail.inView;
	};
	const scrollCurrentIntoView = () => {
		document.getElementById('current-time')?.scrollIntoView({
			block: 'center',
			behavior: 'smooth'
		});
	};

	async function handleDropOnTime(e: Event, timeSlot: Date) {
		e.preventDefault();
		if (!dragging) return;
		loading.increase();
		await fetch(`/event/${dragging.eventId}/date`, {
			method: 'PUT',
			body: JSON.stringify({ from: formatISO(timeSlot) })
		});

		dragging = undefined;
		hoverTime = undefined;
		// TODO manage error
		loading.decrease();
		await invalidateAll();
	}

	function handleTimeDoubleClick(time: Date) {
		upsert.create(time);
	}

	const notypecheck = (x: any) => x;
</script>

<div>
	<div class="flex sticky top-0 bg-gray-900 py-3 px-1" style:z-index={modalZIndex - 2}>
		<div class="flex-1">
			<p class="text-lg md:text-4xl dark:text-white">
				{format('E do MMM yy ', data.date)}
			</p>
		</div>
		<ButtonGroup size="xs">
			<Button size="xs" href="/day?date={formatISO(subDays(1, startOfDay(data.date)))}">
				<AngleLeftOutline />
			</Button>
			<Button size="xs" href="/day?date={formatISO(startOfDay(currentTime))}">Today</Button>
			<Button size="xs" href="/day?date={formatISO(addDays(1, startOfDay(data.date)))}">
				<AngleRightOutline />
			</Button>
		</ButtonGroup>
	</div>

	<Modal open={$isLoading} dismissable={false} autoclose={false}>
		<div class="text-center text-lg font-bold">
			<p>Working...</p>
			<Spinner color="green" size={10} class="mt-2" />
		</div>
	</Modal>

	<div class="flex">
		<div class="hidden md:block w-80 flex-0 mt-6 sticky top-16 self-start">
			<TaskList
				tasks={data.tasks}
				on:dragtask={(e) => (dragging = e.detail)}
				on:dragend={() => {
					dragging = undefined;
					hoverTime = undefined;
				}}
			/>
		</div>

		<div class="schedule flex-1">
			<span
				class=" block bg-white p-1 pt-2 text-center text-gray-600 antialiased dark:bg-gray-900 dark:text-gray-400"
				aria-hidden="true"
				style="grid-column: event; grid-row: tracks;">Events</span
			>
			<span
				class=" block bg-white p-1 pt-2 text-center text-gray-600 antialiased dark:bg-gray-900 dark:text-gray-400"
				aria-hidden="true"
				style="grid-column: task; grid-row: tracks;">Tasks</span
			>
			<span
				class=" block bg-white p-1 pt-2 text-center text-gray-600 antialiased dark:bg-gray-900 dark:text-gray-400"
				aria-hidden="true"
				style="grid-column: reminder; grid-row: tracks;">Reminder</span
			>

			{#if !currentTimeInView && !dragging}
				<Button class="fixed bottom-[6rem] end-6 z-40" on:click={scrollCurrentIntoView}>
					Current Time
				</Button>
			{/if}
			<!-- Time indicator -->
			<div
				class="pointer-events-none"
				style:z-index={modalZIndex - 3}
				style:grid-column="times / reminder"
				style:grid-row="time-{format('HHmm', timeIndicator.nearestSlot)}"
			>
				<div class="relative w-full" style:top="calc({timeIndicator.offset}% - 25px)">
					<span class="relative px-2 text-pink-600 font-bold">
						{format('HH:mm', currentTime)}
					</span>
				</div>
			</div>
			<!-- Dotted line for current time -->
			<div
				use:inview={inviewOption}
				on:inview_change={handleViewChange}
				id="current-time"
				class="pointer-events-none"
				style:z-index={modalZIndex - 3}
				style:grid-column="times / reminder"
				style:grid-row="time-{format('HHmm', timeIndicator.nearestSlot)}"
			>
				<div
					style:top="{timeIndicator.offset}%"
					class="relative w-full border-b-2 border border-pink-600"
				/>
			</div>

			{#each timeBlocks as { time, check } (time)}
				<h2
					on:dblclick={() => !dragging && handleTimeDoubleClick(time)}
					class="time-slot m-0.5 text-center text-xs cursor-pointer select-none"
					class:brightness-50={timeIndicator.nearestSlot >= time}
					style:grid-row={`time-${format('HHmm', time)}`}
				>
					{format('HH:mm', time)}
				</h2>
				<div
					class:hidden={hoverTime !== time}
					class="z-40 text-center rounded-lg font-bold text-lg bg-blue-800"
					style:grid-column="event / reminder"
					style:grid-row="time-{format('HHmm', time)}"
				>
					{format('HH:mm', time)}
				</div>
				{#each sortedEvents as [type, events], i}
					<div
						aria-hidden="true"
						class="border-t border-dotted"
						class:z-50={dragging}
						class:border-gray-600={getMinutes(time) === 0}
						class:border-gray-300={getMinutes(time) === 30}
						style:grid-column={type}
						style:grid-row="time-{format('HHmm', time)}"
						on:dragenter={() => {
							hoverTime = time;
						}}
						on:drop={(e) => {
							handleDropOnTime(e, time);
						}}
						{...notypecheck({ ondragover: 'return false' })}
					></div>
					{#each events.filter((e) => timeCheck(e, check)) as e, k}
						<div
							tabindex={i * 10 + k}
							role="button"
							class="{getEventCardClass(e)} group relative rounded-lg border p-0.5 shadow-2xl"
							class:brightness-50={timeIndicator.nearestSlot > time}
							class:m-px={type === EType.BLOCK}
							class:m-0.5={type !== EType.BLOCK}
							class:glass={type !== EType.BLOCK}
							style:grid-column={type === EType.BLOCK ? 'event / reminder' : type}
							style:grid-row={getScheduleSlot(e)}
							style:z-index={type === EType.BLOCK ? 0 : k}
							on:click={() => ($selectedEvent = e)}
							on:keypress={(event) => {
								if (event.code === 'Enter') $selectedEvent = e;
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
</div>

{#if dragging}
	<div
		aria-hidden="true"
		class="flex fixed w-full z-[51] bg-rose-900 border-rose-800 h-16 max-w-lg -translate-x-1/2 rtl:translate-x-1/2 border rounded-full bottom-4 start-1/2"
		on:dragenter={() => {
			hoverTime = undefined;
		}}
		on:drop={() => {
			dragging = undefined;
		}}
		{...notypecheck({ ondragover: 'return false ' })}
	>
		<div class="flex-1"></div>
		<CloseOutline class="self-center" />
		<div class="flex-1"></div>
	</div>
{/if}

<style>
	.glass {
		/* From https://css.glass */
		/* background: rgba(255, 255, 255, 0.47); */

		box-shadow: 0 4px 30px rgba(0, 0, 0, 0.1);
		backdrop-filter: blur(1.5px);
		-webkit-backdrop-filter: blur(1.5px);
	}

	.blurred-time {
		/* background-color: rgba(0, 0, 0, 0.4); */
		/* background: rgb(0, 0, 0, 0.4); */
		/* background: linear-gradient(0deg, rgba(0,0,0,0.4) 0%, rgba(0,0,0,0.1) 100%); */
		background: repeating-linear-gradient(
			45deg,
			rgba(0, 0, 0, 0.2),
			rgba(0, 0, 0, 0.2) 10px,
			rgba(0, 0, 0, 0.3) 10px,
			rgba(0, 0, 0, 0.3) 20px
		);
	}

	.card__bg-work {
		background-position: center;
		background-image: url('$lib/assets/work.jpg');
	}

	/** Taken from https://css-tricks.com/building-a-conference-schedule-with-css-grid/ */

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
