<script>
	import { enhance } from '$app/forms';
	import { Card, Dropdown, DropdownItem, Avatar, Button, Modal, Input, Popover, Label } from 'flowbite-svelte';
	import { DotsHorizontalOutline, FileCopyAltOutline } from 'flowbite-svelte-icons';
	import { copy } from 'svelte-copy';

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
		<Avatar size="lg" src="/images/profile-picture-3.webp" />
		<h5 class="mb-1 text-xl font-medium text-gray-900 dark:text-white">Mimi</h5>
		<span class="text-sm text-gray-500 dark:text-gray-400">A Mimi</span>
	</div>
</Card>

<div class="mt-4">
  <Button href={data.googleUrl}>Add Google Calendar (view events only)</Button>
	<form action="?/addCalendarView" method="POST" use:enhance>
		<Label>Add another calendar to view from your account.</Label>
		<Input placeholder="Calendar name" name="calendarName" type="text"></Input>
		<Button type="submit">Add</Button>
	</form>
</div>

<Modal title="Access Token" bind:open={showTokenModal} autoclose>
	<p class="text-base leading-relaxed text-gray-500 dark:text-gray-400">
		Copy this token, and use it on any other platform to copy login.
	</p>
	<p class="text-lg font-bold">DO NOT SHARE THIS!!!!</p>
	<Input type="text" value={data.token} class="select-all">
		<button
			slot="right"
			use:copy={data.token}
			class="pointer-events-auto"
			offset="30"
			id="copy-token"
		>
			<FileCopyAltOutline />
		</button>
	</Input>
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
