<script>
	import '../app.pcss';

	import { RectangleListOutline, CalendarEditOutline, PlusOutline, FileCopyAltOutline } from 'flowbite-svelte-icons';
	import BottomNav from 'flowbite-svelte/BottomNav.svelte';
	import BottomNavItem from 'flowbite-svelte/BottomNavItem.svelte';
	import Navbar from 'flowbite-svelte/Navbar.svelte';
	import NavBrand from 'flowbite-svelte/NavBrand.svelte';
	import NavLi from 'flowbite-svelte/NavLi.svelte';
	import NavUl from 'flowbite-svelte/NavUl.svelte';
	import NavHamburger from 'flowbite-svelte/NavHamburger.svelte';
	import { formatISO } from 'date-fns/fp';
	import { page } from '$app/stores';
	import { copy } from 'svelte-copy';
	import { Button, Modal, Input, Popover } from 'flowbite-svelte';

	/** @type {import('./$types').LayoutData} */
	export let data;

	/** @type {string} */
	let date;
	$: date = $page.url.searchParams.get('date') ?? formatISO(new Date());
	let authModal = false;
</script>

<div class="container mx-auto h-full">
	<Navbar>
		<NavBrand href="/">
			<span class="self-center whitespace-nowrap text-xl font-semibold dark:text-white">MimiDo</span
			>
		</NavBrand>
		<NavHamburger />
		{#if data.token}
			<NavUl>
				<NavLi on:click={() => (authModal = true)}>Export Auth</NavLi>
			</NavUl>
			<Modal title="Access Token" bind:open={authModal} autoclose>
				<p class="text-base leading-relaxed text-gray-500 dark:text-gray-400">
					Copy this token, and use it on any other platform to copy login.
				</p>
				<p class="font-bold text-lg">DO NOT SHARE THIS!!!!</p>
				<Input type="text" value={data.token} class="select-all">
					<button slot="right" use:copy={data.token} class="pointer-events-auto" offset="30" id="copy-token">
						<FileCopyAltOutline />
					</button>
				</Input>
				<Popover class="w-64 text-sm font-light " title="Copied" triggeredBy="#copy-token" trigger="click">
					Paste in a new browser in the import token to login.
				</Popover>
				<svelte:fragment slot="footer">
					<Button color="alternative" on:click={() => (authModal = false)}>Close</Button>
				</svelte:fragment>
			</Modal>
		{/if}
	</Navbar>
	<div class="mb-16">
		<slot />
	</div>
	{#if data.token}
		<BottomNav position="fixed" classInner="grid-cols-3">
			<BottomNavItem btnName="ListTask" href="/list?date={date}">
				<RectangleListOutline
					class="mb-1 h-5 w-5 text-gray-500 group-hover:text-primary-600 dark:text-gray-400 dark:group-hover:text-primary-500"
				/>
			</BottomNavItem>
			<BottomNavItem btnName="CalenderView" href="/day?date={date}">
				<CalendarEditOutline
					class="mb-1 h-5 w-5 text-gray-500 group-hover:text-primary-600 dark:text-gray-400 dark:group-hover:text-primary-500"
				/>
			</BottomNavItem>
			<BottomNavItem btnName="Add" href="/form?date={date}">
				<PlusOutline
					class="mb-1 h-5 w-5 text-gray-500 group-hover:text-primary-600 dark:text-gray-400 dark:group-hover:text-primary-500"
				/>
			</BottomNavItem>
		</BottomNav>
	{/if}
</div>
