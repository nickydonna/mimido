<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { EStatus } from '$lib/parser/index.js';
	import { isDefined, isDone } from '$lib/util.js';
	import {
		Table,
		TableBody,
		TableBodyCell,
		TableBodyRow,
		TableHead,
		TableHeadCell,
		Toggle
	} from 'flowbite-svelte';
	import type { PageData } from './$types';
	import type { TAllTypesWithId } from '$lib/server/calendar';
	import { selectedEvent } from '$lib/stores';
	import FilterDropwdown from '$lib/components/filter-dropdown';

	export let data: PageData;

	let tags: string[] = [];
	let tagFilter: string | undefined;
	let events = data.events;
	let showDone = false;
	let groupedEvents: Array<[EStatus, TAllTypesWithId[]]>;

	$: {
		showDone = $page.url.searchParams.get('showDone') === 'true';
		tagFilter = $page.url.searchParams.get('tag') ?? undefined;
		events = !showDone ? data.events.filter((e) => e.status !== EStatus.DONE) : data.events;
		tags = [...new Set(events.map((e) => e.tags).flat())];
		if (isDefined(tagFilter)) {
			events = events.filter((e) => e.tags.includes(tagFilter as string));
		} else {
			events = events;
		}
		groupedEvents = [
			[EStatus.DOING, events.filter((e) => e.status === EStatus.DOING)],
			[EStatus.TODO, events.filter((e) => e.status === EStatus.TODO)],
			[EStatus.DONE, showDone ? events.filter((e) => e.status === EStatus.DONE) : []],
			[EStatus.BACK, events.filter((e) => e.status === EStatus.BACK)]
		];
		if ($selectedEvent) {
			$selectedEvent = data.events.find((e) => e.eventId === $selectedEvent?.eventId);
		}
	}

	function setTag(tag?: string) {
		let query = new URLSearchParams($page.url.searchParams.toString());
		if (tag) {
			query.set('tag', tag);
		} else {
			query.delete('tag');
		}
		goto(`?${query.toString()}`);
	}

	function toggleDone() {
		let query = new URLSearchParams($page.url.searchParams.toString());
		if (showDone) {
			query.delete('showDone');
		} else {
			query.set('showDone', 'true');
		}
		goto(`?${query.toString()}`);
	}
</script>

<div class="flex mb-4">
	<div class="flex items-center gap-2 mb-2 z-10">
		<h5
			id="drawer-label"
			class="inline-flex text-base font-semibold text-gray-500 dark:text-gray-400"
		>
			Tasks
		</h5>
		<FilterDropwdown {tags} on:select={(e) => setTag(e.detail)} on:clear={() => setTag()} />
	</div>
	<div class="flex-1"></div>
	<div class="flex">
		<Toggle checked={showDone} on:change={toggleDone}>Show Done</Toggle>
	</div>
</div>
<Table hoverable>
	<TableHead>
		<TableHeadCell>Title</TableHeadCell>
	</TableHead>
	<TableBody>
		{#each groupedEvents as [status, events]}
			{#if events.length > 0}
				<TableBodyRow color="purple">
					<TableBodyCell class="text-lg">
						{status.toUpperCase()}
					</TableBodyCell>
				</TableBodyRow>
			{/if}
			{#each events as event}
				<TableBodyRow
					class="cursor-pointer {isDone(event) ? 'line-through !text-gray-400' : ''}"
					on:click={() => ($selectedEvent = event)}
				>
					<TableBodyCell class={isDone(event) ? 'line-through !text-gray-400' : ''}
						>{event.title}</TableBodyCell
					>
				</TableBodyRow>
			{/each}
		{/each}
	</TableBody>
</Table>
