<script lang="ts">
	import { enhance } from '$app/forms';
	import {
		Card,
		Avatar,
		Button,
		Input,
		Label,
		Table,
		TableBody,
		TableBodyCell,
		TableBodyRow,
		Spinner
	} from 'flowbite-svelte';
	import frog from '$lib/assets/frog-avatar.jpg';
	import { invalidateAll } from '$app/navigation';
	import type { PageData } from './$types';

	export let data: PageData;
	let syncing = false;
</script>

<Card padding="sm" class="mx-auto">
	<div class="flex flex-col items-center pb-4">
		<Avatar size="lg" src={frog} />
		<h5 class="mb-1 text-xl font-medium text-gray-900 dark:text-white">Mimi</h5>
		<span class="text-sm text-gray-500 dark:text-gray-400">A Mimi</span>
		<div class="mt-4 flex space-x-3 lg:mt-6 rtl:space-x-reverse">
			<form
				action="?/resync"
				method="POST"
				use:enhance={() => {
					syncing = true;
					return () => {
						syncing = false;
						invalidateAll();
					};
				}}
			>
				<Button color="light" class="dark:text-white" disabled={syncing} type="submit">
					{#if syncing}
						<Spinner class="me-3" size="4" />
					{/if}
					Resync
				</Button>
			</form>
		</div>
	</div>
</Card>
{#if !data.main}
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
{:else}
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
{/if}
