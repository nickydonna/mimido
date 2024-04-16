<script lang="ts">
	import { Badge, Button, ButtonGroup, Modal } from 'flowbite-svelte';
	import { createEventDispatcher } from 'svelte';
	import { formatDuration } from 'date-fns';
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
		TrashBinOutline
	} from 'flowbite-svelte-icons';
	import { Editor, rootCtx, defaultValueCtx, editorViewOptionsCtx } from '@milkdown/core';
	import { commonmark } from '@milkdown/preset-commonmark';
	import { nord } from '@milkdown/theme-nord';
	import { EStatus } from '$lib/parser/index.js';
	import { rruleToText } from '$lib/utils/rrule.js';
	import { selectedEvent } from '$lib/stores';
	import type { EventDispatcher } from 'svelte';
	import type { TAllTypesWithId } from '$lib/server/calendar';
	import { format } from 'date-fns/fp';
	import { invalidateAll } from '$app/navigation';

	const dispatch: EventDispatcher<{
		close: null;
		delete: null;
		statuschange: { status: EStatus };
		removeDate: null;
	}> = createEventDispatcher();

	const onClose = () => dispatch('close');
	const onDelete = () => dispatch('delete');
	/** @param {EStatus} status */
	const onStatusChange = (status: EStatus) => dispatch('statuschange', { status });

	/** @type {TAllTypesWithId | undefined} */
	export let event: TAllTypesWithId | undefined;
	export let loading = false;

	let open = false;
	let color = '';
	$: open = !!event;
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

	async function handleRemoveDate() {
		if (!event) return;
		loading = true;
		await fetch(`/event/${event.eventId}/removeDate`, {
			method: 'PUT'
		});

		await invalidateAll()
		loading = false;
		dispatch('removeDate');
	}
</script>

<Modal bind:open dismissable={false} on:close={() => (open = false)}>
	<svelte:fragment slot="header">
		<div class="flex w-full">
			<div class="flex-1 self-center">
				<div class="text-lg mb-1.5">
					<span class="mr-1 text-{color}-600">{event?.type.toUpperCase()}:</span>
					{event?.title}
				</div>
				<div>
					{#if event?.date && event?.endDate}
						<p class="font-semibold text-base leading-relaxed text-gray-500 dark:text-gray-400">
							from
							<span class="underline">
								{dateFormat(event?.date)}
							</span>
							until
							<span class="underline">
								{dateFormat(event?.endDate)}
							</span>
						</p>
					{:else if event?.date}
						<p class="text-base leading-relaxed text-gray-500 dark:text-gray-400">
							on
							<span class="underline">
								{dateFormat(event?.date)}
							</span>
						</p>
					{/if}
				</div>
			</div>
			{#if status && isDefined(statusIdx)}
				<div class="self-center">
					<div class="block mb-1.5">
						<ButtonGroup>
							{#if status !== EStatus.BACK}
								<Button disabled={loading} on:click={() => onStatusChange(EStatus.BACK)}>
									<ArrowLeftToBracketOutline class=" me-2 h-4 w-4 rotate-180" />
								</Button>
								<Button
									disabled={loading}
									on:click={() => isDefined(statusIdx) && onStatusChange(statuses[statusIdx - 1])}
									aria-label="Move to {statuses[statusIdx - 1]}"
								>
									<AngleLeftOutline class="me-2 h-4 w-4" />
								</Button>
							{/if}
							<Button>{status.toUpperCase()}</Button>
							{#if status !== EStatus.DONE}
								<Button
									disabled={loading}
									on:click={() => isDefined(statusIdx) && onStatusChange(statuses[statusIdx + 1])}
									aria-label="Move to {statuses[statusIdx + 1]}"
								>
									<AngleRightOutline class="me-2 h-4 w-4" />
								</Button>
								<Button
									disabled={loading}
									on:click={() => onStatusChange(EStatus.DONE)}
									aria-label="move to done"
								>
									<CheckOutline class="me-2 h-4 w-4" />
								</Button>
							{/if}
						</ButtonGroup>
					</div>
					{#if event?.date}
						<div class="text-right">
							<Button on:click={() => handleRemoveDate()} size="xs" color="purple"
								>Remove Dates</Button
							>
						</div>
					{/if}
				</div>
			{/if}
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
				<Button disabled={loading} color="red" type="button" on:click={onDelete}>
					<TrashBinOutline />
				</Button>
			</div>
			<div>
				<Button disabled={loading} on:click={onClose}>Close</Button>
				<Button
					disabled={loading}
					on:click={() => {
						event && selectedEvent.set(event);
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
