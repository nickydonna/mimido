<script>
	import { enhance } from '$app/forms';
	import {
		Card,
		Dropdown,
		DropdownItem,
		Avatar,
		Button,
		Modal,
		Input,
		Popover,
		Label,
		Table,
		TableBody,
		TableBodyCell,
		TableBodyRow
	} from 'flowbite-svelte';
	import { DotsHorizontalOutline, FileCopyAltOutline } from 'flowbite-svelte-icons';
	import { copy } from 'svelte-copy';
	import frog from '$lib/assets/frog-avatar.jpg';

	/** @type {import('./$types').PageData} */
	export let data;

	let showTokenModal = false;
</script>

<Card padding="sm" class="mx-auto">
	<div class="flex justify-end">
		<DotsHorizontalOutline />
		<Dropdown class="w-36">
			<DropdownItem on:click={() => (showTokenModal = true)}>Export Auth Token</DropdownItem>
		</Dropdown>
	</div>
	<div class="flex flex-col items-center pb-4">
		<Avatar size="lg" src={frog} />
		<h5 class="mb-1 text-xl font-medium text-gray-900 dark:text-white">Mimi</h5>
		<span class="text-sm text-gray-500 dark:text-gray-400">A Mimi</span>
	</div>
</Card>

<form class="flex flex-col space-y-6" method="POST" action="?/setCalendar">
	<h3 class="p-0 text-xl font-medium text-gray-900 dark:text-white">Login</h3>
	<Label class="space-y-2">
		<span>CalDav Server</span>
		<Input type="url" name="server" placeholder="https://caldav.fastmail.com/" required />
	</Label>
	<Label class="space-y-2">
		<span>Your email</span>
		<Input type="email" name="email" placeholder="name@company.com" required />
	</Label>
	<Label class="space-y-2">
		<span>Your password</span>
		<Input type="password" name="password" placeholder="•••••" required />
	</Label>
	<Label class="space-y-2">
		<span>Calendar</span>
		<Input type="text" name="calendar" placeholder="Calendar to use as store" required />
	</Label>
	<Button type="submit" class="w-full1">Connect</Button>
</form>

<div class="mt-4">
	<form action="?/addCalendarView" method="POST" use:enhance>
		<Label class="my-2 text-lg">Add another calendar to view from your account.</Label>
		<div class="flex">
			<Input placeholder="Calendar name" class="mr-2 flex-1" name="calendarName" type="text"
			></Input>
			<Button type="submit">Add</Button>
		</div>
	</form>
</div>

<Table class="mt-3">
	<caption
		class="border-b border-gray-400 bg-white p-5 text-left text-lg font-semibold text-gray-900 dark:bg-gray-800 dark:text-white"
	>
		Added Calendars
	</caption>
	<TableBody>
		<TableBodyRow>
			{#each data.calendars as cal}
				<TableBodyCell>
					{cal.type}
				</TableBodyCell>
				<TableBodyCell>
					{cal.name}
				</TableBodyCell>
			{/each}
		</TableBodyRow>
	</TableBody>
</Table>

<Modal title="Access Token" bind:open={showTokenModal} autoclose>
	<p class="text-base leading-relaxed text-gray-500 dark:text-gray-400">
		Copy this token, and use it on any other platform to copy login.
	</p>
	<p class="text-lg font-bold">DO NOT SHARE THIS!!!!</p>
	<!-- <Input type="text" value={data.token} class="select-all">
		<button
			slot="right"
			use:copy={data.token}
			class="pointer-events-auto"
			offset="30"
			id="copy-token"
		>
			<FileCopyAltOutline />
		</button>
	</Input> -->
	<Popover
		class="w-64 text-sm font-light "
		title="Copied"
		triggeredBy="#copy-token"
		trigger="click"
	>
		Paste in a new browser in the import token to login.
	</Popover>
	<svelte:fragment slot="footer">
		<Button color="alternative" on:click={() => (showTokenModal = false)}>Close</Button>
	</svelte:fragment>
</Modal>
