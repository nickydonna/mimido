<script>
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import DetailModal from '$lib/components/details-modal';
	import {
		Button,
		Table,
		TableBody,
		TableBodyCell,
		TableBodyRow,
		TableHead,
		TableHeadCell,
		CloseButton,
		Drawer,
		Sidebar,
		SidebarGroup,
		SidebarItem,
		SidebarWrapper,

		Modal

	} from 'flowbite-svelte';
	import {
		GridSolid,
		MailBoxSolid,
		BarsFromLeftOutline,

		ExclamationCircleOutline

	} from 'flowbite-svelte-icons';
	import { sineIn } from 'svelte/easing';

	/** @typedef {import('$lib/server/calendar').TAllTypesWithId} TAllTypesWithId */

	/** @type {import('./$types').PageData} */
	export let data;

	/** @type {TAllTypesWithId| undefined} */
	let selectedEvent;
	/** @type {string | undefined} */
	let idOfDeleting;
	let showDelete = false;
	let loading = false;
	$: {
		showDelete = !!idOfDeleting;
	}

	/** @type {import('./$types').SubmitFunction} */
	const onDelete = () => {
		loading = true;
		return async ({ update }) => {
			loading = false;
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
		</SidebarWrapper>
	</Sidebar>
</Drawer>

<Button on:click={() => (hideDrawer = false)}>
	<BarsFromLeftOutline />
</Button>
<Table>
	<TableHead>
		<TableHeadCell>Type</TableHeadCell>
		<TableHeadCell>Title</TableHeadCell>
		<TableHeadCell>Status</TableHeadCell>
	</TableHead>
	<TableBody>
		{#each data.events as event}
			<TableBodyRow on:click={() => (selectedEvent = event)}>
				<TableBodyCell>{event.type}</TableBodyCell>
				<TableBodyCell>{event.title}</TableBodyCell>
				<TableBodyCell>{event.status}</TableBodyCell>
			</TableBodyRow>
		{/each}
	</TableBody>
</Table>

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
