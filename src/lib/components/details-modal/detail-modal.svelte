<script lang="ts">
	import { Badge, Button, ButtonGroup, Indicator, Modal, Tooltip } from 'flowbite-svelte';
	import { createEventDispatcher } from 'svelte';
	import { formatDuration, formatISO, isSameDay } from 'date-fns';
	import {
		getEventColor,
		importanceToString,
		isBlock,
		isDefined,
		isReminder,
		isTask,
		loadToString,
		urgencyToString
	} from '$lib/util.js';
	import {
		AngleLeftOutline,
		AngleRightOutline,
		ArrowLeftToBracketOutline,
		CheckOutline,
		ChevronRightOutline,
		CloseOutline,
		ExclamationCircleOutline,
		ForwardOutline,
		TrashBinOutline,
		UndoOutline
	} from 'flowbite-svelte-icons';
	import { Editor, rootCtx, defaultValueCtx, editorViewOptionsCtx } from '@milkdown/core';
	import { commonmark } from '@milkdown/preset-commonmark';
	import { nord } from '@milkdown/theme-nord';
	import { EStatus } from '$lib/parser/index.js';
	import { rruleToText } from '$lib/utils/rrule.js';
	import { isLoading, loading, upsert } from '$lib/stores';
	import type { EventDispatcher } from 'svelte';
	import type { TAllTypesWithId } from '$lib/server/calendar';
	import { add, format } from 'date-fns/fp';
	import { invalidateAll } from '$app/navigation';

	const dispatch: EventDispatcher<{
		close: null;
		delete: null;
		statuschange: null;
		removeDate: null;
	}> = createEventDispatcher();

	const onClose = () => dispatch('close');
	export let event: TAllTypesWithId | undefined;

	let showDelete = false;
	let open = false;
	let postponing = false;
	let color = '';
	$: open = !!event && !showDelete;
	$: color = event ? getEventColor(event) : '';
	const dateFormat = format("E dd LLL yy 'at' HH:mm");
	const statuses = Object.values(EStatus);
	let status: EStatus | undefined;
	let statusIdx: number | undefined = undefined;
	$: {
		if (event && (isTask(event) || isReminder(event))) {
			status = event.status as EStatus;
			statusIdx = statuses.indexOf(status);
		}
	}

	function editor(dom: HTMLElement) {
		// to obtain the editor instance we need to store a reference of the editor.
		Editor.make()
			.config((ctx) => {
				ctx.set(rootCtx, dom);
				ctx.set(editorViewOptionsCtx, { editable: () => false });
				if (event?.description) {
					ctx.set(defaultValueCtx, event.description);
				}
			})
			.config(nord)
			.use(commonmark)
			.create();
	}

	const handleStatusChange = async (status: EStatus) => {
		if (!event) return;
		loading.increase();
		await fetch(`/event/${event?.eventId}/status`, {
			method: 'PUT',
			body: JSON.stringify({ status })
		});

		// TODO manage error
		loading.decrease();
		await invalidateAll();
		dispatch('statuschange');
	};

	async function handleRemoveDate() {
		if (!event) return;
		loading.increase();
		await fetch(`/event/${event.eventId}/removeDate`, {
			method: 'PUT'
		});

		await invalidateAll();
		loading.decrease();
		dispatch('removeDate');
	}

	async function handlePostpone(amount: 'days' | 'weeks') {
		if (!event || !event?.date) return;
		loading.increase();
		const from = formatISO(add({ [amount]: 1 }, event.date));
		const to = event.endDate ? formatISO(add({ [amount]: 1 }, event.endDate)) : undefined;
		await fetch(`/event/${event.eventId}/date`, {
			method: 'PUT',
			body: JSON.stringify({ from, to, postponing: true })
		});

		// TODO manage error
		loading.decrease();
		await invalidateAll();
		postponing = false;
	}

	async function handleDelete() {
		if (!event) return;
		loading.increase();
		await fetch(`/event/${event.eventId}`, {
			method: 'DELETE'
		});

		await invalidateAll();
		loading.decrease();
		showDelete = false;
		dispatch('delete');
	}
</script>

<Modal bind:open={showDelete} size="xs" on:close={() => (showDelete = false)}>
	<div class="text-center">
		<ExclamationCircleOutline class="mx-auto mb-4 h-12 w-12 text-gray-400 dark:text-gray-200" />
		<h3 class="mb-5 text-lg font-normal text-gray-500 dark:text-gray-400">
			Are you sure you want to delete this event?
		</h3>
		<form class="inline-block" on:submit|preventDefault={handleDelete}>
			<Button disabled={$isLoading} color="red" class="me-2" type="submit">Yes, I'm sure</Button>
			<Button
				disabled={$isLoading}
				type="button"
				on:click={() => (showDelete = false)}
				color="alternative"
			>
				No, cancel
			</Button>
		</form>
	</div>
</Modal>

<Modal bind:open dismissable={false}>
	<svelte:fragment slot="header">
		<div class="flex w-full">
			<div class="flex-1 self-center">
				<div class="text-lg mb-1.5">
					<span class="mr-1 text-{color}-600">{event?.type.toUpperCase()}:</span>
					{event?.title}
				</div>
				<div>
					{#if event?.date && event?.endDate}
						<span class="font-semibold text-base leading-relaxed text-gray-500 dark:text-gray-400">
							from
							<span class="underline">
								{dateFormat(event?.date)}
							</span>
							until
							<span class="underline">
								{isSameDay(event.endDate, event.date)
									? format('HH:mm', event.endDate)
									: dateFormat(event?.endDate)}
							</span>
						</span>
					{:else if event?.date}
						<span class="text-base leading-relaxed text-gray-500 dark:text-gray-400">
							on
							<span class="underline">
								{dateFormat(event?.date)}
							</span>
						</span>
					{/if}
					{#if event?.postponed && event?.postponed > 0}
						<Indicator color="red" border size="xl">
							<span class="text-white font-bold">{event?.postponed}</span>
						</Indicator>
						<Tooltip>Postponed {event?.postponed} times</Tooltip>
					{/if}
				</div>
			</div>
			<div class="self-center">
				{#if status && isDefined(statusIdx)}
					<div class="block mb-1.5">
						<ButtonGroup>
							{#if status !== EStatus.BACK}
								<Button disabled={$isLoading} on:click={() => handleStatusChange(EStatus.BACK)}>
									<ArrowLeftToBracketOutline class=" me-2 h-4 w-4 rotate-180" />
								</Button>
								<Button
									disabled={$isLoading}
									on:click={() =>
										isDefined(statusIdx) && handleStatusChange(statuses[statusIdx - 1])}
									aria-label="Move to {statuses[statusIdx - 1]}"
								>
									<AngleLeftOutline class="me-2 h-4 w-4" />
								</Button>
							{/if}
							<Button>{status.toUpperCase()}</Button>
							{#if status !== EStatus.DONE}
								<Button
									disabled={$isLoading}
									on:click={() =>
										isDefined(statusIdx) && handleStatusChange(statuses[statusIdx + 1])}
									aria-label="Move to {statuses[statusIdx + 1]}"
								>
									<AngleRightOutline class="me-2 h-4 w-4" />
								</Button>
								<Button
									disabled={$isLoading}
									on:click={() => handleStatusChange(EStatus.DONE)}
									aria-label="move to done"
								>
									<CheckOutline class="me-2 h-4 w-4" />
								</Button>
							{/if}
						</ButtonGroup>
					</div>
				{/if}
				{#if event?.date}
					{#if !postponing}
						<div class="text-right self-center">
							<Button size="xs" color="yellow" on:click={() => (postponing = true)}>
								<ForwardOutline class="w-4 h-4"></ForwardOutline>
							</Button>
							<Tooltip>Postpone</Tooltip>
							<Button on:click={() => handleRemoveDate()} size="xs" color="purple">
								<UndoOutline class="w-4 h-4"></UndoOutline>
							</Button>
							<Tooltip>Remove Dates</Tooltip>
						</div>
					{:else}
						<Button
							disable={$isLoading}
							size="xs"
							color="blue"
							on:click={() => handlePostpone('days')}
						>
							<ChevronRightOutline class="w-4 h-4"></ChevronRightOutline>
						</Button>
						<Tooltip>Tomorrow</Tooltip>
						<Button
							disable={$isLoading}
							size="xs"
							color="purple"
							on:click={() => handlePostpone('weeks')}
						>
							<ArrowLeftToBracketOutline class="w-4 h-4"></ArrowLeftToBracketOutline>
						</Button>
						<Tooltip>Next Week</Tooltip>
						<Button
							disable={$isLoading}
							size="xs"
							color="green"
							on:click={() => (postponing = false)}
						>
							<CloseOutline class="w-4 h-4"></CloseOutline>
						</Button>
						<Tooltip>Cancel Postpone</Tooltip>
					{/if}
				{/if}
			</div>
		</div>
	</svelte:fragment>
	<div>
		{#if event?.tags && event.tags.length > 0}
			<div class="mb-1 flex">
				<p class="mr-1 font-semibold">Tags: {' '}</p>
				{#each event.tags as t (t)}
					<Badge rounded class="mr-1" color="purple">{t}</Badge>
				{/each}
			</div>
		{/if}
	</div>
	<p class="text-base leading-relaxed text-gray-500 dark:text-gray-400">
		{#if event?.recur}
			<div class="flex-0 px-1">
				<div class="mb-1 border-b border-solid border-gray-400">Recur</div>
				<div>
					{rruleToText(event.recur)}
				</div>
			</div>
		{/if}
		{#if event?.alarms && event?.alarms.length > 0}
			<div class="flex-0 px-1">
				<div class="mb-1 border-b border-solid border-gray-400">Alarms</div>
				{#each event?.alarms as alarm}
					{formatDuration({ ...alarm.duration }, { format: ['days', 'hours', 'minutes'] })} before |
				{/each}
				<div></div>
			</div>
		{/if}
	</p>
	{#if !isBlock(event)}
		<p class="text-base leading-relaxed text-gray-500 dark:text-gray-400">
			{importanceToString(event?.importance, '|')}
			{urgencyToString(event?.urgency, '|')}
			{loadToString(event?.load)}
		</p>
	{/if}
	<div use:editor />
	<svelte:fragment slot="footer">
		<div class="flex w-full">
			<div class="flex-1">
				<Button
					disabled={$isLoading}
					color="red"
					type="button"
					on:click={() => (showDelete = true)}
				>
					<TrashBinOutline />
				</Button>
			</div>
			<div>
				<Button disabled={$isLoading} on:click={onClose}>Close</Button>
				<Button
					disabled={$isLoading}
					on:click={() => {
						event && upsert.update(event);
						onClose();
					}}
					class="mr-2"
					color="alternative"
				>
					Edit
				</Button>
			</div>
		</div>
	</svelte:fragment>
</Modal>
