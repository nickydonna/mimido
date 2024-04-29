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
	import {
		AngleLeftOutline,
		AngleRightOutline,
		BarsFromLeftOutline,
		CloseOutline
	} from 'flowbite-svelte-icons';
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
	import {
		getEventCardClass,
		isBlock,
		isDone,
		isEvent,
		isReminder,
		isTask,
		timeStore
	} from '$lib/util.js';
	import DetailModal from '$lib/components/details-modal/detail-modal.svelte';
	import { invalidateAll } from '$app/navigation';
	import {
		GradientButton,
		Modal,
		Spinner,
		Table,
		TableBody,
		TableBodyCell,
		TableBodyRow,
		TableHead,
		TableHeadCell
	} from 'flowbite-svelte';
	import { inview } from 'svelte-inview';
	import type { Options, ObserverEventDetails } from 'svelte-inview';
	import type { PageData } from './$types';
	import type { TAllTypesWithId } from '$lib/server/calendar';

	import { Drawer, CloseButton } from 'flowbite-svelte';
	import { sineIn } from 'svelte/easing';
	import { isLoading, loading, upsert } from '$lib/stores';

	let hideTaskDrawer = true;
	let transitionParams = {
		x: -320,
		duration: 200,
		easing: sineIn
	};

	export let data: PageData;

	let dragging: TAllTypesWithId | undefined;
	let showEventDetail: TAllTypesWithId | undefined;
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
			check: isWithinInterval({ start: subSeconds(1, h), end: subSeconds(1, addMinutes(30, h)) })
		}));

		if (showEventDetail) {
			showEventDetail = data.events.find((e) => e.eventId === showEventDetail?.eventId);
		}
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

	async function handleDragDrop(e: Event, timeSlot: Date) {
		e.preventDefault();
		if (!dragging) return;
		loading.increase();
		await fetch(`/event/${dragging.eventId}/date`, {
			method: 'PUT',
			body: JSON.stringify({ from: formatISO(timeSlot) })
		});

		dragging = undefined;
		hideTaskDrawer = true;
		hoverTime = undefined;
		// TODO manage error
		loading.decrease();
		await invalidateAll();
	}

	function handleTimeDoubleClick(time: Date) {
		upsert.create(time);
	}
</script>

<div>
	<div class="flex sticky top-0 bg-gray-900 py-3 px-1" style:z-index={modalZIndex - 2}>
		<GradientButton
			class="flex-0 mr-2"
			color="greenToBlue"
			size="sm"
			disabled={$isLoading}
			on:click={() => (hideTaskDrawer = false)}
		>
			<BarsFromLeftOutline class="w-4 h-4" />
		</GradientButton>
		<div class="flex-1">
			<p class="text-lg md:text-4xl dark:text-white">
				{format('do MMM yy ', data.date)}
			</p>
		</div>
		<ButtonGroup size="xs">
			<Button size="xs" href="/day?date={formatISO(subDays(1, startOfDay(data.date)))}">
				<AngleLeftOutline />
			</Button>
			<Button size="xs" href="/day">Today</Button>
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

	<DetailModal
		event={showEventDetail}
		on:close={() => (showEventDetail = undefined)}
		on:delete={() => (showEventDetail = undefined)}
	/>

	<div class="schedule">
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
		{#if showingToday && !dragging}
			<!-- Blur time before current slot -->
			<div
				class="blurred-time pointer-events-none"
				style:z-index={modalZIndex - 4}
				style:grid-column="times / reminder"
				style:grid-row="time-{format('HHmm', timeBlocks[0].time)} / time-{format(
					'HHmm',
					timeIndicator.nearestSlot
				)}"
			/>
			<!-- Blur percentage time of current slot -->
			<div
				class="pointer-events-none"
				style:z-index={modalZIndex - 4}
				style:grid-column="times / reminder"
				style:grid-row="time-{format('HHmm', timeIndicator.nearestSlot)}"
			>
				<div class="blurred-time relative w-full" style:height="{timeIndicator.offset}%" />
			</div>
		{/if}
		{#if !currentTimeInView}
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
				<span class="relative px-2">
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
				class="relative w-full border-b-2 border-dotted border-gray-700"
			/>
		</div>

		{#each timeBlocks as { time, check } (time)}
			<h2
				on:dblclick={() => !dragging && handleTimeDoubleClick(time)}
				class="time-slot m-0.5 text-center text-xs cursor-pointer select-none"
				style:grid-row={`time-${format('HHmm', time)}`}
			>
				{format('HH:mm', time)}
			</h2>
			<div
				class:hidden={hoverTime !== time}
				class="z-40 text-center font-bold text-lg bg-blue-800"
				style:grid-column="event / time"
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
					on:dragenter={() => (hoverTime = time)}
					on:drop={(e) => {
						handleDragDrop(e, time);
					}}
					ondragover="return false"
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
						on:click={() => (showEventDetail = e)}
						on:keypress={(event) => {
							if (event.code === 'Enter') showEventDetail = e;
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

{#if dragging}
	<div
		aria-hidden="true"
		class="flex fixed w-full z-[51] bg-rose-900 border-rose-800 h-16 max-w-lg -translate-x-1/2 rtl:translate-x-1/2 border rounded-full bottom-4 start-1/2"
		on:dragenter={() => (hoverTime = undefined) }
		on:drop={() => { dragging = undefined}}
		ondragover="return false"
	>
		<div class="flex-1"></div>
		<CloseOutline class="self-center" />
		<div class="flex-1"></div>
	</div>
{/if}

<Drawer
	backdrop={false}
	transitionType="fly"
	{transitionParams}
	hidden={hideTaskDrawer || !!dragging}
	id="sidebar1"
>
	<div class="flex items-center">
		<h5
			id="drawer-label"
			class="inline-flex items-center mb-4 text-base font-semibold text-gray-500 dark:text-gray-400"
		>
			Tasks
		</h5>
		<CloseButton on:click={() => (hideTaskDrawer = true)} class="mb-4 dark:text-white" />
	</div>
	<p class="mb-6 text-sm text-gray-500 dark:text-gray-400">Drop your task on time slots</p>

	<div class="pr-1">
		<Table hoverable divClass="overflow-hidden">
			<TableHead>
				<TableHeadCell>Title</TableHeadCell>
			</TableHead>
			<TableBody>
				{#each data.tasks as event}
					<TableBodyRow class="cursor-pointer {isDone(event) ? 'line-through !text-gray-400' : ''}">
						<TableBodyCell
							on:click={() => {
								upsert.update(event);
								hideTaskDrawer = true;
							}}
							class={isDone(event) ? 'text-ellipsis line-through !text-gray-400' : 'text-ellipsis'}
						>
							<div
								draggable="true"
								aria-hidden="true"
								on:dragstart={() => (dragging = event)}
								on:dragend={() => {
									dragging = undefined;
									hoverTime = undefined;
								}}
							>
								{event.title}
							</div>
						</TableBodyCell>
					</TableBodyRow>
				{/each}
			</TableBody>
		</Table>
	</div>
</Drawer>

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
