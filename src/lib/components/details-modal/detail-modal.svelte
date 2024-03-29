<script>
	import { Badge, Button, ButtonGroup, Modal } from 'flowbite-svelte';
	import { createEventDispatcher } from 'svelte';
	import * as pkg from 'rrule';
	import { formatDuration } from 'date-fns';
	import { getEventColor, importanceToString, loadToString, urgencyToString } from '$lib/util';
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
	import { EStatus } from '$lib/parser';
	// @ts-expect-error - see https://github.com/jkbrzt/rrule/issues/548
	const { RRule } = pkg.default || pkg;

	/** @typedef {import('$lib/server/calendar').TEventSchema} TEventSchema */

	/**
	 * @type {import('svelte').EventDispatcher<{ close: null, delete: null, statuschange: { status: EStatus }}>}
	 */
	const dispatch = createEventDispatcher();

	const onClose = () => dispatch('close');
	const onDelete = () => dispatch('delete');
	/** @param {EStatus} status */
	const onStatusChange = (status) => dispatch('statuschange', { status });

	/** @type {TEventSchema | undefined} */
	export let event;
	export let loading = false;

	let open = false;
	let color = '';
	$: open = !!event;
	$: color = event ? getEventColor(event) : '';
	const statuses = Object.values(EStatus);
	let statusIdx = -1;
	$: statusIdx = statuses.indexOf(event?.status ?? '');

	/** @param {HTMLElement} dom */
	function editor(dom) {
		// to obtain the editor instance we need to store a reference of the editor.
		const MakeEditor = Editor.make()
			.config((ctx) => {
				ctx.set(rootCtx, dom);
				ctx.set(editorViewOptionsCtx, { editable: () => false })
				if (event?.description) {
					ctx.set(defaultValueCtx, event.description);
				}
			})
			.config(nord)
			.use(commonmark)
			.create();
	}

</script>

<Modal bind:open dismissable={false} classDialog="z-[12000]" on:close={() => (open = false)}>
	<svelte:fragment slot="header">
		<div class="flex w-full">
			<div class="flex-1 self-center">
				<span class="mr-1 text-{color}-600">{event?.type.toUpperCase()}:</span>
				{event?.title}
			</div>
			<div>
				<ButtonGroup>
					{#if event?.status !== EStatus.BACK}
						<Button disabled={loading} on:click={() => onStatusChange(EStatus.BACK)}>
							<ArrowLeftToBracketOutline class=" me-2 h-4 w-4 rotate-180" />
						</Button>
						<Button
							disabled={loading}
							on:click={() => onStatusChange(statuses[statusIdx - 1])}
							aria-label="Move to {statuses[statusIdx - 1]}"
						>
							<AngleLeftOutline class="me-2 h-4 w-4" />
						</Button>
					{/if}
					<Button>{event?.status.toUpperCase()}</Button>
					{#if event?.status !== EStatus.DONE}
						<Button
							disabled={loading}
							on:click={() => onStatusChange(statuses[statusIdx + 1])}
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
		</div>
	</svelte:fragment>
	<div>
		{#if event?.tag && event.tag.length > 0}
			<div class="mb-1 flex">
				<p class="mr-1 font-semibold">Tags: {' '}</p>
				{#each event.tag as t (t)}
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
					{RRule.fromString(event.recur).toText()}
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
	<p class="text-base leading-relaxed text-gray-500 dark:text-gray-400">
		{importanceToString(event?.importance, '|')}
		{urgencyToString(event?.urgency, '|')}
		{loadToString(event?.load)}
	</p>
	<div use:editor class="prose-sm" />
	<svelte:fragment slot="footer">
		<div class="flex w-full">
			<div class="flex-1">
				<Button
					disabled={loading}
					color="red"
					type="button"
					on:click={onDelete}
				>
					<TrashBinOutline />
				</Button>
			</div>
			<div>
				<Button disabled={loading} on:click={onClose}>Close</Button>
				<Button disabled={loading} href="/form/{event?.eventId}" class="mr-2" color="alternative"
					>Edit</Button
				>
			</div>
		</div>
	</svelte:fragment>
</Modal>
