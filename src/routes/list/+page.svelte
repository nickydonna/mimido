<script lang="ts">
	import { enhance } from '$app/forms';
	import { goto, invalidateAll } from '$app/navigation';
	import { page } from '$app/stores';
	import DetailModal from '$lib/components/details-modal/index.js';
	import { EStatus } from '$lib/parser/index.js';
	import { isDefined, isDone } from '$lib/util.js';
	import {
		Button,
		CloseButton,
		Drawer,
		Sidebar,
		SidebarGroup,
		SidebarItem,
		SidebarWrapper,
		Modal,
		Table,
		TableBody,
		TableBodyCell,
		TableBodyRow,
		TableHead,
		TableHeadCell
	} from 'flowbite-svelte';
	import {
		GridSolid,
		MailBoxSolid,
		BarsFromLeftOutline,
		ExclamationCircleOutline,
		CloseOutline
	} from 'flowbite-svelte-icons';
	import { sineIn } from 'svelte/easing';
	import type { PageData, SubmitFunction } from './$types';
	import type { TAllTypesWithId } from '$lib/server/calendar';

	export let data: PageData;

	let selectedEvent: TAllTypesWithId | undefined;
	let idOfDeleting: string | undefined;
	let showDelete = false;
	let loading = false;
	let tags: string[] = [];
	let tagFilter: string | undefined;
	let events = data.events;
	let groupedEvents: Array<[EStatus, TAllTypesWithId[]]>;

	$: {
		tags = [...new Set(data.events.map((e) => e.tags).flat())];
		showDelete = !!idOfDeleting;
		tagFilter = $page.url.searchParams.get('tag') ?? undefined;
		if (isDefined(tagFilter)) {
			events = data.events.filter((e) => e.tags.includes(tagFilter as string));
		} else {
			events = data.events;
		}
		groupedEvents = [
			[EStatus.DOING, events.filter((e) => e.status === EStatus.DOING)],
			[EStatus.TODO, events.filter((e) => e.status === EStatus.TODO)],
			[EStatus.DONE, events.filter((e) => e.status === EStatus.DONE)],
			[EStatus.BACK, events.filter((e) => e.status === EStatus.BACK)]
		];
	}

	const onDelete: SubmitFunction = () => {
		loading = true;
		return async ({ update }) => {
			loading = false;
			idOfDeleting = undefined;
			selectedEvent = undefined;
			update();
		};
	};

	let hideDrawer = true;
	let spanClass = 'flex-1 ms-3 whitespace-nowrap';
	let transitionParams = {
		x: -320,
		duration: 200,
		easing: sineIn
	};

	const handleStatusChange = async (event: CustomEvent<{ status: EStatus }>) => {
		if (!selectedEvent) return;
		loading = true;
		const res = await fetch(`/event/${selectedEvent.eventId}/status`, {
			method: 'PUT',
			body: JSON.stringify({ status: event.detail.status })
		});

		const updatedEvent = await res.json();
		events = events.map((e) => (e.eventId === updatedEvent.eventId ? updatedEvent : e));
		selectedEvent = updatedEvent;
		// TODO manage error
		loading = false;
		await invalidateAll();
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
			<SidebarGroup>
				<SidebarItem label="Today" {spanClass}>
					<svelte:fragment slot="icon">
						<GridSolid
							class="h-5 w-5 text-gray-500 transition duration-75 group-hover:text-gray-900 dark:text-gray-400 dark:group-hover:text-white"
						/>
					</svelte:fragment>
					<svelte:fragment slot="subtext">
						<span
							class="ms-3 inline-flex items-center justify-center rounded-full bg-gray-200 px-2 text-sm font-medium text-gray-800 dark:bg-gray-700 dark:text-gray-300"
						>
						</span>
					</svelte:fragment>
				</SidebarItem>
				<SidebarItem label="Inbox" {spanClass}>
					<svelte:fragment slot="icon">
						<MailBoxSolid
							class="h-5 w-5 text-gray-500 transition duration-75 group-hover:text-gray-900 dark:text-gray-400 dark:group-hover:text-white"
						/>
					</svelte:fragment>
					<svelte:fragment slot="subtext">
						<span
							class="ms-3 inline-flex h-3 w-3 items-center justify-center rounded-full bg-primary-200 p-3 text-sm font-medium text-primary-600 dark:bg-primary-900 dark:text-primary-200"
						>
							3
						</span>
					</svelte:fragment>
				</SidebarItem>
			</SidebarGroup>
			<SidebarGroup border>
				{#each tags as tag (tag)}
					<SidebarItem on:click={() => setTag(tag)} label={tag}></SidebarItem>
				{/each}
			</SidebarGroup>
		</SidebarWrapper>
	</Sidebar>
</Drawer>

<div class="flex mb-4">
	<Button on:click={() => (hideDrawer = false)}>
		<BarsFromLeftOutline />
		Menu
	</Button>
	{#if tagFilter}
		<Button class="ml-2" color="alternative" on:click={() => setTag()}>
			Filtering by: {tagFilter}
			<CloseOutline />
		</Button>
	{/if}
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
					on:click={() => (selectedEvent = event)}
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
	{loading}
	event={!idOfDeleting ? selectedEvent : undefined}
	on:close={() => (selectedEvent = undefined)}
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
