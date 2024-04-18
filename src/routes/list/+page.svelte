<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import DetailModal from '$lib/components/details-modal/index.js';
	import { EStatus } from '$lib/parser/index.js';
	import { isDefined, isDone } from '$lib/util.js';
	import {
		CloseButton,
		Drawer,
		Sidebar,
		SidebarGroup,
		SidebarItem,
		SidebarWrapper,
		Table,
		TableBody,
		TableBodyCell,
		TableBodyRow,
		TableHead,
		TableHeadCell,
		Toggle,
		GradientButton
	} from 'flowbite-svelte';
	import { BarsFromLeftOutline, CloseOutline } from 'flowbite-svelte-icons';
	import { sineIn } from 'svelte/easing';
	import type { PageData } from './$types';
	import type { TAllTypesWithId } from '$lib/server/calendar';

	export let data: PageData;

	let showEventDetail: TAllTypesWithId | undefined;
	let tags: string[] = [];
	let tagFilter: string | undefined;
	let events = data.events;
	let showDone = false;
	let groupedEvents: Array<[EStatus, TAllTypesWithId[]]>;

	$: {
		tags = [...new Set(data.events.map((e) => e.tags).flat())];
		showDone = $page.url.searchParams.get('showDone') === 'true';
		tagFilter = $page.url.searchParams.get('tag') ?? undefined;
		if (isDefined(tagFilter)) {
			events = data.events.filter((e) => e.tags.includes(tagFilter as string));
		} else {
			events = data.events;
		}
		groupedEvents = [
			[EStatus.DOING, events.filter((e) => e.status === EStatus.DOING)],
			[EStatus.TODO, events.filter((e) => e.status === EStatus.TODO)],
			[EStatus.DONE, showDone ? events.filter((e) => e.status === EStatus.DONE) : []],
			[EStatus.BACK, events.filter((e) => e.status === EStatus.BACK)]
		];
		if (showEventDetail) {
			showEventDetail = data.events.find((e) => e.eventId === showEventDetail?.eventId);
		}
	}

	let hideDrawer = true;
	let transitionParams = {
		x: -320,
		duration: 100,
		easing: sineIn
	};

	function setTag(tag?: string) {
		let query = new URLSearchParams($page.url.searchParams.toString());
		if (tag) {
			query.set('tag', tag);
		} else {
			query.delete('tag');
		}
		hideDrawer = true;
		goto(`?${query.toString()}`);
	}

	function toggleDone() {
		let query = new URLSearchParams($page.url.searchParams.toString());
		if (showDone) {
			query.delete('showDone');
		} else {
			query.set('showDone', 'true');
		}
		hideDrawer = true;
		goto(`?${query.toString()}`);
	}
</script>

<Drawer transitionType="fly" {transitionParams} bind:hidden={hideDrawer} id="sidebar1">
	<div class="flex items-center">
		<h5
			id="drawer-navigation-label-3"
			class="text-base font-semibold uppercase text-gray-500 dark:text-gray-400"
		>
			Menu
		</h5>
		<CloseButton on:click={() => (hideDrawer = true)} class="mb-4 dark:text-white" />
	</div>
	<Sidebar>
		<SidebarWrapper divClass="overflow-y-auto py-4 px-3 rounded dark:bg-gray-800">
			<SidebarGroup border>
				{#each tags as tag (tag)}
					<SidebarItem on:click={() => setTag(tag)} label={tag}></SidebarItem>
				{/each}
			</SidebarGroup>
		</SidebarWrapper>
	</Sidebar>
</Drawer>

<div class="flex mb-4">
	<div>
		<GradientButton color="greenToBlue" on:click={() => (hideDrawer = false)}>
			<BarsFromLeftOutline class="w-3.5 h-3.5 me-2" />
			Menu
		</GradientButton>
	</div>
	<div>
		{#if tagFilter}
			<GradientButton class="ml-2" color="tealToLime" on:click={() => setTag()}>
				#{tagFilter}
				<CloseOutline class="w-3.5 h-3.5 ms-2" />
			</GradientButton>
		{/if}
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
					on:click={() => (showEventDetail = event)}
				>
					<TableBodyCell class={isDone(event) ? 'line-through !text-gray-400' : ''}
						>{event.title}</TableBodyCell
					>
				</TableBodyRow>
			{/each}
		{/each}
	</TableBody>
</Table>

<DetailModal
	event={showEventDetail}
	on:close={() => (showEventDetail = undefined)}
	on:delete={() => (showEventDetail = undefined)}
/>
